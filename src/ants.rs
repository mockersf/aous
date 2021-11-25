use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use rand::Rng;

use crate::{
    terrain_spawner::{EmptyLot, ObstacleMap},
    DEF,
};

pub struct AntsPlugin;

impl Plugin for AntsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AntHandles>()
            .add_system(spawn_ant)
            .add_system(move_ants);
    }
}

struct AntHandles {
    body_mesh: Handle<bevy::render2::mesh::Mesh>,
    body_color: Handle<bevy::pbr2::StandardMaterial>,
    eye_mesh: Handle<bevy::render2::mesh::Mesh>,
    eye_color: Handle<bevy::pbr2::StandardMaterial>,
}

impl FromWorld for AntHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap();
        let body_mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Capsule {
                radius: 0.015,
                depth: 0.015,
                ..Default::default()
            },
        ));
        let eye_mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.008,
                subdivisions: 5,
            },
        ));

        let mut materials = world
            .get_resource_mut::<Assets<bevy::pbr2::StandardMaterial>>()
            .unwrap();
        let body_color = materials.add(bevy::pbr2::StandardMaterial {
            base_color: bevy::render2::color::Color::rgb(0.3, 0.3, 0.3),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        });
        let eye_color = materials.add(bevy::render2::color::Color::YELLOW.into());

        Self {
            body_mesh,
            body_color,
            eye_mesh,
            eye_color,
        }
    }
}

#[derive(Component)]
struct Creature {
    velocity: Vec3,
    desired_direction: Vec3,
    wander_strength: f32,
}

fn spawn_ant(
    mut commands: Commands,
    ant_handles: Res<AntHandles>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        commands
            .spawn_bundle((Transform::identity(), GlobalTransform::default()))
            .with_children(|creature| {
                creature
                    .spawn_bundle(bevy::pbr2::PbrBundle {
                        mesh: ant_handles.body_mesh.clone_weak(),
                        material: ant_handles.body_color.clone_weak(),
                        transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
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
                wander_strength: 0.1,
            });
    }
}

fn move_ants(
    mut commands: Commands,
    mut ants: Query<(&mut Transform, &mut Creature)>,
    time: Res<Time>,
    obstacle_map: Res<ObstacleMap>,
) {
    let max_speed = 0.25;
    let steer_strength = 2.0;
    for (mut transform, mut ant) in ants.iter_mut() {
        ant.desired_direction = (ant.desired_direction
            - Quat::from_rotation_y(rand::thread_rng().gen_range(0.0..(2.0 * PI)))
                .mul_vec3(Vec3::X)
                * ant.wander_strength)
            .normalize();

        let desired_velocity = ant.desired_direction * max_speed;
        let desired_steering_force = (desired_velocity - ant.velocity) * steer_strength;
        let acceleration = desired_steering_force.clamp_length_max(steer_strength);

        ant.velocity =
            (ant.velocity + acceleration * time.delta_seconds()).clamp_length_max(max_speed);

        let angle = if ant.velocity.x < 0.0 {
            -ant.velocity.angle_between(Vec3::new(0.0, 0.0, 1.0))
        } else {
            ant.velocity.angle_between(Vec3::new(0.0, 0.0, 1.0))
        };
        let forward = transform.translation + ant.velocity * time.delta_seconds();
        let forward_forward = transform.translation + ant.velocity / DEF * 2.0;
        if !obstacle_map.is_obstacle(forward_forward.x, forward_forward.z, 0.0) {
            transform.rotation = Quat::from_rotation_y(angle);
            transform.translation = forward;
            ant.wander_strength = 0.1;
            let position = IVec2::new(
                transform.translation.x as i32,
                transform.translation.z as i32,
            );
            commands.spawn_bundle((EmptyLot::new(position, true),));
        } else {
            ant.wander_strength += 0.5;
        }
    }
}
