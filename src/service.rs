//! Blackmagic Videohub Service - unified service handling both videohub connection and rship integration

use anyhow::Result;
use rship_sdk::{ActionArgs, EmitterArgs, InstanceArgs, SdkClient, TargetArgs};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use videohub::{DeviceInfo, VideohubMessage};

use crate::actions::{
    SetInputAction, SetInputLabelAction, SetLabelAction, SetLockAction, SetOutputLabelAction,
    SetOutputLockAction, SetRouteAction, SetTakeModeAction, SetTakeModeOnThisOutputAction,
};
use crate::client::{NetworkInterface, VideohubClient};
use crate::emitters::{
    DeviceStatusEmitter, InputChangedEmitter, LabelChangedEmitter, LockChangedEmitter,
    NetworkInterfaceEmitter, TakeModeOnThisOutputEmitter,
};

// Commands sent to the videohub client task
#[derive(Debug)]
pub enum VideohubCommand {
    Route { output: u32, input: u32 },
    SetInput { output: u32, input: u32 }, // For output subtargets - output is implicit
    InputLabel { input: u32, label: String },
    OutputLabel { output: u32, label: String },
    OutputLock { output: u32, locked: bool },
    TakeMode { output: u32, enabled: bool },
}

// Events emitted from the videohub client task
#[derive(Debug)]
pub enum VideohubEvent {
    Route {
        output: u32,
        input: u32,
        input_label: Option<String>,
    },
    DeviceStatus {
        connected: bool,
        model_name: Option<String>,
        video_inputs: Option<u32>,
        video_outputs: Option<u32>,
    },
    Label {
        port_type: String,
        port: u32,
        label: String,
    },
    OutputLock {
        output: u32,
        locked: bool,
    },
    TakeMode {
        output: u32,
        enabled: bool,
    },
    NetworkInterface {
        interface: NetworkInterface,
    },
}

// Main service for integrating Videohub with rship
pub struct VideohubService {
    sdk_client: SdkClient,
    rship_address: String,
    rship_port: u16,
    videohub_host: String,
    videohub_port: u16,
}

impl VideohubService {
    pub async fn new(
        videohub_host: String,
        videohub_port: u16,
        rship_address: String,
        rship_port: u16,
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
        let (rship_reconnect_tx, rship_reconnect_rx) = mpsc::channel::<()>(10);

        // Setup the rship instance with both command and event handling
        self.setup_rship_instance(command_tx, event_rx).await?;

        // Start the videohub task
        self.start_videohub_task(command_rx, event_tx, rship_reconnect_rx)
            .await?;

        // Start watching rship connection status for reconnections
        self.start_connection_monitoring(rship_reconnect_tx).await?;

        // Keep the service running indefinitely
        log::info!("Service started successfully, running indefinitely...");
        std::future::pending::<()>().await;

        Ok(())
    }

    async fn setup_rship_connection(&self) -> Result<()> {
        let url = format!("ws://{}:{}/myko", self.rship_address, self.rship_port);
        log::debug!("Connecting to rship at: {url}");

        self.sdk_client.set_address(Some(url));
        self.sdk_client.await_connection().await;

        log::debug!("Connected to rship successfully");
        Ok(())
    }

