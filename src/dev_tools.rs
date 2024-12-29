use avian2d::prelude::*;
use bevy::{
    dev_tools::{
        states::log_transitions,
        ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    },
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{game::Game, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<Game>);

    app.add_plugins(DebugUiPlugin);
    app.add_systems(Update, toggle_debug.run_if(input_just_pressed(TOGGLE_KEY)));

    // bevy_inspector_egui
    app.add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, TOGGLE_KEY)));

    // Avian
    app.add_plugins((FrameTimeDiagnosticsPlugin, PhysicsDebugPlugin::default()));
    app.insert_gizmo_config(PhysicsGizmos::default(), GizmoConfig {
        // Off by default, enables with toggle key
        enabled: false,
        ..default()
    });
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug(mut config_store: ResMut<GizmoConfigStore>, mut options: ResMut<UiDebugOptions>) {
    let config = config_store.config_mut::<PhysicsGizmos>().0;
    config.enabled = !config.enabled;
    options.toggle();
}
