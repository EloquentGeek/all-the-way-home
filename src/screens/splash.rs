use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

use crate::{AppSet, screens::Screen, ui::Containers};

pub fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(SPLASH_BACKGROUND_COLOR));
    app.add_systems(OnEnter(Screen::Splash), spawn_splash_screen);
    app.add_systems(
        Update,
        (
            tick_fade_in_out.in_set(AppSet::TickTimers),
            apply_fade_in_out.in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Splash)),
    );
    app.register_type::<SplashTimer>();
    app.add_systems(OnEnter(Screen::Splash), insert_splash_timer);
    app.add_systems(OnExit(Screen::Splash), remove_splash_timer);
    app.add_systems(
        Update,
        (
            tick_splash_timer.in_set(AppSet::TickTimers),
            check_splash_timer.in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Splash)),
    );
    app.add_systems(
        Update,
        continue_to_loading_screen
            .run_if(input_just_pressed(KeyCode::Escape).and(in_state(Screen::Splash))),
    );
}

const SPLASH_BACKGROUND_COLOR: Color = Color::srgb(0.157, 0.157, 0.157);
const SPLASH_DURATION_SECS: f32 = 1.8;
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

fn spawn_splash_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .ui_root()
        .insert((
            Name::new("Splash screen"),
            BackgroundColor(SPLASH_BACKGROUND_COLOR),
            StateScoped(Screen::Splash),
        ))
        .with_children(|children| {
            children.spawn((
                Name::new("Splash image"),
                Node {
                    margin: UiRect::all(Val::Auto),
                    width: Val::Percent(70.0),
                    ..default()
                },
                ImageNode::new(asset_server.load_with_settings(
                    "textures/splash.png",
                    |settings: &mut ImageLoaderSettings| {
                        // Make an exception for the splash image in case
                        // `ImagePlugin::default_nearest()` is used for pixel art.
                        settings.sampler = ImageSampler::linear();
                    },
                )),
                ImageNodeFadeInOut {
                    total_duration: SPLASH_DURATION_SECS,
                    fade_duration: SPLASH_FADE_DURATION_SECS,
                    t: 0.0,
                },
            ));
        });
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ImageNodeFadeInOut {
    total_duration: f32,
    fade_duration: f32,
    t: f32,
}

impl ImageNodeFadeInOut {
    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

fn tick_fade_in_out(time: Res<Time>, mut animation_query: Query<&mut ImageNodeFadeInOut>) {
    for mut anim in &mut animation_query {
        anim.t += time.delta_secs();
    }
}

fn apply_fade_in_out(mut animation_query: Query<(&ImageNodeFadeInOut, &mut ImageNode)>) {
    for (anim, mut image) in &mut animation_query {
        image.color.set_alpha(anim.alpha())
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SplashTimer(Timer);

impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once))
    }
}

fn insert_splash_timer(mut commands: Commands) {
    commands.init_resource::<SplashTimer>();
}

fn remove_splash_timer(mut commands: Commands) {
    commands.remove_resource::<SplashTimer>();
}

fn tick_splash_timer(time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    timer.0.tick(time.delta());
}

fn check_splash_timer(timer: ResMut<SplashTimer>, mut next_screen: ResMut<NextState<Screen>>) {
    if timer.0.just_finished() {
        next_screen.set(Screen::Loading);
    }
}

fn continue_to_loading_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Loading);
}