use std::{f32::consts::PI, ops::Deref};

use bevy::{pbr2::NotShadowCaster, prelude::*, utils::HashSet};
use rand::Rng;

use crate::{
    ant_hill::HillEvents,
    food::{FoodHeap, FoodPellet},
    terrain_spawner::{EmptyLot, ObstacleMap},
    DEF,
};

pub struct AntsPlugin;

impl Plugin for AntsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AntHandles>()
            .add_system(update_ant_state)
            .add_system(move_ants)
            .add_system_to_stage(CoreStage::PostUpdate, aging_ants);
    }
}

pub struct AntHandles {
    pub body_mesh: Handle<bevy::render2::mesh::Mesh>,
    pub body_color: Handle<bevy::pbr2::StandardMaterial>,
    pub eye_mesh: Handle<bevy::render2::mesh::Mesh>,
    pub eye_color: Handle<bevy::pbr2::StandardMaterial>,
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
                latitudes: 4,
                longitudes: 8,
                ..Default::default()
            },
        ));
        let eye_mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.008,
                subdivisions: 1,
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

pub enum AntState {
    Wander,
    PickFood(Vec3, Entity),
    HasFood,
}

impl PartialEq for AntState {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Component)]
pub struct Creature {
    pub velocity: Vec3,
    pub desired_direction: Vec3,
    pub wander_strength: f32,
    pub state: AntState,
    pub birth: f64,
    pub gene: CreatureGene,
}

#[derive(Clone, Copy, Debug)]
pub struct CreatureGene {
    pub life_expectancy: f64,
    pub max_speed: f32,
    pub wander_strength: f32,
}

#[derive(Component)]
struct PickedFood;

fn update_ant_state(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut Creature, Entity, &Children)>,
    food_heaps: Query<(&Transform, &Children), (With<FoodHeap>, Without<Creature>)>,
    mut foods: Query<(&GlobalTransform, &mut FoodPellet), (Without<Creature>, Without<FoodHeap>)>,
    picked_foods: Query<Entity, With<PickedFood>>,
    mut hill_events: EventWriter<HillEvents>,
) {
    for (transform, mut ant, entity, children) in ants.iter_mut() {
        let mut near = 10.0;
        let mut target_heap = None;
        match ant.state {
            AntState::Wander => {
                // search for food nearby
                for (food_heap, children) in food_heaps.iter() {
                    let distance = food_heap
                        .translation
                        .distance_squared(transform.translation);
                    if distance < near {
                        near = distance;
                        target_heap = Some(children);
                    }
                }
                if near < (1.0 / DEF * 5.0).powf(2.0) {
                    for food_entity in Deref::deref(target_heap.unwrap()) {
                        if let Ok((food, mut pellet)) = foods.get_mut(*food_entity) {
                            if !pellet.targeted {
                                pellet.targeted = true;
                                ant.state = AntState::PickFood(food.translation, *food_entity);
                                break;
                            }
                        }
                    }
                }
            }
            AntState::PickFood(target, food_entity) => {
                // pick food if close enough
                if transform.translation.distance_squared(target) < (1.0 / DEF).powf(2.0) {
                    ant.state = AntState::HasFood;
                    commands
                        .entity(food_entity)
                        .insert_bundle((
                            Parent(entity),
                            Transform {
                                translation: Vec3::new(0.0, 0.01, 0.02),
                                scale: Vec3::splat(0.8),
                                rotation: Default::default(),
                            },
                            PickedFood,
                        ))
                        .remove::<FoodPellet>();
                }
            }
            AntState::HasFood => {
                // drop food at home if close enough
                if transform.translation.distance_squared(Vec3::ZERO) < (1.0 / DEF).powf(2.0) {
                    hill_events.send(HillEvents::StoreFood(1, ant.gene));
                    ant.state = AntState::Wander;
                    for child in children.iter() {
                        if picked_foods.get(*child).is_ok() {
                            commands.entity(*child).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

fn move_ants(
    mut commands: Commands,
    mut ants: Query<(&mut Transform, &mut Creature)>,
    time: Res<Time>,
    obstacle_map: Res<ObstacleMap>,
) {
    let steer_strength = 2.0;
    for (mut transform, mut ant) in ants.iter_mut() {
        // find where we want to go
        let moving_towards = match ant.state {
            AntState::Wander => {
                // TODO: look for pheromons
                Quat::from_rotation_y(rand::thread_rng().gen_range(0.0..(2.0 * PI)))
                    .mul_vec3(Vec3::X)
                    * ant.wander_strength
            }
            AntState::PickFood(position, _) => {
                (-position + transform.translation) * 2.0
                    + Quat::from_rotation_y(rand::thread_rng().gen_range(0.0..(2.0 * PI)))
                        .mul_vec3(Vec3::X)
                        * ant.wander_strength
                        / 2.0
            }
            AntState::HasFood => {
                // TODO: look for pheromons
                transform.translation.normalize()
                    + Quat::from_rotation_y(rand::thread_rng().gen_range(0.0..(2.0 * PI)))
                        .mul_vec3(Vec3::X)
                        * ant.wander_strength
                        / 2.0
            }
        };
        ant.desired_direction = (ant.desired_direction - moving_towards).normalize();

        let desired_velocity = ant.desired_direction * ant.gene.max_speed;
        let desired_steering_force = (desired_velocity - ant.velocity) * steer_strength;
        let acceleration = desired_steering_force.clamp_length_max(steer_strength);

        ant.velocity = (ant.velocity + acceleration * time.delta_seconds())
            .clamp_length_max(ant.gene.max_speed);

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
            ant.wander_strength = ant.gene.wander_strength;
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

fn aging_ants(
    mut commands: Commands,
    ants: Query<(Entity, &Creature)>,
    mut foods: Query<&mut FoodPellet, (Without<Creature>, Without<FoodHeap>)>,
    time: Res<Time>,
) {
    for (entity, ant) in ants.iter() {
        // if ant.state == AntState::Wander || ant.state == AntState::HasFood {
        if time.seconds_since_startup() - ant.birth > ant.gene.life_expectancy {
            if let AntState::PickFood(_, food_entity) = ant.state {
                foods.get_mut(food_entity).unwrap().targeted = false;
            }
            commands.entity(entity).despawn_recursive();
        }
        // }
    }
}
