use bevy::prelude::*;
use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;
use lib::components::{FloorTile, Path, Tile, Untraversable};
use num_traits::Zero;
use rustc_hash::FxHasher;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::iter::FusedIterator;
use std::usize;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

#[allow(clippy::needless_collect)]
fn reverse_path<N, V, F>(parents: &FxIndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let mut i = start;
    let path = std::iter::from_fn(|| {
        parents.get_index(i).map(|(node, value)| {
            i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();
    // Collecting the going through the vector is needed to revert the path because the
    // unfold iterator is not double-ended due to its iterative nature.
    path.into_iter().rev().cloned().collect()
}
struct SmallestCostHolder<K> {
    estimated_cost: K,
    cost: K,
    index: usize,
}

impl<K: PartialEq> PartialEq for SmallestCostHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
    }
}

impl<K: PartialEq> Eq for SmallestCostHolder<K> {}

impl<K: Ord> PartialOrd for SmallestCostHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestCostHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.estimated_cost.cmp(&self.estimated_cost) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            s => s,
        }
    }
}
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
                neighbours.push((*current, 10));
            }
            //North East
            if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 + 1 {
                neighbours.push((*current, 14));
            }

            //East
            if tile.cell.2 == current.cell.2 + 1 && tile.cell.0 == current.cell.0 {
                neighbours.push((*current, 10));
            }
            //South East
            if current.cell.0 != 0 {
                if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 + 1 {
                    neighbours.push((*current, 14));
                }
                //South
                if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 {
                    neighbours.push((*current, 10));
                }
            }
            //South West
            if current.cell.0 != 0 && current.cell.2 != 0 {
                if tile.cell.0 == current.cell.0 - 1 && tile.cell.2 == current.cell.2 - 1 {
                    neighbours.push((*current, 14));
                }
            }
            if current.cell.2 != 0 {
                //West
                if tile.cell.2 == current.cell.2 - 1 && tile.cell.0 == current.cell.0 {
                    neighbours.push((*current, 10));
                }
                //North West
                if tile.cell.0 == current.cell.0 + 1 && tile.cell.2 == current.cell.2 - 1 {
                    neighbours.push((*current, 14));
                }
            }
        }
        //neighbours.iter().map(move |&n| n)
        println!("neighbours: {:?}", neighbours.len());
        neighbours.into_iter()
    }
    fn heuristic(&self, pos: &Tile) -> u32 {
        println!("heuristic");
        let dx = pos.cell.0.abs_diff(self.start.cell.0);
        let dz = pos.cell.2.abs_diff(self.start.cell.2);
        let g_cost = dx + dz;
        let dx = pos.cell.0.abs_diff(self.goal.cell.0);
        let dz = pos.cell.2.abs_diff(self.goal.cell.2);
        let h_cost = dx + dz;
        let f_cost = g_cost + h_cost;
        println!("f cost: {:?}", f_cost);
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
) {
    if let Ok(path_info) = path_query.get_single() {
        let nodes: Nodes = Nodes {
            tiles: tiles.iter().map(|tile| *tile).collect(),
            start: path_info.origin,
            goal: path_info.destination,
        };

        let path = astar(
            &nodes.start,
            |current_node| nodes.successors(current_node),
            |pos| nodes.heuristic(pos),
            |node| nodes.success(node),
        );
        println!("path: {:?}", path);
    }
}

#[allow(clippy::missing_panics_doc)]
pub fn astar<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestCostHolder {
        estimated_cost: Zero::zero(),
        cost: Zero::zero(),
        index: 0,
    });
    let mut parents: FxIndexMap<N, (usize, C)> = FxIndexMap::default();
    parents.insert(start.clone(), (usize::max_value(), Zero::zero()));
    while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
        let successors = {
            let (node, &(_, c)) = parents.get_index(index).unwrap(); // Cannot fail
            if success(node) {
                let path = reverse_path(&parents, |&(p, _)| p, index);
                return Some((path, cost));
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            successors(node)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let h; // heuristic(&successor)
            let n; // index for successor
                    println!("for successor");
            match parents.entry(successor) {
                Vacant(e) => {
                    println!("vacant");
                    h = heuristic(e.key());
                    n = e.index();
                    e.insert((index, new_cost));
                }
                Occupied(mut e) => {
                    println!("occupied");
                    if e.get().1 > new_cost {
                        h = heuristic(e.key());
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(SmallestCostHolder {
                estimated_cost: new_cost + h,
                cost: new_cost,
                index: n,
            });
        }
    }
    None
}
