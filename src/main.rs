#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
use home::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
