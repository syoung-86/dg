use std::{collections::BinaryHeap, sync::Mutex};

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use lazy_static::lazy_static;
use lib::components::{FloorTile, Path, Tile, Untraversable};

lazy_static! {
    static ref GOAL: Mutex<(u32, u32)> = Mutex::new((0, 0));
}
#[derive(Copy, Hash, Clone, Eq, PartialEq, PartialOrd, Debug)]
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
        let mut goal = GOAL.lock().unwrap();
        *goal = (10, 10);
        Node {
            pos: *pos,
            g_cost,
            h_cost,
            f_cost,
            parent,
        }
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

fn success(pos: (u32, u32)) -> bool {
    if let Ok(goal) = GOAL.into_inner() {
        if goal == pos {
            true
        } else {
            false
        }
    } else {
        false
    }
}
pub fn get_neighbours(
    path_info: &Path,
    current: &Tile,
    tiles: &Query<&Tile, (With<FloorTile>, Without<Untraversable>)>,
) -> BinaryHeap<Node> {
    let mut neighbours: BinaryHeap<Node> = BinaryHeap::new();
    for tile in tiles.iter() {
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

//pub fn a_star(
//path_query: Query<&Path, Changed<Path>>,
//tiles: Query<&Tile, (With<FloorTile>, Without<Untraversable>)>,
//) {
//if let Ok(path_info) = path_query.get_single() {
//let mut open_set: BinaryHeap<Node> = BinaryHeap::new();
//let mut open_check: HashSet<Node> = HashSet::new();
//let mut closed_set: HashSet<Node> = HashSet::new();
//let mut came_from: HashMap<Tile, Option<Tile>> = HashMap::new();
//open_set.push(Node::new(&path_info.origin, path_info));
//open_check.insert(Node::new(&path_info.origin, path_info));

//while let Some(current) = open_set.pop() {
//println!("current {:?}", current);
//closed_set.insert(current);
//if current.pos == path_info.destination {
//let mut path = vec![current.pos];
//let mut current = current.pos;

//while let Some(Some(prev)) = came_from.get(&current) {
//path.push(*prev);
//current = *prev;
//}

//path.reverse();
//println!("path: {:?}", path);
//}
//let mut neighbours: BinaryHeap<Node> = get_neighbours(&path_info, &current.pos, &tiles);
//while let Some(mut neighbour) = neighbours.pop() {
//if closed_set.contains(&neighbour) {
//continue;
//}
//if !open_check.contains(&neighbour) {
//neighbour.parent = Some(current.pos);
//came_from.insert(neighbour.pos, Some(current.pos));
//open_set.push(neighbour);
//open_check.insert(neighbour);
//break;
//}
//}
//}
//}
//A
//}
