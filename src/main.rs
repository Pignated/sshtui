mod app;
mod config;
use std::{fs, sync::Arc};

use ssh_ui::AppServer;
use tokio::{spawn, sync::broadcast};

use crate::{app::TestApp, config::Config};

#[tokio::main]
async fn main() {
    let config_str =
        fs::read_to_string("config.toml").expect("ERROR: Could not read in the config file");
    let config: Config = toml::from_str(&config_str)
        .expect("ERROR: Could not parse config file. Please make sure variables are valid");
    let key_pair = ssh_ui::russh_keys::load_secret_key(config.get_sk(), None).unwrap();
    let palette = config.generate_palette();
    let mut server = AppServer::new_with_port(config.get_port());
    let (broadcast_tx, _) = broadcast::channel(1024);
    let (user_tx, mut user_rx) = broadcast::channel(1024);
    let app = TestApp::new(broadcast_tx.clone(), user_tx.clone(), palette);
    spawn(async move {
        while let Ok(msg) = user_rx.recv().await {
            let _ = broadcast_tx.send(msg.clone());
        }
    });
    server.run(&[key_pair], Arc::new(app)).await.unwrap();
}
