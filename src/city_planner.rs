use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use bevy_prototype_debug_lines::DebugShapes;
use bevy_vox_mesh::VoxMeshPlugin;
use bracket_lib::{
    prelude::{FastNoise, FractalType, Interp, NoiseType},
    random::RandomNumberGenerator,
};
use rayon::prelude::*;
use voronoice::{BoundingBox, Point, VoronoiBuilder};

pub struct CityPlannerPlugin;

impl Plugin for CityPlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(VoxMeshPlugin::default());
        app.init_resource::<NoiseGeneration>();
        app.init_resource::<CityBlocks>();
        app.add_startup_system(generate_heightmap);
        app.add_startup_system(spawn_ground_plane);
        app.add_startup_system(generate_city_blocks);
        app.add_startup_system(generate_city_blocks_buildings.after(generate_city_blocks));
        app.add_startup_system(generate_block_meshes.after(generate_city_blocks_buildings));
        app.add_system(spawn_wireframes);
    }
}

const ENABLE_BLOCK_WIREFRAME: bool = false;
const ENABLE_FLOOR_WIREFRAME: bool = false;
const ENABLE_BUILDING_WIREFRAME: bool = true;
const CITY_BLOCK_COUNT: i32 = 1;
const CITY_BLOCK_SIZE_X: i32 = 100;
const CITY_BLOCK_SIZE_Z: i32 = 200;
const CITY_BLOCK_GAP: i32 = 10;
const CITY_BLOCK_FLOOR_HEIGHT: i32 = 4;
const CITY_BLOCK_BUILD_MIN_COUNT: i32 = 20;
const CITY_BLOCK_BUILD_MAX_COUNT: i32 = 50;
const BUILDING_SLOT_MIN_SIZE: i32 = 8;
const BUILDING_MIN_WIDTH: i32 = 7;
const BUILDING_MIN_DEPTH: i32 = 32;
const LLOYD_RELAXATION_ITERATIONS: usize = 5;

fn spawn_ground_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Plane::from_size((CITY_BLOCK_COUNT * CITY_BLOCK_SIZE_Z) as f32 * 22.5).into(),
        ),
        material: materials.add(Color::rgb_u8(30, 30, 30).into()),
        ..default()
    });
}

fn generate_city_blocks(mut city_blocks: ResMut<CityBlocks>, noise_gen: Res<NoiseGeneration>) {
    for x in -CITY_BLOCK_COUNT..=CITY_BLOCK_COUNT {
        for z in -CITY_BLOCK_COUNT..=CITY_BLOCK_COUNT {
            let height = (noise_gen.noise.get_noise(x as f32 / 10., z as f32 / 10.) * 100.0).abs();

            city_blocks.blocks.push(CityBlock {
                block_x: x,
                block_z: z,
                height,
                ..Default::default()
            });
        }
    }
}

enum SquareEdges {
    North,
    South,
    East,
    West,
}

