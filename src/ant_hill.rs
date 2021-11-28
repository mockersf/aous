use bevy::prelude::*;

pub struct AntHillPlugin;

impl Plugin for AntHillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AntHillHandles>()
            .add_startup_system(spawn_ant_hill);
    }
}

struct AntHillHandles {
    mesh: Handle<bevy::render2::mesh::Mesh>,
    color: Handle<bevy::pbr2::StandardMaterial>,
}

impl FromWorld for AntHillHandles {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut meshes = world
            .get_resource_mut::<Assets<bevy::render2::mesh::Mesh>>()
            .unwrap();
        let mesh = meshes.add(bevy::render2::mesh::Mesh::from(
            bevy::render2::mesh::shape::Icosphere {
                radius: 0.1,
                ..Default::default()
            },
        ));

        let mut materials = world
            .get_resource_mut::<Assets<bevy::pbr2::StandardMaterial>>()
            .unwrap();
        let color = materials.add(bevy::pbr2::StandardMaterial {
            base_color: bevy::render2::color::Color::rgb(0.545, 0.271, 0.075),
            perceptual_roughness: 1.0,
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
        .spawn_bundle(bevy::pbr2::PbrBundle {
            mesh: ant_hill_handles.mesh.clone_weak(),
            material: ant_hill_handles.color.clone_weak(),
            transform: Transform::from_xyz(0.0, -0.02, 0.0),
            ..Default::default()
        })
        .insert(AntHill);
}
