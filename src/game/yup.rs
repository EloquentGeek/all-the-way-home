use bevy::prelude::*;

use crate::{GameSet, assets::Characters, physics::Gravity, screens::Screen};

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
    app.add_systems(OnEnter(Screen::InGame), init.in_set(GameSet::Init));
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
