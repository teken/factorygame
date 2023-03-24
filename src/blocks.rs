use bevy::{math::vec3, prelude::*, render::primitives::Aabb};
use bevy_mod_raycast::RaycastMesh;
use bevy_prototype_debug_lines::DebugShapes;

use crate::{
    materials::{Item, Reaction},
    player::{self, Modes, SpawnerOptions},
    MyRaycastSet,
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(furnace_system);
        app.add_system(conveyor_system);
        app.add_system(input_feed_system);
        app.add_system(display_build_ghost_system);
        app.add_system(block_clicked_event_handler);
        app.add_system(highlight_selected_block);
    }
}

#[derive(Component)]
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub block_type: BlockType,
}

#[derive(Component, Default)]
pub struct Input {
    pub output_entity: Option<Entity>,
    pub accepts: Option<Item>,
    pub inventory: Vec<Item>,
}

#[derive(Component, Default)]
pub struct Output {
    pub inventory: Vec<Item>,
}

impl Output {
    pub fn contains(&self, accept: &Item) -> bool {
        self.inventory.iter().any(|item| {
            item.material == accept.material
                && item.energy == accept.energy
                && item.quantity >= accept.quantity
        })
    }

    pub fn transfer(&mut self, accept: &Item, destination: &mut Vec<Item>) {
        let mut item = self
            .inventory
            .iter_mut()
            .find(|item| {
                item.material == accept.material
                    && item.energy == accept.energy
                    && item.quantity >= accept.quantity
            })
            .unwrap();
        item.quantity -= accept.quantity;
        destination.push(accept.clone());
    }

    pub fn transfer_first(&mut self, destination: &mut Vec<Item>) {
        let item = self.inventory.remove(0);
        destination.push(item);
    }
}

#[derive(Component, Default)]
pub struct Process {
    pub reaction: Option<Reaction>,
    pub time: f32,
    pub timer: Timer,
}

#[derive(Debug, Clone)]
pub enum BlockType {
    Debug,
    Furnace,
    Conveyor,
    Splitter,
    Storage,
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
struct Inventory {
    items: Vec<Item>,
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
                },
                Input::default(),
                Output::default(),
                Process::default(),
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
                },
                Inventory::default(),
                Input::default(),
                Output::default(),
                RaycastMesh::<MyRaycastSet>::default(),
            )),
        };
    }
}

fn furnace_system(
    mut query: Query<(&mut Input, &mut Output, &mut Process), With<Furnace>>,
    time: Res<Time>,
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

        process.timer.tick(time.delta());
        if !process.timer.finished() {
            continue;
        }

        process
            .reaction
            .as_ref()
            .unwrap()
            .run(&mut input.inventory, &mut output.inventory);
    }
}

fn conveyor_system(mut query: Query<(&mut Input, &mut Output), With<Conveyor>>) {
    for (mut input, mut output) in query.iter_mut() {
        if let Some(item) = input.inventory.pop() {
            output.inventory.push(item);
        }
    }
}

fn input_feed_system(
    mut input_query: Query<&mut Input, With<Block>>,
    mut output_query: Query<(&mut Output, Entity), With<Block>>,
) {
    for mut input in input_query.iter_mut() {
        let Some(entity_id) = input.output_entity else {
            continue;
        };

        let Ok((mut output, _)) = output_query.get_mut(entity_id) else {
            continue;
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
    let d = |x: f32, y: f32, z: f32| -> player::Direction {
        let f_x = (x - x.floor()).abs();
        let f_y = (y - y.floor()).abs();
        let f_z = (z - z.floor()).abs();
        let c_x = (x - x.ceil()).abs();
        let c_y = (y - y.ceil()).abs();
        let c_z = (z - z.ceil()).abs();

        // println!("{} {} {} : {} {} {}", f_x, f_y, f_z, c_x, c_y, c_z);

        return if f_x == c_x {
            player::Direction::North
        } else if f_y == c_y {
            player::Direction::Up
        } else {
            //if f_z == c_z
            player::Direction::East
        };
    };

    let modified_x = mod_coord(position.x);
    let modified_y = mod_coord(position.y);
    let modified_z = mod_coord(position.z);

    let di = d(modified_x, modified_y, modified_z);

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

    let modified_position = vec3(modified_x, modified_y, position.z);
    shapes.cuboid().min_max(
        modified_position.floor(),
        (modified_position
            + match di {
                player::Direction::North => vec3(0.1, 0., 0.),
                player::Direction::South => vec3(-0.1, 0., 0.),
                player::Direction::East => vec3(0., 0., 0.1),
                player::Direction::West => vec3(0., 0., -0.1),
                player::Direction::Up => vec3(0., 0.1, 0.),
                player::Direction::Down => vec3(0., -0.1, 0.),
            })
        .ceil(),
    );
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
