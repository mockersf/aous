use std::{f32::consts::PI, time::Duration};

use bevy::{pbr2::NotShadowCaster, prelude::*};
use rand::Rng;

use crate::{BORDER, DEF};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoodHandles>()
            .add_system(spawn_food.config(|(_, _, _, timer)| {
                let duration = Duration::from_secs_f32(5.0);
                let mut new_timer = Timer::new(duration, true);
                new_timer.set_elapsed(duration * 95 / 100);
                *timer = Some(new_timer);
            }));
    }
}

struct FoodHandles {
    mesh: Handle<bevy::render2::mesh::Mesh>,
    color: Handle<bevy::pbr2::StandardMaterial>,
}

impl FromWorld for FoodHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap();
        let mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.015,
                ..Default::default()
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

#[derive(Component)]
pub struct FoodPellet;

#[derive(Component)]
pub struct FoodHeap;

fn spawn_food(
    mut commands: Commands,
    food_handles: Res<FoodHandles>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.tick(time.delta()).just_finished() {
        let mut rn = rand::thread_rng();
        commands
            .spawn_bundle((
                Transform::from_xyz(
                    rn.gen_range(-BORDER..BORDER),
                    0.0,
                    rn.gen_range(-BORDER..BORDER),
                ),
                GlobalTransform::default(),
                FoodHeap,
            ))
            .with_children(|heap| {
                for _ in 0..rn.gen_range(80..100) {
                    let transform = Transform::from_translation(
                        Quat::from_rotation_y(rn.gen_range(0.0..(2.0 * PI)))
                            .mul_vec3(Vec3::X * rn.gen_range(0.0..(1.0 / DEF * 2.0))),
                    );
                    heap.spawn_bundle(bevy::pbr2::PbrBundle {
                        mesh: food_handles.mesh.clone_weak(),
                        material: food_handles.color.clone_weak(),
                        transform,
                        ..Default::default()
                    })
                    .insert_bundle((FoodPellet, NotShadowCaster));
                }
            });
    }
}
