use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use videohub::{DeviceInfo, Label, Route, VideohubCodec, VideohubMessage};

// Represents the current state of a Videohub device
#[derive(Debug, Clone, Default)]
pub struct VideohubState {
    pub device_info: Option<DeviceInfo>,
    pub input_labels: HashMap<u32, String>,
    pub output_labels: HashMap<u32, String>,
    pub video_output_routing: HashMap<u32, u32>, // output -> input
    pub connected: bool,
}

// Client for communicating with a Blackmagic Videohub device
pub struct VideohubClient {
    host: String,
    port: u16,
    state: VideohubState,
    connection: Option<Framed<TcpStream, VideohubCodec>>,
}

impl VideohubClient {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            state: VideohubState::default(),
            connection: None,
        }
    }

    // Connect to the videohub device
    pub async fn connect(&mut self) -> Result<()> {
        log::debug!("Connecting to videohub at {}:{}", self.host, self.port);

        let stream = TcpStream::connect(format!("{}:{}", self.host, self.port)).await?;
        let framed = Framed::new(stream, VideohubCodec);

        self.connection = Some(framed);
        self.state.connected = true;

        log::debug!("Connected to videohub successfully");
        Ok(())
    }

    // Disconnect from the videohub device
    #[allow(dead_code)]
    pub async fn disconnect(&mut self) {
        if let Some(mut conn) = self.connection.take() {
            let _ = conn.close().await;
        }
        self.state.connected = false;
        log::info!("Disconnected from videohub");
    }

    // Check if connected to the videohub
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.state.connected && self.connection.is_some()
    }

    // Get the current videohub state
    #[allow(dead_code)]
    pub fn state(&self) -> &VideohubState {
        &self.state
    }

    // Send a message to the videohub
    pub async fn send_message(&mut self, message: VideohubMessage) -> Result<()> {
        if let Some(conn) = &mut self.connection {
            conn.send(message)
                .await
                .map_err(|e| anyhow!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err(anyhow!("Not connected to videohub"))
        }
    }

    // Receive the next message from the videohub
    pub async fn receive_message(&mut self) -> Result<Option<VideohubMessage>> {
        if let Some(conn) = &mut self.connection {
            match conn.next().await {
                Some(Ok(message)) => {
                    self.handle_message(&message);
                    Ok(Some(message))
                }
                Some(Err(e)) => Err(anyhow!("Failed to receive message: {}", e)),
                None => {
                    // Connection closed
                    self.state.connected = false;
                    Ok(None)
                }
            }
        } else {
            Err(anyhow!("Not connected to videohub"))
        }
    }

    // Handle incoming messages and update state
    fn handle_message(&mut self, message: &VideohubMessage) {
        match message {
            VideohubMessage::DeviceInfo(info) => {
                log::info!(
                    "Device connected: {} | Inputs: {} | Outputs: {} | ID: {}",
                    info.model_name.as_deref().unwrap_or("Unknown"),
                    info.video_inputs.unwrap_or(0),
                    info.video_outputs.unwrap_or(0),
                    info.unique_id.as_deref().unwrap_or("Unknown")
                );
                self.state.device_info = Some(info.clone());
            }
            VideohubMessage::InputLabels(labels) => {
                log::debug!("Received input labels: {} labels", labels.len());
                self.state.input_labels.clear();
                for label in labels {
                    self.state.input_labels.insert(label.id, label.name.clone());
                }
            }
            VideohubMessage::OutputLabels(labels) => {
                log::debug!("Received output labels: {} labels", labels.len());
                self.state.output_labels.clear();
                for label in labels {
                    self.state
                        .output_labels
                        .insert(label.id, label.name.clone());
                }
            }
            VideohubMessage::VideoOutputRouting(routes) => {
                log::debug!("Received video output routing: {} routes", routes.len());
                self.state.video_output_routing.clear();
                for route in routes {
                    self.state
                        .video_output_routing
                        .insert(route.to_output, route.from_input);
                }
            }
            VideohubMessage::ACK => {
                log::debug!("Received ACK");
            }
            VideohubMessage::NAK => {
                log::warn!("Received NAK");
            }
            VideohubMessage::Ping => {
                log::debug!("Received ping");
            }
            _ => {
                log::debug!("Received unhandled message: {:?}", message);
            }
        }
    }

    // Set a video output route
    pub async fn set_route(&mut self, output: u32, input: u32) -> Result<()> {
        log::info!("Setting route: output {} -> input {}", output, input);

        let route = Route {
            to_output: output,
            from_input: input,
        };

        let message = VideohubMessage::VideoOutputRouting(vec![route]);
        self.send_message(message).await?;

        Ok(())
    }

    // Set an input label
    pub async fn set_input_label(&mut self, input: u32, label: String) -> Result<()> {
        log::info!("Setting input {} label to: {}", input, label);

        let label_msg = Label {
            id: input,
            name: label,
        };

        let message = VideohubMessage::InputLabels(vec![label_msg]);
        self.send_message(message).await?;

        Ok(())
    }

    // Set an output label
    pub async fn set_output_label(&mut self, output: u32, label: String) -> Result<()> {
        log::info!("Setting output {} label to: {}", output, label);

        let label_msg = Label {
            id: output,
            name: label,
        };

        let message = VideohubMessage::OutputLabels(vec![label_msg]);
        self.send_message(message).await?;

        Ok(())
    }

    // Request device information
    #[allow(dead_code)]
    pub async fn request_device_info(&mut self) -> Result<()> {
        log::debug!("Requesting device info");
        // Videohub protocol sends device info automatically on connection
        // We can send a ping to trigger a response
        let message = VideohubMessage::Ping;
        self.send_message(message).await?;
        Ok(())
    }
}
