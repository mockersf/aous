use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

mod camera;
mod terrain_spawner;

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
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            FrameTimeDiagnosticsPlugin::FPS,
        ]))
        .add_plugin(camera::CameraPlugin)
        .add_plugin(terrain_spawner::TerrainSpawnerPlugin)
        .run();
}
