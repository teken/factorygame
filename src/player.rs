use std::{
    f32::consts::PI,
    fmt::{Debug, Display},
};

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::PickingCameraBundle;
use enum_iterator::{all, Sequence};

use crate::{
    blocks::BlockType,
    components::{self, Block, BlockClicked, Process},
    grid::GridSelectMode,
    materials::{self, Element, Energy, Inventory, Reaction},
    reactions::PROCESS_IRON_TO_GOLD,
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

#[derive(Default, Reflect, PartialEq, Clone, Debug, Sequence)]
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
    #[inline]
    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Direction {
    pub fn to_quat(&self) -> Quat {
        match self {
            Direction::North => Quat::from_rotation_y(0.0),
            Direction::East => Quat::from_rotation_y(PI * 1.5),
            Direction::South => Quat::from_rotation_y(PI),
            Direction::West => Quat::from_rotation_y(PI * 0.5),
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
            let window = get_primary_window_size(windows);
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
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(windows);
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
    Vec2::new(window.width(), window.height())
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
            camera: Camera {
                ..Default::default()
            },
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
        } else if keys.just_pressed(KeyCode::Escape) {
            ele.player_mode = Modes::Overview;
            ele.grid_select_mode = GridSelectMode::Block;
        }
    }
}

#[derive(Default)]
struct UiState {
    selected_quantity: u32,
    selected_element: Element,
    selected_state: materials::State,
    selected_energy: Energy,
    selected_reaction: Option<Reaction>,
}

fn dev_ui(
    mut egui_ctx: EguiContexts,
    mut player_query: Query<&mut SpawnerOptions, With<Player>>,
    block_selected_query: Query<(&Block, Entity), With<BlockClicked>>,
    mut process_selected_query: Query<&mut Process, With<BlockClicked>>,
    mut input_selected_query: Query<&mut components::Input, With<BlockClicked>>,
    mut output_selected_query: Query<&mut components::Output, With<BlockClicked>>,
    mut ui_state: Local<UiState>,
) {
    let Ok(mut spawn_options) = player_query.get_single_mut() else { return; };

    egui::SidePanel::right("selected_block_panel")
        .default_width(200.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.group(|ui| {
                ui.heading("Player Settings");
                ui.separator();
                ui.label(format!("Mode (Q): {:?}", spawn_options.player_mode));
                ui.label(format!("Grid Mode: {:?}", spawn_options.grid_select_mode));

                enum_dropdown::<Direction>(
                    ui,
                    "rot".to_string(),
                    "Rotation (R)",
                    &mut spawn_options.block_rotation,
                );
                enum_dropdown::<BlockType>(
                    ui,
                    "bt".to_string(),
                    "Block (Num Keys)",
                    &mut spawn_options.block_selection,
                );
            });
            block_selected_query.iter().for_each(|(block, ent)| {
                ui.group(|ui| {
                    ui.heading("Selected Block");
                    ui.separator();
                    ui.label(format!("Block Type: {:?}", block.block_type));
                    ui.label(format!("Block Rotation: {:?}", block.direction));

                    if let Ok(mut process) = process_selected_query.get_mut(ent) {
                        ui.heading("Process");
                        if process.reaction.is_some() {
                            ui.add(
                                egui::ProgressBar::new(process.timer.percent())
                                    .animate(process.timer.percent() > 0.),
                            );
                        }
                        if let BlockType::Furnace = block.block_type {
                            egui::ComboBox::from_id_source("furance_process")
                                .selected_text(match &ui_state.selected_reaction {
                                    Some(reaction) => reaction.to_string(),
                                    None => "None".to_string(),
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut ui_state.selected_reaction,
                                        None,
                                        "None",
                                    );
                                    ui.selectable_value(
                                        &mut ui_state.selected_reaction,
                                        Some(PROCESS_IRON_TO_GOLD.clone()),
                                        format!("{}", PROCESS_IRON_TO_GOLD.clone()),
                                    );
                                });
                        }
                        if ui_state.selected_reaction.is_some()
                            && process.reaction != ui_state.selected_reaction
                        {
                            process.set_reaction(ui_state.selected_reaction.as_ref().unwrap());
                        }
                    }

                    if let Ok(mut input) = input_selected_query.get_mut(ent) {
                        ui.heading("Input");
                        inventory_table(
                            ui,
                            &mut ui_state,
                            "input".to_string(),
                            &mut input.inventory,
                        );
                    }
                    if let Ok(mut output) = output_selected_query.get_mut(ent) {
                        ui.heading("Output");
                        inventory_table(
                            ui,
                            &mut ui_state,
                            "output".to_string(),
                            &mut output.inventory,
                        );
                    }
                });
            });
        });
}

#[inline]
fn inventory_table(
    ui: &mut egui::Ui,
    ui_state: &mut Local<UiState>,
    id: String,
    inventory: &mut Inventory,
) {
    ui.horizontal(|ui| {
        ui.label("Item");
        ui.separator();
        ui.label("Amount");
    });
    ui.separator();
    for stack in inventory.items.iter() {
        ui.horizontal(|ui| {
            ui.label(format!("{}", stack.item_type));
            ui.separator();
            ui.label(format!("{}", stack.quantity));
        });
    }
    ui.collapsing(format!("Add {} Item", id), |ui| {
        ui.add(
            egui::DragValue::new(&mut ui_state.selected_quantity)
                .speed(0.1)
                .clamp_range(1..=64),
        );
        ui.horizontal(|ui| {
            enum_dropdown::<Element>(
                ui,
                format!("{}-el", id),
                "Element",
                &mut ui_state.selected_element,
            );
            enum_dropdown::<materials::State>(
                ui,
                format!("{}-st", id),
                "State",
                &mut ui_state.selected_state,
            );
            if ui.button("Add").clicked() {
                inventory.push(
                    ui_state
                        .selected_element
                        .clone()
                        .to_item_stack(ui_state.selected_state.clone(), ui_state.selected_quantity),
                );
            }
        });
        ui.horizontal(|ui| {
            enum_dropdown::<Energy>(
                ui,
                format!("{}-en", id),
                "Energy",
                &mut ui_state.selected_energy,
            );
            if ui.button("Add").clicked() {
                inventory.push(
                    ui_state
                        .selected_energy
                        .clone()
                        .to_item_stack(ui_state.selected_quantity),
                );
            }
        });
    });
}

#[inline]
fn enum_dropdown<T: Sequence + PartialEq + Display + Clone + Debug>(
    ui: &mut egui::Ui,
    id: String,
    label: &str,
    value: &mut T,
) {
    ui.label(label);
    egui::ComboBox::from_id_source(id)
        .selected_text(format!("{}", value))
        .show_ui(ui, |ui| {
            for option in all::<T>().collect::<Vec<T>>() {
                ui.selectable_value(value, option.clone(), format!("{}", option));
            }
        });
}
