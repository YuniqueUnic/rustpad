use bevy::app::Plugin;

fn dummy_fn() {
    println!("Hello, world!");
}

#[derive(Clone)]
pub struct Breackout;

impl Plugin for Breackout {
    fn build(&self, app: &mut bevy::prelude::App) {
        todo!()
    }
}
