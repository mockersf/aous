use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use rand::Rng;

use crate::{
    ant_hill::HillEvents,
    ants::{AntState, Creature},
    food::{FoodHeap, FoodPellet, WorldEvents},
    game_state::GameState,
    terrain_spawner::{EmptyLot, ObstacleMap},
    DEF,
};

pub struct AntEatersPlugin;

impl Plugin for AntEatersPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AntEaterHandles>()
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(spawn_anteaters)
                    .with_system(move_anteaters),
            )
            .add_system_to_stage(CoreStage::PostUpdate, anteaters_die)
            .add_system_to_stage(CoreStage::Update, anteaters_consume_food)
            .add_system_to_stage(CoreStage::PreUpdate, anteaters_consume_ants);
    }
}

pub struct AntEaterHandles {
    pub body_mesh: Handle<bevy::render2::mesh::Mesh>,
    pub body_color: Handle<bevy::pbr2::StandardMaterial>,
    pub eye_mesh: Handle<bevy::render2::mesh::Mesh>,
    pub eye_color: Handle<bevy::pbr2::StandardMaterial>,
}

impl FromWorld for AntEaterHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap();
        let body_mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Capsule {
                radius: 0.015,
                depth: 0.015,
                latitudes: 8,
                longitudes: 16,
                ..Default::default()
            },
        ));
        let eye_mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.008,
                subdivisions: 2,
            },
        ));

        let mut materials = world
            .get_resource_mut::<Assets<bevy::pbr2::StandardMaterial>>()
            .unwrap();
        let body_color = materials.add(bevy::pbr2::StandardMaterial {
            base_color: bevy::render2::color::Color::rgb(0.9, 0.1, 0.1),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        });
        let eye_color = materials.add(bevy::render2::color::Color::BLACK.into());

        Self {
            body_mesh,
            body_color,
            eye_mesh,
            eye_color,
        }
    }
}

#[derive(Component)]
pub struct AntEater {
    pub velocity: Vec3,
    pub desired_direction: Vec3,
    pub wander_strength: f32,
    pub food_picked: u32,
    pub ant_killed: u32,
}

fn spawn_anteaters(
    mut commands: Commands,
    handles: Res<AntEaterHandles>,
    mut events: EventReader<WorldEvents>,
) {
    for event in events.iter() {
        match event {
            WorldEvents::SpawnFood => (),
            WorldEvents::SpawnAntEater(position) => {
                commands
                    .spawn_bundle((
                        Transform {
                            translation: *position,
                            scale: Vec3::splat(5.0),
                            ..Default::default()
                        },
                        GlobalTransform::default(),
                    ))
                    .with_children(|creature| {
                        creature.spawn_bundle(bevy::pbr2::PbrBundle {
                            mesh: handles.body_mesh.clone_weak(),
                            material: handles.body_color.clone_weak(),
                            transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                            ..Default::default()
                        });
                        creature.spawn_bundle(bevy::pbr2::PbrBundle {
                            mesh: handles.eye_mesh.clone_weak(),
                            material: handles.eye_color.clone_weak(),
                            transform: Transform::from_xyz(0.0075, 0.0075, 0.01875),
                            ..Default::default()
                        });
                        creature.spawn_bundle(bevy::pbr2::PbrBundle {
                            mesh: handles.eye_mesh.clone_weak(),
                            material: handles.eye_color.clone_weak(),
                            transform: Transform::from_xyz(-0.0075, 0.0075, 0.01875),
                            ..Default::default()
                        });
                    })
                    .insert(AntEater {
                        velocity: Vec3::ZERO,
                        desired_direction: Vec3::ZERO,
                        wander_strength: 0.2,
                        food_picked: 0,
                        ant_killed: 0,
                    });
            }
        }
    }
}

