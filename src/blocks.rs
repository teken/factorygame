use bevy::{math::vec3, prelude::*, utils::HashMap};
use bevy_mod_raycast::RaycastMesh;
use bevy_prototype_debug_lines::DebugShapes;

use crate::{
    materials::{self, ItemStack, Reaction},
    player::{self, Modes, Player, SpawnerOptions},
    reactions::PROCESS_IRON_TO_GOLD,
    MyRaycastSet,
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
        app.add_system(block_clicked_event_handler);
        app.add_system(highlight_selected_block);
        app.add_system(logger_system);
    }
}

#[derive(Component, Reflect)]
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub block_type: BlockType,
    pub direction: player::Direction,
}

#[derive(Component, Default)]
pub struct Input {
    pub accepts: Option<ItemStack>,
    pub inventory: Vec<ItemStack>,
}

#[derive(Component, Default)]
pub struct Output {
    pub inventory: Vec<ItemStack>,
}

#[derive(Component, Default, Reflect)]
pub struct LogInput;

#[derive(Component, Default, Reflect)]
pub struct LogOutput;

impl Output {
    pub fn contains(&self, accept: &ItemStack) -> bool {
        self.inventory
            .iter()
            .any(|item| item.item_type == accept.item_type && item.quantity >= accept.quantity)
    }

    pub fn transfer(&mut self, accept: &ItemStack, destination: &mut Vec<ItemStack>) {
        // todo : cover when requested quantity is more than a single stack size
        let Some(mut item) = self
            .inventory
            .iter_mut()
            .find(|item| item.item_type == accept.item_type && item.quantity >= accept.quantity)
            else {
                return;
        };
        item.quantity -= accept.quantity;

        let item_c = item.clone();
        if item.quantity == 0 {
            let index = self.inventory.iter().position(|x| x == &item_c);
            if let Some(index) = index {
                self.inventory.remove(index);
            }
        }
        // todo: cover when stack size is greater than 1 stack or the distation doesn't have space
        destination.push(accept.clone());
    }

    // todo: same as above
    pub fn transfer_first(&mut self, destination: &mut Vec<ItemStack>) {
        if self.inventory.is_empty() {
            return;
        }
        let item = self.inventory.remove(0);
        destination.push(item);
    }
}

#[derive(Component, Default)]
pub struct Process {
    pub reaction: Option<Reaction>,
    // pub time: f32,
    // pub timer: Timer,
}

#[derive(Debug, Clone, Reflect, Copy)]
pub enum BlockType {
    Debug,
    Furnace,
    Conveyor,
    Splitter,
    Storage,
    Grabber,
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
                    min: click_position.floor(),
                    max: click_position.ceil(),
                    block_type: BlockType::Debug,
                    direction: spawner_options.block_rotation.clone(),
                },
                RaycastMesh::<MyRaycastSet>::default(),
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
                    min: click_position.floor(),
                    max: click_position.ceil(),
                    block_type: BlockType::Furnace,
                    direction: spawner_options.block_rotation.clone(),
                },
                Input::default(),
                Output::default(),
                Process {
                    reaction: Some(PROCESS_IRON_TO_GOLD.clone()),
                },
                RaycastMesh::<MyRaycastSet>::default(),
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
                    min: click_position.floor(),
                    max: click_position.ceil(),
                    block_type: BlockType::Conveyor,
                    direction: spawner_options.block_rotation.clone(),
                },
                Input::default(),
                Output::default(),
                RaycastMesh::<MyRaycastSet>::default(),
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
                    min: click_position.floor(),
                    max: click_position.ceil() + vec3(0., 0., 1.),
                    block_type: BlockType::Splitter,
                    direction: spawner_options.block_rotation.clone(),
                },
                Input::default(),
                Output::default(),
                RaycastMesh::<MyRaycastSet>::default(),
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
                    min: click_position.floor(),
                    max: click_position.ceil(),
                    block_type: BlockType::Storage,
                    direction: spawner_options.block_rotation.clone(),
                },
                // Inventory::default(),
                Input::default(),
                Output {
                    inventory: vec![
                        materials::Element::Iron.to_item_stack(materials::State::Solid, 10)
                    ],
                },
                LogInput::default(),
                RaycastMesh::<MyRaycastSet>::default(),
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
                    min: click_position.floor(),
                    max: click_position.ceil(),
                    block_type: BlockType::Grabber,
                    direction: spawner_options.block_rotation.clone(),
                },
                RaycastMesh::<MyRaycastSet>::default(),
            )),
        };
    }
}

