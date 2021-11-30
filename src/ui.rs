use std::{collections::VecDeque, time::Duration};

use bevy::{
    core::{Time, Timer},
    prelude::{
        debug, ConfigurableSystem, EventWriter, Local, Plugin, Query, Res, ResMut, State, SystemSet,
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
    ant_hill::{AntHill, EvolveTimer, HillEvents},
    ants::Creature,
    game_state::GameState,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Bonuses {
            spawn_cost: 10,
            spawn: 15,
            improve_speed_cost: 5,
            improve_speed: 0.002,
            improve_life_cost: 5,
            improve_life: 3.0,
        })
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(overall_ui)
                .with_system(update_graph_data.config(|(_, _, timer, _, _, _)| {
                    let duration = Duration::from_secs_f32(1.0);
                    let mut new_timer = Timer::new(duration, true);
                    new_timer.set_elapsed(duration * 99 / 100);
                    *timer = Some(new_timer);
                })),
        );
    }
}

pub struct GraphData {
    nb_ants: VecDeque<u32>,
    queen_food: u32,
    food: u32,
    genome_speed: f32,
    genome_expectancy: f64,
    pub max_ants: u32,
    pub start_time: Duration,
    pub end_time: Duration,
}
impl GraphData {
    pub fn from_anthill(anthill: AntHill, time: &Time) -> Self {
        let mut nb_ants = VecDeque::new();
        nb_ants.extend([0; HISTORY_SIZE]);
        let queen_food = 0;
        let genome_speed = anthill.gene.max_speed;
        let genome_expectancy = anthill.gene.life_expectancy;
        let food = anthill.food;
        Self {
            nb_ants,
            queen_food,
            food,
            genome_speed,
            genome_expectancy,
            max_ants: 0,
            start_time: time.time_since_startup(),
            end_time: time.time_since_startup(),
        }
    }
}

struct Bonuses {
    spawn_cost: u32,
    spawn: u32,
    improve_speed_cost: u32,
    improve_speed: f32,
    improve_life_cost: u32,
    improve_life: f64,
}

fn update_graph_data(
    creatures: Query<&Creature>,
    mut data: ResMut<GraphData>,
    mut timer: Local<Timer>,
    time: Res<Time>,
    anthill: Res<AntHill>,
    mut state: ResMut<State<GameState>>,
) {
    if timer.tick(time.delta()).just_finished() {
        let creature_count = creatures.iter().len() as u32;
        if data.max_ants > 0 && creature_count == 0 {
            data.end_time = time.time_since_startup();
            state.set(GameState::Lost).unwrap();
        }
        if creature_count > data.max_ants {
            data.max_ants = creature_count;
        }
        data.nb_ants.push_back(creature_count);
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
    mut bonuses: ResMut<Bonuses>,
    mut events: EventWriter<HillEvents>,
    evolve_timer: Res<EvolveTimer>,
) {
    egui::SidePanel::left("left-panel")
        .resizable(false)
        .show(egui_context.ctx(), |ui| {
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
                ui.add(ProgressBar::new(evolve_timer.0.percent()).text("Mutate"));
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
                        if data.queen_food < bonuses.spawn_cost {
                            ui.add(
                                Button::new(format!("Spawn {} ants", bonuses.spawn)).enabled(false),
                            );
                        } else if ui.button(format!("Spawn {} ants", bonuses.spawn)).clicked() {
                            debug!("spawn ants!");
                            events.send(HillEvents::SpawnAnts {
                                count: bonuses.spawn,
                            });
                            events.send(HillEvents::RemoveQueenFood(bonuses.spawn_cost));
                        }
                        ui.label(format!("{}", bonuses.spawn_cost));
                        ui.end_row();
                        if data.queen_food < bonuses.improve_speed_cost {
                            ui.add(Button::new("Improve speed").enabled(false));
                        } else if ui.button("Improve speed").clicked() {
                            debug!("Improve speed!");
                            events.send(HillEvents::ImproveMaxSpeed(bonuses.improve_speed));
                            events.send(HillEvents::RemoveQueenFood(bonuses.improve_speed_cost));
                            bonuses.improve_speed_cost += 5;
                        }
                        ui.label(format!("{}", bonuses.improve_speed_cost));
                        ui.end_row();
                        if data.queen_food < bonuses.improve_life_cost {
                            ui.add(Button::new("Improve life expectancy").enabled(false));
                        } else if ui.button("Improve life expectancy").clicked() {
                            debug!("Improve life expectancy!");
                            events.send(HillEvents::ImproveLifeExpectancy(bonuses.improve_life));
                            events.send(HillEvents::RemoveQueenFood(bonuses.improve_life_cost));
                            bonuses.improve_life_cost += 5;
                        }
                        ui.label(format!("{}", bonuses.improve_life_cost));
                        ui.end_row();
                    });
            });
        });
}
