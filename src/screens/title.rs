use bevy::prelude::*;

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen);
}

#[derive(Component)]
pub struct Button;

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

            p.spawn((Name::new("Start Button"), Button, Node {
                align_items: AlignItems::Center,
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                width: Val::Px(200.0),
                ..default()
            }))
            .with_children(|p| {
                p.spawn((Name::new("Button Text"), Text::new("Start")));
            })
            .observe(
                |_ev: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<Screen>>| {
                    next_state.set(Screen::Playing);
                },
            );
        });
}
