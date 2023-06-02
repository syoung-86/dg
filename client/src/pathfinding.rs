use std::collections::BinaryHeap;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use lib::components::{FloorTile, Path, Tile, Untraversable};

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug)]
pub struct Node {
    pos: Tile,
    g_cost: u32,
    h_cost: u32,
    f_cost: u32,
    parent: Option<Tile>,
}

impl Node {
    fn new(pos: &Tile, path: &Path) -> Node {
        let dx = pos.cell.0.abs_diff(path.origin.cell.0);
        let dz = pos.cell.2.abs_diff(path.origin.cell.2);
        let g_cost = dx + dz;
        let dx = pos.cell.0.abs_diff(path.destination.cell.0);
        let dz = pos.cell.2.abs_diff(path.destination.cell.2);
        let h_cost = dx + dz;
        let f_cost = g_cost + h_cost;
        let parent = None;

        Node {
            pos: *pos,
            g_cost,
            h_cost,
            f_cost,
            parent,
        }
    }
    fn success(&self, path: &Path) -> bool {
        if self.pos == path.destination {
            true
        } else {
            false
        }
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

pub fn get_neighbours(
    path_info: &Path,
    current: &Tile,
    tiles: &Query<&Tile, (With<FloorTile>, Without<Untraversable>)>,
) -> Vec<Node> {
    let mut neighbours: Vec<Node> = vec![];
    for tile in tiles.iter() {
        
        //exlucde current tile?
        if tile == current {
            continue 
        }
        //North
        if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 {
            neighbours.push(Node::new(tile, path_info));
        }
        //North East
        if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 + 1 {
            neighbours.push(Node::new(tile, path_info));
        }
        //East
        if tile.cell.2 == current.cell.2 + 1 && tile.cell.0 == current.cell.0 {
            neighbours.push(Node::new(tile, path_info));
        }
        //South East
        if current.cell.0 != 0 {
            if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 + 1 {
                neighbours.push(Node::new(tile, path_info));
            }
            //South
            if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 {
                neighbours.push(Node::new(tile, path_info));
            }
        }
        //South West
        if current.cell.0 != 0 && current.cell.2 != 0 {
            if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 - 1 {
                neighbours.push(Node::new(tile, path_info));
            }
        }
        if current.cell.2 != 0 {
            //West
            if tile.cell.2 == current.cell.2 - 1 && tile.cell.0 == current.cell.0 {
                neighbours.push(Node::new(tile, path_info));
            }
            //North West
            if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 - 1 {
                neighbours.push(Node::new(tile, path_info));
            }
        }
    }
    //println!("neighbours: {:?}", neighbours.len());
    neighbours
}

fn success(path: &Path) -> bool {
    false
}

pub fn a_star(
    path_query: Query<&Path, Changed<Path>>,
    tiles: Query<&Tile, (With<FloorTile>, Without<Untraversable>)>,
) {
    if let Ok(path_info) = path_query.get_single() {
        //let sucessor_nodes = successors(&path, tiles);
        let mut open_set: BinaryHeap<Node> = BinaryHeap::new();
        let mut closed_set: HashSet<Tile> = HashSet::new();
        let mut came_from: HashMap<Tile, Option<Tile>> = HashMap::new();
        open_set.push(Node::new(&path_info.origin, path_info));

        while let Some(node) = open_set.pop() {
            if node.pos == path_info.destination {
                let mut path = vec![node.pos];
                let mut current = node.pos;

                while let Some(Some(prev)) = came_from.get(&current) {
                    path.push(*prev);
                    current = *prev;
                }

                path.reverse();
                //println!("path: {:?}", path);
            }
            let neighbours: Vec<Node> = get_neighbours(&path_info, &node.pos, &tiles);
        }
    }
}
