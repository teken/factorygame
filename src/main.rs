mod blocks;
mod materials;
mod player;

use std::f32::consts::PI;

use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{DebugCursorPickingPlugin, DefaultPickingPlugins, Hover, PickableBundle};
use bevy_obj::ObjPlugin;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin, DebugShapes};
use bevy_rapier3d::prelude::*;
use blocks::{Block, Spawn};
use materials::Item;
use player::{Player, PlayerPlugin, PlayerPluginCamera, SpawnerOptions};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugin(PlayerPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_startup_system(setup_graphics)
        .add_system(bevy::window::close_on_esc)
        .add_event::<EmptyGridCellClickedEvent>()
        .add_event::<BlockClickedEvent>()
        .add_system(grid)
        .add_system(empty_grid_cell_event_spawner)
        .add_system(empty_grid_cell_event_handler)
        .add_system(build_plane_manipulation)
        .add_system(block_clicked_event_handler)
        .run();
}

#[derive(Component)]
struct BuildPlane {}

#[derive(Component)]
struct Inventory {
    items: Vec<Item>,
}

const RENDER_GRID: bool = true;
const GRID_SIZE: i32 = 100;
const GRID_CELL_SIZE: usize = 1;

fn setup_graphics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(0.0).into()),
            ..default()
        },
        BuildPlane {},
        PickableBundle::default(),
        Name::new("Build Plane"),
    ));
    // light
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 1.0;

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 40.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn grid(
    mut lines: ResMut<DebugLines>,
    build_plane_query: Query<(&Transform, Entity), With<BuildPlane>>,
) {
    if !RENDER_GRID {
        return;
    }

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

struct EmptyGridCellClickedEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
}

struct BlockClickedEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
}

#[derive(Component)]
struct BlockClicked {}

fn empty_grid_cell_event_spawner(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Player>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    build_plane_query: Query<(&GlobalTransform, Entity), (With<BuildPlane>, With<Hover>)>,
    mut ev_emptygridcellclicked: EventWriter<EmptyGridCellClickedEvent>,
    mut ev_blockclicked: EventWriter<BlockClickedEvent>,
    objects_query: Query<(&Block, Entity)>,
) {
    let drag_time = keys.pressed(KeyCode::LControl);
    let mouse_trigger = if drag_time {
        mouse.pressed(MouseButton::Left)
    } else {
        mouse.just_pressed(MouseButton::Left)
    };
    if mouse_trigger {
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

        let clicked_block = objects_query.iter().find(|(block, _)| {
            i.x >= block.min.x
                && i.x <= block.max.x
                && i.y >= block.min.y
                && i.y <= block.max.y
                && i.z >= block.min.z
                && i.z <= block.max.z
        });
        match clicked_block {
            Some((_, entity)) => {
                if drag_time {
                    return;
                }
                // ev_blockclicked.send(BlockClickedEvent {
                //     grid_cell: i.floor(),
                //     world_pos: i,
                // });
                // commands.entity(entity).insert(BlockClicked {});
                return;
            }
            None => {
                ev_emptygridcellclicked.send(EmptyGridCellClickedEvent {
                    grid_cell: i.floor(),
                    world_pos: i,
                });
            }
        }
    }
}

fn block_clicked_event_handler(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    objects_query: Query<Entity, (With<Block>, With<BlockClicked>)>,
) {
    // if keys.pressed(KeyCode::LControl) {
    //     for entity in objects_query.iter() {
    //         commands.entity(entity).despawn_recursive();
    //     }
    // }
}

fn empty_grid_cell_event_handler(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cell_clicked_event: EventReader<EmptyGridCellClickedEvent>,
    mut shapes: ResMut<DebugShapes>,
    mut player_query: Query<&SpawnerOptions, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    for event in cell_clicked_event.iter() {
        shapes
            .cuboid()
            .min_max(event.world_pos.floor(), event.world_pos.ceil())
            .color(Color::rgba(1.0, 0.0, 0.0, 0.5))
            .duration(3.0);

        for ele in player_query.iter_mut() {
            match &ele.block_selection {
                Some(i) => i.clone().spawn(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &asset_server,
                    ele,
                    Block {
                        min: event.world_pos.floor(),
                        max: event.world_pos.ceil(),
                        block_type: i.clone(),
                    },
                ),
                None => {}
            }
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
