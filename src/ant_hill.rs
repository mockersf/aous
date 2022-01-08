use std::{collections::VecDeque, f32::consts::FRAC_PI_2, time::Duration};

use bevy::prelude::*;
use rand::Rng;

use crate::{
    ants::{AntHandles, AntState, Creature, CreatureGene},
    game_state::GameState,
    ui::GraphData,
};

pub struct AntHillPlugin;

impl Plugin for AntHillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AntHillHandles>()
            .add_event::<HillEvents>()
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_ant_hill))
            .insert_resource(EvolveTimer(Timer::new(Duration::from_secs_f32(30.0), true)))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(hill_events)
                    .with_system(use_food)
                    // used for debugging
                    // .with_system(spawn_ant)
                    .with_system(evolve_hills),
            );
    }
}

struct AntHillHandles {
    mesh: Handle<bevy::render::mesh::Mesh>,
    color: Handle<bevy::pbr::StandardMaterial>,
}

impl FromWorld for AntHillHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render::mesh::Mesh>>()
            .unwrap();
        let mesh = meshes.add(bevy::render::mesh::Mesh::from(
            bevy::render::mesh::shape::Icosphere {
                radius: 0.15,
                ..Default::default()
            },
        ));

        let mut materials = world
            .get_resource_mut::<Assets<bevy::pbr::StandardMaterial>>()
            .unwrap();
        let color = materials.add(bevy::pbr::StandardMaterial {
            base_color: bevy::render::color::Color::rgb(0.545, 0.271, 0.075),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        });

        Self { mesh, color }
    }
}

pub struct AntHill {
    pub food: u32,
    pub queen_food: u32,
    pub gene: CreatureGene,
    pub spawn_per_wave: f32,
    pub mutation_improvement: f32,
    pub gatherer_genes: VecDeque<CreatureGene>,
}

impl Default for AntHill {
    fn default() -> Self {
        AntHill {
            food: 50,
            queen_food: 5,
            gene: CreatureGene {
                life_expectancy: 30.0,
                max_speed: 0.25,
                wander_strength: 0.1,
                antennas: 8.0,
            },
            spawn_per_wave: 10.0,
            mutation_improvement: 0.0,
            gatherer_genes: VecDeque::new(),
        }
    }
}

fn spawn_ant_hill(mut commands: Commands, ant_hill_handles: Res<AntHillHandles>) {
    commands.spawn_bundle(bevy::pbr::PbrBundle {
        mesh: ant_hill_handles.mesh.clone_weak(),
        material: ant_hill_handles.color.clone_weak(),
        transform: Transform::from_xyz(0.0, -0.02, 0.0),
        ..Default::default()
    });
}

pub enum HillEvents {
    SpawnAnts { count: u32 },
    RemoveQueenFood(u32),
    ImproveMaxSpeed(f32),
    ImproveLifeExpectancy(f64),
    ImproveAntennas(f32),
    ImproveWave(f32),
    ImproveMutation(f32),
    ReplenishFood(u32, f64, Option<CreatureGene>),
}

fn use_food(mut hill: ResMut<AntHill>, mut events: EventWriter<HillEvents>) {
    if hill.food >= 10 {
        hill.food -= 10;
        events.send(HillEvents::SpawnAnts {
            count: hill.spawn_per_wave as u32,
        });
    }
}

#[allow(dead_code)]
fn spawn_ant(keyboard_input: Res<Input<KeyCode>>, mut events: EventWriter<HillEvents>) {
    if keyboard_input.pressed(KeyCode::Space) {
        events.send(HillEvents::SpawnAnts { count: 1 });
    }
}

