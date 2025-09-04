use crate::engine::flow_graph::FlowGraph;
use crate::engine::min_heap::MinHeap;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DijkstraNode {
    distance: i32,
    node_id: usize,
}

impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.cmp(&other.distance)
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

pub fn dijkstra(graph: &FlowGraph, source: usize, sink: usize) -> Option<Vec<usize>> {
    let num_nodes = graph.graph.len();
    let mut distances = vec![i32::MAX; num_nodes];
    let mut parent = vec![None; num_nodes];
    let mut heap = MinHeap::new();

    distances[source] = 0;
    heap.insert(DijkstraNode { distance: 0, node_id: source}).ok()?;

    while heap.heap_size() > 0 {
        let current = heap.extract_min().ok()?;
        let u = current.node_id;

        if current.distance > distances[u] {
            continue;
        }

        if u == sink {
            break;
        }

        for &edge_idx in &graph.graph[u] {
            let edge = &graph.edges[edge_idx];

            if graph.residual_capacity(edge_idx) > 0 {
                let new_distance = distances[u].saturating_add(edge.cost);

                if new_distance < distances[edge.to] {
                    distances[edge.to] = new_distance;
                    parent[edge.to] = Some(u);
                    heap.insert(DijkstraNode {
                        distance: new_distance,
                        node_id: edge.to
                    }).ok()?;
                }
            }
        }
    }

    if distances[sink] == i32::MAX {
        return None;
    }

    let mut path = vec![];
    let mut current = sink;

    while let Some(prev) = parent[current] {
        path.push(current);
        current = prev;
    }

    path.push(source);
    path.reverse();

    Some(path)
}