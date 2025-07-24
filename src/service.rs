//! Blackmagic Videohub Service - unified service handling both videohub connection and rship integration

use anyhow::Result;
use rship_sdk::{ActionArgs, EmitterArgs, InstanceArgs, SdkClient, TargetArgs};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use videohub::{DeviceInfo, VideohubMessage};

use crate::client::VideohubClient;
use crate::actions::{SetRouteAction, SetInputLabelAction, SetOutputLabelAction};
use crate::emitters::{RouteChangedEmitter, DeviceStatusEmitter, LabelChangedEmitter};

// Commands sent to the videohub client task
#[derive(Debug)]
pub enum VideohubCommand {
    SetRoute { output: u32, input: u32 },
    SetInputLabel { input: u32, label: String },
    SetOutputLabel { output: u32, label: String },
}

// Events emitted from the videohub client task
#[derive(Debug)]
pub enum VideohubEvent {
    RouteChanged { output: u32, input: u32, output_label: Option<String>, input_label: Option<String> },
    DeviceStatusChanged { connected: bool, model_name: Option<String>, video_inputs: Option<u32>, video_outputs: Option<u32> },
    LabelChanged { port_type: String, port: u32, label: String },
}

// Main service for integrating Videohub with rship
pub struct VideohubService {
    sdk_client: SdkClient,
    rship_address: String,
    rship_port: String,
    videohub_host: String,
    videohub_port: u16,
}

impl VideohubService {
    pub async fn new(
        videohub_host: String,
        videohub_port: u16,
        rship_address: String,
        rship_port: String,
    ) -> Result<Self> {
        let sdk_client = SdkClient::init();
        
        Ok(Self {
            sdk_client,
            rship_address,
            rship_port,
            videohub_host,
            videohub_port,
        })
    }

