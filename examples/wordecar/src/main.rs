use bevy::{prelude::*, state::commands};

fn main() {
    // println!("Hello, world!");
    let p = Position { x: 1.0, y: 2.0 };
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_person);
        app.add_systems(
            Update,
            (
                print_hello,
                (greet_person, update_people, greet_person).chain(),
                print_position_system,
            ),
        );
    }
}

fn update_people(mut query: Query<&mut Name, (With<Person>, With<Position>)>) {
    for mut name in &mut query {
        if name.0 == "Alice" {
            name.0 = "Alice 1".to_string();
            break;
        }
    }
}

fn greet_person(query: Query<&Name, (With<Person>, With<Position>)>) {
    for name in &query {
        println!("Hello, {}!", name.0);
    }
}

fn add_person(mut commands: Commands) {
    commands.spawn((
        Person,
        Name("Alice".to_string()),
        Position { x: 1.0, y: 2.0 },
    ));
    commands.spawn((Person, Name("Bob".to_string()), Position { x: 2.0, y: 4.0 }));
    commands.spawn((
        Person,
        Name("Unic".to_string()),
        Position { x: 3.0, y: 6.0 },
    ));
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn print_hello() {
    println!("Hello, world in bevy!");
}

#[derive(Component, Clone)]
struct Position {
    x: f32,
    y: f32,
}

fn print_position_system(query: Query<&Position>) {
    for p in &query {
        println!("position: {} {}", p.x, p.y);
    }
}

struct Entity(u64);
