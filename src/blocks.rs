use bevy::{math::vec3, prelude::*, render::primitives::Aabb};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_debug_lines::DebugShapes;
use enum_iterator::Sequence;
use std::fmt::Display;

use crate::{
    grid::GridCellHoveredEvent,
    materials::{Inventory, ItemStack, Reaction},
    player::{self, Modes, Player, SpawnerOptions},
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(furnace_system);
        app.add_system(internal_conveyor_system);
        app.add_system(external_conveyor_system);
        // app.add_system(input_feed_system);
        app.add_system(grabber_system);
        app.add_system(display_build_ghost_system);
        app.add_system(highlight_selected_block);
        app.add_system(logger_system);
        app.add_system(display_dep_chains);
        app.register_type::<Block>()
            .register_type::<Input>()
            .register_type::<Output>()
            .register_type::<Process>();
    }
}

#[derive(Component, Reflect)]
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub block_type: BlockType,
    pub direction: player::Direction,
}

#[derive(Component, Default, Reflect, Debug)]
pub struct Input {
    pub accepts: Option<ItemStack>,
    pub inventory: Inventory,
}

#[derive(Component, Default, Reflect, Debug)]
pub struct Output {
    pub inventory: Inventory,
}

#[derive(Component, Default, Reflect)]
pub struct LogInput;

#[derive(Component, Default, Reflect)]
pub struct LogOutput;

#[derive(Component, Default, Reflect)]
pub struct Process {
    pub reaction: Option<Reaction>,
    pub timer: Timer,
}

impl Process {
    pub fn set_reaction(&mut self, reaction: &Reaction) {
        self.reaction = Some(reaction.clone());
        self.timer = Timer::new(reaction.duration.clone(), TimerMode::Repeating);
    }
}

#[derive(Debug, Clone, Reflect, Copy, Default, PartialEq, Eq, Hash, Sequence)]
pub enum BlockType {
    #[default]
    Debug,
    Furnace,
    Conveyor,
    Splitter,
    Storage,
    Grabber,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component, Default)]
pub struct Furnace;

#[derive(Component, Default)]
pub struct Conveyor;

#[derive(Component, Default)]
pub struct Splitter;

#[derive(Component, Default)]
pub struct Storage;

#[derive(Component, Default)]
pub struct Grabber;

pub trait Spawn {
    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut Assets<StandardMaterial>,
        asset_server: &Res<AssetServer>,
        spawner_options: &SpawnerOptions,
        click_position: Vec3,
    );
}

impl Spawn for BlockType {
    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut Assets<StandardMaterial>,
        asset_server: &Res<AssetServer>,
        spawner_options: &SpawnerOptions,
        click_position: Vec3,
    ) {
        let default_block = Block {
            min: click_position.floor(),
            max: click_position.ceil(),
            block_type: BlockType::Debug,
            direction: spawner_options.block_rotation.clone(),
        };
        match self {
            BlockType::Debug => commands.spawn((
                SceneBundle {
                    scene: asset_server.load(r"models\test.gltf#Scene0"),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 0.5, 0.5),
                    )
                    .with_rotation(spawner_options.block_rotation.to_quat()),
                    ..default()
                },
                Name::new("Debug Block"),
                Block {
                    block_type: BlockType::Debug,
                    ..default_block
                },
                PickableBundle::default(),
            )),
            BlockType::Furnace => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 0.5, 0.5),
                    )
                    .with_rotation(spawner_options.block_rotation.to_quat()),
                    ..default()
                },
                Name::new("Furnace"),
                Furnace::default(),
                Block {
                    block_type: BlockType::Furnace,
                    ..default_block
                },
                Input::default(),
                Output::default(),
                Process::default(),
                PickableBundle::default(),
            )),
            BlockType::Conveyor => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Box::new(1.0, 0.2, 0.2).into()),
                    material: materials.add(Color::BLUE.into()),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 0.5, 0.5),
                    )
                    .with_rotation(spawner_options.block_rotation.to_quat()),
                    ..default()
                },
                Name::new("Conveyor"),
                Conveyor::default(),
                Block {
                    block_type: BlockType::Conveyor,
                    ..default_block
                },
                Input::default(),
                Output::default(),
                PickableBundle::default(),
            )),
            BlockType::Splitter => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Box::new(1.0, 1.0, 2.0).into()),
                    material: materials.add(Color::GREEN.into()),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 0.5, 0.),
                    )
                    .with_rotation(spawner_options.block_rotation.to_quat()),
                    ..default()
                },
                Name::new("Splitter"),
                Splitter::default(),
                Block {
                    max: click_position.ceil() + vec3(0., 0., 1.),
                    block_type: BlockType::Splitter,
                    ..default_block
                },
                Input::default(),
                Output::default(),
                PickableBundle::default(),
            )),
            BlockType::Storage => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Box::new(1.0, 0.8, 1.0).into()),
                    material: materials.add(Color::YELLOW.into()),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 0.4, 0.5),
                    )
                    .with_rotation(spawner_options.block_rotation.to_quat()),
                    ..default()
                },
                Name::new("Storage"),
                Storage::default(),
                Block {
                    block_type: BlockType::Storage,
                    ..default_block
                },
                // Inventory::default(),
                Input::default(),
                Output::default(),
                PickableBundle::default(),
            )),
            BlockType::Grabber => commands.spawn((
                SceneBundle {
                    scene: asset_server.load(r"models\grabber.gltf#Scene0"),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 0.5, 0.5),
                    )
                    .with_rotation(spawner_options.block_rotation.to_quat()),
                    ..default()
                },
                Name::new("Grabber Block"),
                Grabber::default(),
                Block {
                    block_type: BlockType::Grabber,
                    ..default_block
                },
                PickableBundle::default(),
            )),
        };
    }
}