    pub async fn start(&self) -> Result<()> {
        log::info!("Starting Videohub service");

        // First, establish connection to rship
        self.setup_rship_connection().await?;

        // Create the mpsc channels for command and event communication
        let (command_tx, command_rx) = mpsc::channel::<VideohubCommand>(100);
        let (event_tx, event_rx) = mpsc::channel::<VideohubEvent>(100);

        // Setup the rship instance with both command and event handling
        self.setup_rship_instance(command_tx, event_rx).await?;

        // Start the videohub task
        self.start_videohub_task(command_rx, event_tx).await?;

        // Keep the service running indefinitely
        log::info!("Service started successfully, running indefinitely...");
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            log::debug!("Service heartbeat - still running");
        }
    }

    async fn setup_rship_connection(&self) -> Result<()> {
        let url = format!("ws://{}:{}/myko", self.rship_address, self.rship_port);
        log::info!("Connecting to rship at: {}", url);
        
        self.sdk_client.set_address(Some(url));
        self.sdk_client.await_connection().await;
        
        log::info!("Connected to rship successfully");
        Ok(())
    }

    async fn setup_rship_instance(&self, command_tx: mpsc::Sender<VideohubCommand>, mut event_rx: mpsc::Receiver<VideohubEvent>) -> Result<()> {
        // Create the main instance
        let instance = self.sdk_client
            .add_instance(InstanceArgs {
                name: "Blackmagic Videohub".into(),
                short_id: "blackmagic-videohub".into(),
                code: "blackmagic-videohub".into(),
                service_id: "blackmagic-videohub-service".into(),
                cluster_id: None,
                color: "#FF6B35".into(),
                machine_id: format!("videohub-{}", std::process::id()),
                message: Some("Blackmagic Videohub Controller".into()),
                status: rship_sdk::InstanceStatus::Available,
            })
            .await;

        // Create the videohub target
        let mut target = instance
            .add_target(TargetArgs {
                name: "Videohub Device".into(),
                short_id: "videohub-device".into(),
                category: "video".into(),
                parent_targets: None,
            })
            .await;

        // Clone command senders for action handlers
        let tx_for_route = command_tx.clone();
        let tx_for_input_label = command_tx.clone();
        let tx_for_output_label = command_tx.clone();

        // Add route action
        target
            .add_action(
                ActionArgs::<SetRouteAction>::new("Set Video Route".into(), "set-route".into()),
                move |_action, data| {
                    let tx = tx_for_route.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(VideohubCommand::SetRoute { 
                            output: data.output, 
                            input: data.input 
                        }).await {
                            log::error!("Failed to send route command: {}", e);
                        }
                    });
                },
            )
            .await;

        // Add input label action
        target
            .add_action(
                ActionArgs::<SetInputLabelAction>::new("Set Input Label".into(), "set-input-label".into()),
                move |_action, data| {
                    let tx = tx_for_input_label.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(VideohubCommand::SetInputLabel { 
                            input: data.input, 
                            label: data.label 
                        }).await {
                            log::error!("Failed to send input label command: {}", e);
                        }
                    });
                },
            )
            .await;

        // Add output label action
        target
            .add_action(
                ActionArgs::<SetOutputLabelAction>::new("Set Output Label".into(), "set-output-label".into()),
                move |_action, data| {
                    let tx = tx_for_output_label.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(VideohubCommand::SetOutputLabel { 
                            output: data.output, 
                            label: data.label 
                        }).await {
                            log::error!("Failed to send output label command: {}", e);
                        }
                    });
                },
            )
            .await;

        // Create emitters using EmitterArgs
        let route_emitter = target
            .add_emitter(EmitterArgs::<RouteChangedEmitter>::new(
                "Route Changed".into(),
                "route-changed".into(),
            ))
            .await;

        let status_emitter = target
            .add_emitter(EmitterArgs::<DeviceStatusEmitter>::new(
                "Device Status".into(),
                "device-status".into(),
            ))
            .await;

        let label_emitter = target
            .add_emitter(EmitterArgs::<LabelChangedEmitter>::new(
                "Label Changed".into(),
                "label-changed".into(),
            ))
            .await;

        // Start the event emission task with the emitters
        tokio::spawn(async move {
            log::info!("Event emission task started");
            
            while let Some(event) = event_rx.recv().await {
                log::debug!("Processing event: {:?}", event);
                
                match event {
                    VideohubEvent::RouteChanged { output, input, output_label, input_label } => {
                        let data = RouteChangedEmitter {
                            output,
                            input,
                            output_label,
                            input_label,
                        };
                        if let Err(e) = route_emitter.pulse(data).await {
                            log::error!("Failed to emit route changed event: {}", e);
                        } else {
                            log::debug!("Emitted route changed: output {} -> input {}", output, input);
                        }
                    }
                    VideohubEvent::DeviceStatusChanged { connected, model_name, video_inputs, video_outputs } => {
                        let data = DeviceStatusEmitter {
                            connected,
                            model_name,
                            video_inputs,
                            video_outputs,
                        };
                        if let Err(e) = status_emitter.pulse(data).await {
                            log::error!("Failed to emit device status event: {}", e);
                        } else {
                            log::debug!("Emitted device status: connected={}", connected);
                        }
                    }
                    VideohubEvent::LabelChanged { port_type, port, label } => {
                        let data = LabelChangedEmitter {
                            port_type: port_type.clone(),
                            port,
                            label: label.clone(),
                        };
                        if let Err(e) = label_emitter.pulse(data).await {
                            log::error!("Failed to emit label changed event: {}", e);
                        } else {
                            log::debug!("Emitted label changed: {} port {}", port_type, port);
                        }
                    }
                }
            }
        });

        log::info!("rship instance and targets setup complete");
        Ok(())
    }

    async fn start_videohub_task(&self, mut command_rx: mpsc::Receiver<VideohubCommand>, event_tx: mpsc::Sender<VideohubEvent>) -> Result<()> {
        let host = self.videohub_host.clone();
        let port = self.videohub_port;
        
        tokio::spawn(async move {
            let mut client = VideohubClient::new(host, port);
            
            // Connect to videohub
            if let Err(e) = client.connect().await {
                log::error!("Failed to connect to videohub: {}", e);
                return;
            }
            
            log::info!("Videohub client task started");
            
            // Track current state to detect changes
            let mut current_device_info: Option<DeviceInfo> = None;
            let mut current_routes: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
            let mut current_input_labels: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
            let mut current_output_labels: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
            
            loop {
                tokio::select! {
                    // Handle incoming commands
                    Some(command) = command_rx.recv() => {
                        match command {
                            VideohubCommand::SetRoute { output, input } => {
                                if let Err(e) = client.set_route(output, input).await {
                                    log::error!("Failed to set route: {}", e);
                                }
                            }
                            VideohubCommand::SetInputLabel { input, label } => {
                                if let Err(e) = client.set_input_label(input, label).await {
                                    log::error!("Failed to set input label: {}", e);
                                }
                            }
                            VideohubCommand::SetOutputLabel { output, label } => {
                                if let Err(e) = client.set_output_label(output, label).await {
                                    log::error!("Failed to set output label: {}", e);
                                }
                            }
                        }
                    }
                    // Handle incoming videohub messages
                    message_result = client.receive_message() => {
                        match message_result {
                            Ok(Some(message)) => {
                                log::debug!("Received videohub message: {:?}", message);
                                
                                // Process messages and emit events on changes
                                match &message {
                                    VideohubMessage::DeviceInfo(info) => {
                                        if current_device_info.as_ref() != Some(info) {
                                            current_device_info = Some(info.clone());
                                            let _ = event_tx.send(VideohubEvent::DeviceStatusChanged {
                                                connected: true,
                                                model_name: info.model_name.clone(),
                                                video_inputs: info.video_inputs,
                                                video_outputs: info.video_outputs,
                                            }).await;
                                        }
                                    }
                                    VideohubMessage::VideoOutputRouting(routes) => {
                                        for route in routes {
                                            if current_routes.get(&route.to_output) != Some(&route.from_input) {
                                                current_routes.insert(route.to_output, route.from_input);
                                                let output_label = current_output_labels.get(&route.to_output).cloned();
                                                let input_label = current_input_labels.get(&route.from_input).cloned();
                                                let _ = event_tx.send(VideohubEvent::RouteChanged {
                                                    output: route.to_output,
                                                    input: route.from_input,
                                                    output_label,
                                                    input_label,
                                                }).await;
                                            }
                                        }
                                    }
                                    VideohubMessage::InputLabels(labels) => {
                                        for label in labels {
                                            if current_input_labels.get(&label.id) != Some(&label.name) {
                                                current_input_labels.insert(label.id, label.name.clone());
                                                let _ = event_tx.send(VideohubEvent::LabelChanged {
                                                    port_type: "input".to_string(),
                                                    port: label.id,
                                                    label: label.name.clone(),
                                                }).await;
                                            }
                                        }
                                    }
                                    VideohubMessage::OutputLabels(labels) => {
                                        for label in labels {
                                            if current_output_labels.get(&label.id) != Some(&label.name) {
                                                current_output_labels.insert(label.id, label.name.clone());
                                                let _ = event_tx.send(VideohubEvent::LabelChanged {
                                                    port_type: "output".to_string(),
                                                    port: label.id,
                                                    label: label.name.clone(),
                                                }).await;
                                            }
                                        }
                                    }
                                    _ => {
                                        // Handle other message types as needed
                                    }
                                }
                            }
                            Ok(None) => {
                                log::warn!("Videohub connection closed, attempting to reconnect...");
                                // Emit disconnection event
                                let _ = event_tx.send(VideohubEvent::DeviceStatusChanged {
                                    connected: false,
                                    model_name: current_device_info.as_ref().and_then(|info| info.model_name.clone()),
                                    video_inputs: current_device_info.as_ref().and_then(|info| info.video_inputs),
                                    video_outputs: current_device_info.as_ref().and_then(|info| info.video_outputs),
                                }).await;
                                
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                if let Err(e) = client.connect().await {
                                    log::error!("Failed to reconnect to videohub: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Error receiving videohub message: {}", e);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    async fn start_monitoring(&self) -> Result<()> {
        log::info!("Starting monitoring loops");
        
        // Start status monitoring
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                log::debug!("Status monitoring tick");
                // TODO: Emit status updates via rship
            }
        });

        // Keep the main thread alive
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            log::debug!("Executor running...");
        }
    }
}