fn furnace_system(
    mut query: Query<(&mut Input, &mut Output, &mut Process), With<Furnace>>,
    // time: Res<Time>,
) {
    for (mut input, mut output, mut process) in query.iter_mut() {
        if !process.reaction.is_some() {
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

        // process.timer.tick(time.delta());
        // if !process.timer.finished() {
        //     continue;
        // }

        process
            .reaction
            .as_ref()
            .unwrap()
            .run(&mut input.inventory, &mut output.inventory);
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
            if !output.inventory.is_empty() && output.contains(&accepts) {
                output.transfer(&accepts, &mut input.inventory);
            }
        } else {
            output.transfer_first(&mut input.inventory);
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
            if !output.inventory.is_empty() && output.contains(&accepts) {
                output.transfer(&accepts, &mut input.inventory);
            }
        } else {
            output.transfer_first(&mut input.inventory);
        }
    }
}

fn display_build_ghost_system(
    objects_query: Query<(&Block, Entity)>,
    mode_states: Res<State<Modes>>,
    mut shapes: ResMut<DebugShapes>,
    intersect_query: Query<&bevy_mod_raycast::Intersection<MyRaycastSet>>,
    player_query: Query<&SpawnerOptions, With<Player>>,
) {
    if mode_states.0 != Modes::Build {
        return;
    }

    let Ok(inter) = intersect_query.get_single() else {
        return;
    };

    let Some(position) = inter.position() else {
        return;
    };

    let mod_coord = |c: f32| {
        if (c - c.floor()).abs() < 0.001 {
            c.floor()
        } else if (c - c.ceil()).abs() < 0.001 {
            c.ceil()
        } else {
            c
        }
    };

    // a function that take 3 floats and return the number closest to the whole number
    // let d = |x: f32, y: f32, z: f32| -> player::Direction {
    //     let f_x = (x - x.floor()).abs();
    //     let f_y = (y - y.floor()).abs();
    //     let f_z = (z - z.floor()).abs();
    //     let c_x = (x - x.ceil()).abs();
    //     let c_y = (y - y.ceil()).abs();
    //     let c_z = (z - z.ceil()).abs();

    //     // println!("{} {} {} : {} {} {}", f_x, f_y, f_z, c_x, c_y, c_z);

    //     return if f_x == c_x {
    //         player::Direction::North
    //     } else if f_y == c_y {
    //         player::Direction::Up
    //     } else {
    //         //if f_z == c_z
    //         player::Direction::East
    //     };
    // };

    let modified_x = mod_coord(position.x);
    let modified_y = mod_coord(position.y);
    let modified_z = mod_coord(position.z);

    // let di = d(modified_x, modified_y, modified_z);

    let current_block = objects_query.iter().find(|(block, _)| {
        modified_x >= block.min.x
            && modified_x <= block.max.x
            && modified_y >= block.min.y
            && modified_y <= block.max.y
            && modified_z >= block.min.z
            && modified_z <= block.max.z
    });

    if current_block.is_some() {
        return;
    }

    // println!("{} {} {}", modified_x, modified_y, modified_z);

    // use the position detect which face of the block was clicked

    let modified_position = vec3(modified_x, modified_y, modified_z);
    // shapes.cuboid().min_max(
    //     modified_position.floor(),
    //     (modified_position
    //         + match di {
    //             player::Direction::North => vec3(0.1, 0., 0.),
    //             player::Direction::South => vec3(-0.1, 0., 0.),
    //             player::Direction::East => vec3(0., 0., 0.1),
    //             player::Direction::West => vec3(0., 0., -0.1),
    //             player::Direction::Up => vec3(0., 0.1, 0.),
    //             player::Direction::Down => vec3(0., -0.1, 0.),
    //         })
    //     .ceil(),
    // );

    let Ok(spawner_options) = player_query.get_single() else {
        return;
    };

    let base = modified_position.floor() + vec3(0.5, 0.5, 0.5);

    match spawner_options.block_rotation {
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

fn block_clicked_event_handler(
    mut commands: Commands,
    mut ev_blockclicked: EventReader<BlockClickedEvent>,
    objects_query: Query<(&Block, Entity)>,
    current_selected_query: Query<(&Block, Entity), With<BlockClicked>>,
) {
    for ele in ev_blockclicked.iter() {
        let i = ele.world_pos;
        let Some(clicked) = objects_query.iter().find(|(block, _)| {
            i.x >= block.min.x
                && i.x <= block.max.x
                && i.y >= block.min.y
                && i.y <= block.max.y
                && i.z >= block.min.z
                && i.z <= block.max.z
        }) else {
            return;
        };
        for ele in current_selected_query.iter() {
            commands.entity(ele.1).remove::<BlockClicked>();
        }
        commands.entity(clicked.1).insert(BlockClicked {});
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

pub struct BlockClickedEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
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
