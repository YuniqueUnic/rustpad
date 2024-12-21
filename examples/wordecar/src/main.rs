use std::env;

use bevy::{prelude::*, state::commands, time};

fn main() {
    env::set_var("WGPU_BACKEND", "vulkan");
    App::new().add_plugins(DefaultPlugins).run();
}
