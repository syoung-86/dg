use bevy::prelude::*;
use lib::{
    components::{ControlledEntity, FloorTile, LeftClick, Path, Tile, Untraversable},
    resources::Tick,
};
use pathfinding::prelude::astar;
use std::hash::Hash;

use crate::movement::PathMap;

#[derive(Eq, PartialEq, Hash, Clone)]
struct Nodes {
    tiles: Vec<Tile>,
    start: Tile,
    goal: Tile,
}
impl Nodes {
    fn successors(&self, current: &Tile) -> impl Iterator<Item = (Tile, u32)> {
        let mut neighbours: Vec<(Tile, u32)> = vec![];
        for tile in self.tiles.iter() {
            //North
            if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 {
                neighbours.push((*tile, 10));
            }
            //North East
            if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 + 1 {
                neighbours.push((*tile, 14));
            }

            //East
            if tile.cell.2 == current.cell.2 + 1 && tile.cell.0 == current.cell.0 {
                neighbours.push((*tile, 10));
            }
            //South East
            if current.cell.0 != 0 {
                if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 + 1 {
                    neighbours.push((*tile, 14));
                }
                //South
                if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 {
                    neighbours.push((*tile, 10));
                }
            }
            //South West
            if current.cell.0 != 0 && current.cell.2 != 0 {
                if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 - 1 {
                    neighbours.push((*tile, 14));
                }
            }
            if current.cell.2 != 0 {
                //West
                if tile.cell.2 == current.cell.2 - 1 && tile.cell.0 == current.cell.0 {
                    neighbours.push((*tile, 10));
                }
                //North West
                if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 - 1 {
                    neighbours.push((*tile, 14));
                }
            }
        }
        //neighbours.iter().map(move |&n| n)
        neighbours.into_iter()
    }
    fn heuristic(&self, pos: &Tile) -> u32 {
        let dx = pos.cell.0.abs_diff(self.start.cell.0);
        let dz = pos.cell.2.abs_diff(self.start.cell.2);
        let g_cost = dx + dz;
        let dx = pos.cell.0.abs_diff(self.goal.cell.0);
        let dz = pos.cell.2.abs_diff(self.goal.cell.2);
        let h_cost = dx + dz;
        let f_cost = g_cost + h_cost;
        f_cost
    }

    fn success(&self, current: &Tile) -> bool {
        if self.goal == *current {
            true
        } else {
            false
        }
    }
}

pub fn find_path(
    path_query: Query<&Path, Changed<Path>>,
    tiles: Query<&Tile, (With<FloorTile>, Without<Untraversable>)>,
    tick: Res<Tick>,
    player: Query<Entity, With<ControlledEntity>>,
    mut commands: Commands,
) {
    if let Ok(path_info) = path_query.get_single() {
        let nodes: Nodes = Nodes {
            tiles: tiles.iter().map(|tile| *tile).collect(),
            start: path_info.origin,
            goal: path_info.destination,
        };

        if let Some(path) = astar(
            &nodes.start,
            |current_node| nodes.successors(current_node),
            |pos| nodes.heuristic(pos),
            |node| nodes.success(node),
        ) {
            let mut path_map: PathMap = PathMap::default();
            let mut step_tick = tick.clone();
            for step in path.0 {
                step_tick.tick += 1;
                path_map.steps.push((step_tick, LeftClick::Walk, step));
            }
            if let Some(last) = path_map.steps.last_mut() {
                last.1 = path_info.left_click;
            }
            if let Ok(player_entity) = player.get_single() {
                commands.entity(player_entity).insert(path_map);
            }
        }
    }
}
