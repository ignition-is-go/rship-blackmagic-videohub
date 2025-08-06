use anyhow::Result;
use std::env;

mod actions;
mod client;
mod emitters;
mod service;

use service::VideohubService;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Get configuration from environment variables
    let videohub_address = env::var("VIDEOHUB_ADDRESS").expect("VIDEOHUB_ADDRESS must be set");
    let videohub_port: u16 = env::var("VIDEOHUB_PORT")
        .expect("VIDEOHUB_PORT must be set")
        .parse()
        .expect("Failed to parse VIDEOHUB_PORT");

    let rship_address = env::var("RSHIP_ADDRESS").expect("RSHIP_ADDRESS must be set");
    let rship_port: u16 = env::var("RSHIP_PORT")
        .expect("RSHIP_PORT must be set")
        .parse()
        .expect("Failed to parse RSHIP_PORT");

    log::info!("Starting rship-blackmagic-videohub service");
    log::info!("Videohub: {videohub_address}:{videohub_port}");
    log::info!("Rship: {rship_address}:{rship_port}");

    // Create and start the service
    let service =
        VideohubService::new(videohub_address, videohub_port, rship_address, rship_port).await?;

    service.start().await?;

    Ok(())
}
