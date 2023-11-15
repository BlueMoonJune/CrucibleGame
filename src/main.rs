use bevy::prelude::*;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, hello_world)
        .run();
}

fn hello_world() {
    println!("Hello, World!");
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Betty Parris".to_owned())));
    commands.spawn((Person, Name("Abigail Williams".to_owned())));
}

fn introduce_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("Welcome {}!", name.0);
    }
    println!("Fight!")
}
