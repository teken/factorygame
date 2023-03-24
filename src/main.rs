mod blocks;
mod materials;
mod player;

use std::f32::consts::PI;

use bevy::{input::mouse::MouseWheel, math::vec3, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_raycast::{
    DefaultPluginState, DefaultRaycastingPlugin, RaycastMesh, RaycastMethod, RaycastSource,
    RaycastSystem,
};
use bevy_obj::ObjPlugin;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin, DebugShapes};
use bevy_rapier3d::prelude::*;
use blocks::{Block, BlockClickedEvent, BlockPlugin, Spawn};
use player::{Modes, Player, PlayerPlugin, SpawnerOptions};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugin(PlayerPlugin)
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .add_plugin(BlockPlugin)
        .add_startup_system(setup_graphics)
        .add_system(bevy::window::close_on_esc)
        .add_event::<EmptyGridCellClickedEvent>()
        .add_event::<BlockClickedEvent>()
        .add_system(
            update_raycast_with_cursor
                .in_base_set(CoreSet::First)
                .before(RaycastSystem::BuildRays::<MyRaycastSet>),
        )
        .add_system(grid)
        .add_system(empty_grid_cell_event_spawner)
        .add_system(empty_grid_cell_event_handler)
        .add_system(build_plane_manipulation)
        .run();
}

#[derive(Clone, Reflect)]
struct MyRaycastSet;

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<MyRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

#[derive(Component)]
struct BuildPlane {}

const RENDER_GRID: bool = true;
const GRID_SIZE: i32 = 100;
const GRID_CELL_SIZE: usize = 1;

fn setup_graphics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ambient_light: ResMut<AmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(10000.0).into()),
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        },
        BuildPlane {},
        RaycastMesh::<MyRaycastSet>::default(),
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

fn empty_grid_cell_event_spawner(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    intersect_query: Query<&bevy_mod_raycast::Intersection<MyRaycastSet>>,
    mut ev_emptygridcellclicked: EventWriter<EmptyGridCellClickedEvent>,
    mut ev_blockclicked: EventWriter<BlockClickedEvent>,
    objects_query: Query<(&Block, Entity)>,
    mode_states: Res<State<Modes>>,
) {
    let drag_time = keys.pressed(KeyCode::LControl);
    let mouse_trigger = if drag_time {
        mouse.pressed(MouseButton::Left)
    } else {
        mouse.just_pressed(MouseButton::Left)
    };

    if !mouse_trigger {
        return;
    }
    let Ok(inter) = intersect_query.get_single() else {
            return;
        };

    let Some(position) = inter.position() else {
            return;
        };

    let clicked_block = objects_query.iter().find(|(block, _)| {
        position.x >= block.min.x
            && position.x <= block.max.x
            && position.y >= block.min.y
            && position.y <= block.max.y
            && position.z >= block.min.z
            && position.z <= block.max.z
    });

    match clicked_block {
        Some((_, entity)) => {
            if drag_time {
                return;
            }
            if mode_states.0 == Modes::Destroy {
                commands.entity(entity).despawn_recursive();
            } else if mode_states.0 == Modes::Overview {
                ev_blockclicked.send(BlockClickedEvent {
                    grid_cell: position.floor(),
                    world_pos: position.clone(),
                });
            }

            return;
        }
        None => {
            ev_emptygridcellclicked.send(EmptyGridCellClickedEvent {
                grid_cell: position.floor(),
                world_pos: position.clone(),
            });
        }
    }
}

fn empty_grid_cell_event_handler(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cell_clicked_event: EventReader<EmptyGridCellClickedEvent>,
    mut player_query: Query<&SpawnerOptions, With<Player>>,
    asset_server: Res<AssetServer>,
    mode_states: Res<State<Modes>>,
) {
    for event in cell_clicked_event.iter() {
        if mode_states.0 != Modes::Build {
            return;
        }
        for ele in player_query.iter_mut() {
            match &ele.block_selection {
                Some(i) => i.clone().spawn(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &asset_server,
                    ele,
                    event.world_pos,
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
