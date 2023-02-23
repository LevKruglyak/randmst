use crate::graph::Edge;

pub mod rem_union_find;

// Generic union find struct that can be used in Kruskal's algorithm
pub trait KruskalUnionFind {
    fn new(size: u32) -> Self;

    fn unite(&mut self, u: u32, v: u32) -> bool;
}

// Some generator of sorted edges for use in Kruskal's algorithm
pub trait KruskalEdgeGenerator: IntoIterator<Item = Edge> {
    fn size(&self) -> u32;
}

pub fn mst_total_length<U: KruskalUnionFind>(edges: impl KruskalEdgeGenerator) -> f64 {
    let mut set = U::new(edges.size());
    let mut total_count = edges.size() - 1;
    let mut total_weight = 0.0;

    for edge in edges.into_iter() {
        if set.unite(edge.u, edge.v) {
            // Add edge to total graph
            total_weight += edge.w;
            total_count -= 1;
        }

        // If we have a tree, we're done
        if total_count == 0 {
            break;
        }
    }

    total_weight
}
