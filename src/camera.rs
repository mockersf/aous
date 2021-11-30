use std::collections::hash_map::Entry;

use bevy::{prelude::*, utils::HashMap};
// use bevy_mod_raycast::RayCastSource;

use crate::{game_state::GameState, terrain_spawner::EmptyLot, BORDER};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup));

        #[cfg(target_arch = "wasm32")]
        app.insert_resource(bevy::pbr2::PointLightShadowMap {
            size: 2_usize.pow(11),
        });
        #[cfg(not(target_arch = "wasm32"))]
        app.insert_resource(bevy::pbr2::PointLightShadowMap {
            size: 2_usize.pow(12),
        });
        #[cfg(target_arch = "wasm32")]
        app.insert_resource(bevy::pbr2::DirectionalLightShadowMap {
            size: 2_usize.pow(1),
        });
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_camera)
                .with_system(refresh_visible_lots)
                .with_system(rotator),
        );
    }
}

#[derive(Component)]
struct CameraParent;

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle((
            Transform::default(),
            GlobalTransform::default(),
            CameraParent,
        ))
        .with_children(|camera_placer| {
            camera_placer.spawn_bundle(bevy::render2::camera::PerspectiveCameraBundle {
                transform: Transform::from_xyz(0.45, 4.3, -1.5)
                    .looking_at(Vec3::new(0.45, 0.0, -0.2), Vec3::Y),
                ..Default::default()
            });
            // .insert(RayCastSource::<crate::RaycastCameraToGround>::new_transform_empty());
            camera_placer
                .spawn_bundle(bevy::pbr2::PointLightBundle {
                    transform: Transform::from_xyz(-10.0, 3.0, 0.0),
                    point_light: bevy::pbr2::PointLight {
                        intensity: 1600.0,
                        range: 100.0,
                        shadows_enabled: true,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Rotates);
        });
}

#[derive(Default)]
pub struct VisibleLots(HashMap<IVec2, Entity>);

fn refresh_visible_lots(
    mut commands: Commands,
    windows: Res<Windows>,
    camera: Query<(&bevy::render2::camera::Camera, &GlobalTransform)>,
    mut visible_lots: ResMut<VisibleLots>,
) {
    let window_width = windows.get_primary().unwrap().width();
    let window_heigth = windows.get_primary().unwrap().height();
    let margin = 200.0;
    let is_on_screen = |position: Vec2| {
        if position.x - margin > window_width {
            return false;
        }
        if position.x + margin < 0.0 {
            return false;
        }
        if position.y - margin > window_heigth {
            return false;
        }
        if position.y + margin < 0.0 {
            return false;
        }

        true
    };

    let (camera, gt) = camera.single();

    let mut updated_lots: HashMap<IVec2, Entity> = visible_lots
        .0
        .drain()
        .filter(|(position, entity)| {
            if let Some(screen_position) = camera.world_to_screen(
                &*windows,
                gt,
                Vec3::new(position.x as f32, 0.0, position.y as f32),
            ) {
                if !is_on_screen(screen_position) {
                    debug!("despawning {:?} ({:?})", position, entity);
                    commands.entity(*entity).despawn_recursive();
                    return false;
                }
            }
            true
        })
        .collect();

    let span = 5;
    for i in -span..span {
        for j in -(span / 2)..span {
            let position = IVec2::new(gt.translation.x as i32 + i, gt.translation.z as i32 + j);
            if let Some(screen_position) = camera.world_to_screen(
                &*windows,
                gt,
                Vec3::new(position.x as f32, 0.0, position.y as f32),
            ) {
                if is_on_screen(screen_position) {
                    if let Entry::Vacant(vacant) = updated_lots.entry(position) {
                        debug!("spawning {:?}", position);
                        vacant.insert(
                            commands
                                .spawn_bundle((
                                    EmptyLot::new(position, false),
                                    Transform::from_xyz(position.x as f32, 0.0, position.y as f32),
                                    GlobalTransform::identity(),
                                ))
                                .id(),
                        );
                    }
                }
            }
        }
    }

    visible_lots.0 = updated_lots;
}

// Marker for a light to rotate
#[derive(Component)]
pub struct Rotates;

// Make the light rotate
fn rotator(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in query.iter_mut() {
        *transform = Transform::from_rotation(Quat::from_rotation_y(
            (4.0 * std::f32::consts::PI / 500.0) * time.delta_seconds(),
        )) * *transform;
    }
}

fn move_camera(
    mut query: QuerySet<(
        QueryState<&mut Transform, With<CameraParent>>,
        QueryState<&Transform, With<CameraParent>>,
    )>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let transform = query.q1().single();
    let move_by = time.delta_seconds();
    let mut move_to = Vec3::ZERO;
    let mut moving = false;
    if input.pressed(KeyCode::Left) && transform.translation.x < BORDER {
        moving = true;
        move_to.x = 1.0;
    } else if input.pressed(KeyCode::Right) && transform.translation.x > -BORDER {
        moving = true;
        move_to.x = -1.0;
    }
    if input.pressed(KeyCode::Up) && transform.translation.z < BORDER {
        moving = true;
        move_to.z = 1.0;
    } else if input.pressed(KeyCode::Down) && transform.translation.z > -BORDER {
        moving = true;
        move_to.z = -1.0;
    }
    if moving {
        query.q0().single_mut().translation += move_to.normalize() * move_by;
    }
}
