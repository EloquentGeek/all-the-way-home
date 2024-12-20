use bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen);
}

fn spawn_title_screen(mut commands: Commands) {
    commands
        .spawn((StateScoped(Screen::Title), Name::new("Title"), Node {
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            height: Val::Percent(100.),
            justify_content: JustifyContent::Start,
            justify_self: JustifySelf::Center,
            padding: UiRect::all(Val::Px(10.)),
            width: Val::Percent(100.),
            ..default()
        }))
        .with_children(|p| {
            p.spawn((Text::new("All The Way Home"), TextFont {
                font_size: 30.,
                ..default()
            }));
        });
}

// fn enter_gameplay_screen(_trigger: Trigger<OnPress>, mut next_screen: ResMut<NextState<Screen>>) {
//     next_screen.set(Screen::Playing);
// }
//
// fn enter_credits_screen(_trigger: Trigger<OnPress>, mut next_screen: ResMut<NextState<Screen>>) {
//     next_screen.set(Screen::Credits);
// }
//
// #[cfg(not(target_family = "wasm"))]
// fn exit_app(_trigger: Trigger<OnPress>, mut app_exit: EventWriter<AppExit>) {
//     app_exit.send(AppExit::Success);
// }
