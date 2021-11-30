use std::{collections::VecDeque, time::Duration};

use bevy::{
    core::{Time, Timer},
    prelude::{ConfigurableSystem, FromWorld, Local, Plugin, Query, Res, ResMut},
};
use bevy_egui::{
    egui::{
        self,
        plot::{Line, Plot, Value, Values},
    },
    EguiContext,
};

use crate::{ant_hill::AntHill, ants::Creature};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<GraphData>()
            .add_system(overall_ui)
            .add_system(update_graph_data.config(|(_, _, timer, _, _)| {
                let duration = Duration::from_secs_f32(1.0);
                let mut new_timer = Timer::new(duration, true);
                new_timer.set_elapsed(duration * 99 / 100);
                *timer = Some(new_timer);
            }));
    }
}

struct GraphData {
    nb_ants: VecDeque<u32>,
    queen_food: VecDeque<u32>,
    genome_speed: f32,
    genome_expectancy: f64,
}
impl FromWorld for GraphData {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut nb_ants = VecDeque::new();
        nb_ants.extend([0; 60]);
        let mut queen_food = VecDeque::new();
        queen_food.extend([0; 60]);
        let base_food_value = world
            .get_resource::<AntHill>()
            .map(|ah| ah.food)
            .unwrap_or(0);
        let mut food = VecDeque::new();
        food.extend([base_food_value; 60]);
        let anthill = world.get_resource::<AntHill>().unwrap();
        let genome_speed = anthill.gene.max_speed;
        let genome_expectancy = anthill.gene.life_expectancy;
        Self {
            nb_ants,
            queen_food,
            genome_speed,
            genome_expectancy,
        }
    }
}

fn update_graph_data(
    creatures: Query<&Creature>,
    mut data: ResMut<GraphData>,
    mut timer: Local<Timer>,
    time: Res<Time>,
    anthill: Res<AntHill>,
) {
    if timer.tick(time.delta()).just_finished() {
        let creature_count = creatures.iter().len();
        data.nb_ants.push_back(creature_count as u32);
        if data.nb_ants.len() > 60 {
            data.nb_ants.pop_front();
        }
        data.queen_food.push_back(anthill.queen_food);
        if data.queen_food.len() > 60 {
            data.queen_food.pop_front();
        }
        data.genome_speed = anthill.gene.max_speed;
        data.genome_expectancy = anthill.gene.life_expectancy;
    }
}

fn overall_ui(egui_context: ResMut<EguiContext>, data: Res<GraphData>) {
    egui::TopBottomPanel::top("top-panel").show(egui_context.ctx(), |ui| {
        ui.label("Feed them all");
    });
    egui::SidePanel::left("left-panel").show(egui_context.ctx(), |ui| {
        ui.label("");
        ui.label(format!("Ants: {}", data.nb_ants.back().unwrap_or(&0)));
        ui.add(
            Plot::new("ant count")
                .height(150.0)
                .allow_zoom(false)
                .allow_drag(false)
                .show_axes([false, true])
                .line(Line::new(Values::from_values_iter(
                    data.nb_ants
                        .iter()
                        .enumerate()
                        .map(|(i, c)| Value::new(i as f64, (*c) as f32)),
                ))),
        );
        ui.label("");
        ui.group(|ui| {
            ui.label("Genome");
            ui.separator();
            ui.label(format!("Max Speed: {}", data.genome_speed));
            ui.label(format!("Life Expectancy: {}", data.genome_expectancy));
        });
        ui.label("");
        ui.group(|ui| {
            ui.label(format!(
                "Queen Food Storage: {}",
                data.queen_food.back().unwrap_or(&0)
            ));
            ui.add(
                Plot::new("queen food store")
                    .height(100.0)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .show_axes([false, true])
                    .line(Line::new(Values::from_values_iter(
                        data.queen_food
                            .iter()
                            .enumerate()
                            .map(|(i, c)| Value::new(i as f64, (*c) as f32)),
                    ))),
            );
        });
        ui.label("");
        ui.group(|ui| {
            ui.label("Actions");
            ui.separator();
            ui.button("aaa");
            ui.label(format!("Max Speed: {}", data.genome_speed));
            ui.label(format!("Life Expectancy: {}", data.genome_expectancy));
        });
    });
}
