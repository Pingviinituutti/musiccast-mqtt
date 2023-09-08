use color_eyre::Result;

mod musiccast;
mod app_config;
mod mqtt;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let (mqtt_config, musiccast_config) = app_config::read_config()?;

    let mqtt_client = mqtt::init(&mqtt_config).await?;
    musiccast::init(mqtt_client, musiccast_config);

    tokio::signal::ctrl_c().await?;

    Ok(())
}
