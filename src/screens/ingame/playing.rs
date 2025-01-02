use avian2d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::prelude::*;

use crate::{game::yup::Yup, screens::Screen};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::InGame), init);
    app.insert_resource(Gravity(Vector::NEG_Y * 1000.0));
}

#[derive(Component, Debug)]
pub struct Platform;

#[derive(Component, Debug)]
pub struct Obstacle;

#[derive(Component)]
pub struct MovementSpeed(pub Scalar);

fn init(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut time: ResMut<Time<Physics>>,
) {
    let platform = Sprite {
        color: Color::srgba(0.7, 0.7, 0.8, 0.25),
        custom_size: Some(Vec2::splat(50.0)),
        ..default()
    };
    commands.spawn((
        Name::new("Platform"),
        Platform,
        platform.clone(),
        Collider::rectangle(50., 50.),
        RigidBody::Static,
        StateScoped(Screen::InGame),
        Transform::from_xyz(0., 16. * 6., 0.).with_scale(Vec3::new(10., 0.5, 10.)),
    ));

    commands.spawn((
        Name::new("Platform"),
        Platform,
        platform.clone(),
        Collider::rectangle(50., 50.),
        RigidBody::Static,
        StateScoped(Screen::InGame),
        Transform::from_xyz(400., -150., 0.).with_scale(Vec3::new(10., 0.5, 10.)),
    ));

    commands.spawn((
        Name::new("Obstacle"),
        Obstacle,
        platform.clone(),
        Collider::rectangle(50., 50.),
        RigidBody::Static,
        StateScoped(Screen::InGame),
        Transform::from_xyz(500., -112., 0.),
    ));

    commands.spawn((
        Name::new("Obstacle"),
        Obstacle,
        platform.clone(),
        Collider::rectangle(50., 50.),
        RigidBody::Static,
        StateScoped(Screen::InGame),
        Transform::from_xyz(200., -112., 0.),
    ));

    commands.spawn((
        Name::new("Yup"),
        Yup,
        Collider::circle(20.),
        LockedAxes::ROTATION_LOCKED,
        Mesh2d(meshes.add(Circle::new(20.))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
        MovementSpeed(100.),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        RigidBody::Dynamic,
        StateScoped(Screen::InGame),
        Transform::from_xyz(0., 200., 0.),
    ));

    commands.spawn((
        Name::new("Yup"),
        Yup,
        Collider::circle(20.),
        LockedAxes::ROTATION_LOCKED,
        Mesh2d(meshes.add(Circle::new(20.))),
        MeshMaterial2d(materials.add(Color::srgb(0.1, 0.6, 0.9))),
        MovementSpeed(100.),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        RigidBody::Dynamic,
        StateScoped(Screen::InGame),
        Transform::from_xyz(-100., 500., 0.),
    ));
    time.unpause();
}
