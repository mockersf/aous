use std::collections::hash_map::Entry;

use bevy::{
    ecs::component::SparseStorage,
    math::const_vec3,
    prelude::*,
    render::{
        mesh::Indices,
        pipeline::PrimitiveTopology,
        texture::{Extent3d, TextureDimension, TextureFormat},
    },
    utils::HashMap,
};
// use bevy_mod_raycast::{BoundVol, RayCastMesh};
use bracket_noise::prelude::{FastNoise, FractalType, NoiseType};
use rand::Rng;

use crate::{BORDER, DEF};

#[derive(Debug)]
pub struct EmptyLot {
    x: i32,
    z: i32,
    offscreen: bool,
}

impl Component for EmptyLot {
    type Storage = SparseStorage;
}

impl EmptyLot {
    pub fn new(position: IVec2, offscreen: bool) -> Self {
        EmptyLot {
            x: position.x,
            z: position.y,
            offscreen,
        }
    }
}

pub struct TerrainSpawnerPlugin;

#[derive(Default)]
pub struct ObstacleMap {
    pub obstacle_map: HashMap<IVec2, bool>,
}

impl ObstacleMap {
    pub fn is_obstacle(&self, x: f32, z: f32, _width: f32) -> bool {
        *self
            .obstacle_map
            .get(&IVec2::new(
                (x * DEF + DEF / 2.0) as i32,
                (z * DEF + DEF / 2.0) as i32,
            ))
            .unwrap_or(&false)
    }
}

pub struct NoiseSeeds {
    elevation: u64,
    moisture: u64,
}

impl Plugin for TerrainSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NoiseSeeds {
            elevation: rand::thread_rng().gen(),
            moisture: rand::thread_rng().gen(),
        })
        .init_resource::<ObstacleMap>()
        .add_system(fill_empty_lots);
    }
}

struct Lot {
    mesh: bevy::render2::mesh::Mesh,
    color: bevy::render2::texture::Image,
    metallic_roughness: bevy::render2::texture::Image,
    obstacle_map: HashMap<IVec2, bool>,
}

struct HandledLot {
    mesh: Handle<bevy::render2::mesh::Mesh>,
    color: Handle<bevy::pbr2::StandardMaterial>,
}

fn fill_empty_lots(
    mut commands: Commands,
    lots: Query<(Entity, &EmptyLot)>,
    mut meshes: ResMut<Assets<bevy::render2::mesh::Mesh>>,
    mut textures: ResMut<Assets<bevy::render2::texture::Image>>,
    mut materials: ResMut<Assets<bevy::pbr2::StandardMaterial>>,
    mut mesh_cache: Local<HashMap<IVec2, HandledLot>>,
    mut obstacle_map: ResMut<ObstacleMap>,
    noise_seeds: Res<NoiseSeeds>,
) {
    for (entity, position) in lots.iter() {
        let mesh = mesh_cache
            .entry(IVec2::new(position.x, position.z))
            .or_insert_with(|| {
                let lot = generate_lot(position.x, position.z, &*noise_seeds);
                obstacle_map.obstacle_map.extend(lot.obstacle_map);
                HandledLot {
                    mesh: meshes.add(lot.mesh),
                    color: materials.add(bevy::pbr2::StandardMaterial {
                        base_color: bevy::render2::color::Color::WHITE,
                        base_color_texture: Some(textures.add(lot.color)),
                        perceptual_roughness: 1.0,
                        metallic: 1.0,
                        metallic_roughness_texture: Some(textures.add(lot.metallic_roughness)),
                        ..Default::default()
                    }),
                }
            });
        if !position.offscreen {
            commands
                .entity(entity)
                .with_children(|lot| {
                    lot.spawn_bundle(bevy::pbr2::PbrBundle {
                        mesh: mesh.mesh.clone_weak(),
                        material: mesh.color.clone_weak(),
                        ..Default::default()
                    });
                    // .insert_bundle((
                    //     BoundVol { sphere: None },
                    //     RayCastMesh::<crate::RaycastCameraToGround>::default(),
                    // ));
                })
                .remove::<EmptyLot>();
        } else {
            commands.entity(entity).remove::<EmptyLot>();
        }
    }
}

