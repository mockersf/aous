use std::time::Duration;

use bevy::{
    core::{Time, Timer},
    prelude::{Commands, Entity, Plugin, Query, Res, ResMut, State, SystemSet},
};
use bevy_egui::{egui, EguiContext};

use crate::{
    ant_hill::AntHill,
    camera::VisibleLots,
    food::{FoodDelay, FoodTimer},
    ui::{Bonuses, GraphData},
};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Splash,
    Playing,
    Lost,
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(GameState::Splash)
            .add_system_set(SystemSet::on_update(GameState::Lost).with_system(lost_stats))
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(restart_game))
            .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(despawn_all));
    }
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
            ui.label(format!("You had a maximum of {} ants ", data.max_ants));
            ui.label("");
            if ui.button("Restart!").clicked() {
                let _ = state.set(GameState::Playing);
            }
        });
}

fn restart_game(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(AntHill::default());
    commands.insert_resource(FoodDelay::default());
    commands.insert_resource(GraphData::from_anthill(AntHill::default(), &*time));
    commands.insert_resource(VisibleLots::default());
    let duration = Duration::from_secs_f32(20.0);
    let mut new_timer = Timer::new(duration, true);
    new_timer.set_elapsed(duration * 99 / 100);
    commands.insert_resource(FoodTimer(new_timer));
    commands.insert_resource(Bonuses::default())
}

fn despawn_all(mut commands: Commands, all: Query<Entity>) {
    for entity in all.iter() {
        commands.entity(entity).despawn();
    }
}