fn generate_city_blocks_buildings(
    mut city_blocks: ResMut<CityBlocks>,
    mut noise_gen: ResMut<NoiseGeneration>,
    mut debug_shapes: ResMut<DebugShapes>,
) {
    let x_length = CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP;
    let z_length = CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP;
    let x_line_offset = x_length / 4;
    let z_line_offset = x_length / 4;

    for block in city_blocks.blocks.iter_mut() {
        if block.height < 8. {
            continue;
        }

        let building_count = noise_gen
            .rng
            .range(CITY_BLOCK_BUILD_MIN_COUNT, CITY_BLOCK_BUILD_MAX_COUNT);

        let mut points = vec![];

        for _ in 0..building_count {
            let edge = match noise_gen.rng.range(0, 4) {
                0 => SquareEdges::North,
                1 => SquareEdges::South,
                2 => SquareEdges::East,
                3 => SquareEdges::West,
                _ => panic!("Invalid edge"),
            };

            points.push(match edge {
                SquareEdges::North => Vec2::new(
                    noise_gen.rng.range(
                        (block.min_x() + x_line_offset) as f32,
                        (block.max_x() - x_line_offset) as f32,
                    ),
                    (block.min_z() + z_line_offset) as f32,
                ),
                SquareEdges::South => Vec2::new(
                    noise_gen.rng.range(
                        (block.min_x() + x_line_offset) as f32,
                        (block.max_x() - x_line_offset) as f32,
                    ),
                    (block.max_z() - z_line_offset) as f32,
                ),
                SquareEdges::East => Vec2::new(
                    (block.min_x() + x_line_offset) as f32,
                    noise_gen.rng.range(
                        (block.min_z() + z_line_offset) as f32,
                        (block.max_z() - z_line_offset) as f32,
                    ),
                ),
                SquareEdges::West => Vec2::new(
                    (block.max_x() - x_line_offset) as f32,
                    noise_gen.rng.range(
                        (block.min_z() + z_line_offset) as f32,
                        (block.max_z() - z_line_offset) as f32,
                    ),
                ),
            });
        }

        let voronoi = VoronoiBuilder::default()
            .set_sites(
                points
                    .iter()
                    .map(|p| Point {
                        x: p.x as f64,
                        y: p.y as f64,
                    })
                    .collect(),
            )
            .set_bounding_box(BoundingBox::new(
                Point {
                    x: (block.block_x * CITY_BLOCK_SIZE_X) as f64,
                    y: (block.block_z * CITY_BLOCK_SIZE_Z) as f64,
                },
                (CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP) as f64,
                (CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP) as f64,
            ))
            .build()
            .unwrap();

        for cell in voronoi.iter_cells() {
            block.buildings.push(BuildingSlot {
                points: cell
                    .iter_vertices()
                    .map(|p| Vec2::new(p.x as f32, p.y as f32))
                    .collect(),
                height: if block.height as i32 <= BUILDING_MIN_DEPTH {
                    BUILDING_MIN_DEPTH
                } else {
                    noise_gen.rng.range(BUILDING_MIN_DEPTH, block.height as i32)
                },
            });
        }
    }
}

fn generate_block_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    city_blocks: Res<CityBlocks>,
) {
    let x_length = CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP;
    let z_length = CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP;

    for block in city_blocks.blocks.iter() {
        if block.height < 8. {
            continue;
        }

        for slot in block.buildings.iter() {
            // let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            // let mut triangulation: DelaunayTriangulation<_> = DelaunayTriangulation::new();
            // slot.verts.iter().for_each(|p| {
            //     let x = p.x.0 as i32 + block.x * CITY_BLOCK_SIZE_X + x_length / 2;
            //     let z = p.y.0 as i32 + block.z * CITY_BLOCK_SIZE_Z + z_length / 2;

            //     let y = slot.height;
            //     triangulation.insert(spade::Point2::new(x as f32, z as f32));
            // });

            // triangulation.inner_faces().map(|face| {
            //     let edge = face.adjacent_edges();

            //     [
            //         edge[0].origin().clone(),
            //         edge[1].origin().clone(),
            //         edge[2].origin().clone(),
            //     ]
            // });

            // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

            // commands.spawn(PbrBundle {
            //     mesh: meshes.add(mesh),
            //     material: materials.add(Color::rgb_u8(30, 30, 30).into()),
            //     ..default()
            // });
        }

        // let mut vertices = block
        //     .buildings
        //     .iter()
        //     .map(|x| {
        //         x.verts
        //             .iter()
        //             .zip(x.verts.clone().iter_mut().map(|_| x.height))
        //             .collect::<Vec<_>>()
        //     })
        //     .flatten()
        //     .map(|(p, h)| {
        //         let x = p.x.0 as i32 + block.x * CITY_BLOCK_SIZE_X + x_length / 2;
        //         let z = p.y.0 as i32 + block.z * CITY_BLOCK_SIZE_Z + z_length / 2;

        //         let y = h;

        //         [x as f32, y as f32, z as f32]
        //     })
        //     .collect::<Vec<[f32; 3]>>();

        // // let start_x = (block.x * CITY_BLOCK_SIZE_X) as f32;
        // // let end_x = start_x + x_length as f32;
        // // let start_z = (block.z * CITY_BLOCK_SIZE_Z) as f32;
        // // let end_z = start_z + z_length as f32;
        // // vertices.push([start_x, 0., start_z]);
        // // vertices.push([start_x, 0., end_z]);
        // // vertices.push([end_x, 0., end_z]);
        // // vertices.push([end_x, 0., start_z]);

        // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        // commands.spawn(PbrBundle {
        //     mesh: meshes.add(mesh),
        //     material: materials.add(Color::rgb_u8(30, 30, 30).into()),
        //     ..default()
        // });
    }
}