fn generate_lot(x: i32, z: i32, noise_seeds: &NoiseSeeds) -> Lot {
    debug!("generating mesh for {} / {}", x, z);
    let mut elevation_noise = FastNoise::seeded(noise_seeds.elevation);
    elevation_noise.set_noise_type(NoiseType::PerlinFractal);
    elevation_noise.set_fractal_type(FractalType::FBM);
    elevation_noise.set_fractal_octaves(7);
    elevation_noise.set_fractal_gain(0.4);
    elevation_noise.set_fractal_lacunarity(2.0);
    elevation_noise.set_frequency(2.0);

    let mut moisture_noise = FastNoise::seeded(noise_seeds.moisture);
    moisture_noise.set_noise_type(NoiseType::PerlinFractal);
    moisture_noise.set_fractal_type(FractalType::FBM);
    moisture_noise.set_fractal_octaves(5);
    moisture_noise.set_fractal_gain(0.75);
    moisture_noise.set_fractal_lacunarity(2.0);
    moisture_noise.set_frequency(2.0);

    const fn color_to_vec3(color: Color) -> Vec3 {
        if let Color::Rgba {
            red,
            green,
            blue,
            alpha: _,
        } = color
        {
            const_vec3!([red, green, blue])
        } else {
            const_vec3!([0.0, 0.0, 0.0])
        }
    }
    let moisture_mountain = color_to_vec3(Color::ALICE_BLUE);
    let moisture_prairie = color_to_vec3(Color::GREEN);
    let arid_mountain = color_to_vec3(Color::ANTIQUE_WHITE);
    let arid_prairie = color_to_vec3(Color::GRAY);

    let mut vertices = Vec::new();
    let mut colors = Vec::new();
    let mut metallic_roughness = Vec::new();

    let mut obstacle_map = HashMap::default();

    for i in 0..=(DEF as i32) {
        for j in 0..=(DEF as i32) {
            let nx = x as f32 + i as f32 / DEF;
            let nz = z as f32 + j as f32 / DEF;
            let get_elevation = |x: f32, z: f32, dx: f32, dz: f32| {
                let px = x + dx - 0.5;
                let pz = z + dz - 0.5;
                if px.powf(2.0) + pz.powf(2.0) < 0.05 {
                    (0.0, 0.005)
                } else {
                    let elevation = elevation_noise.get_noise(px, pz);
                    if !(-BORDER..=BORDER).contains(&px) || !(-BORDER..=BORDER).contains(&pz) {
                        (elevation + 0.4, 0.41 + elevation / 10.0)
                    } else {
                        (
                            elevation,
                            elevation / 25.0 + if elevation > 0.25 { 0.4 } else { 0.0 },
                        )
                    }
                }
            };

            let (elevation, elevation_mod) =
                get_elevation(x as f32, z as f32, i as f32 / DEF, j as f32 / DEF);

            let mut neighbours = Vec::new();
            let mut has_obstacle_in_neighbours = false;
            for di in -1..=1 {
                for dj in -1..=1 {
                    if di != 0 || dj != 0 {
                        let de = get_elevation(nx, nz, di as f32 / DEF, dj as f32 / DEF).1;
                        neighbours.push([di as f32 / DEF, de, dj as f32 / DEF]);
                        if de > 0.4 {
                            has_obstacle_in_neighbours = true;
                        }
                    }
                }
            }
            obstacle_map.insert(
                IVec2::new(x * DEF as i32 + i, z * DEF as i32 + j),
                elevation_mod > 0.4 || has_obstacle_in_neighbours,
            );

            let mut normal = Vec3::ZERO;
            for (b, c) in [
                (0, 1),
                (1, 2),
                (2, 4),
                (4, 7),
                (7, 6),
                (6, 5),
                (5, 3),
                (3, 0),
            ] {
                let pa = Vec3::from([0.0, elevation_mod, 0.0]);
                let pb = Vec3::from(neighbours[b]);
                let pc = Vec3::from(neighbours[c]);

                let ab = pb - pa;
                let bc = pc - pb;
                let ca = pa - pc;
                let normal_face = ab.cross(bc) + bc.cross(ca) + ca.cross(ab);

                normal += normal_face;
            }
            let normal = normal.normalize();

            vertices.push((
                [i as f32 / DEF - 0.5, elevation_mod, j as f32 / DEF - 0.5],
                [normal.x, normal.y, normal.z],
                [j as f32 / DEF, i as f32 / DEF],
            ));

            let moisture = moisture_noise.get_noise(nx, nz);

            let elevation = elevation + 0.5;
            let moisture = moisture + 0.5;
            let mountain = arid_mountain.lerp(moisture_mountain, (moisture * 2.0).clamp(0.0, 1.0));
            let prairie = arid_prairie.lerp(moisture_prairie, (moisture * 2.0).clamp(0.0, 1.0));
            let lerped = prairie.lerp(mountain, elevation);

            colors.extend_from_slice(&[
                (lerped.x * 255.0) as u8,
                (lerped.y * 255.0) as u8,
                (lerped.z * 255.0) as u8,
                255,
            ]);

            let roughness = ((1.0 - elevation) * 2.0).clamp(0.0, 1.0);
            let metallic = 1.0 - moisture;
            metallic_roughness.extend_from_slice(&[
                0,
                (roughness * 255.0) as u8,
                (metallic * 255.0) as u8,
                255,
            ]);
        }
    }

    Lot {
        mesh: vertices_as_mesh(vertices, DEF as u32),
        color: bevy::render2::texture::Image::new(
            bevy::render2::render_resource::Extent3d {
                width: DEF as u32 + 1,
                height: DEF as u32 + 1,
                depth_or_array_layers: 1,
            },
            bevy::render2::render_resource::TextureDimension::D2,
            colors,
            bevy::render2::render_resource::TextureFormat::Rgba8UnormSrgb,
        ),
        metallic_roughness: bevy::render2::texture::Image::new(
            bevy::render2::render_resource::Extent3d {
                width: DEF as u32 + 1,
                height: DEF as u32 + 1,
                depth_or_array_layers: 1,
            },
            bevy::render2::render_resource::TextureDimension::D2,
            metallic_roughness,
            bevy::render2::render_resource::TextureFormat::Rgba8UnormSrgb,
        ),
        obstacle_map,
    }
}
type Node = ([f32; 3], [f32; 3], [f32; 2]);

