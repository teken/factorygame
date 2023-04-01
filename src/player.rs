use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::PickingCameraBundle;

use crate::{
    blocks::{Block, BlockClicked, BlockType},
    grid::GridSelectMode,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player);
        app.add_system(dev_ui);
        app.add_system(player_controller);
        app.add_system(player_hotkeys);
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Modes {
    #[default]
    Overview,
    Build,
    Destroy,
}

#[derive(Component)]
pub struct ModeReadOut;

#[derive(Component)]
pub struct RotationReadOut;

#[derive(Component)]
pub struct BlockReadOut;

#[derive(Component)]
pub struct Player {}

#[derive(Component, Default, Clone)]
pub struct SpawnerOptions {
    pub block_selection: BlockType,
    pub block_rotation: Direction,
    pub grid_select_mode: GridSelectMode,
    pub player_mode: Modes,
}

#[derive(Default, Reflect, PartialEq, Clone, Debug)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
    Up,
    Down,
}

impl Direction {
    pub fn to_quat(&self) -> Quat {
        match self {
            Direction::North => Quat::from_rotation_y(0.0),
            Direction::East => Quat::from_rotation_y(PI * 0.5),
            Direction::South => Quat::from_rotation_y(PI),
            Direction::West => Quat::from_rotation_y(PI * 1.5),
            Direction::Up => Quat::from_rotation_z(PI * 0.5),
            Direction::Down => Quat::from_rotation_z(PI * 1.5),
        }
    }
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct PlayerPluginCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PlayerPluginCamera {
    fn default() -> Self {
        PlayerPluginCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn player_controller(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut PlayerPluginCamera, &mut Transform, &Projection), With<Player>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    if !keys.pressed(KeyCode::LShift) {
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    let Ok(windows) = primary_query.get_single() else {
        return;
    };

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

fn get_primary_window_size(window: &Window) -> Vec2 {
    Vec2::new(window.width() as f32, window.height() as f32)
}

/// Spawn a camera like this
fn spawn_player(mut commands: Commands) {
    let translation = Vec3::new(-2.0, 2.5, 5.0);
    let radius = translation.length();

    // commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
    commands.spawn((
        Name::new("Player"),
        Player {},
        SpawnerOptions::default(),
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PlayerPluginCamera {
            radius,
            ..Default::default()
        },
        PickingCameraBundle::default(), // RaycastSource::<MyRaycastSet>::default(),
    ));
}

#[derive(Component)]
pub struct UICamera;

fn player_hotkeys(keys: Res<Input<KeyCode>>, mut query: Query<&mut SpawnerOptions, With<Player>>) {
    for mut ele in query.iter_mut() {
        if keys.just_pressed(KeyCode::Key1) {
            ele.block_selection = BlockType::Debug;
        } else if keys.just_pressed(KeyCode::Key2) {
            ele.block_selection = BlockType::Furnace;
        } else if keys.just_pressed(KeyCode::Key3) {
            ele.block_selection = BlockType::Conveyor;
        } else if keys.just_pressed(KeyCode::Key4) {
            ele.block_selection = BlockType::Splitter;
        } else if keys.just_pressed(KeyCode::Key5) {
            ele.block_selection = BlockType::Storage;
        } else if keys.just_pressed(KeyCode::Key6) {
            ele.block_selection = BlockType::Grabber;
        } else if keys.just_pressed(KeyCode::R) {
            ele.block_rotation = match ele.block_rotation {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::Up,
                Direction::Up => Direction::Down,
                Direction::Down => Direction::North,
            }
        } else if keys.just_pressed(KeyCode::Q) {
            ele.player_mode = match ele.player_mode {
                Modes::Overview => Modes::Build,
                Modes::Build => Modes::Destroy,
                Modes::Destroy => Modes::Overview,
            };
            ele.grid_select_mode = match ele.player_mode {
                Modes::Overview => GridSelectMode::Block,
                Modes::Build => GridSelectMode::OnTopOfBlock,
                Modes::Destroy => GridSelectMode::Block,
            }
        }
    }
}

fn dev_ui(
    mut egui_ctx: EguiContexts,
    mut player_query: Query<&mut SpawnerOptions, With<Player>>,
    block_selected_query: Query<(&Block, Entity), With<BlockClicked>>,
) {
    let Ok(mut spawn_options) = player_query.get_single_mut() else { return; };
    let mut mode_value = spawn_options.player_mode.clone();
    let mut direction_value = spawn_options.block_rotation.clone();
    let mut block_value = spawn_options.block_selection.clone();

    egui::SidePanel::right("selected_block_panel")
        .default_width(200.0)
        .show(&egui_ctx.ctx_mut(), |ui| {
            ui.group(|ui| {
                ui.heading("Player Settings");
                ui.separator();
                ui.label("Mode");
                egui::ComboBox::from_id_source("mode")
                    .selected_text(format!("{:?}", mode_value))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut mode_value, Modes::Overview, "Overview");
                        ui.selectable_value(&mut mode_value, Modes::Build, "Build");
                        ui.selectable_value(&mut mode_value, Modes::Destroy, "Destroy");
                    });
                ui.label("Rotation");
                egui::ComboBox::from_id_source("rotation")
                    .selected_text(format!("{:?}", direction_value))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut direction_value, Direction::North, "North");
                        ui.selectable_value(&mut direction_value, Direction::East, "East");
                        ui.selectable_value(&mut direction_value, Direction::South, "South");
                        ui.selectable_value(&mut direction_value, Direction::West, "West");
                        ui.selectable_value(&mut direction_value, Direction::Up, "Up");
                        ui.selectable_value(&mut direction_value, Direction::Down, "Down");
                    });
                ui.label("Block");
                egui::ComboBox::from_id_source("block")
                    .selected_text(format!("{:?}", block_value))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut block_value, BlockType::Debug, "Debug");
                        ui.selectable_value(&mut block_value, BlockType::Furnace, "Furnace");
                        ui.selectable_value(&mut block_value, BlockType::Conveyor, "Conveyor");
                        ui.selectable_value(&mut block_value, BlockType::Splitter, "Splitter");
                        ui.selectable_value(&mut block_value, BlockType::Storage, "Storage");
                        ui.selectable_value(&mut block_value, BlockType::Grabber, "Grabber");
                    });
            });
            block_selected_query.iter().for_each(|(block, _)| {
                ui.group(|ui| {
                    ui.heading("Selected Block");
                    ui.separator();
                    ui.label(format!("Block Type: {:?}", block.block_type));
                    ui.label(format!("Block Rotation: {:?}", block.direction));
                });
            });
        });

    spawn_options.player_mode = mode_value;
    spawn_options.block_rotation = direction_value;
    spawn_options.block_selection = block_value;
}
