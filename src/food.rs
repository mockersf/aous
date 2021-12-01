use std::{f32::consts::PI, iter, time::Duration};

use bevy::{pbr2::NotShadowCaster, prelude::*};
use rand::Rng;

use crate::{game_state::GameState, terrain_spawner::ObstacleMap, BORDER, DEF};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoodHandles>()
            .add_event::<WorldEvents>()
            .add_system_to_stage(CoreStage::PostUpdate, remove_empty_heaps)
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(spawn_food)
                    .with_system(food_gone_bad)
                    .with_system(enter_the_anteater)
                    .with_system(pop_food)
                    .with_system(faster_decay.config(|(_, timer, _)| {
                        *timer = Some(Timer::new(Duration::from_secs_f32(30.0), true));
                    })),
            );
    }
}

pub struct FoodHandles {
    pub mesh: Handle<bevy::render2::mesh::Mesh>,
    pub color: Handle<bevy::pbr2::StandardMaterial>,
    pub warning: Handle<bevy::render2::texture::Image>,
    pub warning_material: Handle<bevy::pbr2::StandardMaterial>,
    pub warning_mesh: Handle<bevy::render2::mesh::Mesh>,
}

pub struct FoodDelay {
    gone_bad: f32,
    summon_anteater: f32,
}
impl Default for FoodDelay {
    fn default() -> Self {
        FoodDelay {
            gone_bad: 45.0,
            summon_anteater: 15.0,
        }
    }
}

impl FromWorld for FoodHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap();
        let mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.015,
                subdivisions: 1,
            },
        ));

        let mut materials = world
            .get_resource_mut::<Assets<bevy::pbr2::StandardMaterial>>()
            .unwrap();
        let color = materials.add(bevy::pbr2::StandardMaterial {
            base_color: bevy::render2::color::Color::BLUE,
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        });

        let warning = world
            .get_resource::<AssetServer>()
            .unwrap()
            .load("hazard-sign.png");

        let warning_material = world
            .get_resource_mut::<Assets<bevy::pbr2::StandardMaterial>>()
            .unwrap()
            .add(bevy::pbr2::StandardMaterial {
                base_color: bevy::render2::color::Color::rgba(1.0, 0.0, 0.0, 0.3),
                base_color_texture: Some(warning.clone()),
                unlit: true,
                alpha_mode: bevy::pbr2::AlphaMode::Blend,
                ..Default::default()
            });

        let warning_mesh = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap()
            .add(bevy::render2::mesh::Mesh::from(
                bevy::render2::mesh::shape::Quad::new(Vec2::new(0.2, 0.2)),
            ));

        Self {
            mesh,
            color,
            warning,
            warning_material,
            warning_mesh,
        }
    }
}

pub enum WorldEvents {
    SpawnFood(bool),
    SpawnAntEater(Vec3),
}

#[derive(Component)]
pub struct FoodPellet {
    pub targeted: bool,
}

#[derive(Component)]
pub struct FoodHeap {
    start_count: usize,
}

#[derive(Component)]
pub struct FoodGoneBadTimer(Timer);

#[derive(Component)]
pub struct AntEaterTimer(Timer);

