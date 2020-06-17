use crate::{error::Error, NodeID};
use bimap::BiMap;
pub use petgraph::{
    algo::condensation,
    graph::{EdgeIndex, NodeIndex},
    stable_graph::{EdgeIndices, StableDiGraph},
    Direction, IntoWeightedEdge,
};

pub trait Graph {
    fn from_edges<I>(iterable: I) -> Self
    where
        I: IntoIterator,
        I::Item: IntoWeightedEdge<(), NodeId = NodeID>;
    fn edge_count(&self) -> usize;
    fn edge_endpoints(&self, e: EdgeIndex) -> Option<(NodeID, NodeID)>;
    fn edge_indices(&self) -> EdgeIndices<()>;
    fn add_edge(&mut self, node1: NodeID, node2: NodeID) -> EdgeIndex;
    fn remove_edge(&mut self, e: EdgeIndex) -> Option<()>;
    fn game_beatable(&self) -> Result<(), Error>;
}

pub struct GameGraph {
    graph: StableDiGraph<NodeID, ()>,
    node_map: BiMap<NodeID, NodeIndex>,
}

fn insert_edge(
    graph: &mut StableDiGraph<NodeID, ()>,
    node_map: &mut BiMap<NodeID, NodeIndex>,
    a: NodeID,
    b: NodeID,
) -> EdgeIndex {
    match node_map.get_by_left(&a) {
        Some(a_idx) => ensure_end_exists_and_insert_edge(*a_idx, b, node_map, graph),
        None => {
            let a_index = graph.add_node(a);
            node_map.insert(a, a_index);
            ensure_end_exists_and_insert_edge(a_index, b, node_map, graph)
        }
    }
}

fn ensure_end_exists_and_insert_edge(
    a_idx: NodeIndex,
    b: NodeID,
    node_map: &mut BiMap<NodeID, NodeIndex>,
    graph: &mut StableDiGraph<NodeID, ()>,
) -> EdgeIndex {
    match node_map.get_by_left(&b) {
        Some(b_idx) => graph.add_edge(a_idx, *b_idx, ()),
        None => {
            let b_index = graph.add_node(b);
            node_map.insert(b, b_index);
            graph.add_edge(a_idx, b_index, ())
        }
    }
}

impl Graph for GameGraph {
    fn from_edges<I>(iterable: I) -> Self
    where
        I: IntoIterator,
        I::Item: IntoWeightedEdge<(), NodeId = NodeID>,
    {
        let mut graph = StableDiGraph::new();

        let mut node_map = BiMap::new();

        for i in iterable.into_iter() {
            let (a, b, _) = i.into_weighted_edge();
            insert_edge(&mut graph, &mut node_map, a, b);
        }

        GameGraph { graph, node_map }
    }

    fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    fn edge_endpoints(&self, e: EdgeIndex) -> Option<(NodeID, NodeID)> {
        let (n1, n2) = self.graph.edge_endpoints(e)?;

        let id1 = self.node_map.get_by_right(&n1)?;
        let id2 = self.node_map.get_by_right(&n2)?;
        Some((*id1, *id2))
    }

    fn add_edge(&mut self, a: NodeID, b: NodeID) -> EdgeIndex {
        insert_edge(&mut self.graph, &mut self.node_map, a, b)
    }

    fn remove_edge(&mut self, e: EdgeIndex) -> Option<()> {
        self.graph.remove_edge(e)
    }

    fn game_beatable(&self) -> Result<(), Error> {
        let condensed_graph = condensation(
            self.graph.map(|_, n| n, |_, e| e).into(),
            /*make_acyclic*/ true,
        );

        if condensed_graph.externals(Direction::Incoming).count() == 1 {
            Ok(())
        } else {
            let node_ids = condensed_graph
                .externals(Direction::Incoming)
                .flat_map(|idx| condensed_graph.node_weight(idx).unwrap())
                .map(|id| **id)
                .collect::<Vec<NodeID>>();
            Err(Error::GameUnbeatable(node_ids))
        }
    }

    fn edge_indices(&self) -> EdgeIndices<()> {
        self.graph.edge_indices()
    }
}
