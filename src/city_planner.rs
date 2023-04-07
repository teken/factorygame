use bevy::{
    prelude::*,
    render::{mesh, render_resource::PrimitiveTopology},
};
use bevy_prototype_debug_lines::DebugShapes;
use bracket_lib::{
    prelude::{FastNoise, FractalType, Interp, NoiseType},
    random::RandomNumberGenerator,
};
use voronoi::Point;

pub struct CityPlannerPlugin;

impl Plugin for CityPlannerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoiseGeneration>();
        app.init_resource::<CityBlocks>();
        app.add_startup_system(generate_heightmap);
        app.add_startup_system(spawn_ground_plane);
        app.add_startup_system(generate_city_blocks);
        app.add_startup_system(generate_city_blocks_buildings.after(generate_city_blocks));
        app.add_system(spawn_block_wireframes);
    }
}

const ENABLE_BLOCK_WIREFRAME: bool = false;
const ENABLE_FLOOR_WIREFRAME: bool = false;
const ENABLE_BUILDING_WIREFRAME: bool = true;
const CITY_BLOCK_COUNT: i32 = 1;
const CITY_BLOCK_SIZE_X: i32 = 100;
const CITY_BLOCK_SIZE_Z: i32 = 100;
const CITY_BLOCK_GAP: i32 = 10;
const CITY_BLOCK_FLOOR_HEIGHT: i32 = 4;
const CITY_BLOCK_BUILD_MIN_COUNT: i32 = 10;
const CITY_BLOCK_BUILD_MAX_COUNT: i32 = 30;
const BUILDING_SLOT_MIN_SIZE: i32 = 8;
const BUILDING_MIN_WIDTH: i32 = 8;
const BUILDING_MIN_DEPTH: i32 = 32;

fn spawn_ground_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Plane::from_size((CITY_BLOCK_COUNT * CITY_BLOCK_SIZE_Z) as f32 * 2.5).into(),
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
                x,
                z,
                height,
                ..Default::default()
            });
        }
    }
}

fn generate_city_blocks_buildings(
    mut city_blocks: ResMut<CityBlocks>,
    mut noise_gen: ResMut<NoiseGeneration>,
) {
    let x_length = CITY_BLOCK_SIZE_X - CITY_BLOCK_GAP;
    let z_length = CITY_BLOCK_SIZE_Z - CITY_BLOCK_GAP;

    for ele in city_blocks.blocks.iter_mut() {
        if ele.height < 8. {
            continue;
        }

        let building_count = noise_gen
            .rng
            .range(CITY_BLOCK_BUILD_MIN_COUNT, CITY_BLOCK_BUILD_MAX_COUNT);

        let mut points = (0..building_count)
            .into_iter()
            .map(|_| {
                Point::new(
                    noise_gen.rng.range(0, x_length) as f64,
                    noise_gen.rng.range(0, z_length) as f64,
                )
            })
            .collect::<Vec<_>>();
        for _ in 0..5 {
            points = voronoi::lloyd_relaxation(points, x_length as f64);
        }

        let diagram = voronoi::voronoi(points, x_length as f64);
        for buildings in voronoi::make_polygons(&diagram).iter() {
            ele.buildings.push(BuildingSlot {
                verts: buildings.to_vec(),
                height: noise_gen.rng.range(8, ele.height as i32),
            })
        }
    }
}

#[derive(Resource, Default)]
struct CityBlocks {
    blocks: Vec<CityBlock>,
}

#[derive(Default)]
struct CityBlock {
    x: i32,
    z: i32,
    height: f32,
    buildings: Vec<BuildingSlot>,
}

struct BuildingSlot {
    height: i32,
    verts: Vec<Point>,
}

fn spawn_block_wireframes(city_blocks: Res<CityBlocks>, mut debug_shapes: ResMut<DebugShapes>) {
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
                    Vec3::new(
                        ((block.x * CITY_BLOCK_SIZE_X) - (x_length / 2)) as f32,
                        0.,
                        ((block.z * CITY_BLOCK_SIZE_Z) - (z_length / 2)) as f32,
                    ),
                    Vec3::new(
                        ((block.x * CITY_BLOCK_SIZE_X) + (x_length / 2)) as f32,
                        block.height,
                        ((block.z * CITY_BLOCK_SIZE_Z) + (z_length / 2)) as f32,
                    ),
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
                        Vec3::new(
                            ((block.x * CITY_BLOCK_SIZE_X) - (x_length / 2)) as f32,
                            y as f32,
                            ((block.z * CITY_BLOCK_SIZE_Z) - (z_length / 2)) as f32,
                        ),
                        Vec3::new(
                            ((block.x * CITY_BLOCK_SIZE_X) + (x_length / 2)) as f32,
                            y as f32,
                            ((block.z * CITY_BLOCK_SIZE_Z) + (z_length / 2)) as f32,
                        ),
                    )
                    .color(Color::rgb_u8(201, 201, 201));
            }
        }
        if ENABLE_BUILDING_WIREFRAME {
            // for building in block.buildings.iter() {
            //     debug_shapes
            //         .cuboid()
            //         .min_max(
            //             Vec3::new(
            //                 ((block.x * CITY_BLOCK_SIZE_X) - (x_length / 2)) as f32
            //                     + building.x as f32,
            //                 0.,
            //                 ((block.z * CITY_BLOCK_SIZE_Z) - (z_length / 2)) as f32
            //                     + building.z as f32,
            //             ),
            //             Vec3::new(
            //                 ((block.x * CITY_BLOCK_SIZE_X) - (x_length / 2)) as f32
            //                     + building.x as f32
            //                     + building.width as f32,
            //                 building.height as f32,
            //                 ((block.z * CITY_BLOCK_SIZE_Z) - (z_length / 2)) as f32
            //                     + building.z as f32
            //                     + building.depth as f32,
            //             ),
            //         )
            //         .color(Color::rgb_u8(0, 0, 201));
            // }

            for slot in block.buildings.iter() {
                let mut last_point: Option<&Point> = None;
                for line in slot.verts.iter() {
                    if last_point.is_none() {
                        last_point = Some(line);
                        continue;
                    }
                    debug_shapes
                        .line()
                        .start_end(
                            Vec3::new(
                                ((block.x * CITY_BLOCK_SIZE_X) - (x_length / 2)) as f32
                                    + last_point.unwrap().x.0 as f32,
                                slot.height as f32,
                                ((block.z * CITY_BLOCK_SIZE_Z) - (z_length / 2)) as f32
                                    + last_point.unwrap().y.0 as f32,
                            ),
                            Vec3::new(
                                ((block.x * CITY_BLOCK_SIZE_X) - (x_length / 2)) as f32
                                    + line.x.0 as f32,
                                slot.height as f32,
                                ((block.z * CITY_BLOCK_SIZE_Z) - (z_length / 2)) as f32
                                    + line.y.0 as f32,
                            ),
                        )
                        .color(Color::rgb_u8(0, 0, 201));
                    last_point = Some(line);
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
        let mut noise = FastNoise::seeded(rng.next_u64());
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
