use crate::{
    error::Result,
    graph::{GameGraph, Graph},
};
use linked_hash_set::LinkedHashSet;
use petgraph::graph::EdgeIndex;
use rand::{rngs::StdRng, seq::IteratorRandom, Rng};
use std::{fmt::Debug, hash::Hash};

trait Swappable {
    fn swap<G: Graph>(&self, other: &Self, graph: &mut G) -> EdgePair<Self>
    where
        Self: Sized;
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct OneWay {
    idx: EdgeIndex,
}

impl OneWay {
    pub fn new(idx: EdgeIndex) -> Self {
        OneWay { idx }
    }

    pub fn get_idx(self) -> EdgeIndex {
        self.idx
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct TwoWay {
    idx1: EdgeIndex,
    idx2: EdgeIndex,
}

impl TwoWay {
    pub fn new(idx1: EdgeIndex, idx2: EdgeIndex) -> Self {
        TwoWay { idx1, idx2 }
    }

    pub fn get_idx1(self) -> EdgeIndex {
        self.idx1
    }

    pub fn get_idx2(self) -> EdgeIndex {
        self.idx2
    }
}

impl Swappable for OneWay {
    fn swap<G: Graph>(&self, other: &Self, graph: &mut G) -> EdgePair<Self> {
        let (e1, e2) = swap_edges(self.idx, other.idx, graph);

        EdgePair(Self::new(e1), Self::new(e2))
    }
}

impl Swappable for TwoWay {
    fn swap<G: Graph>(&self, other: &Self, graph: &mut G) -> EdgePair<Self> {
        let (e1, e2) = swap_edges(self.idx1, other.idx1, graph);
        let (e3, e4) = swap_edges(self.idx2, other.idx2, graph);

        EdgePair(Self::new(e1, e4), Self::new(e2, e3))
    }
}

fn swap_edges<G: Graph>(
    edge1: EdgeIndex,
    edge2: EdgeIndex,
    graph: &mut G,
) -> (EdgeIndex, EdgeIndex) {
    let (edge1a, edge1b) = graph.edge_endpoints(edge1).unwrap();
    let (edge2a, edge2b) = graph.edge_endpoints(edge2).unwrap();

    graph
        .remove_edge(edge1)
        .unwrap_or_else(|| panic!("Failed to remove edge ({:?}, {:?})", edge1a, edge1b));
    graph
        .remove_edge(edge2)
        .unwrap_or_else(|| panic!("Failed to remove edge ({:?}, {:?})", edge2a, edge2b));

    let new_edge_id1 = graph.add_edge(edge1a, edge2b);
    let new_edge_id2 = graph.add_edge(edge2a, edge1b);
    (new_edge_id1, new_edge_id2)
}

pub struct GameWorld {
    pub graph: GameGraph,
    pub swappable_one_ways: LinkedHashSet<OneWay>,
    pub swappable_two_ways: LinkedHashSet<TwoWay>,
}

struct EdgePair<T: Swappable>(T, T);

fn pick_random_edges<T>(swap_edges: &LinkedHashSet<T>, rng: &mut StdRng) -> Option<EdgePair<T>>
where
    T: Copy + Clone + Hash + Eq + Swappable,
{
    if swap_edges.len() >= 2 {
        let edge_vec: Vec<&T> = swap_edges.iter().choose_multiple(rng, 2);
        Some(EdgePair(*edge_vec[0], *edge_vec[1]))
    } else {
        None
    }
}

fn try_swap_edges<T, G>(graph: &mut G, swap_edges: &mut LinkedHashSet<T>, rng: &mut StdRng)
where
    G: Graph,
    T: Debug + Copy + Clone + Hash + Eq + Swappable,
{
    if let Some(EdgePair(edge1, edge2)) = pick_random_edges(swap_edges, rng) {
        let EdgePair(new_edge1, new_edge2) = edge1.swap(&edge2, graph);

        if graph.game_beatable().is_err() {
            new_edge1.swap(&new_edge2, graph);
        } else {
            if !swap_edges.remove(&edge1) {
                panic!(format!(
                    "Failed to remove {:?} from swappable edges",
                    &edge1
                ))
            }
            if !swap_edges.remove(&edge2) {
                panic!(format!(
                    "Failed to remove {:?} from swappable edges",
                    &edge2
                ))
            }

            swap_edges.insert(new_edge1);
            swap_edges.insert(new_edge2);
        }
    }
}

pub fn build_game(
    mut game_world: GameWorld,
    rng: &mut StdRng,
    iterations: usize,
) -> Result<GameWorld> {
    let _ = game_world.graph.game_beatable()?;

    for _ in 0..iterations {
        if rng.gen::<bool>() {
            try_swap_edges(
                &mut game_world.graph,
                &mut game_world.swappable_one_ways,
                rng,
            );
        } else {
            try_swap_edges(
                &mut game_world.graph,
                &mut game_world.swappable_two_ways,
                rng,
            );
        }
    }
    Ok(game_world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn graph() -> GameGraph {
        GameGraph::from_edges(&[
            (0, 1),
            (1, 2),
            (2, 1),
            (2, 3),
            (3, 4),
            (4, 3),
            (4, 5),
            (5, 4),
            (3, 5),
            (5, 3),
            (4, 6),
            (6, 8),
            (8, 6),
            (5, 7),
            (7, 9),
            (9, 7),
            (8, 10),
            (9, 10),
        ])
    }

    #[test]
    fn test_swap() {
        let mut graph = graph();

        let edge1 = OneWay::new(graph.add_edge(4, 6));

        let edge2 = OneWay::new(graph.add_edge(8, 10));

        let (idx1, idx2) = swap_edges(edge1.idx, edge2.idx, &mut graph);
        assert!(graph.edge_endpoints(idx1).unwrap() == (4, 10));
        assert!(graph.edge_endpoints(idx2).unwrap() == (8, 6));
    }

    #[test]
    fn test_swap_two_ways() {
        let mut graph = graph();

        let edge1 = TwoWay::new(graph.add_edge(4, 6), graph.add_edge(6, 4));
        let edge2 = TwoWay::new(graph.add_edge(8, 10), graph.add_edge(10, 8));

        let (idx1, idx2) = swap_edges(edge1.idx1, edge2.idx1, &mut graph);
        let (idx3, idx4) = swap_edges(edge1.idx2, edge2.idx2, &mut graph);
        assert!(graph.edge_endpoints(idx1).unwrap() == (4, 10));
        assert!(graph.edge_endpoints(idx2).unwrap() == (8, 6));
        assert!(graph.edge_endpoints(idx3).unwrap() == (6, 8));
        assert!(graph.edge_endpoints(idx4).unwrap() == (10, 4));
    }

    // Make sure we don't add edges before removing old edges, otherwise we start losing edges
    #[test]
    fn test_swap_same_endpoint() {
        let mut graph = graph();

        let edge1 = OneWay::new(graph.add_edge(9, 10));

        let edge2 = OneWay::new(graph.add_edge(8, 10));

        let (idx1, idx2) = swap_edges(edge1.idx, edge2.idx, &mut graph);

        assert_eq!(graph.edge_endpoints(idx1).unwrap(), (9, 10));
        assert_eq!(graph.edge_endpoints(idx2).unwrap(), (8, 10));
    }

    #[test]
    fn test_swap_twice() {
        let mut graph = graph();

        let edge1 = OneWay::new(graph.add_edge(4, 6));

        let edge2 = OneWay::new(graph.add_edge(8, 10));

        let EdgePair(new_edge1, new_edge2) = edge1.swap(&edge2, &mut graph);
        let EdgePair(final_edge1, final_edge2) = new_edge1.swap(&new_edge2, &mut graph);

        assert!(graph.edge_endpoints(final_edge1.idx).unwrap() == (4, 6));
        assert!(graph.edge_endpoints(final_edge2.idx).unwrap() == (8, 10));
    }

    #[test]
    fn test_swap_twice_two_ways() {
        let mut graph = graph();

        let edge1 = TwoWay::new(graph.add_edge(4, 6), graph.add_edge(6, 4));
        let edge2 = TwoWay::new(graph.add_edge(8, 10), graph.add_edge(10, 8));

        let EdgePair(new_edge1, new_edge2) = edge1.swap(&edge2, &mut graph);
        let EdgePair(final_edge1, final_edge2) = new_edge1.swap(&new_edge2, &mut graph);

        assert_eq!(graph.edge_endpoints(final_edge1.idx1).unwrap(), (4, 6));
        assert_eq!(graph.edge_endpoints(final_edge1.idx2).unwrap(), (6, 4));
        assert_eq!(graph.edge_endpoints(final_edge2.idx1).unwrap(), (8, 10));
        assert_eq!(graph.edge_endpoints(final_edge2.idx2).unwrap(), (10, 8));
    }

    #[test]
    fn test_beatable() {
        let graph = graph();

        assert!(graph.game_beatable().is_ok());
    }

    #[test]
    fn test_shuffle() {
        let mut graph = GameGraph::from_edges(&[
            (1, 2),
            (2, 1),
            (3, 4),
            (4, 3),
            (4, 5),
            (5, 4),
            (3, 5),
            (5, 3),
            (6, 8),
            (8, 6),
            (7, 9),
            (9, 7),
        ]);

        let ow1 = OneWay::new(graph.add_edge(0, 1));
        let ow2 = OneWay::new(graph.add_edge(2, 3));
        let ow3 = OneWay::new(graph.add_edge(4, 6));
        let ow4 = OneWay::new(graph.add_edge(5, 7));
        let ow5 = OneWay::new(graph.add_edge(8, 10));
        let ow6 = OneWay::new(graph.add_edge(9, 10));

        let mut ow_hashset = LinkedHashSet::new();
        ow_hashset.insert(ow1);
        ow_hashset.insert(ow2);
        ow_hashset.insert(ow3);
        ow_hashset.insert(ow4);
        ow_hashset.insert(ow5);
        ow_hashset.insert(ow6);

        let mut rng = StdRng::seed_from_u64(3);

        let game_world = GameWorld {
            graph,
            swappable_one_ways: ow_hashset,
            swappable_two_ways: LinkedHashSet::new(),
        };

        let game = build_game(game_world, &mut rng, 500).unwrap();

        assert_eq!(game.graph.edge_count(), 18);
        assert!(game.graph.game_beatable().is_ok());
    }
}
