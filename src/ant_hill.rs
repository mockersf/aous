use bevy::prelude::*;

use crate::terrain_spawner::ObstacleMap;

pub struct AntHillPlugin;

impl Plugin for AntHillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AntHillHandles>()
            .add_startup_system(spawn_ant_hill);
    }
}

struct AntHillHandles {
    mesh: Handle<Mesh>,
    color: Handle<StandardMaterial>,
}

impl FromWorld for AntHillHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.add(Mesh::from(shape::Icosphere {
            radius: 0.1,
            ..Default::default()
        }));

        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let color = materials.add(StandardMaterial {
            base_color: Color::rgb(0.545, 0.271, 0.075),
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        });

        Self { mesh, color }
    }
}

#[derive(Component)]
pub struct AntHill;

fn spawn_ant_hill(mut commands: Commands, ant_hill_handles: Res<AntHillHandles>) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: ant_hill_handles.mesh.clone_weak(),
            material: ant_hill_handles.color.clone_weak(),
            transform: Transform::from_xyz(0.0, -0.02, 0.0),
            ..Default::default()
        })
        .insert(AntHill);
}