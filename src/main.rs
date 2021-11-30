// disable console opening on windows
#![windows_subsystem = "windows"]
#![allow(clippy::type_complexity)]

use bevy::{
    diagnostic::{
        // EntityCountDiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin,
    },
    prelude::*,
    PipelinedDefaultPlugins,
};
use bevy_egui::EguiPlugin;
// use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMethod, RayCastSource, RaycastSystem};

mod ant_eaters;
mod ant_hill;
mod ants;
mod camera;
mod food;
mod game_state;
mod splash;
mod terrain_spawner;
mod ui;

// struct RaycastCameraToGround;

const BORDER: f32 = 2.0;
const DEF: f32 = 20.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Ants Of Unusual Shape".to_string(),
            #[cfg(target_arch = "wasm32")]
            width: 1024.0,
            ..Default::default()
        })
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::TRACE,
            filter: "wgpu=warn,bevy=info,winit=info,naga=info".to_string(),
        })
        .add_plugins_with(PipelinedDefaultPlugins, |group| {
            return group.add_before::<bevy::asset::AssetPlugin, _>(asset_io::InMemoryAssetPlugin);
        })
        // .add_plugin(DefaultRaycastingPlugin::<RaycastCameraToGround>::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            FrameTimeDiagnosticsPlugin::FPS,
            // EntityCountDiagnosticsPlugin::ENTITY_COUNT,
        ]))
        .add_plugin(EguiPlugin)
        .add_plugin(game_state::GameStatePlugin)
        .add_plugin(splash::SplashPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(terrain_spawner::TerrainSpawnerPlugin)
        .add_plugin(ants::AntsPlugin)
        .add_plugin(ant_hill::AntHillPlugin)
        .add_plugin(food::FoodPlugin)
        .add_plugin(ant_eaters::AntEatersPlugin)
        // .init_resource::<CursorPosition>()
        // .add_system_to_stage(
        //     CoreStage::PreUpdate,
        //     update_raycast_with_cursor.before(RaycastSystem::BuildRays),
        // )
        // .add_system_to_stage(
        //     CoreStage::PreUpdate,
        //     update_debug_cursor::<RaycastCameraToGround>
        //         .label(RaycastSystem::UpdateDebugCursor)
        //         .after(RaycastSystem::UpdateRaycast),
        // )
        .add_plugin(ui::UiPlugin)
        .run();
}

// fn update_raycast_with_cursor(
//     mut cursor: EventReader<CursorMoved>,
//     mut query: Query<&mut RayCastSource<RaycastCameraToGround>>,
// ) {
//     for mut pick_source in &mut query.iter_mut() {
//         // Grab the most recent cursor event if it exists:
//         if let Some(cursor_latest) = cursor.iter().last() {
//             pick_source.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
//         }
//     }
// }

// #[derive(Default)]
// pub struct CursorPosition {
//     pub pos: Option<Vec3>,
// }

// pub fn update_debug_cursor<T: 'static + Send + Sync>(
//     mut pos: ResMut<CursorPosition>,
//     raycast_source_query: Query<&RayCastSource<T>>,
// ) {
//     // Set the cursor translation to the top pick's world coordinates
//     for raycast_source in raycast_source_query.iter() {
//         if let Some(top_intersection) = raycast_source.intersect_top() {
//             let transform_new = top_intersection.1.normal_ray().to_transform();
//             pos.pos = Some(Transform::from_matrix(transform_new).translation);
//         }
//     }
// }
