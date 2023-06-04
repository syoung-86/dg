use bevy::prelude::*;
use lib::components::{FloorTile, Path, Tile, Untraversable};
use std::hash::Hash;
use pathfinding::prelude::astar;


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

//#[allow(clippy::missing_panics_doc)]
//pub fn astar<N, C, FN, IN, FH, FS>(
    //start: &N,
    //mut successors: FN,
    //mut heuristic: FH,
    //mut success: FS,
//) -> Option<(Vec<N>, C)>
//where
    //N: Eq + Hash + Clone,
    //C: Zero + Ord + Copy,
    //FN: FnMut(&N) -> IN,
    //IN: IntoIterator<Item = (N, C)>,
    //FH: FnMut(&N) -> C,
    //FS: FnMut(&N) -> bool,
//{
    //let mut to_see = BinaryHeap::new();
    //to_see.push(SmallestCostHolder {
        //estimated_cost: Zero::zero(),
        //cost: Zero::zero(),
        //index: 0,
    //});
    //let mut parents: FxIndexMap<N, (usize, C)> = FxIndexMap::default();
    //parents.insert(start.clone(), (usize::max_value(), Zero::zero()));
    //while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
        //let successors = {
            //let (node, &(_, c)) = parents.get_index(index).unwrap(); // Cannot fail
            //if success(node) {
                //let path = reverse_path(&parents, |&(p, _)| p, index);
                //return Some((path, cost));
            //}
            //// We may have inserted a node several time into the binary heap if we found
            //// a better way to access it. Ensure that we are currently dealing with the
            //// best path and discard the others.
            //if cost > c {
                //continue;
            //}
            //successors(node)
        //};
        //for (successor, move_cost) in successors {
            //let new_cost = cost + move_cost;
            //let h; // heuristic(&successor)
            //let n; // index for successor
            //match parents.entry(successor) {
                //Vacant(e) => {
                    //h = heuristic(e.key());
                    //n = e.index();
                    //e.insert((index, new_cost));
                //}
                //Occupied(mut e) => {
                    //if e.get().1 > new_cost {
                        //h = heuristic(e.key());
                        //n = e.index();
                        //e.insert((index, new_cost));
                    //} else {
                        //continue;
                    //}
                //}
            //}

            //to_see.push(SmallestCostHolder {
                //estimated_cost: new_cost + h,
                //cost: new_cost,
                //index: n,
            //});
        //}
    //}
    //None
//}
