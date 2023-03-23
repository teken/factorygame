mod materials;
mod panorbitcamera;

use bevy::{
    input::{self, mouse::MouseWheel},
    math::vec3,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{DebugCursorPickingPlugin, DefaultPickingPlugins, Hover, PickableBundle};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin, DebugShapes};
use bevy_rapier3d::prelude::*;
use materials::{Item, Reaction};
use panorbitcamera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_startup_system(setup_graphics)
        .add_system(bevy::window::close_on_esc)
        .add_event::<EmptyGridCellClickedEvent>()
        .add_system(grid)
        .add_system(empty_grid_cell_event_spawner)
        .add_system(empty_grid_cell_event_handler)
        .add_system(build_plane_manipulation)
        .run();
}

#[derive(Component)]
struct BuildPlane {}

#[derive(Component)]
struct Block {
    min: Vec3,
    max: Vec3,
    block_type: BlockType,
}

#[derive(Component)]
struct Inventory {
    items: Vec<Item>,
}

enum BlockType {
    Debug,
    Furnace,
}

trait Processor {
    fn process(&self, reaction: &Reaction, input: Vec<Item>) -> Option<Vec<Item>>;
}

impl Processor for BlockType {
    fn process(&self, reaction: &Reaction, input: Vec<Item>) -> Option<Vec<Item>> {
        match self {
            BlockType::Furnace => _process(reaction, input),
            _ => None,
        }
    }
}

fn _process(reaction: &Reaction, input: Vec<Item>) -> Option<Vec<Item>> {
    let mut block_input: Vec<Item> = vec![];
    let mut param_input: Vec<Item> = input.clone();
    param_input.append(&mut block_input);
    if reaction.valid_input(param_input) {
        Some(reaction.output.clone())
    } else {
        None
    }
}

const RENDER_GRID: bool = true;
const GRID_SIZE: i32 = 100;
const GRID_CELL_SIZE: usize = 1;

fn setup_graphics(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(100.0).into()),
            ..default()
        },
        BuildPlane {},
        PickableBundle::default(),
        Name::new("Build Plane"),
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn grid(
    mut lines: ResMut<DebugLines>,
    build_plane_query: Query<(&Transform, Entity), With<BuildPlane>>,
) {
    if RENDER_GRID {
        let Ok((trans, _)) = build_plane_query.get_single() else {
            return;
        };

        for x in (-GRID_SIZE..GRID_SIZE).step_by(GRID_CELL_SIZE) {
            lines.line_colored(
                Vec3::new(x as f32, trans.translation.y, -GRID_SIZE as f32),
                Vec3::new(x as f32, trans.translation.y, GRID_SIZE as f32),
                0.0,
                Color::rgb(0.5, 0.5, 0.5),
            );
        }
        for z in (-GRID_SIZE..GRID_SIZE).step_by(GRID_CELL_SIZE) {
            lines.line_colored(
                Vec3::new(-GRID_SIZE as f32, trans.translation.y, z as f32),
                Vec3::new(GRID_SIZE as f32, trans.translation.y, z as f32),
                0.0,
                Color::rgb(0.5, 0.5, 0.5),
            );
        }
    }
}

struct EmptyGridCellClickedEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
}

fn empty_grid_cell_event_spawner(
    mouse: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PanOrbitCamera>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    build_plane_query: Query<(&GlobalTransform, Entity), (With<BuildPlane>, With<Hover>)>,
    mut ev_emptygridcellclick: EventWriter<EmptyGridCellClickedEvent>,
    objects_query: Query<(&Block, Entity)>,
) {
    if mouse.pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_query.get_single() else {
            return;
        };
        let Ok(primary) = primary_query.get_single() else {
            return;
        };
        let Some(ray) = camera
            .viewport_to_world(camera_transform, primary.cursor_position().unwrap()) else {
                return;
            };

        let Ok((plane_transform,_)) = build_plane_query.get_single() else {
            return;
        };

        let Some(distance) =
            ray.intersect_plane(plane_transform.translation(), plane_transform.up()) else {
                return;
            };

        let i = camera_transform.translation() + ray.direction * distance;

        if objects_query.iter().any(|(block, _)| {
            i.x >= block.min.x
                && i.x <= block.max.x
                && i.y >= block.min.y
                && i.y <= block.max.y
                && i.z >= block.min.z
                && i.z <= block.max.z
        }) {
            return;
        }

        ev_emptygridcellclick.send(EmptyGridCellClickedEvent {
            grid_cell: i.floor(),
            world_pos: i,
        });
    }
}

fn empty_grid_cell_event_handler(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cell_clicked_event: EventReader<EmptyGridCellClickedEvent>,
    mut shapes: ResMut<DebugShapes>,
) {
    for event in cell_clicked_event.iter() {
        shapes
            .cuboid()
            .min_max(event.world_pos.floor(), event.world_pos.ceil())
            .color(Color::rgba(1.0, 0.0, 0.0, 0.5))
            .duration(3.0);

        spawn_block(
            &mut commands,
            &mut meshes,
            &mut materials,
            event.grid_cell,
            Block {
                min: event.world_pos.floor(),
                max: event.world_pos.ceil(),
                block_type: BlockType::Debug,
            },
        );
    }
}

fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    grid_cell: Vec3,
    block: Block,
) {
    match block.block_type {
        BlockType::Debug => {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_translation(grid_cell + vec3(0.5, 0.5, 0.5)),
                    ..default()
                },
                Name::new("Debug Block"),
                block,
            ));
        }
    }
}

fn build_plane_manipulation(
    mut build_plane_query: Query<(&mut Transform, Entity), With<BuildPlane>>,
    mut ev_scroll: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.pressed(KeyCode::LShift) {
        let mut scroll = 0.0;
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }

        for (mut transform, _) in build_plane_query.iter_mut() {
            if scroll.abs() > 0.0 {
                transform.translation.y += scroll * GRID_CELL_SIZE as f32;
            }
        }
    }
}
