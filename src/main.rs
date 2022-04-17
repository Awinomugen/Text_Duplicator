//ターミナルが出なくなるおまじない
#![windows_subsystem = "windows"]

mod core;
mod parser;
use crate::core::AppBody;

use eframe::egui::Vec2;

fn main() {
    let app = AppBody::default();
    let native_options = eframe::NativeOptions{
        initial_window_size:Some(Vec2{x:640.0,y:480.0}),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