fn spawn_food(
    mut commands: Commands,
    food_handles: Res<FoodHandles>,
    obstacle_map: Res<ObstacleMap>,
    mut events: EventReader<WorldEvents>,
    food_delay: Res<FoodDelay>,
) {
    for event in events.iter() {
        let mut rn = rand::thread_rng();
        match event {
            WorldEvents::SpawnFood(is_nearby) => {
                let range = if *is_nearby {
                    0.25
                } else if rn.gen_bool(0.05) {
                    BORDER / 2.0
                } else {
                    BORDER * 10.0 / 11.0
                };
                let (x, z) = iter::repeat(())
                    .map(|_| (rn.gen_range(-range..range), rn.gen_range(-range..range)))
                    .find(|(x, z)| !obstacle_map.is_obstacle(*x, *z, 0.0))
                    .unwrap();
                let nb = rn.gen_range(80..100);
                commands
                    .spawn_bundle((
                        Transform::from_xyz(x, 0.0, z),
                        GlobalTransform::default(),
                        FoodHeap { start_count: nb },
                        FoodGoneBadTimer(Timer::new(
                            Duration::from_secs_f32(food_delay.gone_bad + rn.gen_range(-5.0..5.0)),
                            false,
                        )),
                    ))
                    .with_children(|heap| {
                        iter::repeat(())
                            .map(|_| {
                                Quat::from_rotation_y(rn.gen_range(0.0..(2.0 * PI)))
                                    .mul_vec3(Vec3::X * rn.gen_range(0.0..(1.0 / DEF * 2.0)))
                            })
                            .filter(|pos| !obstacle_map.is_obstacle(x + pos.x, z + pos.z, 0.0))
                            .take(nb)
                            .for_each(|pos| {
                                let transform = Transform::from_translation(pos);
                                heap.spawn_bundle(bevy::pbr2::PbrBundle {
                                    mesh: food_handles.mesh.clone_weak(),
                                    material: food_handles.color.clone_weak(),
                                    transform,
                                    ..Default::default()
                                })
                                .insert_bundle((FoodPellet { targeted: false }, NotShadowCaster));
                            });
                    });
            }
            WorldEvents::SpawnAntEater(_) => (),
        }
    }
}

pub struct FoodTimer(pub Timer);

fn pop_food(time: Res<Time>, mut timer: ResMut<FoodTimer>, mut events: EventWriter<WorldEvents>) {
    if timer.0.tick(time.delta()).just_finished() {
        for _ in 0..(BORDER as i32).pow(2) {
            events.send(WorldEvents::SpawnFood(false));
        }
    }
}

fn remove_empty_heaps(
    mut commands: Commands,
    mut heaps: Query<(Entity, &Children), With<FoodHeap>>,
    is_warning: Query<(), With<Warning>>,
) {
    for (entity, children) in heaps.iter_mut() {
        if children.len() == 0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        if children.len() == 1 && is_warning.get(children[0]).is_ok() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct Warning;

fn food_gone_bad(
    mut commands: Commands,
    mut food_heaps: Query<(Entity, &mut FoodGoneBadTimer, &FoodHeap)>,
    time: Res<Time>,
    food_handles: Res<FoodHandles>,
    food_delay: Res<FoodDelay>,
) {
    for (entity, mut timer, food_heap) in food_heaps.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            commands
                .entity(entity)
                .with_children(|heap| {
                    heap.spawn_bundle((
                        Transform {
                            translation: Vec3::new(0.0, 0.05, 0.0),
                            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                            ..Default::default()
                        },
                        GlobalTransform::default(),
                    ))
                    .with_children(|rotated| {
                        rotated
                            .spawn_bundle(bevy::pbr2::PbrBundle {
                                mesh: food_handles.warning_mesh.clone(),
                                material: food_handles.warning_material.clone(),
                                transform: Transform {
                                    rotation: Quat::from_rotation_z(std::f32::consts::PI),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(NotShadowCaster);
                    })
                    .insert(Warning);
                })
                .insert(AntEaterTimer(Timer::new(
                    Duration::from_secs_f32(
                        food_delay.summon_anteater + 100.0 - food_heap.start_count as f32,
                    ),
                    false,
                )));
        }
    }
}

fn enter_the_anteater(
    mut food_heaps: Query<(&mut AntEaterTimer, &Transform)>,
    time: Res<Time>,
    mut events: EventWriter<WorldEvents>,
) {
    for (mut timer, transform) in food_heaps.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            events.send(WorldEvents::SpawnAntEater(transform.translation));
        }
    }
}

fn faster_decay(time: Res<Time>, mut timer: Local<Timer>, mut food_delay: ResMut<FoodDelay>) {
    if timer.tick(time.delta()).just_finished() {
        food_delay.gone_bad = (food_delay.gone_bad - 5.0).max(10.0);
        food_delay.summon_anteater = (food_delay.summon_anteater - 2.0).max(5.0);
    }
}
