use std::{collections::VecDeque, time::Duration};

use bevy::{
    core::{Time, Timer},
    prelude::{
        debug, ConfigurableSystem, EventWriter, FromWorld, Local, Plugin, Query, Res, ResMut,
    },
};
use bevy_egui::{
    egui::{
        self,
        plot::{Line, Plot, Value, Values},
        Button, ProgressBar,
    },
    EguiContext,
};

const HISTORY_SIZE: usize = 240;

use crate::{
    ant_hill::{AntHill, HillEvents},
    ants::Creature,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<GraphData>()
            .insert_resource(Costs {
                spawn: 10,
                improve_speed: 5,
                improve_life: 5,
            })
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
    queen_food: u32,
    food: u32,
    genome_speed: f32,
    genome_expectancy: f64,
}
impl FromWorld for GraphData {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut nb_ants = VecDeque::new();
        nb_ants.extend([0; HISTORY_SIZE]);
        let queen_food = 0;
        let anthill = world.get_resource::<AntHill>().unwrap();
        let genome_speed = anthill.gene.max_speed;
        let genome_expectancy = anthill.gene.life_expectancy;
        let food = anthill.food;
        Self {
            nb_ants,
            queen_food,
            food,
            genome_speed,
            genome_expectancy,
        }
    }
}

struct Costs {
    spawn: u32,
    improve_speed: u32,
    improve_life: u32,
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
        if data.nb_ants.len() > HISTORY_SIZE {
            data.nb_ants.pop_front();
        }
    }
    data.queen_food = anthill.queen_food;
    data.genome_speed = anthill.gene.max_speed;
    data.genome_expectancy = anthill.gene.life_expectancy;
    data.food = anthill.food;
}

fn overall_ui(
    egui_context: ResMut<EguiContext>,
    data: Res<GraphData>,
    mut costs: ResMut<Costs>,
    mut events: EventWriter<HillEvents>,
) {
    egui::SidePanel::left("left-panel").show(egui_context.ctx(), |ui| {
        ui.label("");
        ui.group(|ui| {
            egui::Grid::new("Ants_grid")
                .num_columns(2)
                .spacing([30.0, 0.0])
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Ants");
                    ui.label(format!("{}", data.nb_ants.back().unwrap_or(&0)));
                    ui.end_row();
                });
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
            ui.add(ProgressBar::new(data.food as f32 / 10.0).text("Spawn new ants"));
        });
        ui.label("");
        ui.group(|ui| {
            ui.label("Genome");
            ui.separator();
            egui::Grid::new("genome_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Max Speed");
                    ui.label(format!("{:.2}", data.genome_speed * 100.0));
                    ui.end_row();
                    ui.label("Life Expectancy");
                    ui.label(format!("{:.2}", data.genome_expectancy));
                    ui.end_row();
                });
        });
        ui.label("");
        ui.group(|ui| {
            ui.label("Actions");
            ui.separator();
            egui::Grid::new("actions_grid")
                .num_columns(2)
                .spacing([10.0, 0.0])
                .striped(false)
                .min_row_height(25.0)
                .show(ui, |ui| {
                    ui.label("Available");
                    ui.label(format!("{}", data.queen_food));
                    ui.end_row();
                    if data.queen_food < costs.spawn {
                        ui.add(Button::new("Spawn 12 ants").enabled(false));
                    } else if ui.button("Spawn 12 ants").clicked() {
                        debug!("spawn ants!");
                        events.send(HillEvents::SpawnAnts { count: 12 });
                        events.send(HillEvents::RemoveQueenFood(costs.spawn));
                    }
                    ui.label(format!("{}", costs.spawn));
                    ui.end_row();
                    if data.queen_food < costs.improve_speed {
                        ui.add(Button::new("Improve speed").enabled(false));
                    } else if ui.button("Improve speed").clicked() {
                        debug!("Improve speed!");
                        events.send(HillEvents::ImproveMaxSpeed(0.01));
                        events.send(HillEvents::RemoveQueenFood(costs.improve_speed));
                        costs.improve_speed += 5;
                    }
                    ui.label(format!("{}", costs.improve_speed));
                    ui.end_row();
                    if data.queen_food < costs.improve_life {
                        ui.add(Button::new("Improve life expectancy").enabled(false));
                    } else if ui.button("Improve life expectancy").clicked() {
                        debug!("Improve life expectancy!");
                        events.send(HillEvents::ImproveLifeExpectancy(5.0));
                        events.send(HillEvents::RemoveQueenFood(costs.improve_life));
                        costs.improve_life += 5;
                    }
                    ui.label(format!("{}", costs.improve_life));
                    ui.end_row();
                });
        });
    });
}
