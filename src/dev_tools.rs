use bevy::{
    dev_tools::{
        states::log_transitions,
        ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    },
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<Screen>);

    app.add_plugins(DebugUiPlugin);
    app.add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, TOGGLE_KEY)));
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
