use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMethod, RayCastSource, RaycastSystem};

mod ant_hill;
mod ants;
mod camera;
mod terrain_spawner;

struct RaycastCameraToGround;

const BORDER: f32 = 3.0;
const DEF: f32 = 20.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Ants Of Unusual Size".to_string(),
            ..Default::default()
        })
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::TRACE,
            filter: "wgpu=warn,bevy=info".to_string(),
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultRaycastingPlugin::<RaycastCameraToGround>::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            FrameTimeDiagnosticsPlugin::FPS,
        ]))
        .add_plugin(camera::CameraPlugin)
        .add_plugin(terrain_spawner::TerrainSpawnerPlugin)
        .add_plugin(ants::AntsPlugin)
        .add_plugin(ant_hill::AntHillPlugin)
        .init_resource::<CursorPosition>()
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_raycast_with_cursor.before(RaycastSystem::BuildRays),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_debug_cursor::<RaycastCameraToGround>
                .label(RaycastSystem::UpdateDebugCursor)
                .after(RaycastSystem::UpdateRaycast),
        )
        .run();
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RayCastSource<RaycastCameraToGround>>,
) {
    for mut pick_source in &mut query.iter_mut() {
        // Grab the most recent cursor event if it exists:
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
        }
    }
}

#[derive(Default)]
pub struct CursorPosition {
    pub pos: Option<Vec3>,
}

pub fn update_debug_cursor<T: 'static + Send + Sync>(
    mut pos: ResMut<CursorPosition>,
    raycast_source_query: Query<&RayCastSource<T>>,
) {
    // Set the cursor translation to the top pick's world coordinates
    for raycast_source in raycast_source_query.iter() {
        if let Some(top_intersection) = raycast_source.intersect_top() {
            let transform_new = top_intersection.1.normal_ray().to_transform();
            pos.pos = Some(Transform::from_matrix(transform_new).translation);
        }
    }
}
