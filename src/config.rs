use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Config {
    serverConfig:ServerConfig
}
impl Config {
    pub fn get_sk(&self) -> String{
        self.config.secret_key.clone()
    }
    pub fn get_port(&self) -> u16 {
        self.config.port
    }
}
#[derive(Debug,Deserialize)]
pub struct ServerConfig {
    secret_key: String,
    port:u16
}
