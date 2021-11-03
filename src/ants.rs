use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::CursorPosition;

pub struct AntsPlugin;

impl Plugin for AntsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AntHandles>()
            .add_startup_system(setup)
            .add_system(move_ants);
    }
}

struct AntHandles {
    body_mesh: Handle<Mesh>,
    body_color: Handle<StandardMaterial>,
    eye_mesh: Handle<Mesh>,
    eye_color: Handle<StandardMaterial>,
}

impl FromWorld for AntHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let body_mesh = meshes.add(Mesh::from(shape::Capsule {
            radius: 0.015,
            depth: 0.015,
            ..Default::default()
        }));
        let eye_mesh = meshes.add(Mesh::from(shape::Icosphere {
            radius: 0.008,
            subdivisions: 5,
        }));

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let body_color = materials.add(StandardMaterial {
            base_color: Color::BLUE,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        });
        let eye_color = materials.add(Color::YELLOW.into());

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
}

fn setup(mut commands: Commands, ant_handles: Res<AntHandles>) {
    commands
        .spawn_bundle((Transform::identity(), GlobalTransform::default()))
        .with_children(|creature| {
            creature.spawn_bundle(PbrBundle {
                mesh: ant_handles.body_mesh.clone_weak(),
                material: ant_handles.body_color.clone_weak(),
                transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                ..Default::default()
            });
            creature.spawn_bundle(PbrBundle {
                mesh: ant_handles.eye_mesh.clone_weak(),
                material: ant_handles.eye_color.clone_weak(),
                transform: Transform::from_xyz(0.0075, 0.0075, 0.01875),
                ..Default::default()
            });
            creature.spawn_bundle(PbrBundle {
                mesh: ant_handles.eye_mesh.clone_weak(),
                material: ant_handles.eye_color.clone_weak(),
                transform: Transform::from_xyz(-0.0075, 0.0075, 0.01875),
                ..Default::default()
            });
        })
        .insert(Creature {
            velocity: Vec3::ZERO,
        });
}

fn move_ants(
    mut ants: Query<(&mut Transform, &mut Creature)>,
    time: Res<Time>,
    cursor_position: Res<CursorPosition>,
) {
    let max_speed = 0.25;
    let steer_strength = 2.0;
    if let Some(target) = cursor_position.pos {
        let target = Vec3::new(target.x, 0.1, target.z);
        for (mut transform, mut ant) in ants.iter_mut() {
            let desired_direction = (target - transform.translation).normalize();

            let desired_velocity = desired_direction * max_speed;
            let desired_steering_force = (desired_velocity - ant.velocity) * steer_strength;
            let acceleration = desired_steering_force.clamp_length_max(steer_strength);

            ant.velocity =
                (ant.velocity + acceleration * time.delta_seconds()).clamp_length_max(max_speed);

            let angle = if ant.velocity.x < 0.0 {
                -ant.velocity.angle_between(Vec3::new(0.0, 0.0, 1.0))
            } else {
                ant.velocity.angle_between(Vec3::new(0.0, 0.0, 1.0))
            };
            transform.rotation = Quat::from_rotation_y(angle);
            transform.translation += ant.velocity * time.delta_seconds();
        }
    }
}