fn furnace_system(
    mut query: Query<(&mut Input, &mut Output, &mut Process), With<Furnace>>,
    time: Res<Time>,
) {
    for (mut input, mut output, mut process) in query.iter_mut() {
        if process.reaction.is_none() {
            continue;
        };

        if !process
            .reaction
            .as_ref()
            .unwrap()
            .valid_input(&input.inventory)
        {
            continue;
        }

        process.timer.tick(time.delta());
        if process.timer.just_finished() {
            process
                .reaction
                .as_ref()
                .unwrap()
                .run(&mut input.inventory, &mut output.inventory);
            process.timer.reset();
        }
    }
}

fn internal_conveyor_system(mut query: Query<(&mut Input, &mut Output), With<Conveyor>>) {
    for (mut input, mut output) in query.iter_mut() {
        if let Some(item) = input.inventory.pop() {
            output.inventory.push(item);
        }
    }
}

fn external_conveyor_system(
    mut input_query: Query<(&Block, &mut Input), With<Conveyor>>,
    mut output_query: Query<(&Block, &mut Output), With<Conveyor>>,
) {
    for (block, mut input) in input_query.iter_mut() {
        let output;
        match block.direction {
            player::Direction::North => {
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x - 1. && t.min.y == block.min.y && t.min.z == block.min.z
                });
            }
            player::Direction::South => {
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x + 1. && t.min.y == block.min.y && t.min.z == block.min.z
                });
            }
            player::Direction::East => {
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y && t.min.z == block.min.z - 1.
                });
            }
            player::Direction::West => {
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y && t.min.z == block.min.z + 1.
                });
            }
            player::Direction::Up => {
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y - 1. && t.min.z == block.min.z
                });
            }
            player::Direction::Down => {
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y + 1. && t.min.z == block.min.z
                });
            }
        }

        let Some((_, mut output)) = output else {
            return;
        };

        if let Some(accepts) = input.accepts.clone() {
            if !output.inventory.is_empty() && output.inventory.contains(&accepts) {
                output.inventory.transfer(&accepts, &mut input.inventory);
            }
        } else {
            output.inventory.transfer_first(&mut input.inventory);
        }
    }
}

fn grabber_system(
    grabber_query: Query<&Block, With<Grabber>>,
    mut input_query: Query<(&Block, &mut Input)>,
    mut output_query: Query<(&Block, &mut Output)>,
) {
    for block in grabber_query.iter() {
        let input;
        let output;
        match block.direction {
            player::Direction::North => {
                input = input_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x + 1. && t.min.y == block.min.y && t.min.z == block.min.z
                });
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x - 1. && t.min.y == block.min.y && t.min.z == block.min.z
                });
            }
            player::Direction::South => {
                input = input_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x - 1. && t.min.y == block.min.y && t.min.z == block.min.z
                });
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x + 1. && t.min.y == block.min.y && t.min.z == block.min.z
                });
            }
            player::Direction::East => {
                input = input_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y && t.min.z == block.min.z + 1.
                });
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y && t.min.z == block.min.z - 1.
                });
            }
            player::Direction::West => {
                input = input_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y && t.min.z == block.min.z - 1.
                });
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y && t.min.z == block.min.z + 1.
                });
            }
            player::Direction::Up => {
                input = input_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y + 1. && t.min.z == block.min.z
                });
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y - 1. && t.min.z == block.min.z
                });
            }
            player::Direction::Down => {
                input = input_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y - 1. && t.min.z == block.min.z
                });
                output = output_query.iter_mut().find(|(t, _)| {
                    t.min.x == block.min.x && t.min.y == block.min.y + 1. && t.min.z == block.min.z
                });
            }
        }

        let Some((_, mut input)) = input else {
            return;
        };

        let Some((_, mut output)) = output else {
            return;
        };

        if let Some(accepts) = input.accepts.clone() {
            if !output.inventory.is_empty() && output.inventory.contains(&accepts) {
                output.inventory.transfer(&accepts, &mut input.inventory);
            }
        } else {
            output.inventory.transfer_first(&mut input.inventory);
        }
    }
}

