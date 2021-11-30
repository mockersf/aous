use std::{f32::consts::PI, iter, time::Duration};

use bevy::{pbr2::NotShadowCaster, prelude::*};
use rand::Rng;

use crate::{terrain_spawner::ObstacleMap, BORDER, DEF};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoodHandles>()
            .add_event::<WorldEvents>()
            .add_system_to_stage(CoreStage::PostUpdate, remove_empty_heaps)
            .add_system(spawn_food)
            .add_system(pop_food.config(|(_, timer, _)| {
                let duration = Duration::from_secs_f32(15.0);
                let mut new_timer = Timer::new(duration, true);
                new_timer.set_elapsed(duration * 99 / 100);
                *timer = Some(new_timer);
            }));
    }
}

pub struct FoodHandles {
    pub mesh: Handle<bevy::render2::mesh::Mesh>,
    pub color: Handle<bevy::pbr2::StandardMaterial>,
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

        Self { mesh, color }
    }
}

enum WorldEvents {
    SpawnFood,
}

#[derive(Component)]
pub struct FoodPellet {
    pub targeted: bool,
}

#[derive(Component)]
pub struct FoodHeap;

fn spawn_food(
    mut commands: Commands,
    food_handles: Res<FoodHandles>,
    obstacle_map: Res<ObstacleMap>,
    mut events: EventReader<WorldEvents>,
) {
    for event in events.iter() {
        let mut rn = rand::thread_rng();
        match event {
            WorldEvents::SpawnFood => {
                let (x, z) = iter::repeat(())
                    .map(|_| (rn.gen_range(-BORDER..BORDER), rn.gen_range(-BORDER..BORDER)))
                    .find(|(x, z)| !obstacle_map.is_obstacle(*x, *z, 0.0))
                    .unwrap();
                commands
                    .spawn_bundle((
                        Transform::from_xyz(x, 0.0, z),
                        GlobalTransform::default(),
                        FoodHeap,
                    ))
                    .with_children(|heap| {
                        let nb = rn.gen_range(80..100);
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
        }
    }
}

fn pop_food(time: Res<Time>, mut timer: Local<Timer>, mut events: EventWriter<WorldEvents>) {
    if timer.tick(time.delta()).just_finished() {
        for _ in 0..(BORDER as i32).pow(2) {
            events.send(WorldEvents::SpawnFood);
        }
    }
}

fn remove_empty_heaps(
    mut commands: Commands,
    mut heaps: Query<(Entity, &Children), With<FoodHeap>>,
) {
    for (entity, children) in heaps.iter_mut() {
        if children.len() == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
