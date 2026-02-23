use serde::Deserialize;
use ssh_ui::cursive::theme::{Color, Palette};
#[derive(Debug, Deserialize)]
pub struct Config {
    server_config:ServerConfig,
    colors:ColorConfig
}
impl Config {
    pub fn get_sk(&self) -> String{
        self.server_config.secret_key.clone()
    }
    pub fn get_port(&self) -> u16 {
        self.server_config.port
    }
    pub fn generate_palette(&self) -> Palette {
        let mut palette = Palette::retro();
        if let Some(color_str) = &self.colors.bg && let Some(color) = Color::parse(color_str) {
            palette.set_color("Background",color);
        }
        if let Some(color_str) = &self.colors.text && let Some(color) = Color::parse(color_str) {
            palette.set_color("Primary",color);
        }
        if let Some(color_str) = &self.colors.textbox && let Some(color) = Color::parse(color_str) {
            palette.set_color("Secondary",color);
        }
        if let Some(color_str) = &self.colors.highlight && let Some(color) = Color::parse(color_str) {
            palette.set_color("Highlight",color);
        }
        if let Some(color_str) = &self.colors.title && let Some(color) = Color::parse(color_str) {
            palette.set_color("TitlePrimary",color);
        }
        if let Some(color_str) = &self.colors.view_window && let Some(color) = Color::parse(color_str) {
            palette.set_color("View",color);
        }
        palette
    }
}
#[derive(Debug,Deserialize)]
pub struct ServerConfig {
    secret_key: String,
    port:u16
}

#[derive(Debug,Deserialize)]
pub struct ColorConfig {
    bg: Option<String>, //Background, the background color
    text: Option<String>, //Primary, text color in the chat box
    textbox: Option<String>, //Secondary, Color of textbox itself
    highlight: Option<String>, //Highlight, color of selected button (if button is selected)
    title: Option<String>, //TitlePrimary, color of title text
    view_window: Option<String>, //view, color of text window
}



