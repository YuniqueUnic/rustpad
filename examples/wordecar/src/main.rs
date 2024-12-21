use std::env;

use bevy::{
    core::FrameCount,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    state::commands,
    time,
    window::{PresentMode, WindowLevel, WindowTheme},
};

fn main() {
    env::set_var("WGPU_BACKEND", "vulkan");
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Word E-car".to_string(),
                    resolution: (800., 600.).into(),
                    present_mode: PresentMode::AutoVsync,
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Light),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    visible: false, // avoid flashing
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(
            Update,
            (switch_win_level, keyboard_input_system, make_visible),
        )
        .run();
}

#[derive(Component)]
struct Player;

fn make_visible(mut windows: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        windows.single_mut().visible = true;
    }
}

fn switch_win_level(input: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>) {
    if input.just_pressed(KeyCode::KeyT) {
        let mut win = windows.single_mut();

        win.window_level = match win.window_level {
            WindowLevel::AlwaysOnBottom => WindowLevel::Normal,
            WindowLevel::Normal => WindowLevel::AlwaysOnTop,
            WindowLevel::AlwaysOnTop => WindowLevel::Normal,
        };

        info!("Window level changed to {:?}", win.window_level);
    }
}

fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    info!("Keyboard input: {:?}", keyboard_input);
    for mut transform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            transform.translation.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= 1.0;
        }
        println!("{:?}", transform.translation);
    }
}
