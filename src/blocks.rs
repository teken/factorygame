use bevy::{math::vec3, prelude::*, window::PrimaryWindow};
use bevy_prototype_debug_lines::DebugShapes;

use crate::{
    materials::{Item, Reaction},
    player::{self, Modes, Player, SpawnerOptions},
    BuildPlane,
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(furnace_system);
        app.add_system(conveyor_system);
        app.add_system(input_feed_system);
        app.add_system(display_build_host_system);
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
            )),
        };
    }
}

fn furnace_system(mut query: Query<(&mut Input, &mut Output, &Process), With<Furnace>>) {
    for (mut input, mut output, process) in query.iter_mut() {
        let Some(reaction) = &process.reaction else {
            continue;
        };

        if !reaction.valid_input(&input.inventory) {
            continue;
        }

        reaction.run(&mut input.inventory, &mut output.inventory);
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

fn display_build_host_system(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<Player>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    build_plane_query: Query<(&GlobalTransform, Entity), With<BuildPlane>>,
    objects_query: Query<(&Block, Entity)>,
    mode_states: Res<State<Modes>>,
    mut shapes: ResMut<DebugShapes>,
) {
    if mode_states.0 != Modes::Build {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
            return;
        };
    let Ok(primary) = primary_query.get_single() else {
            return;
        };

    let Some(cursor_position) = primary.cursor_position() else {
            return;
        };

    let Some(ray) = camera
        .viewport_to_world(camera_transform, cursor_position) else {
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

    let current_block = objects_query.iter().find(|(block, _)| {
        i.x >= block.min.x
            && i.x <= block.max.x
            && i.y >= block.min.y
            && i.y <= block.max.y
            && i.z >= block.min.z
            && i.z <= block.max.z
    });

    if current_block.is_some() {
        return;
    }

    shapes.cuboid().min_max(i.floor(), i.ceil());
}
