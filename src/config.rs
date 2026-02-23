use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Config {
    server_config:ServerConfig
}
impl Config {
    pub fn get_sk(&self) -> String{
        self.server_config.secret_key.clone()
    }
    pub fn get_port(&self) -> u16 {
        self.server_config.port
    }
}
#[derive(Debug,Deserialize)]
pub struct ServerConfig {
    secret_key: String,
    port:u16
}
