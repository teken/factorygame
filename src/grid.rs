use bevy::{input::mouse::MouseWheel, math::vec3, prelude::*, render::primitives::Aabb};
use bevy_mod_picking::{Highlighting, Hover, PickableBundle, PickingRaycastSet};
use bevy_prototype_debug_lines::DebugLines;

use crate::{
    blocks::Spawn,
    components::{Block, BlockClicked},
    player::{Modes, Player, SpawnerOptions},
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_build_plane)
            .add_system(grid)
            .add_system(build_plane_manipulation)
            .add_system(grid_cell_select)
            .add_event::<EmptyGridCellClickedEvent>()
            .add_event::<GridCellHoveredEvent>()
            .add_event::<GridCellClickedEvent>()
            .add_system(grid_cell_hover)
            .add_system(grid_cell_clicked);
    }
}

#[derive(Component)]
struct BuildPlane {}

const RENDER_GRID: bool = false;
const GRID_SIZE: i32 = 1000;
const GRID_CELL_SIZE: usize = 1;

fn setup_build_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = materials.add(Color::NONE.into());
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(10000.0).into()),
            material: mat.clone(),
            ..Default::default()
        },
        BuildPlane {},
        PickableBundle::default(),
        Highlighting {
            initial: mat.clone(),
            hovered: Some(mat.clone()),
            pressed: Some(mat.clone()),
            selected: Some(mat),
        },
        Name::new("Build Plane"),
    ));
}

fn grid(
    mut lines: ResMut<DebugLines>,
    build_plane_query: Query<(&Transform, Entity), With<BuildPlane>>,
    intersect_query: Query<&bevy_mod_raycast::Intersection<PickingRaycastSet>>,
) {
    if !RENDER_GRID {
        return;
    }

    let Ok((trans, _)) = build_plane_query.get_single() else {
        return;
    };

    let pos = if intersect_query.get_single().is_ok() {
        intersect_query
            .get_single()
            .unwrap()
            .position()
            .unwrap_or(&vec3(0., 0., 0.))
            .floor()
    } else {
        vec3(0., 0., 0.)
    };

    let start_grid_z = pos.z as i32 - GRID_SIZE;
    let end_grid_z = pos.z as i32 + GRID_SIZE;

    let start_grid_x = pos.x as i32 - GRID_SIZE;
    let end_grid_x = pos.x as i32 + GRID_SIZE;

    for x in (start_grid_x..=end_grid_x).step_by(GRID_CELL_SIZE) {
        lines.line_colored(
            Vec3::new(x as f32, trans.translation.y, start_grid_z as f32),
            Vec3::new(x as f32, trans.translation.y, end_grid_z as f32),
            0.0,
            if x == 0 {
                Color::rgb(0., 0., 1.)
            } else {
                Color::rgb(0.5, 0.5, 0.5)
            },
        );
    }
    for z in (start_grid_z..=end_grid_z).step_by(GRID_CELL_SIZE) {
        lines.line_colored(
            Vec3::new(start_grid_x as f32, trans.translation.y, z as f32),
            Vec3::new(end_grid_x as f32, trans.translation.y, z as f32),
            0.0,
            if z == 0 {
                Color::rgb(1., 0., 0.)
            } else {
                Color::rgb(0.5, 0.5, 0.5)
            },
        );
    }
}

pub struct EmptyGridCellClickedEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
}

pub struct GridCellHoveredEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
    pub entity: Option<Entity>,
}

pub struct GridCellClickedEvent {
    pub grid_cell: Vec3,
    pub world_pos: Vec3,
    pub entity: Option<Entity>,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub enum GridSelectMode {
    #[default]
    Block,
    OnTopOfBlock,
}

fn grid_cell_clicked(
    mut reader: EventReader<GridCellClickedEvent>,
    player_query: Query<&SpawnerOptions, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    current_selected_query: Query<(&Block, Entity), With<BlockClicked>>,
) {
    let Ok(spawner_opts) = player_query.get_single() else {
        return;
    };

    for ele in reader.iter() {
        match spawner_opts.player_mode {
            Modes::Overview => {
                if let Some(ent) = ele.entity {
                    for ele in current_selected_query.iter() {
                        commands.entity(ele.1).remove::<BlockClicked>();
                    }
                    commands.entity(ent).insert(BlockClicked {});
                }
            }
            Modes::Build => spawner_opts.block_selection.spawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                spawner_opts,
                ele.grid_cell,
            ),
            Modes::Destroy => {
                if let Some(ent) = ele.entity {
                    commands.entity(ent).despawn_recursive();
                }
            }
        }
    }
    reader.clear();
}

fn grid_cell_hover(
    mut reader: EventReader<GridCellHoveredEvent>,
    mouse: Res<Input<MouseButton>>,
    mut writer: EventWriter<GridCellClickedEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for ele in reader.iter() {
        writer.send(GridCellClickedEvent {
            grid_cell: ele.grid_cell,
            world_pos: ele.world_pos,
            entity: ele.entity,
        });
    }
    reader.clear();
}

fn grid_cell_select(
    intersect_query: Query<&bevy_mod_raycast::Intersection<PickingRaycastSet>>,
    blocks_query: Query<(&Aabb, &GlobalTransform, Entity, &Hover), With<Block>>,
    player_query: Query<&SpawnerOptions, With<Player>>,
    mut grid_cell_hovered_event_writer: EventWriter<GridCellHoveredEvent>,
) {
    let Ok(inter) = intersect_query.get_single() else {
        return;
    };

    let Some(position) = inter.position() else {
        return;
    };

    let Ok(spawner_opts) = player_query.get_single() else {
        return;
    };

    let hovered_block = blocks_query.iter().find(|x| x.3.hovered());

    if let Some((aabb, trans, entity, _)) = hovered_block {
        if spawner_opts.grid_select_mode == GridSelectMode::Block {
            grid_cell_hovered_event_writer.send(GridCellHoveredEvent {
                grid_cell: trans.transform_point(aabb.center.into()),
                world_pos: *position,
                entity: Some(entity),
            });
        } else {
            let normal = (trans.transform_point(aabb.center.into()) - *position)
                .normalize_or_zero()
                .round();

            grid_cell_hovered_event_writer.send(GridCellHoveredEvent {
                grid_cell: trans.transform_point(aabb.center.into()) - normal,
                world_pos: *position,
                entity: Some(entity),
            });
        }
        return;
    }

    let mod_coord = |c: f32| {
        if (c - c.floor()).abs() < 0.001 {
            c.floor()
        } else if (c - c.ceil()).abs() < 0.001 {
            c.ceil()
        } else {
            c
        }
    };

    let mod_position = vec3(
        mod_coord(position.x),
        mod_coord(position.y),
        mod_coord(position.z),
    );

    grid_cell_hovered_event_writer.send(GridCellHoveredEvent {
        grid_cell: mod_position.floor() + vec3(0.5, 0.5, 0.5),
        world_pos: *position,
        entity: None,
    });
}

fn build_plane_manipulation(
    mut build_plane_query: Query<(&mut Transform, Entity), With<BuildPlane>>,
    mut ev_scroll: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
) {
    if !keys.pressed(KeyCode::LShift) {
        return;
    }

    let mut scroll = 0.0;
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    for (mut transform, _) in build_plane_query.iter_mut() {
        if scroll.abs() > 0.0 {
            transform.translation.y += scroll * GRID_CELL_SIZE as f32;
        }
    }
}
