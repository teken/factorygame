use bevy::{
    math::{vec3, Vec3A},
    prelude::*,
    render::primitives::Aabb,
};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_debug_lines::DebugShapes;
use enum_iterator::Sequence;
use std::fmt::Display;

use crate::{
    components::{
        self, Block, BlockClicked, Conveyor, Furnace, Grabber, Output, Process, Splitter, Storage,
    },
    grid::GridCellHoveredEvent,
    player::{self, Modes, Player, SpawnerOptions},
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(furnace_system);
        app.add_system(internal_conveyor_system);
        app.add_system(external_conveyor_system);
        app.add_system(grabber_system);
        app.add_system(display_build_ghost_system);
        app.add_system(highlight_selected_block);
    }
}

pub fn is_next_block_in_direction(
    a: (&Aabb, &GlobalTransform),
    b: (&Aabb, &GlobalTransform),
    direction: player::Direction,
) -> bool {
    let mut target_vec = a.1.transform_point(a.0.center.into());
    match direction {
        player::Direction::North => {
            target_vec.x = (target_vec.x + a.0.half_extents.x).ceil() + 0.5;
        }
        player::Direction::South => {
            target_vec.x = (target_vec.x - a.0.half_extents.x).floor() - 0.5;
        }
        player::Direction::East => {
            target_vec.z = (target_vec.z + a.0.half_extents.z).ceil() + 0.5;
        }
        player::Direction::West => {
            target_vec.z = (target_vec.z - a.0.half_extents.z).floor() - 0.5;
        }
        player::Direction::Up => {
            target_vec.y = (target_vec.y + a.0.half_extents.y).ceil() + 0.5;
        }
        player::Direction::Down => {
            target_vec.y = (target_vec.y - a.0.half_extents.y).floor() - 0.5;
        }
    }

    let block_aabb = (
        b.1.transform_point(b.0.min().into()).floor(),
        b.1.transform_point(b.0.max().into()).ceil(),
    );

    target_vec.x >= block_aabb.0.x
        && target_vec.x <= block_aabb.1.x
        && target_vec.y >= block_aabb.0.y
        && target_vec.y <= block_aabb.1.y
        && target_vec.z >= block_aabb.0.z
        && target_vec.z <= block_aabb.1.z
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
                Aabb {
                    half_extents: Vec3A::new(0.5, 0.5, 0.5),
                    ..Default::default()
                },
            )),
            BlockType::Furnace => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(3.).into()),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_translation(
                        click_position.floor() + vec3(0.5, 1.5, 0.5),
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
                components::Input::default(),
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
                components::Input::default(),
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
                    block_type: BlockType::Splitter,
                    ..default_block
                },
                components::Input::default(),
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
                components::Input::default(),
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
                Aabb {
                    half_extents: Vec3A::new(0.5, 0.5, 0.5),
                    ..Default::default()
                },
            )),
        };
    }
}

fn furnace_system(
    mut query: Query<(&mut components::Input, &mut Output, &mut Process), With<Furnace>>,
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

fn internal_conveyor_system(
    mut query: Query<(&mut components::Input, &mut Output, &mut Conveyor)>,
    time: Res<Time>,
) {
    for (mut input, mut output, mut conveyor) in query.iter_mut() {
        conveyor.timer.tick(time.delta());
        if conveyor.timer.finished() {
            if let Some(item) = input.inventory.pop() {
                output.inventory.push(item);
            }
            conveyor.timer.reset();
        }
    }
}

fn external_conveyor_system(
    mut input_query: Query<
        (&Aabb, &GlobalTransform, &Block, &mut components::Input),
        With<Conveyor>,
    >,
    mut output_query: Query<(&Aabb, &GlobalTransform, &Block, &mut Output), With<Conveyor>>,
) {
    for (aabb, trans, block, mut input) in input_query.iter_mut() {
        let output = output_query.iter_mut().find(|(ab, tr, _, _)| {
            is_next_block_in_direction((aabb, trans), (ab, tr), block.direction.reverse())
        });

        let Some((_,_,_, mut output)) = output else {
            continue;
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
    grabber_query: Query<(&Block, &Aabb, &GlobalTransform), With<Grabber>>,
    mut input_query: Query<(&Aabb, &GlobalTransform, &mut components::Input)>,
    mut output_query: Query<(&Aabb, &GlobalTransform, &mut Output)>,
    // mut debug_lines: ResMut<DebugShapes>,
) {
    for (block, aabb, trans) in grabber_query.iter() {
        let input = input_query.iter_mut().find(|(ab, tr, _)| {
            is_next_block_in_direction((aabb, trans), (ab, tr), block.direction.clone())
        });
        let output = output_query.iter_mut().find(|(ab, tr, _)| {
            is_next_block_in_direction((aabb, trans), (ab, tr), block.direction.reverse())
        });

        let Some((_, _, mut input)) = input else {
            continue;
        };

        let Some((_, _, mut output)) = output else {
            continue;
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
    objects_query: Query<(&Aabb, &GlobalTransform, Entity), With<BlockClicked>>,
    mut shapes: ResMut<DebugShapes>,
) {
    for (ab, trans, _) in objects_query.iter() {
        shapes
            .cuboid()
            .min_max(
                trans.transform_point(ab.min().into()).floor(),
                trans.transform_point(ab.max().into()).ceil(),
            )
            .color(Color::rgba(0.0, 0.0, 1.0, 0.5))
            .duration(0.);
    }
}