fn hill_events(
    mut commands: Commands,
    ant_handles: Res<AntHandles>,
    mut hill: ResMut<AntHill>,
    mut events: EventReader<HillEvents>,
    time: Res<Time>,
    mut data: ResMut<GraphData>,
) {
    for event in events.iter() {
        match event {
            HillEvents::SpawnAnts { count } => {
                data.total_ants += count;
                let mut rn = rand::thread_rng();
                for _ in 0..*count {
                    commands
                        .spawn_bundle((Transform::identity(), GlobalTransform::default()))
                        .with_children(|creature| {
                            creature
                                .spawn_bundle(bevy::pbr::PbrBundle {
                                    mesh: ant_handles.body_mesh.clone_weak(),
                                    material: ant_handles.body_color.clone_weak(),
                                    transform: Transform::from_rotation(Quat::from_rotation_x(
                                        FRAC_PI_2,
                                    )),
                                    ..Default::default()
                                })
                                .insert(bevy::pbr::NotShadowCaster);
                            creature
                                .spawn_bundle(bevy::pbr::PbrBundle {
                                    mesh: ant_handles.eye_mesh.clone_weak(),
                                    material: ant_handles.eye_color.clone_weak(),
                                    transform: Transform::from_xyz(0.0075, 0.0075, 0.01875),
                                    ..Default::default()
                                })
                                .insert(bevy::pbr::NotShadowCaster);
                            creature
                                .spawn_bundle(bevy::pbr::PbrBundle {
                                    mesh: ant_handles.eye_mesh.clone_weak(),
                                    material: ant_handles.eye_color.clone_weak(),
                                    transform: Transform::from_xyz(-0.0075, 0.0075, 0.01875),
                                    ..Default::default()
                                })
                                .insert(bevy::pbr::NotShadowCaster);
                        })
                        .insert(Creature {
                            velocity: Vec3::ZERO,
                            desired_direction: Vec3::ZERO,
                            wander_strength: hill.gene.wander_strength,
                            state: AntState::Wander,
                            birth: time.seconds_since_startup(),
                            gene: CreatureGene {
                                life_expectancy: hill.gene.life_expectancy
                                    + rn.gen_range(
                                        -mutations::LIFE_EXPECTANCY..mutations::LIFE_EXPECTANCY,
                                    ) / 2.0,
                                max_speed: hill.gene.max_speed
                                    + rn.gen_range(-mutations::MAX_SPEED..mutations::MAX_SPEED)
                                        / 2.0,
                                wander_strength: hill.gene.wander_strength
                                    + rn.gen_range(
                                        -mutations::WANDER_STRENGTH..mutations::WANDER_STRENGTH,
                                    ) / 2.0,
                                antennas: hill.gene.antennas
                                    + rn.gen_range(-mutations::ANTENNAS..mutations::ANTENNAS) / 2.0,
                            },
                        });
                }
            }
            HillEvents::RemoveQueenFood(consumed) => hill.queen_food -= consumed,
            HillEvents::ImproveMaxSpeed(boost) => {
                hill.gene.max_speed = (hill.gene.max_speed + boost).max(0.15)
            }
            HillEvents::ImproveLifeExpectancy(boost) => {
                hill.gene.life_expectancy = (hill.gene.life_expectancy + boost).max(10.0)
            }
            HillEvents::ImproveAntennas(boost) => {
                hill.gene.antennas = (hill.gene.antennas + boost).max(3.0)
            }
            HillEvents::ImproveWave(boost) => hill.spawn_per_wave += boost,
            HillEvents::ImproveMutation(boost) => hill.mutation_improvement += boost,
            HillEvents::ReplenishFood(count, ratio, gene) => {
                for _ in 0..*count {
                    if rand::thread_rng().gen_bool(*ratio) {
                        hill.queen_food += 1;
                    } else {
                        hill.food += 1;
                    }
                }
                if let Some(gene) = gene {
                    hill.gatherer_genes.push_back(*gene);
                    if hill.gatherer_genes.len() > 100 {
                        hill.gatherer_genes.pop_front();
                    }
                }
            }
        }
    }
}

mod mutations {
    pub const MAX_SPEED: f32 = 0.08;
    pub const WANDER_STRENGTH: f32 = 0.02;
    pub const LIFE_EXPECTANCY: f64 = 10.0;
    pub const ANTENNAS: f32 = 3.0;
}

pub struct EvolveTimer(pub Timer);

fn evolve_hills(mut hill: ResMut<AntHill>, time: Res<Time>, mut timer: ResMut<EvolveTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        let mean_gene =
            hill.gatherer_genes
                .iter()
                .fold((hill.gene, 1), |(current, count), gene| {
                    (
                        CreatureGene {
                            life_expectancy: (current.life_expectancy * count as f64
                                + gene.life_expectancy)
                                / (count + 1) as f64,
                            max_speed: (current.max_speed * count as f32 + gene.max_speed)
                                / (count + 1) as f32,
                            wander_strength: (current.wander_strength * count as f32
                                + gene.wander_strength)
                                / (count + 1) as f32,
                            antennas: (current.antennas * count as f32 + gene.antennas)
                                / (count + 1) as f32,
                        },
                        count + 1,
                    )
                });
        hill.gene = CreatureGene {
            life_expectancy: mean_gene.0.life_expectancy + hill.mutation_improvement as f64,
            max_speed: mean_gene.0.max_speed + hill.mutation_improvement / 100.0,
            wander_strength: mean_gene.0.wander_strength,
            antennas: mean_gene.0.antennas + hill.mutation_improvement / 10.0,
        };
        info!("current gene: {:?}", hill.gene);
    }
}
