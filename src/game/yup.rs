use bevy::prelude::*;

use crate::{
    assets::Characters,
    physics::Gravity,
    screens::{Screen, ingame::playing},
};

#[derive(Component, Debug, Default, Eq, PartialEq)]
pub enum CharacterState {
    #[default]
    Falling,
    Walking,
}

#[derive(Component, Debug)]
#[require(CharacterState)]
pub struct Yup;

pub fn plugin(app: &mut App) {
    // TODO: this .after is a bit cursed, come up with better spawning logic.
    app.add_systems(OnEnter(Screen::InGame), init.after(playing::init));
}

fn init(mut commands: Commands, characters: Res<Characters>) {
    commands.spawn((
        Name::new("Yup"),
        Yup,
        Gravity,
        Sprite {
            image: characters.yup.clone(),
            ..default()
        },
        // TODO: should all yups be spawned on specific Z-value for easy handling?
        Transform::from_xyz(0., 800., 1.),
    ));
}