    async fn setup_rship_instance(
        &self,
        command_tx: mpsc::Sender<VideohubCommand>,
        mut event_rx: mpsc::Receiver<VideohubEvent>,
    ) -> Result<()> {
        // We'll need to create output subtargets dynamically once we know device capabilities
        let command_tx_for_subtargets = command_tx.clone();
        // Create the main instance
        let instance = self
            .sdk_client
            .add_instance(InstanceArgs {
                name: "Blackmagic Videohub".into(),
                short_id: "blackmagic-videohub-02".into(),
                code: "blackmagic-videohub".into(),
                service_id: "blackmagic-videohub-service-02".into(),
                cluster_id: None,
                color: "#FF6B35".into(),
                machine_id: hostname::get()
                    .map(|h| h.to_string_lossy().into_owned())
                    .unwrap_or("unknown-host".to_string()),
                message: Some("Hello from Blackmagic Videohub!".into()),
                status: rship_sdk::InstanceStatus::Available,
            })
            .await;

        // Create the main videohub device target
        let mut device_target = instance
            .add_target(TargetArgs {
                name: "Videohub Device".into(),
                short_id: "videohub-device".into(),
                category: "video".into(),
                parent_targets: None,
            })
            .await;

        // Add all actions to the main device target
        let device_tx_for_route = command_tx.clone();
        let device_tx_for_input_label = command_tx.clone();
        let device_tx_for_output_label = command_tx.clone();
        let device_tx_for_output_lock = command_tx.clone();
        let device_tx_for_take_mode = command_tx.clone();

        device_target
            .add_action(
                ActionArgs::<SetRouteAction>::new("Set Video Route".into(), "set-route".into()),
                move |_action, data| {
                    let tx = device_tx_for_route.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(VideohubCommand::Route {
                                output: data.output.clamp(1, u32::MAX) - 1,
                                input: data.input.clamp(1, u32::MAX) - 1,
                            })
                            .await
                        {
                            log::error!("Failed to send route command: {e}");
                        }
                    });
                },
            )
            .await;

        device_target
            .add_action(
                ActionArgs::<SetInputLabelAction>::new(
                    "Set Input Label".into(),
                    "set-input-label".into(),
                ),
                move |_action, data| {
                    let tx = device_tx_for_input_label.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(VideohubCommand::InputLabel {
                                input: data.input.clamp(1, u32::MAX) - 1,
                                label: data.label,
                            })
                            .await
                        {
                            log::error!("Failed to send input label command: {e}");
                        }
                    });
                },
            )
            .await;

        device_target
            .add_action(
                ActionArgs::<SetOutputLabelAction>::new(
                    "Set Output Label".into(),
                    "set-output-label".into(),
                ),
                move |_action, data| {
                    let tx = device_tx_for_output_label.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(VideohubCommand::OutputLabel {
                                output: data.output.clamp(1, u32::MAX) - 1,
                                label: data.label,
                            })
                            .await
                        {
                            log::error!("Failed to send output label command: {e}");
                        }
                    });
                },
            )
            .await;

        device_target
            .add_action(
                ActionArgs::<SetOutputLockAction>::new(
                    "Set Output Lock".into(),
                    "set-output-lock".into(),
                ),
                move |_action, data| {
                    let tx = device_tx_for_output_lock.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(VideohubCommand::OutputLock {
                                output: data.output.clamp(1, u32::MAX) - 1,
                                locked: data.locked,
                            })
                            .await
                        {
                            log::error!("Failed to send output lock command: {e}");
                        }
                    });
                },
            )
            .await;

        device_target
            .add_action(
                ActionArgs::<SetTakeModeAction>::new(
                    "Set Take Mode".into(),
                    "set-take-mode".into(),
                ),
                move |_action, data| {
                    let tx = device_tx_for_take_mode.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(VideohubCommand::TakeMode {
                                output: data.output.clamp(1, u32::MAX) - 1,
                                enabled: data.enabled,
                            })
                            .await
                        {
                            log::error!("Failed to send take mode command: {e}");
                        }
                    });
                },
            )
            .await;

        // Add device-level emitters (device status and network interface)
        let device_status_emitter = device_target
            .add_emitter(EmitterArgs::<DeviceStatusEmitter>::new(
                "Device Status".into(),
                "device-status".into(),
            ))
            .await;

        let device_network_interface_emitter = device_target
            .add_emitter(EmitterArgs::<NetworkInterfaceEmitter>::new(
                "Network Interface".into(),
                "network-interface".into(),
            ))
            .await;

        // Output subtargets will be created dynamically when we receive device info
        log::info!("Output subtargets will be created dynamically based on device capabilities");

        // Store instance and device target for dynamic subtarget creation
        let instance_for_subtargets = instance.clone();
        let device_target_for_subtargets = device_target.clone();

        // Start the event emission task with dynamic output target support
        tokio::spawn(async move {
            log::debug!("Event emission task started");

            // Dynamic storage for output emitters - will be populated when device info is received
            let mut output_emitters = Vec::new();
            let mut targets_created = false;

            while let Some(event) = event_rx.recv().await {
                log::debug!("Processing event");

                match event {
                    VideohubEvent::DeviceStatus {
                        connected,
                        model_name,
                        video_inputs,
                        video_outputs,
                    } => {
                        // Create output subtargets when we first receive device info
                        match video_outputs {
                            Some(num_outputs) if connected && !targets_created => {
                                log::info!("Creating {num_outputs} output subtargets dynamically");

                                for output_id in 1..num_outputs.clamp(0, u32::MAX - 1) + 1 {
                                    // Create output subtarget
                                    let mut output_target = instance_for_subtargets
                                        .add_target(TargetArgs {
                                            name: format!("Output {output_id}"),
                                            short_id: format!("output-{output_id}"),
                                            category: "video".into(),
                                            parent_targets: Some(vec![
                                                device_target_for_subtargets.clone(),
                                            ]),
                                        })
                                        .await;

                                    // Add all actions to each output subtarget
                                    let output_tx_for_route = command_tx_for_subtargets.clone();
                                    let output_tx_for_output_label =
                                        command_tx_for_subtargets.clone();
                                    let output_tx_for_output_lock =
                                        command_tx_for_subtargets.clone();
                                    let output_tx_for_take_mode = command_tx_for_subtargets.clone();

                                    output_target
                                        .add_action(
                                            ActionArgs::<SetInputAction>::new(
                                                "Set Input".into(),
                                                "set-input".into(),
                                            ),
                                            move |_action, data| {
                                                let tx = output_tx_for_route.clone();
                                                let current_output_id = output_id;
                                                tokio::spawn(async move {
                                                    if let Err(e) = tx
                                                        .send(VideohubCommand::SetInput {
                                                            output: current_output_id,
                                                            input: data.input.clamp(1, u32::MAX)
                                                                - 1,
                                                        })
                                                        .await
                                                    {
                                                        log::error!(
                                                            "Failed to send set input command: {e}"
                                                        );
                                                    }
                                                });
                                            },
                                        )
                                        .await;

                                    output_target
                                    .add_action(
                                        ActionArgs::<SetLabelAction>::new(
                                            "Set Label".into(),
                                            "set-label".into(),
                                        ),
                                        move |_action, data| {
                                            let tx = output_tx_for_output_label.clone();
                                            let current_output_id = output_id;
                                            tokio::spawn(async move {
                                                if let Err(e) = tx
                                                    .send(VideohubCommand::OutputLabel {
                                                        output: current_output_id,
                                                        label: data.label,
                                                    })
                                                    .await
                                                {
                                                    log::error!(
                                                        "Failed to send output label command: {e}"
                                                    );
                                                }
                                            });
                                        },
                                    )
                                    .await;

                                    output_target
                                    .add_action(
                                        ActionArgs::<SetLockAction>::new(
                                            "Set Lock".into(),
                                            "set-lock".into(),
                                        ),
                                        move |_action, data| {
                                            let tx = output_tx_for_output_lock.clone();
                                            let current_output_id = output_id;
                                            tokio::spawn(async move {
                                                if let Err(e) = tx
                                                    .send(VideohubCommand::OutputLock {
                                                        output: current_output_id,
                                                        locked: data.locked,
                                                    })
                                                    .await
                                                {
                                                    log::error!(
                                                        "Failed to send output lock command: {e}"
                                                    );
                                                }
                                            });
                                        },
                                    )
                                    .await;

                                    output_target
                                        .add_action(
                                            ActionArgs::<SetTakeModeOnThisOutputAction>::new(
                                                "Set Take Mode".into(),
                                                "set-take-mode".into(),
                                            ),
                                            move |_action, data| {
                                                let tx = output_tx_for_take_mode.clone();
                                                let current_output_id = output_id;
                                                tokio::spawn(async move {
                                                    if let Err(e) = tx
                                                        .send(VideohubCommand::TakeMode {
                                                            output: current_output_id,
                                                            enabled: data.enabled,
                                                        })
                                                        .await
                                                    {
                                                        log::error!(
                                                            "Failed to send take mode command: {e}"
                                                        );
                                                    }
                                                });
                                            },
                                        )
                                        .await;

                                    // Add output-specific emitters (input-only versions)
                                    let input_changed_emitter = output_target
                                        .add_emitter(EmitterArgs::<InputChangedEmitter>::new(
                                            "Input Changed".into(),
                                            "input-changed".into(),
                                        ))
                                        .await;

                                    let label_emitter = output_target
                                        .add_emitter(EmitterArgs::<LabelChangedEmitter>::new(
                                            "Label Changed".into(),
                                            "label-changed".into(),
                                        ))
                                        .await;

                                    let output_lock_emitter = output_target
                                        .add_emitter(EmitterArgs::<LockChangedEmitter>::new(
                                            "Lock Changed".into(),
                                            "lock-changed".into(),
                                        ))
                                        .await;

                                    let take_mode_emitter = output_target
                                        .add_emitter(
                                            EmitterArgs::<TakeModeOnThisOutputEmitter>::new(
                                                "Take Mode Changed".into(),
                                                "take-mode-changed".into(),
                                            ),
                                        )
                                        .await;

                                    output_emitters.push((
                                        input_changed_emitter,
                                        label_emitter,
                                        output_lock_emitter,
                                        take_mode_emitter,
                                    ));
                                }

                                targets_created = true;
                                log::info!("Created {num_outputs} output subtargets");
                            }
                            _ => {}
                        }

                        let data = DeviceStatusEmitter {
                            connected,
                            model_name,
                            video_inputs,
                            video_outputs,
                        };
                        if let Err(e) = device_status_emitter.pulse(data).await {
                            log::error!("Failed to emit device status event: {e}");
                        } else {
                            log::debug!("Emitted device status: connected={connected}");
                        }
                    }
                    VideohubEvent::Route {
                        output,
                        input,
                        input_label,
                    } => {
                        let input_data = InputChangedEmitter {
                            input: input + 1,
                            input_label,
                        };

                        // Emit to the specific output subtarget if it exists
                        if let Some((input_changed_emitter, _, _, _)) =
                            output_emitters.get(output as usize)
                        {
                            if let Err(e) = input_changed_emitter.pulse(input_data).await {
                                log::error!(
                                    "Failed to emit input changed event on output {output}: {e}"
                                );
                            } else {
                                log::debug!(
                                    "Emitted input changed on output {output}: input {input}"
                                );
                            }
                        } else {
                            log::debug!(
                                "Output emitters not ready or output {output} out of range"
                            );
                        }
                    }
                    VideohubEvent::Label {
                        port_type,
                        port,
                        label,
                    } => {
                        let data = LabelChangedEmitter {
                            port_type: port_type.clone(),
                            port,
                            label: label.clone(),
                        };

                        // For output labels, emit to the specific output subtarget
                        if port_type == "output" {
                            if let Some((_, label_emitter, _, _)) =
                                output_emitters.get(port as usize)
                            {
                                if let Err(e) = label_emitter.pulse(data).await {
                                    log::error!(
                                        "Failed to emit label changed event on output {port}: {e}"
                                    );
                                } else {
                                    log::debug!(
                                        "Emitted label changed on output {port}: {port_type} port {port}"
                                    );
                                }
                            } else {
                                log::debug!(
                                    "Output emitters not ready or output {port} out of range for label"
                                );
                            }
                        } else {
                            // For input labels, emit to the first available output target as an example
                            if let Some((_, label_emitter, _, _)) = output_emitters.first() {
                                if let Err(e) = label_emitter.pulse(data).await {
                                    log::error!("Failed to emit input label changed event: {e}");
                                } else {
                                    log::debug!(
                                        "Emitted input label changed: {port_type} port {port}"
                                    );
                                }
                            }
                        }
                    }
                    VideohubEvent::OutputLock { output, locked } => {
                        let data = LockChangedEmitter { locked };

                        // Emit to the specific output subtarget
                        if let Some((_, _, output_lock_emitter, _)) =
                            output_emitters.get(output as usize)
                        {
                            if let Err(e) = output_lock_emitter.pulse(data).await {
                                log::error!(
                                    "Failed to emit lock changed event on output {output}: {e}"
                                );
                            } else {
                                log::debug!(
                                    "Emitted lock changed on output {output}: locked={locked}"
                                );
                            }
                        } else {
                            log::debug!(
                                "Output emitters not ready or output {output} out of range for lock"
                            );
                        }
                    }
                    VideohubEvent::TakeMode { output, enabled } => {
                        let data = TakeModeOnThisOutputEmitter { enabled };

                        // Emit to the specific output subtarget
                        if let Some((_, _, _, take_mode_emitter)) =
                            output_emitters.get(output as usize)
                        {
                            if let Err(e) = take_mode_emitter.pulse(data).await {
                                log::error!(
                                    "Failed to emit take mode changed event on output {output}: {e}"
                                );
                            } else {
                                log::debug!(
                                    "Emitted take mode changed on output {output}: enabled={enabled}"
                                );
                            }
                        } else {
                            log::debug!(
                                "Output emitters not ready or output {output} out of range for take mode"
                            );
                        }
                    }
                    VideohubEvent::NetworkInterface { interface } => {
                        let data = NetworkInterfaceEmitter {
                            interface_id: interface.id,
                            name: interface.name.clone(),
                            mac_address: interface.mac_address.clone(),
                            current_addresses: interface.current_addresses.clone(),
                            current_gateway: interface.current_gateway.clone(),
                            dynamic_ip: interface.dynamic_ip,
                        };
                        // Network interface emitter stays on the main device target
                        if let Err(e) = device_network_interface_emitter.pulse(data).await {
                            log::error!("Failed to emit network interface event: {e}");
                        } else {
                            log::debug!("Emitted network interface: {}", interface.name);
                        }
                    }
                }
            }
        });

        log::debug!("rship instance and targets setup complete");
        Ok(())
    }

    async fn start_videohub_task(
        &self,
        mut command_rx: mpsc::Receiver<VideohubCommand>,
        event_tx: mpsc::Sender<VideohubEvent>,
        mut rship_reconnect_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        let host = self.videohub_host.clone();
        let port = self.videohub_port;

        tokio::spawn(async move {
            let mut client = VideohubClient::new(host, port);

            // Connect to videohub
            if let Err(e) = client.connect().await {
                log::error!("Failed to connect to videohub: {e}");
                return;
            }

            log::debug!("Videohub client task started");

            // Track current state to detect changes
            let mut current_device_info: Option<DeviceInfo> = None;
            let mut current_routes: std::collections::HashMap<u32, u32> =
                std::collections::HashMap::new();
            let mut current_input_labels: std::collections::HashMap<u32, String> =
                std::collections::HashMap::new();
            let mut current_output_labels: std::collections::HashMap<u32, String> =
                std::collections::HashMap::new();
            let mut current_output_locks: std::collections::HashMap<u32, bool> =
                std::collections::HashMap::new();
            let mut current_take_mode: std::collections::HashMap<u32, bool> =
                std::collections::HashMap::new();
            let mut current_network_interfaces: std::collections::HashMap<u32, NetworkInterface> =
                std::collections::HashMap::new();

            loop {
                tokio::select! {
                    // Handle rship reconnection
                    Some(_) = rship_reconnect_rx.recv() => {
                        log::info!("Rship reconnected - forcing full state refresh");
                        client.force_full_state_refresh();
                    }
                    // Handle incoming commands
                    Some(command) = command_rx.recv() => {
                        match command {
                            VideohubCommand::Route { output, input } => {
                                if let Err(e) = client.set_route(output, input).await {
                                    log::error!("Failed to set route: {e}");
                                }
                            }
                            VideohubCommand::SetInput { output, input } => {
                                if let Err(e) = client.set_route(output, input).await {
                                    log::error!("Failed to set input for output {output}: {e}");
                                }
                            }
                            VideohubCommand::InputLabel { input, label } => {
                                if let Err(e) = client.set_input_label(input, label).await {
                                    log::error!("Failed to set input label: {e}");
                                }
                            }
                            VideohubCommand::OutputLabel { output, label } => {
                                if let Err(e) = client.set_output_label(output, label).await {
                                    log::error!("Failed to set output label: {e}");
                                }
                            }
                            VideohubCommand::OutputLock { output, locked } => {
                                log::info!("Output lock command received: output {output} locked={locked}");
                                // Note: Output lock setting would need to be implemented in the client
                                // For now, we'll log this as the protocol might not support setting locks
                            }
                            VideohubCommand::TakeMode { output, enabled } => {
                                log::info!("Take mode command received: output {output} enabled={enabled}");
                                // Note: Take mode setting would need to be implemented in the client
                                // For now, we'll log this as the protocol might not support setting take mode
                            }
                        }
                    }
                    // Handle incoming videohub messages
                    message_result = client.receive_message() => {
                        match message_result {
                            Ok(Some(message)) => {
                                log::debug!("Received videohub message");

                                // Process messages and emit events on changes
                                match &message {
                                    VideohubMessage::DeviceInfo(info) => {
                                        let should_emit = client.just_reconnected() ||
                                            current_device_info.as_ref() != Some(info);

                                        current_device_info = Some(info.clone());

                                        if should_emit
                                            && let Err(e) = event_tx.send(VideohubEvent::DeviceStatus {
                                                connected: true,
                                                model_name: info.model_name.clone(),
                                                video_inputs: info.video_inputs,
                                                video_outputs: info.video_outputs,
                                            }).await {
                                                log::error!("Failed to send device status event: {e}");
                                            }
                                    }
                                    VideohubMessage::VideoOutputRouting(routes) => {
                                        for route in routes {
                                            let should_emit = client.just_reconnected() ||
                                                current_routes.get(&route.to_output) != Some(&route.from_input);

                                            current_routes.insert(route.to_output, route.from_input);

                                            if should_emit {
                                                let input_label = current_input_labels.get(&route.from_input).cloned();
                                                if let Err(e) = event_tx.send(VideohubEvent::Route {
                                                    output: route.to_output,
                                                    input: route.from_input,
                                                    input_label,
                                                }).await {
                                                    log::error!("Failed to send route event for output {} to input {}: {e}", route.to_output, route.from_input);
                                                }
                                            }
                                        }
                                    }
                                    VideohubMessage::InputLabels(labels) => {
                                        for label in labels {
                                            let should_emit = client.just_reconnected() ||
                                                current_input_labels.get(&label.id) != Some(&label.name);

                                            current_input_labels.insert(label.id, label.name.clone());

                                            if should_emit
                                                && let Err(e) = event_tx.send(VideohubEvent::Label {
                                                    port_type: "input".to_string(),
                                                    port: label.id,
                                                    label: label.name.clone(),
                                                }).await {
                                                    log::error!("Failed to send input label event for input {}: {e}", label.id);
                                                }
                                        }
                                    }
                                    VideohubMessage::OutputLabels(labels) => {
                                        for label in labels {
                                            let should_emit = client.just_reconnected() ||
                                                current_output_labels.get(&label.id) != Some(&label.name);

                                            current_output_labels.insert(label.id, label.name.clone());

                                            if should_emit
                                                && let Err(e) = event_tx.send(VideohubEvent::Label {
                                                    port_type: "output".to_string(),
                                                    port: label.id,
                                                    label: label.name.clone(),
                                                }).await {
                                                    log::error!("Failed to send output label event for output {}: {e}", label.id);
                                                }
                                        }
                                    }
                                    VideohubMessage::VideoOutputLocks(locks) => {
                                        for lock in locks {
                                            let is_locked = matches!(lock.state, videohub::LockState::Locked);
                                            let should_emit = client.just_reconnected() ||
                                                current_output_locks.get(&lock.id) != Some(&is_locked);

                                            current_output_locks.insert(lock.id, is_locked);

                                            if should_emit
                                                && let Err(e) = event_tx.send(VideohubEvent::OutputLock {
                                                    output: lock.id,
                                                    locked: is_locked,
                                                }).await {
                                                    log::error!("Failed to send output lock event for output {}: {e}", lock.id);
                                                }
                                        }
                                    }
                                    VideohubMessage::EndPrelude => {
                                        // Clear the reconnected flag after processing all initial state
                                        client.clear_reconnected_flag();
                                        log::debug!("Cleared reconnection flag after receiving full state");
                                    }
                                    _ => {
                                        // Check if client state has new information that we should emit events for
                                        let client_state = client.state();

                                        // Check take mode changes
                                        for (&output, &enabled) in &client_state.take_mode {
                                            let should_emit = client.just_reconnected() ||
                                                current_take_mode.get(&output) != Some(&enabled);

                                            current_take_mode.insert(output, enabled);

                                            if should_emit
                                                && let Err(e) = event_tx.send(VideohubEvent::TakeMode {
                                                    output,
                                                    enabled,
                                                }).await {
                                                    log::error!("Failed to send take mode event for output {output}: {e}");
                                                }
                                        }

                                        // Check network interface changes
                                        for interface in &client_state.network_interfaces {
                                            let should_emit = client.just_reconnected() ||
                                                current_network_interfaces.get(&interface.id) != Some(interface);

                                            current_network_interfaces.insert(interface.id, interface.clone());

                                            if should_emit
                                                && let Err(e) = event_tx.send(VideohubEvent::NetworkInterface {
                                                    interface: interface.clone(),
                                                }).await {
                                                    log::error!("Failed to send network interface event for interface {}: {e}", interface.id);
                                                }
                                        }
                                    }
                                }
                            }
                            Ok(None) => {
                                log::warn!("Videohub connection closed, attempting to reconnect...");
                                // Emit disconnection event
                                if let Err(e) = event_tx.send(VideohubEvent::DeviceStatus {
                                    connected: false,
                                    model_name: current_device_info.as_ref().and_then(|info| info.model_name.clone()),
                                    video_inputs: current_device_info.as_ref().and_then(|info| info.video_inputs),
                                    video_outputs: current_device_info.as_ref().and_then(|info| info.video_outputs),
                                }).await {
                                    log::error!("Failed to send device disconnection event: {e}");
                                }

                                tokio::time::sleep(Duration::from_secs(5)).await;
                                if let Err(e) = client.connect().await {
                                    log::error!("Failed to reconnect to videohub: {e}");
                                } else {
                                    log::info!("Reconnected to videohub - will emit full state on next messages");
                                }
                            }
                            Err(e) => {
                                log::error!("Error receiving videohub message: {e}");
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_connection_monitoring(
        &self,
        rship_reconnect_tx: mpsc::Sender<()>,
    ) -> Result<()> {
        log::info!("Starting rship connection status monitoring");

        let sdk_client = self.sdk_client.clone();
        tokio::spawn(async move {
            let mut was_connected = true; // Assume initially connected
            let mut interval = interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                // Check connection by trying await_connection with timeout
                let connection_result =
                    tokio::time::timeout(Duration::from_millis(100), sdk_client.await_connection())
                        .await;

                let is_connected = connection_result.is_ok();

                if !was_connected && is_connected {
                    log::info!("Rship SDK connection restored - triggering full state refresh");
                    if let Err(e) = rship_reconnect_tx.send(()).await {
                        log::error!("Failed to send rship reconnection signal: {e}");
                        break;
                    }
                } else if was_connected && !is_connected {
                    log::warn!("Rship SDK connection lost");
                }

                was_connected = is_connected;
            }
        });

        Ok(())
    }

    #[allow(dead_code)]
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

        Ok(())
    }
}
