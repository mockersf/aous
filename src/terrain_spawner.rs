use std::collections::{hash_map::Entry, HashMap};

use bevy::{
    ecs::component::SparseStorage,
    prelude::{shape::Plane, *},
    render::{
        mesh::Indices,
        pipeline::PrimitiveTopology,
        texture::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bracket_noise::prelude::{FastNoise, FractalType, NoiseType};

#[derive(Debug)]
pub struct EmptyLot {
    x: i32,
    z: i32,
}

impl Component for EmptyLot {
    type Storage = SparseStorage;
}

impl EmptyLot {
    pub fn new(position: IVec2) -> Self {
        EmptyLot {
            x: position.x,
            z: position.y,
        }
    }
}

pub struct TerrainSpawnerPlugin;

impl Plugin for TerrainSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterHandles>()
            .add_system(fill_empty_lots);
    }
}

struct WaterHandles {
    plane: Handle<Mesh>,
    color: Handle<StandardMaterial>,
}

impl FromWorld for WaterHandles {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let plane = meshes.add(Plane::default().into());
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let color = materials.add(Color::rgba(1.0, 0.84, 0.0, 0.5).into());

        Self { plane, color }
    }
}

struct Lot {
    mesh: Mesh,
    color: Texture,
}

struct HandledLot {
    mesh: Handle<Mesh>,
    color: Handle<StandardMaterial>,
}

fn fill_empty_lots(
    mut commands: Commands,
    lots: Query<(Entity, &EmptyLot)>,
    water: Res<WaterHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_cache: Local<HashMap<IVec2, HandledLot>>,
) {
    for (entity, position) in lots.iter() {
        commands
            .entity(entity)
            .with_children(|lot| {
                lot.spawn_bundle(PbrBundle {
                    mesh: water.plane.clone(),
                    material: water.color.clone(),
                    visible: Visible {
                        is_visible: true,
                        is_transparent: true,
                    },
                    transform: Transform::from_xyz(0.0, -0.05, 0.0),
                    ..Default::default()
                });
                let mesh = mesh_cache
                    .entry(IVec2::new(position.x, position.z))
                    .or_insert_with(|| {
                        let lot = generate_lot(position.x, position.z);
                        HandledLot {
                            mesh: meshes.add(lot.mesh),
                            color: materials.add(StandardMaterial {
                                base_color: Color::WHITE,
                                base_color_texture: Some(textures.add(lot.color)),
                                ..Default::default()
                            }),
                        }
                    });
                lot.spawn_bundle(PbrBundle {
                    mesh: mesh.mesh.clone_weak(),
                    material: mesh.color.clone_weak(),
                    ..Default::default()
                });
            })
            .remove::<EmptyLot>();
    }
}

fn generate_lot(x: i32, z: i32) -> Lot {
    debug!("generating mesh for {} / {}", x, z);
    let mut elevation_noise = FastNoise::seeded(0);
    elevation_noise.set_noise_type(NoiseType::PerlinFractal);
    elevation_noise.set_fractal_type(FractalType::FBM);
    elevation_noise.set_fractal_octaves(7);
    elevation_noise.set_fractal_gain(0.4);
    elevation_noise.set_fractal_lacunarity(2.0);
    elevation_noise.set_frequency(2.0);

    let mut vertices = Vec::new();
    let mut colors = Vec::new();

    let def = 5.0;

    for i in 0..=(def as i32) {
        for j in 0..=(def as i32) {
            let nx = x as f32 + i as f32 / def;
            let nz = z as f32 + j as f32 / def;
            let elevation = elevation_noise.get_noise(nx, nz);
            let elevation_mod = elevation / 20.0
                + if elevation > 0.25 {
                    0.5
                } else if elevation < -0.25 {
                    -0.5
                } else {
                    0.0
                };
            vertices.push((
                [i as f32 / def - 0.5, elevation_mod, j as f32 / def - 0.5],
                [0.0, 0.0, 0.0],
                [j as f32 / def, i as f32 / def],
            ));
            colors.extend_from_slice(&[0 as u8, ((elevation + 1.0) / 2.0 * 255.0) as u8, 0, 255]);
        }
    }
    Lot {
        mesh: vertices_as_mesh(vertices, def as u32),
        color: Texture::new(
            Extent3d::new(def as u32 + 1, def as u32 + 1, 1),
            TextureDimension::D2,
            colors,
            TextureFormat::Rgba8UnormSrgb,
        ),
    }
}

fn vertices_as_mesh(vertices: Vec<([f32; 3], [f32; 3], [f32; 2])>, details: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut indices = vec![];
    for i in 0..details {
        for j in 0..details {
            indices.extend_from_slice(&[
                i + (details + 1) * j,
                i + 1 + (details + 1) * j,
                i + (details + 1) * (j + 1),
            ]);
            indices.extend_from_slice(&[
                i + (details + 1) * (j + 1),
                i + 1 + (details + 1) * j,
                i + 1 + (details + 1) * (j + 1),
            ]);
        }
    }

    let mut indices_iter = indices.iter();
    while let Some(a) = indices_iter.next() {
        let b = indices_iter.next().unwrap();
        let c = indices_iter.next().unwrap();

        let pa = Vec3::from(positions[*a as usize]);
        let pb = Vec3::from(positions[*b as usize]);
        let pc = Vec3::from(positions[*c as usize]);

        let ab = pb - pa;
        let bc = pc - pb;
        let ca = pa - pc;
        let normal_face = ab.cross(bc) + bc.cross(ca) + ca.cross(ab);

        let na = Vec3::from(normals[*a as usize]);
        let nb = Vec3::from(normals[*b as usize]);
        let nc = Vec3::from(normals[*c as usize]);
        (na + normal_face).write_to_slice(&mut normals[*a as usize]);
        (nb + normal_face).write_to_slice(&mut normals[*b as usize]);
        (nc + normal_face).write_to_slice(&mut normals[*c as usize]);
    }

    let normals: Vec<_> = normals
        .into_iter()
        .map(|normal| {
            let normal = Vec3::from(normal);
            let normalized = normal.normalize();
            [normalized.x, normalized.y, normalized.z]
        })
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}
