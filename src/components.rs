use std::time::Duration;

use bevy::prelude::*;

use crate::{
    blocks::BlockType,
    materials::{Inventory, ItemStack, Reaction},
    player,
};

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Block>()
            .register_type::<Input>()
            .register_type::<Output>()
            .register_type::<Process>();
    }
}

#[derive(Component, Default)]
pub struct Furnace;

#[derive(Component)]
pub struct Conveyor {
    pub timer: Timer,
}

impl Default for Conveyor {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default)]
pub struct Splitter;

#[derive(Component, Default)]
pub struct Storage;

#[derive(Component, Default)]
pub struct Grabber;

#[derive(Component)]
pub struct BlockClicked {}

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
pub struct Process {
    pub reaction: Option<Reaction>,
    pub timer: Timer,
}

#[derive(Component, Default, Reflect)]
pub struct Source {
    pub source: Option<ItemStack>,
    pub fequency: Duration,
    pub timer: Timer,
    pub inventory: Inventory,
}

impl Process {
    pub fn set_reaction(&mut self, reaction: &Reaction) {
        self.reaction = Some(reaction.clone());
        self.timer = Timer::new(reaction.duration, TimerMode::Repeating);
    }
}

#[derive(Component, Reflect)]
pub struct Block {
    pub block_type: BlockType,
    pub direction: player::Direction,
}
