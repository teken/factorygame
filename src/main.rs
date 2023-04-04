mod blocks;
mod components;
mod grid;
mod inventory;
mod materials;
mod player;
mod reactions;

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{DebugCursorPickingPlugin, DefaultPickingPlugins};

use bevy_obj::ObjPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::*;
use blocks::BlockPlugin;
use components::ComponentPlugin;
use grid::GridPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(ComponentPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(BlockPlugin)
        .add_plugin(materials::MaterialsPlugin)
        .add_startup_system(setup_lights)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup_lights(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>) {
    // light
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 1.0;

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 40.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}
