use std::{collections::VecDeque, f32::consts::FRAC_PI_2, time::Duration};

use bevy::prelude::*;
use rand::Rng;

use crate::ants::{AntHandles, AntState, Creature, CreatureGene};

pub struct AntHillPlugin;

impl Plugin for AntHillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AntHillHandles>()
            .add_event::<HillEvents>()
            .add_startup_system(spawn_ant_hill)
            .insert_resource(AntHill {
                food: 50,
                queen_food: 2,
                gene: CreatureGene {
                    life_expectancy: 30.0,
                    max_speed: 0.25,
                    wander_strength: 0.1,
                },
                gatherer_genes: VecDeque::new(),
            })
            .add_system(hill_events)
            .add_system(use_food)
            .add_system(spawn_ant)
            .add_system(evolve_hills.config(|(_, _, timer)| {
                *timer = Some(Timer::new(Duration::from_secs_f32(30.0), true));
            }));
    }
}

struct AntHillHandles {
    mesh: Handle<bevy::render2::mesh::Mesh>,
    color: Handle<bevy::pbr2::StandardMaterial>,
}

impl FromWorld for AntHillHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap();
        let mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.1,
                ..Default::default()
            },
        ));

        let mut materials = world
            .get_resource_mut::<Assets<bevy::pbr2::StandardMaterial>>()
            .unwrap();
        let color = materials.add(bevy::pbr2::StandardMaterial {
            base_color: bevy::render2::color::Color::rgb(0.545, 0.271, 0.075),
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
    pub gatherer_genes: VecDeque<CreatureGene>,
}

fn spawn_ant_hill(mut commands: Commands, ant_hill_handles: Res<AntHillHandles>) {
    commands.spawn_bundle(bevy::pbr2::PbrBundle {
        mesh: ant_hill_handles.mesh.clone_weak(),
        material: ant_hill_handles.color.clone_weak(),
        transform: Transform::from_xyz(0.0, -0.02, 0.0),
        ..Default::default()
    });
}

pub enum HillEvents {
    SpawnAnts { count: u32 },
    StoreFood(u32, CreatureGene),
    RemoveQueenFood(u32),
    ImproveMaxSpeed(f32),
    ImproveLifeExpectancy(f64),
}

fn use_food(mut hill: ResMut<AntHill>, mut events: EventWriter<HillEvents>) {
    if hill.food >= 10 {
        hill.food -= 10;
        events.send(HillEvents::SpawnAnts { count: 10 });
    }
}

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
) {
    for event in events.iter() {
        match event {
            HillEvents::SpawnAnts { count } => {
                let mut rn = rand::thread_rng();
                for _ in 0..*count {
                    commands
                        .spawn_bundle((Transform::identity(), GlobalTransform::default()))
                        .with_children(|creature| {
                            creature
                                .spawn_bundle(bevy::pbr2::PbrBundle {
                                    mesh: ant_handles.body_mesh.clone_weak(),
                                    material: ant_handles.body_color.clone_weak(),
                                    transform: Transform::from_rotation(Quat::from_rotation_x(
                                        FRAC_PI_2,
                                    )),
                                    ..Default::default()
                                })
                                .insert(bevy::pbr2::NotShadowCaster);
                            creature
                                .spawn_bundle(bevy::pbr2::PbrBundle {
                                    mesh: ant_handles.eye_mesh.clone_weak(),
                                    material: ant_handles.eye_color.clone_weak(),
                                    transform: Transform::from_xyz(0.0075, 0.0075, 0.01875),
                                    ..Default::default()
                                })
                                .insert(bevy::pbr2::NotShadowCaster);
                            creature
                                .spawn_bundle(bevy::pbr2::PbrBundle {
                                    mesh: ant_handles.eye_mesh.clone_weak(),
                                    material: ant_handles.eye_color.clone_weak(),
                                    transform: Transform::from_xyz(-0.0075, 0.0075, 0.01875),
                                    ..Default::default()
                                })
                                .insert(bevy::pbr2::NotShadowCaster);
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
                            },
                        });
                }
            }
            HillEvents::StoreFood(count, gene) => {
                if rand::thread_rng().gen_bool(0.1) {
                    hill.queen_food += count;
                } else {
                    hill.food += count;
                }
                hill.gatherer_genes.push_back(*gene);
                if hill.gatherer_genes.len() > 100 {
                    hill.gatherer_genes.pop_front();
                }
            }
            HillEvents::RemoveQueenFood(consumed) => hill.queen_food -= consumed,
            HillEvents::ImproveMaxSpeed(boost) => hill.gene.max_speed += boost,
            HillEvents::ImproveLifeExpectancy(boost) => hill.gene.life_expectancy += boost,
        }
    }
}

mod mutations {
    pub const MAX_SPEED: f32 = 0.08;
    pub const WANDER_STRENGTH: f32 = 0.02;
    pub const LIFE_EXPECTANCY: f64 = 10.0;
}

fn evolve_hills(mut hill: ResMut<AntHill>, time: Res<Time>, mut timer: Local<Timer>) {
    if timer.tick(time.delta()).just_finished() {
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
                        },
                        count + 1,
                    )
                });
        hill.gene = mean_gene.0;
        info!("current gene: {:?}", hill.gene);
    }
}