fn vertices_as_mesh(vertices: Vec<Node>, details: u32) -> bevy::render2::mesh::Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut n = 0;
    let mut pushed = HashMap::default();

    let mut push = |data: Node| match pushed.entry(IVec2::new(
        (data.0[0] * details as f32 * 2.0) as i32,
        (data.0[2] * details as f32 * 2.0) as i32,
    )) {
        Entry::Occupied(o) => *o.get(),
        Entry::Vacant(v) => {
            positions.push(data.0);
            normals.push(data.1);
            uvs.push(data.2);
            n += 1;
            *v.insert(n - 1)
        }
    };

    for i in 0..details {
        for j in 0..details {
            let data1 = *vertices.get((i + j * (details + 1)) as usize).unwrap();
            let data2 = *vertices.get((i + 1 + j * (details + 1)) as usize).unwrap();
            let data3 = *vertices
                .get((i + (j + 1) * (details + 1)) as usize)
                .unwrap();
            let data4 = *vertices
                .get((i + 1 + (j + 1) * (details + 1)) as usize)
                .unwrap();

            indices.extend_from_slice(&[push(data1), push(data2), push(data3)]);
            indices.extend_from_slice(&[push(data3), push(data2), push(data4)]);
        }
    }

    let mut mesh = bevy::render2::mesh::Mesh::new(
        bevy::render2::render_resource::PrimitiveTopology::TriangleList,
    );
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(bevy::render2::mesh::Indices::U32(indices)));
    mesh
}