fn display_build_ghost_system(
    mut shapes: ResMut<DebugShapes>,
    player_query: Query<&SpawnerOptions, With<Player>>,
    mut grid_cell_hover_events: EventReader<GridCellHoveredEvent>,
) {
    let Ok(spawner_opts) = player_query.get_single() else {
        return;
    };

    if spawner_opts.player_mode != Modes::Build {
        return;
    }

    for ele in grid_cell_hover_events.iter() {
        let base = ele.grid_cell.floor() + vec3(0.5, 0.5, 0.5);

        match spawner_opts.block_rotation {
            player::Direction::North => {
                shapes
                    .cuboid()
                    .min_max(base + vec3(0.3, 0.1, 0.1), base + vec3(-0.3, -0.1, -0.1));

                shapes
                    .cuboid()
                    .min_max(base + vec3(0.5, 0.5, 0.5), base + vec3(0.3, -0.5, -0.5));
            }
            player::Direction::South => {
                shapes
                    .cuboid()
                    .min_max(base + vec3(0.3, 0.1, 0.1), base + vec3(-0.3, -0.1, -0.1));

                shapes
                    .cuboid()
                    .min_max(base + vec3(-0.5, 0.5, 0.5), base + vec3(-0.3, -0.5, -0.5));
            }
            player::Direction::East => {
                shapes
                    .cuboid()
                    .min_max(base + vec3(0.1, 0.1, 0.3), base + vec3(-0.1, -0.1, -0.3));

                shapes
                    .cuboid()
                    .min_max(base + vec3(0.5, 0.5, 0.5), base + vec3(-0.5, -0.5, 0.3));
            }
            player::Direction::West => {
                shapes
                    .cuboid()
                    .min_max(base + vec3(0.1, 0.1, 0.3), base + vec3(-0.1, -0.1, -0.3));

                shapes
                    .cuboid()
                    .min_max(base + vec3(0.5, 0.5, -0.5), base + vec3(-0.5, -0.5, -0.3));
            }
            player::Direction::Up => {
                shapes
                    .cuboid()
                    .min_max(base + vec3(0.1, 0.3, 0.1), base + vec3(-0.1, -0.3, -0.1));

                shapes
                    .cuboid()
                    .min_max(base + vec3(0.5, 0.5, 0.5), base + vec3(-0.5, 0.3, -0.5));
            }
            player::Direction::Down => {
                shapes
                    .cuboid()
                    .min_max(base + vec3(0.1, 0.3, 0.1), base + vec3(-0.1, -0.3, -0.1));

                shapes
                    .cuboid()
                    .min_max(base + vec3(0.5, -0.5, 0.5), base + vec3(-0.5, -0.3, -0.5));
            }
        }
    }
}

fn highlight_selected_block(
    objects_query: Query<(&Block, Entity), With<BlockClicked>>,
    mut shapes: ResMut<DebugShapes>,
) {
    for (block, _) in objects_query.iter() {
        shapes
            .cuboid()
            .min_max(block.min, block.max)
            .color(Color::rgba(0.0, 0.0, 1.0, 0.5))
            .duration(0.);
    }
}

#[derive(Component)]
pub struct BlockClicked {}

fn logger_system(
    input_query: Query<(&Input, Entity), With<LogInput>>,
    output_query: Query<(&Output, Entity), With<LogOutput>>,
) {
    for (input, ent) in input_query.iter().filter(|x| !x.0.inventory.is_empty()) {
        println!("INPUT {:?}: {:#?}", ent, input.inventory);
    }

    for (output, ent) in output_query.iter().filter(|x| !x.0.inventory.is_empty()) {
        println!("OUTPUT {:?}: {:#?}", ent, output.inventory);
    }
}

fn display_dep_chains(
    mut shapes: ResMut<DebugShapes>,
    input_query: Query<(&GlobalTransform, &Aabb, &Block, Entity), With<Input>>,
    output_query: Query<(&GlobalTransform, &Aabb, &Block, Entity), With<Output>>,
) {
    for (trans, aabb, block, _) in input_query.iter() {
        let output = output_query.iter().find(|(_, _, b, _)| {
            // entity != *e
            match block.direction {
                player::Direction::North => {
                    b.min.x == block.min.x - 1. && b.min.y == block.min.y && b.min.z == block.min.z
                }
                player::Direction::South => {
                    b.min.x == block.min.x + 1. && b.min.y == block.min.y && b.min.z == block.min.z
                }
                player::Direction::East => {
                    b.min.x == block.min.x && b.min.y == block.min.y && b.min.z == block.min.z - 1.
                }
                player::Direction::West => {
                    b.min.x == block.min.x && b.min.y == block.min.y && b.min.z == block.min.z + 1.
                }
                player::Direction::Up => {
                    b.min.x == block.min.x && b.min.y == block.min.y - 1. && b.min.z == block.min.z
                }
                player::Direction::Down => {
                    b.min.x == block.min.x && b.min.y == block.min.y + 1. && b.min.z == block.min.z
                }
            }
        });

        let Some((o_t,o_a,_, _)) = output else {
            continue;
        };

        // println!("{:?} -> {:?}", entity, o_entity);

        shapes
            .line()
            .start_end(
                trans.transform_point(aabb.center.into()),
                o_t.transform_point(o_a.center.into()),
            )
            .gradient(Color::RED, Color::GREEN);
    }
}
