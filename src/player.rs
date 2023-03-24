use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_mod_raycast::{DefaultPluginState, RaycastSource};

use crate::{blocks::BlockType, MyRaycastSet};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player);
        app.add_startup_system(setup_hud);
        app.add_system(player_controller);
        app.add_system(player_hotkeys);
        app.add_state::<Modes>();
        app.add_system(mode_ui_system);
        app.add_system(rotation_ui_system);
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
pub struct Player {}

#[derive(Component, Default)]
pub struct SpawnerOptions {
    pub block_selection: Option<BlockType>,
    pub block_rotation: Direction,
}

#[derive(Default, PartialEq)]
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

    commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());
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
        RaycastSource::<MyRaycastSet>::default(),
    ));
}

const MARGIN: Val = Val::Px(5.);

#[derive(Component)]
pub struct UICamera;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn((
            Name::new("Player UI"),
            NodeBundle {
                style: Style {
                    // fill the entire window
                    size: Size::all(Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..Default::default()
            },
        ))
        .with_children(|builder| {
            // spawn the key
            builder
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::top(MARGIN),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|builder| {
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                padding: UiRect {
                                    top: Val::Px(1.),
                                    left: Val::Px(5.),
                                    right: Val::Px(5.),
                                    bottom: Val::Px(1.),
                                },
                                ..Default::default()
                            },
                            background_color: BackgroundColor(Color::rgb(0.102, 0.522, 1.)),
                            ..Default::default()
                        })
                        .with_children(|builder| {
                            builder.spawn((
                                TextBundle::from_section(
                                    "Overview",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 24.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                ModeReadOut,
                            ));
                        });
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                padding: UiRect {
                                    top: Val::Px(1.),
                                    left: Val::Px(5.),
                                    right: Val::Px(5.),
                                    bottom: Val::Px(1.),
                                },
                                ..Default::default()
                            },
                            background_color: BackgroundColor(Color::rgb(0.102, 1., 0.14)),
                            ..Default::default()
                        })
                        .with_children(|builder| {
                            builder.spawn((
                                TextBundle::from_section(
                                    "North",
                                    TextStyle {
                                        font,
                                        font_size: 24.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                RotationReadOut,
                            ));
                        });
                });
        });
}

fn player_hotkeys(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut SpawnerOptions, With<Player>>,
    modes_state: Res<State<Modes>>,
    mut next_modes_state: ResMut<NextState<Modes>>,
) {
    for mut ele in query.iter_mut() {
        if keys.just_pressed(KeyCode::Key1) {
            ele.block_selection = Some(BlockType::Debug);
        } else if keys.just_pressed(KeyCode::Key2) {
            ele.block_selection = Some(BlockType::Furnace);
        } else if keys.just_pressed(KeyCode::Key3) {
            ele.block_selection = Some(BlockType::Conveyor);
        } else if keys.just_pressed(KeyCode::Key4) {
            ele.block_selection = Some(BlockType::Splitter);
        } else if keys.just_pressed(KeyCode::Key5) {
            ele.block_selection = Some(BlockType::Storage);
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
            next_modes_state.set(match modes_state.0 {
                Modes::Overview => Modes::Build,
                Modes::Build => Modes::Destroy,
                Modes::Destroy => Modes::Overview,
            });
        }
    }
}

fn mode_ui_system(
    mut text_query: Query<&mut Text, With<ModeReadOut>>,
    modes_state: Res<State<Modes>>,
) {
    for mut text in text_query.iter_mut() {
        text.sections[0].value = match modes_state.0 {
            Modes::Overview => "Overview",
            Modes::Build => "Build",
            Modes::Destroy => "Destroy",
        }
        .to_string();
    }
}

fn rotation_ui_system(
    mut text_query: Query<&mut Text, With<RotationReadOut>>,
    player_query: Query<&SpawnerOptions, With<Player>>,
) {
    let Ok(spawn_options) = player_query.get_single() else { return;};
    for mut text in text_query.iter_mut() {
        text.sections[0].value = match spawn_options.block_rotation {
            Direction::North => "North",
            Direction::East => "East",
            Direction::South => "South",
            Direction::West => "West",
            Direction::Up => "Up",
            Direction::Down => "Down",
        }
        .to_string();
    }
}
