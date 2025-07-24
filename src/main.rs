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
    let videohub_host = env::var("VIDEOHUB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let videohub_port: u16 = env::var("VIDEOHUB_PORT")
        .unwrap_or_else(|_| "9990".to_string())
        .parse()
        .unwrap_or(9990);

    let rship_address = env::var("RSHIP_ADDRESS").unwrap_or_else(|_| "nyc.rship.io".to_string());
    let rship_port = env::var("RSHIP_PORT").unwrap_or_else(|_| "5155".to_string());

    log::info!("Starting rship-blackmagic-videohub service");
    log::info!("Videohub: {}:{}", videohub_host, videohub_port);
    log::info!("Rship: {}:{}", rship_address, rship_port);

    // Create and start the service
    let service =
        VideohubService::new(videohub_host, videohub_port, rship_address, rship_port).await?;

    service.start().await?;

    Ok(())
}
