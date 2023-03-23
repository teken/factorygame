use bevy::{math::vec3, prelude::*};

use crate::{
    materials::{Item, Reaction},
    player::SpawnerOptions,
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(furnace_system);
        app.add_system(conveyor_system);
        app.add_system(input_feed_system);
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
        block: Block,
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
        block: Block,
    ) {
        let block_transform = Transform::from_translation(block.min + vec3(0.5, 0.5, 0.5))
            .with_rotation(spawner_options.block_rotation.to_quat());
        match self {
            BlockType::Debug => commands.spawn((
                SceneBundle {
                    scene: asset_server.load(r"models\test.gltf#Scene0"),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Debug Block"),
                block,
            )),
            BlockType::Furnace => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::RED.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Furnace"),
                Furnace::default(),
                block,
                Input::default(),
                Output::default(),
                Process::default(),
            )),
            BlockType::Conveyor => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::BLUE.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Conveyor"),
                Conveyor::default(),
                block,
                Input::default(),
                Output::default(),
            )),
            BlockType::Splitter => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::GREEN.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Splitter"),
                Splitter::default(),
                block,
                Input::default(),
                Output::default(),
            )),
            BlockType::Storage => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::YELLOW.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Storage"),
                Storage::default(),
                block,
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

        if let Some(item) = input.inventory.pop() {
            output.inventory.push(item);
        }
    }
}
