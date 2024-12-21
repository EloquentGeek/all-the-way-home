use bevy::{ecs::system::EntityCommands, prelude::*, ui::Val::*};

/// An extension trait for spawning UI containers.
pub trait Containers {
    /// Spawns a root node that covers the full screen
    /// and centers its content horizontally and vertically.
    fn ui_root(&mut self) -> EntityCommands;
}

impl Containers for Commands<'_, '_> {
    fn ui_root(&mut self) -> EntityCommands {
        self.spawn((Name::new("UI Root"), Node {
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            height: Percent(100.0),
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            row_gap: Px(10.0),
            width: Percent(100.0),
            ..default()
        }))
    }
}
