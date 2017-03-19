use std::fs::File;
use std::io::Read;

use toml;

use error::Result;


#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}


#[derive(Debug, Deserialize)]
pub struct SerialConfig {
    pub path: String,
}


#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    pub interval: i32,
}


#[derive(Debug, Deserialize)]
pub struct FieldConfig {
    pub column: String,
    pub packet_id: String,
    pub field_id: String,
}


#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub serial: SerialConfig,
    pub logger: LoggerConfig,
    pub fields: Vec<FieldConfig>,
}


pub fn read_config() -> Result<Config> {
    let mut file = File::open("rpi-vbus-logger.toml")?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    drop(file);

    let config: Config = toml::from_slice(&bytes)?;

    Ok(config)
}
