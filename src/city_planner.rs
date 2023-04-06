use bevy::{
    prelude::*,
    render::{mesh, render_resource::PrimitiveTopology},
};
use bracket_lib::{
    prelude::{FastNoise, FractalType, Interp, NoiseType},
    random::RandomNumberGenerator,
};

pub struct CityPlannerPlugin;

impl Plugin for CityPlannerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoiseGeneration>();
        app.add_startup_system(generate_heightmap);
        app.add_startup_system(spawn_ground_plane);
    }
}

const ENABLE_WIREFRAME: bool = false;
const CITY_BLOCK_COUNT: i32 = 10;
const CITY_BLOCK_SIZE: i32 = 100;

fn spawn_ground_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut noise_gen: ResMut<NoiseGeneration>,
) {
    // let mut mesh = if ENABLE_WIREFRAME {
    //     Mesh::new(PrimitiveTopology::LineList)
    // } else {
    //     Mesh::new(PrimitiveTopology::TriangleList)
    // };

    // let mut vertices: Vec<[f32; 3]> = Vec::new();
    // let mut indices: Vec<u32> = Vec::new();
    // let mut colors: Vec<[f32; 3]> = Vec::new();

    // for vertex in &terrain_mesh_data.vertices {
    //     vertices.push([vertex.x, vertex.y, vertex.z]);

    //     let color = grad.get(vertex.y);
    //     let raw_float: Srgb<f32> = Srgb::<f32>::from_linear(color.into());
    //     colors.push([raw_float.red, raw_float.green, raw_float.blue]);
    // }

    // // Positions of the vertices
    // // See https://bevy-cheatbook.github.io/features/coords.html
    // mesh.insert_attribute(
    //     Mesh::ATTRIBUTE_POSITION,
    //     vec![[0., 0., 0.], [1., 2., 1.], [2., 0., 0.]],
    // );

    // // In this example, normals and UVs don't matter,
    // // so we just use the same value for all of them
    // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 3]);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 3]);

    // // A triangle using vertices 0, 2, and 1.
    // // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
    // mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1])));

    commands.spawn(PbrBundle {
        mesh: meshes
            .add(shape::Plane::from_size((CITY_BLOCK_COUNT * CITY_BLOCK_SIZE) as f32 * 2.5).into()),
        material: materials.add(Color::rgb_u8(30, 30, 30).into()),
        ..default()
    });

    for x in -CITY_BLOCK_COUNT..CITY_BLOCK_COUNT {
        for z in -CITY_BLOCK_COUNT..CITY_BLOCK_COUNT {
            let height =
                (noise_gen.noise.get_noise(x as f32 / 10., z as f32 / 10.) * 100.0).abs() + 10.;

            commands.spawn(PbrBundle {
                mesh: meshes.add(
                    shape::Box::new(
                        (CITY_BLOCK_SIZE - 10) as f32,
                        height,
                        (CITY_BLOCK_SIZE - 10) as f32,
                    )
                    .into(),
                ),
                material: materials.add(Color::rgb_u8(201, 201, 201).into()),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 100.0,
                    height / 2.,
                    z as f32 * 100.0,
                )),
                ..default()
            });
        }
    }
}

#[derive(Resource)]
struct NoiseGeneration {
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

        Self { noise }
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
