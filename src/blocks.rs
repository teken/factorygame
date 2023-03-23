use bevy::{math::vec3, prelude::*};

use crate::{
    materials::{Item, Reaction},
    player::SpawnerOptions,
};

#[derive(Component)]
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub block_type: BlockType,
}

#[derive(Debug, Clone)]
pub enum BlockType {
    Debug,
    Furnace,
    Conveyor,
    Splitter,
    Storage,
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
                block,
            )),
            BlockType::Conveyor => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::BLUE.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Conveyor"),
                block,
            )),
            BlockType::Splitter => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::GREEN.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Splitter"),
                block,
            )),
            BlockType::Storage => commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::new(1.0).into()),
                    material: materials.add(Color::YELLOW.into()),
                    transform: block_transform,
                    ..default()
                },
                Name::new("Storage"),
                block,
            )),
        };
    }
}

pub trait Processor {
    fn process(&self, reaction: &Reaction, input: Vec<Item>) -> Option<Vec<Item>>;
}

impl Processor for BlockType {
    fn process(&self, reaction: &Reaction, input: Vec<Item>) -> Option<Vec<Item>> {
        match self {
            BlockType::Furnace => _process(reaction, input),
            _ => None,
        }
    }
}

fn _process(reaction: &Reaction, input: Vec<Item>) -> Option<Vec<Item>> {
    let mut block_input: Vec<Item> = vec![];
    let mut param_input: Vec<Item> = input.clone();
    param_input.append(&mut block_input);
    if reaction.valid_input(param_input) {
        Some(reaction.output.clone())
    } else {
        None
    }
}
