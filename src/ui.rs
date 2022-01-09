use std::{collections::VecDeque, time::Duration};

use bevy::{
    core::{Time, Timer},
    math::Vec3,
    prelude::{
        ConfigurableSystem, EventWriter, Local, Or, Plugin, Query, Res, ResMut, State, SystemSet,
        With,
    },
};
use bevy_egui::{
    egui::{
        self,
        plot::{Line, Plot, Value, Values},
        ProgressBar,
    },
    EguiContext,
};
use rand::Rng;

const HISTORY_SIZE: usize = 240;

use crate::{
    ant_eaters::AntEater,
    ant_hill::{AntHill, EvolveTimer, HillEvents},
    ants::Creature,
    food::{FoodPellet, WorldEvents},
    game_state::GameState,
    BORDER,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(overall_ui)
                .with_system(update_graph_data.config(|(_, _, _, timer, _, _, _, _, _)| {
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
    genome_antennas: f32,
    wave: f32,
    pub max_ants: u32,
    pub total_ants: u32,
    pub start_time: Duration,
    pub end_time: Duration,
    can_summon_food: bool,
    appocalypse: bool,
}
impl GraphData {
    pub fn from_anthill(anthill: AntHill, time: &Time) -> Self {
        let mut nb_ants = VecDeque::new();
        nb_ants.extend([0; HISTORY_SIZE]);
        let queen_food = 0;
        let genome_speed = anthill.gene.max_speed;
        let genome_expectancy = anthill.gene.life_expectancy;
        let genome_antennas = anthill.gene.antennas;
        let wave = anthill.spawn_per_wave;
        let food = anthill.food;
        Self {
            nb_ants,
            queen_food,
            food,
            genome_speed,
            genome_expectancy,
            genome_antennas,
            wave,
            max_ants: 0,
            total_ants: 0,
            start_time: time.time_since_startup(),
            end_time: time.time_since_startup(),
            can_summon_food: false,
            appocalypse: false,
        }
    }
}

pub struct Bonuses {
    spawn_cost: u32,
    spawn: u32,
    improve_wave_cost: u32,
    improve_wave: f32,
    improve_speed_cost: u32,
    improve_speed: f32,
    improve_life_cost: u32,
    improve_life: f64,
    improve_antennas_cost: u32,
    improve_antennas: f32,
    improve_mutation_cost: u32,
    improve_mutation: f32,
}
impl Default for Bonuses {
    fn default() -> Self {
        Bonuses {
            spawn_cost: 10,
            spawn: 15,
            improve_wave_cost: 10,
            improve_wave: 1.75,
            improve_speed_cost: 5,
            improve_speed: 0.005,
            improve_life_cost: 5,
            improve_life: 5.0,
            improve_antennas_cost: 5,
            improve_antennas: 1.0,
            improve_mutation_cost: 15,
            improve_mutation: 0.6,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_graph_data(
    creatures: Query<&Creature>,
    todo: Query<(), Or<(With<FoodPellet>, With<AntEater>)>>,
    mut data: ResMut<GraphData>,
    mut timer: Local<Timer>,
    time: Res<Time>,
    anthill: Res<AntHill>,
    mut state: ResMut<State<GameState>>,
    mut events: EventWriter<WorldEvents>,
    mut corner: Local<u8>,
) {
    if timer.tick(time.delta()).just_finished() {
        let creature_count = creatures.iter().len() as u32;
        if data.max_ants > 0 && creature_count == 0 {
            data.end_time = time.time_since_startup();
            state.set(GameState::Lost).unwrap();
        } else if anthill.queen_food >= 200
            || (creature_count > 100 && todo.iter().next().is_none())
        {
            data.end_time = time.time_since_startup();
            state.set(GameState::Won).unwrap();
        }
        if creature_count > data.max_ants {
            data.max_ants = creature_count;
        }
        data.nb_ants.push_back(creature_count);
        if data.nb_ants.len() > HISTORY_SIZE {
            data.nb_ants.pop_front();
        }
        if !data.can_summon_food && rand::thread_rng().gen_bool(0.005) {
            data.can_summon_food = true;
        }
        if !data.appocalypse
            && (time.time_since_startup() - data.start_time > Duration::from_secs(540)
                || creature_count > 1500
                || data.total_ants > 6000)
        {
            data.appocalypse = true;
        }
        if data.appocalypse {
            *corner += 1;
            events.send(WorldEvents::SpawnAntEater(match *corner {
                1 => Vec3::new(BORDER, 0.0, BORDER),
                2 => Vec3::new(BORDER, 0.0, BORDER / 2.0),
                3 => Vec3::new(BORDER, 0.0, 0.0),
                4 => Vec3::new(BORDER, 0.0, -BORDER / 2.0),
                5 => Vec3::new(BORDER, 0.0, -BORDER),
                6 => Vec3::new(BORDER / 2.0, 0.0, -BORDER),
                7 => Vec3::new(0.0, 0.0, -BORDER),
                8 => Vec3::new(-BORDER / 2.0, 0.0, -BORDER),
                9 => Vec3::new(-BORDER, 0.0, -BORDER),
                10 => Vec3::new(-BORDER, 0.0, -BORDER / 2.0),
                11 => Vec3::new(-BORDER, 0.0, 0.0),
                12 => Vec3::new(-BORDER, 0.0, BORDER / 2.0),
                13 => Vec3::new(-BORDER, 0.0, BORDER),
                14 => Vec3::new(-BORDER / 2.0, 0.0, BORDER),
                15 => Vec3::new(0.0, 0.0, BORDER),
                _ => {
                    *corner = 0;
                    Vec3::new(BORDER / 2.0, 0.0, BORDER)
                }
            }));
        }
    }
    data.queen_food = anthill.queen_food;
    data.genome_speed = anthill.gene.max_speed;
    data.genome_expectancy = anthill.gene.life_expectancy;
    data.genome_antennas = anthill.gene.antennas;
    data.food = anthill.food;
    data.wave = anthill.spawn_per_wave;
}

fn overall_ui(
    egui_context: ResMut<EguiContext>,
    mut data: ResMut<GraphData>,
    mut bonuses: ResMut<Bonuses>,
    mut events: EventWriter<HillEvents>,
    mut world_events: EventWriter<WorldEvents>,
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
                Plot::new("ant count")
                    .height(150.0)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .show_axes([false, true])
                    .show(ui, |ui| {
                        ui.line(Line::new(Values::from_values_iter(
                            data.nb_ants
                                .iter()
                                .enumerate()
                                .map(|(i, c)| Value::new(i as f64, (*c) as f32)),
                        )))
                    });
                ui.add(
                    ProgressBar::new(data.food as f32 / 10.0)
                        .text(format!("Spawn {} new ants", data.wave as u32)),
                );
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
                        ui.label("Food Sensibility");
                        ui.label(format!("{:.2}", data.genome_antennas));
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
                        ui.scope(|ui| {
                            if data.queen_food < bonuses.spawn_cost {
                                ui.set_enabled(false);
                            }
                            if ui.button(format!("Spawn {} Ants", bonuses.spawn)).clicked() {
                                events.send(HillEvents::SpawnAnts {
                                    count: bonuses.spawn,
                                });
                                events.send(HillEvents::RemoveQueenFood(bonuses.spawn_cost));
                                bonuses.spawn += 1;
                                bonuses.spawn_cost += 1;
                            }
                        });
                        ui.label(format!("{}", bonuses.spawn_cost));
                        ui.end_row();
                        ui.scope(|ui| {
                            if data.queen_food < bonuses.improve_wave_cost {
                                ui.set_enabled(false);
                            }
                            if ui.button("Improve Ant Spawning").clicked() {
                                events.send(HillEvents::ImproveWave(bonuses.improve_wave));
                                events.send(HillEvents::RemoveQueenFood(bonuses.improve_wave_cost));
                                bonuses.spawn += 2;
                                bonuses.improve_wave_cost += 15;
                            }
                        });
                        ui.label(format!("{}", bonuses.improve_wave_cost));
                        ui.end_row();
                        ui.scope(|ui| {
                            if data.queen_food < bonuses.improve_speed_cost {
                                ui.set_enabled(false);
                            }
                            if ui.button("Improve Speed").clicked() {
                                events.send(HillEvents::ImproveMaxSpeed(bonuses.improve_speed));
                                events
                                    .send(HillEvents::RemoveQueenFood(bonuses.improve_speed_cost));
                                bonuses.improve_speed_cost += 5;
                                bonuses.improve_speed += 0.002;
                            }
                        });

                        ui.label(format!("{}", bonuses.improve_speed_cost));
                        ui.end_row();
                        ui.scope(|ui| {
                            if data.queen_food < bonuses.improve_life_cost {
                                ui.set_enabled(false);
                            }
                            if ui.button("Improve Life Expectancy").clicked() {
                                events
                                    .send(HillEvents::ImproveLifeExpectancy(bonuses.improve_life));
                                events.send(HillEvents::RemoveQueenFood(bonuses.improve_life_cost));
                                bonuses.improve_life_cost += 5;
                                bonuses.improve_life += 2.0;
                            }
                        });
                        ui.label(format!("{}", bonuses.improve_life_cost));
                        ui.end_row();
                        ui.scope(|ui| {
                            if data.queen_food < bonuses.improve_antennas_cost {
                                ui.set_enabled(false);
                            }
                            if ui.button("Improve Food Sensibility").clicked() {
                                events.send(HillEvents::ImproveAntennas(bonuses.improve_antennas));
                                events.send(HillEvents::RemoveQueenFood(
                                    bonuses.improve_antennas_cost,
                                ));
                                bonuses.improve_antennas_cost += 5;
                                bonuses.improve_antennas += 1.0;
                            }
                        });
                        ui.label(format!("{}", bonuses.improve_antennas_cost));
                        ui.end_row();
                        ui.scope(|ui| {
                            if data.queen_food < bonuses.improve_mutation_cost {
                                ui.set_enabled(false);
                            }
                            if ui.button("Improve Mutations").clicked() {
                                events.send(HillEvents::ImproveMutation(bonuses.improve_mutation));
                                events.send(HillEvents::RemoveQueenFood(
                                    bonuses.improve_mutation_cost,
                                ));
                                bonuses.improve_mutation_cost += 15;
                                bonuses.improve_mutation += 0.15;
                            }
                        });
                        ui.label(format!("{}", bonuses.improve_mutation_cost));
                        ui.end_row();
                        if !data.can_summon_food {
                            ui.label("");
                        } else if ui.button("Create Food").clicked() {
                            world_events.send(WorldEvents::SpawnFood(true));
                            world_events
                                .send(WorldEvents::SpawnAntEater(Vec3::new(BORDER, 0.0, BORDER)));
                            world_events
                                .send(WorldEvents::SpawnAntEater(Vec3::new(BORDER, 0.0, -BORDER)));
                            world_events
                                .send(WorldEvents::SpawnAntEater(Vec3::new(-BORDER, 0.0, BORDER)));
                            world_events
                                .send(WorldEvents::SpawnAntEater(Vec3::new(-BORDER, 0.0, -BORDER)));
                            data.can_summon_food = false
                        }
                        ui.end_row();
                    });
            });
            if data.appocalypse {
                ui.label("");
                ui.label("You colony has been found!");
            }
        });
}
