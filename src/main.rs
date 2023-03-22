mod panorbitcamera;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_mod_picking::{DebugCursorPickingPlugin, DefaultPickingPlugins, Hover, PickableBundle};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin, DebugShapes};
use bevy_rapier3d::prelude::*;
use panorbitcamera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_startup_system(setup_graphics)
        .add_system(bevy::window::close_on_esc)
        .add_event::<EmptyGridCellClickEvent>()
        .add_startup_system(grid)
        .add_system(empty_grid_cell_event_spawner)
        .add_system(empty_grid_cell_event_handler)
        .run();
}

#[derive(Component)]
struct BuildPlane {}

const RENDER_GRID: bool = true;
const GRID_SIZE: i32 = 100;
const GRID_CELL_SIZE: usize = 1;
const Y_OFFSET: f32 = 0.0;

fn setup_graphics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(100.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(BuildPlane {})
        .insert(PickableBundle::default());
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
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

fn grid(mut lines: ResMut<DebugLines>) {
    if RENDER_GRID {
        for x in (-GRID_SIZE..GRID_SIZE).step_by(GRID_CELL_SIZE) {
            lines.line_colored(
                Vec3::new(
                    x as f32,
                    Y_OFFSET * GRID_CELL_SIZE as f32,
                    -GRID_SIZE as f32,
                ),
                Vec3::new(x as f32, Y_OFFSET * GRID_CELL_SIZE as f32, GRID_SIZE as f32),
                3600.0,
                Color::rgb(0.5, 0.5, 0.5),
            );
        }
        for z in (-GRID_SIZE..GRID_SIZE).step_by(GRID_CELL_SIZE) {
            lines.line_colored(
                Vec3::new(
                    -GRID_SIZE as f32,
                    Y_OFFSET * GRID_CELL_SIZE as f32,
                    z as f32,
                ),
                Vec3::new(GRID_SIZE as f32, Y_OFFSET * GRID_CELL_SIZE as f32, z as f32),
                3600.0,
                Color::rgb(0.5, 0.5, 0.5),
            );
        }
    }
}

struct EmptyGridCellClickEvent(Vec3, Vec3);

fn empty_grid_cell_event_spawner(
    mouse: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PanOrbitCamera>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    build_plane_query: Query<(&GlobalTransform, Entity), (With<BuildPlane>, With<Hover>)>,
    mut ev_emptygridcellclick: EventWriter<EmptyGridCellClickEvent>,
) {
    if mouse.just_pressed(MouseButton::Left) {
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

        ev_emptygridcellclick.send(EmptyGridCellClickEvent(i.floor(), i));
    }
}

fn empty_grid_cell_event_handler(
    mut cell_clicked_event: EventReader<EmptyGridCellClickEvent>,
    mut shapes: ResMut<DebugShapes>,
) {
    for event in cell_clicked_event.iter() {
        shapes
            .cuboid()
            .min_max(event.1.floor(), event.1.ceil())
            .color(Color::rgb(1.0, 0.0, 0.0))
            .duration(3.0);
    }
}
