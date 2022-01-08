use std::time::Duration;

use bevy::{
    core::{Time, Timer},
    prelude::{Commands, Entity, EventWriter, Plugin, Query, Res, ResMut, State, SystemSet},
    render::camera::OrthographicCameraBundle,
};
use bevy_egui::{egui, EguiContext};

use crate::{
    ant_hill::AntHill,
    camera::VisibleLots,
    food::{FoodDelay, FoodTimer, WorldEvents},
    ui::{Bonuses, GraphData},
};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Splash,
    Playing,
    Lost,
    Won,
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(GameState::Splash)
            .add_system_set(SystemSet::on_enter(GameState::Lost).with_system(background_scene))
            .add_system_set(SystemSet::on_update(GameState::Lost).with_system(lost_stats))
            .add_system_set(SystemSet::on_exit(GameState::Lost).with_system(despawn_all))
            .add_system_set(SystemSet::on_enter(GameState::Won).with_system(background_scene))
            .add_system_set(SystemSet::on_update(GameState::Won).with_system(won_stats))
            .add_system_set(SystemSet::on_exit(GameState::Won).with_system(despawn_all))
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(restart_game))
            .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(despawn_all));
    }
}

fn background_scene(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn lost_stats(
    egui_context: Res<EguiContext>,
    data: Res<GraphData>,
    mut state: ResMut<State<GameState>>,
) {
    egui::Window::new("All your ants died!")
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
            ui.label(format!(
                "You survived for {:.2?}!",
                data.end_time - data.start_time
            ));
            ui.label("");
            ui.label(format!(
                "You had a maximum of {} ants, with {} spawned.",
                data.max_ants, data.total_ants
            ));
            ui.label("");
            if ui.button("Restart!").clicked() {
                let _ = state.set(GameState::Playing);
            }
        });
}

fn won_stats(
    egui_context: Res<EguiContext>,
    data: Res<GraphData>,
    mut state: ResMut<State<GameState>>,
) {
    egui::Window::new("Your colony is now self sufficient!")
        .collapsible(false)
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
            ui.label(format!(
                "It took you {:.2?} to achieve!",
                data.end_time - data.start_time
            ));
            ui.label("");
            ui.label(format!(
                "You had a maximum of {} ants, with {} spawned.",
                data.max_ants, data.total_ants
            ));
            ui.label("");
            if ui.button("Restart!").clicked() {
                let _ = state.set(GameState::Playing);
            }
        });
}

fn restart_game(mut commands: Commands, time: Res<Time>, mut events: EventWriter<WorldEvents>) {
    commands.insert_resource(AntHill::default());
    commands.insert_resource(FoodDelay::default());
    commands.insert_resource(GraphData::from_anthill(AntHill::default(), &*time));
    commands.insert_resource(VisibleLots::default());
    let duration = Duration::from_secs_f32(19.0);
    let mut new_timer = Timer::new(duration, true);
    new_timer.set_elapsed(duration * 99 / 100);
    commands.insert_resource(FoodTimer(new_timer));
    commands.insert_resource(Bonuses::default());
    events.send(WorldEvents::SpawnFood(true));
}

fn despawn_all(mut commands: Commands, all: Query<Entity>) {
    for entity in all.iter() {
        commands.entity(entity).despawn();
    }
}