fn move_anteaters(
    mut commands: Commands,
    mut anteaters: Query<(&mut Transform, &mut AntEater)>,
    time: Res<Time>,
    obstacle_map: Res<ObstacleMap>,
) {
    let steer_strength = 2.0;
    let max_speed = 0.18;
    let wander_strength = 0.5;
    for (mut transform, mut anteater) in anteaters.iter_mut() {
        let moving_towards = transform.translation.normalize()
            + Quat::from_rotation_y(rand::thread_rng().gen_range(0.0..(2.0 * PI)))
                .mul_vec3(Vec3::X)
                * anteater.wander_strength;
        anteater.desired_direction = (anteater.desired_direction - moving_towards).normalize();

        let desired_velocity = anteater.desired_direction * max_speed;
        let desired_steering_force = (desired_velocity - anteater.velocity) * steer_strength;
        let acceleration = desired_steering_force.clamp_length_max(steer_strength);

        anteater.velocity =
            (anteater.velocity + acceleration * time.delta_seconds()).clamp_length_max(max_speed);

        let angle = if anteater.velocity.x < 0.0 {
            -anteater.velocity.angle_between(Vec3::new(0.0, 0.0, 1.0))
        } else {
            anteater.velocity.angle_between(Vec3::new(0.0, 0.0, 1.0))
        };
        let forward = transform.translation + anteater.velocity * time.delta_seconds();
        let forward_forward = transform.translation + anteater.velocity / DEF * 2.0;
        if !obstacle_map.is_obstacle(forward_forward.x, forward_forward.z, 0.0) {
            transform.rotation = Quat::from_rotation_y(angle);
            transform.translation = forward;
            anteater.wander_strength = wander_strength;
            let position = IVec2::new(
                transform.translation.x as i32,
                transform.translation.z as i32,
            );
            commands.spawn_bundle((EmptyLot::new(position, true),));
        } else {
            anteater.wander_strength += 0.5;
        }
    }
}

fn anteaters_die(
    mut commands: Commands,
    anteaters: Query<(Entity, &Transform, &AntEater)>,
    mut events: EventWriter<HillEvents>,
) {
    for (entity, transform, anteater) in anteaters.iter() {
        if transform.translation.distance_squared(Vec3::ZERO) < 0.005 {
            commands.entity(entity).despawn_recursive();
            events.send(HillEvents::ReplenishFood(anteater.ant_killed / 10, 0.8));
            events.send(HillEvents::ReplenishFood(anteater.food_picked / 20, 0.5));
            events.send(HillEvents::ImproveLifeExpectancy(-0.5));
            events.send(HillEvents::ImproveMaxSpeed(-0.001));
        }
    }
}

fn anteaters_consume_food(
    mut commands: Commands,
    mut anteaters: Query<(Entity, &Transform, &mut AntEater)>,
    foods: Query<(Entity, &GlobalTransform), With<FoodPellet>>,
) {
    for (entity, transform, mut anteater) in anteaters.iter_mut() {
        for (food_entity, food_transform) in foods.iter() {
            if transform
                .translation
                .distance_squared(food_transform.translation)
                < 0.025
            {
                commands
                    .entity(food_entity)
                    .insert(Parent(entity))
                    .remove::<Transform>()
                    .remove::<FoodPellet>();
                anteater.food_picked += 1;
            }
        }
    }
}
fn anteaters_consume_ants(
    mut commands: Commands,
    mut anteaters: Query<(&Transform, &mut AntEater)>,
    ants: Query<(Entity, &Transform, &Creature)>,
    mut foods: Query<&mut FoodPellet, (Without<Creature>, Without<FoodHeap>)>,
) {
    for (transform, mut anteater) in anteaters.iter_mut() {
        for (ant_entity, ant_transform, ant) in ants.iter() {
            if transform
                .translation
                .distance_squared(ant_transform.translation)
                < 0.025
            {
                if let AntState::PickFood(_, food_entity) = ant.state {
                    if let Ok(mut food_pellet) = foods.get_mut(food_entity) {
                        food_pellet.targeted = false;
                    }
                }
                commands.entity(ant_entity).despawn_recursive();
                anteater.ant_killed += 1;
            }
        }
    }
}
