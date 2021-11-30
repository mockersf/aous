use bevy::{
    core::{Time, Timer},
    math::Vec3,
    prelude::{
        info, App, AssetServer, Commands, Component, DespawnRecursiveExt, Entity, Query, Res,
        ResMut, State, SystemSet, Transform, With,
    },
    render2::camera::OrthographicCameraBundle,
    sprite2::PipelinedSpriteBundle,
};
use rand::Rng;

use crate::game_state::GameState;

const CURRENT_STATE: GameState = GameState::Splash;

#[derive(Component)]
struct ScreenTag;

struct Screen {
    done: Option<Timer>,
}
impl Default for Screen {
    fn default() -> Self {
        Screen { done: None }
    }
}

pub struct SplashPlugin;
impl bevy::app::Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Screen::default())
            .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
            .add_system_set(
                SystemSet::on_update(CURRENT_STATE)
                    .with_system(done)
                    .with_system(animate_logo),
            );
    }
}

fn setup(mut commands: Commands, mut screen: ResMut<Screen>, asset_server: Res<AssetServer>) {
    info!("Loading screen");

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(ScreenTag);
    commands
        .spawn_bundle(PipelinedSpriteBundle {
            texture: asset_server.load("logo.png"),
            ..Default::default()
        })
        .insert(ScreenTag)
        .insert(SplashGiggle(Timer::from_seconds(0.05, true)));

    screen.done = Some(Timer::from_seconds(0.7, false));
}

#[derive(Component)]
struct SplashGiggle(Timer);

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn done(time: Res<Time>, mut screen: ResMut<Screen>, mut state: ResMut<State<GameState>>) {
    if let Some(ref mut timer) = screen.done {
        timer.tick(time.delta());
        if timer.just_finished() {
            state.set(GameState::Playing).unwrap();
        }
    }
}

fn animate_logo(
    time: Res<Time>,
    mut query: Query<(&mut SplashGiggle, &mut Transform), With<ScreenTag>>,
) {
    for (mut timer, mut transform) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let translation = transform.translation;
            if translation.x != 0. || translation.y != 0. {
                *transform = Transform::identity();
                continue;
            }

            let scale = transform.scale;
            // `scale.0 != 1.` for floating numbers
            if (scale.x - 1.) > 0.01 {
                *transform = Transform::identity();
                continue;
            }

            let mut rng = rand::thread_rng();
            let act = rng.gen_range(0..100);

            if act < 20 {
                let span = 1.;
                let x: f32 = rng.gen_range(-span..span);
                let y: f32 = rng.gen_range(-span..span);
                *transform = Transform::from_translation(Vec3::new(x, y, 0.));
            }
            if act > 80 {
                let scale_diff = 0.02;
                let new_scale: f32 = rng.gen_range((1. - scale_diff)..(1. + scale_diff));
                *transform = Transform::from_scale(Vec3::splat(new_scale));
            }
        }
    }
}