#[derive(Resource, Default)]
struct CityBlocks {
    blocks: Vec<CityBlock>,
}

#[derive(Default)]
struct CityBlock {
    block_x: i32,
    block_z: i32,
    height: f32,
    buildings: Vec<BuildingSlot>,
}

impl CityBlock {
    fn min_x(&self) -> i32 {
        (self.block_x * CITY_BLOCK_SIZE_X) - ((CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP) / 2)
    }

    fn max_x(&self) -> i32 {
        (self.block_x * CITY_BLOCK_SIZE_X) + ((CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP) / 2)
    }

    fn min_z(&self) -> i32 {
        (self.block_z * CITY_BLOCK_SIZE_Z) - ((CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP) / 2)
    }

    fn max_z(&self) -> i32 {
        (self.block_z * CITY_BLOCK_SIZE_Z) + ((CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP) / 2)
    }
}

struct BuildingSlot {
    height: i32,
    points: Vec<Vec2>,
}

fn spawn_wireframes(city_blocks: Res<CityBlocks>, mut debug_shapes: ResMut<DebugShapes>) {
    if !ENABLE_BLOCK_WIREFRAME && !ENABLE_FLOOR_WIREFRAME && !ENABLE_BUILDING_WIREFRAME {
        return;
    }

    let x_length = CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP;
    let z_length = CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP;

    for block in city_blocks.blocks.iter() {
        if block.height < 8. {
            continue;
        }

        if ENABLE_BLOCK_WIREFRAME {
            debug_shapes
                .cuboid()
                .min_max(
                    Vec3::new(block.min_x() as f32, 0., block.min_z() as f32),
                    Vec3::new(block.max_x() as f32, block.height, block.max_z() as f32),
                )
                .color(Color::rgb_u8(201, 201, 201));
        }
        if ENABLE_FLOOR_WIREFRAME {
            for y in (CITY_BLOCK_FLOOR_HEIGHT..((block.height as i32) - CITY_BLOCK_FLOOR_HEIGHT))
                .step_by(CITY_BLOCK_FLOOR_HEIGHT as usize)
            {
                debug_shapes
                    .cuboid()
                    .min_max(
                        Vec3::new(block.min_x() as f32, y as f32, block.min_z() as f32),
                        Vec3::new(block.min_x() as f32, y as f32, block.min_z() as f32),
                    )
                    .color(Color::rgb_u8(201, 201, 201));
            }
        }
        if ENABLE_BUILDING_WIREFRAME {
            for slot in block.buildings.iter() {
                let mut last_point: &Vec2 = slot.points.last().unwrap();
                for point in slot.points.iter() {
                    debug_shapes
                        .line()
                        .start_end(
                            Vec3::new(
                                last_point.x as f32 as f32,
                                slot.height as f32,
                                last_point.y as f32,
                            ),
                            Vec3::new(point.x as f32 as f32, slot.height as f32, point.y as f32),
                        )
                        .color(Color::rgb_u8(0, 0, 201));
                    last_point = point;
                }
            }
        }
    }
}

#[derive(Resource)]
struct NoiseGeneration {
    rng: RandomNumberGenerator,
    noise: FastNoise,
}

impl Default for NoiseGeneration {
    fn default() -> Self {
        let mut rng = RandomNumberGenerator::new();
        let seed = rng.next_u64();
        println!("Seed: {}", seed);
        let mut noise = FastNoise::seeded(seed);
        noise.set_noise_type(NoiseType::SimplexFractal);
        noise.set_fractal_type(FractalType::Billow);
        noise.set_interp(Interp::Quintic);
        noise.set_fractal_octaves(5);
        noise.set_fractal_gain(0.6);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(2.0);

        Self { rng, noise }
    }
}

fn generate_heightmap(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut noise_gen: ResMut<NoiseGeneration>,
) {
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(10000.0).into()),
    //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //     ..default()
    // });

    // noise_gen.noise.

    // for y in 0..50 {
    //     for x in 0..80 {
    //         let n = noise.get_noise((x as f32) / 160.0, (y as f32) / 100.0);
    //         if n < 0.0 {
    //             print_color(RGB::from_f32(0.0, 0.0, 1.0 - (0.0 - n)), "░");
    //         } else {
    //             print_color(RGB::from_f32(0.0, n, 0.0), "░");
    //         }
    //     }
    //     print_color(RGB::named(WHITE), "\n");
    // }
}
