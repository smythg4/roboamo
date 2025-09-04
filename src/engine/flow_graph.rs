#[derive(Debug, Clone)]
pub struct Edge {
    pub to: usize,
    pub capacity: i32, // max flow through edge
    pub flow: i32,     // current flow through edge
    pub cost: i32,     // cost per unit of flow
}

pub struct FlowGraph {
    pub edges: Vec<Edge>,
    pub graph: Vec<Vec<usize>>,
    num_nodes: usize,
}

impl FlowGraph {
    pub fn new(num_nodes: usize) -> Self {
        FlowGraph {
            edges: vec![],
            graph: vec![vec![]; num_nodes],
            num_nodes,
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, capacity: i32, cost: i32) {
        let edge_idx = self.edges.len();

        self.edges.push(Edge {
            to,
            capacity,
            flow: 0,
            cost,
        });
        self.graph[from].push(edge_idx);

        self.edges.push(Edge {
            to: from,
            capacity: 0,
            flow: 0,
            cost: -cost,
        });
        self.graph[to].push(edge_idx + 1);
    }

    // pub fn add_simple_edge(&mut self, from: usize, to: usize) {
    //     self.add_edge(from, to, 1, 0);
    // }

    // pub fn add_undirected_edge(&mut self, u: usize, v: usize, capacity: i32, cost: i32) {
    //     self.add_edge(u, v, capacity, cost);
    //     self.add_edge(v, u, capacity, cost);
    // }

    // pub fn neighbors(&self, u: usize) -> impl Iterator<Item = usize> + '_ {
    //     self.graph[u]
    //         .iter()
    //         .map(|&edge_idx| self.edges[edge_idx].to)
    // }

    pub fn residual_capacity(&self, edge_idx: usize) -> i32 {
        let edge = &self.edges[edge_idx];
        edge.capacity - edge.flow
    }

    // pub fn ford_fulkerson(&mut self, source: usize, sink: usize) -> i32 {
    //     for edge in &mut self.edges {
    //         edge.flow = 0;
    //     }

    //     let mut max_flow_value = 0;

    //     while let Some(path) = self.find_augmenting_path(source, sink) {
    //         let path_flow = self.path_capacity(&path);

    //         self.augment_path(&path, path_flow);

    //         max_flow_value += path_flow;
    //     }

    //     max_flow_value
    // }

    fn find_augmenting_path(&self, source: usize, sink: usize) -> Option<Vec<usize>> {
        // Use Dijkstra's algorithm for better O((V+E)log V) performance vs Bellman-Ford O(VE)
        crate::engine::djikstra::dijkstra(self, source, sink)
    }

    pub fn min_cost_max_flow(&mut self, source: usize, sink: usize) -> (i32, i32) {
        // initialize all flows to 0
        for edge in &mut self.edges {
            edge.flow = 0;
        }

        let mut max_flow_value = 0;
        let mut total_cost = 0;

        while let Some(path) = self.find_augmenting_path(source, sink) {
            let path_flow = self.path_capacity(&path);
            let path_cost = self.path_cost(&path);

            self.augment_path(&path, path_flow);

            max_flow_value += path_flow;
            total_cost += path_flow * path_cost;
        }

        (max_flow_value, total_cost)
    }

    fn path_cost(&self, path: &[usize]) -> i32 {
        let mut total_cost = 0;

        for window in path.windows(2) {
            let u = window[0];
            let v = window[1];

            for &edge_idx in &self.graph[u] {
                if self.edges[edge_idx].to == v && self.residual_capacity(edge_idx) > 0 {
                    total_cost += self.edges[edge_idx].cost;
                    break;
                }
            }
        }

        total_cost
    }
    // dfs path used in original augmenting path algorithm. Not needed for shortest path approach.
    // fn dfs_path(&self, u: usize, sink: usize, visited: &mut [bool], path: &mut Vec<usize>) -> bool {
    //     visited[u] = true;
    //     path.push(u);

    //     if u == sink {
    //         return true;
    //     }

    //     for &edge_idx in &self.graph[u] {
    //         let edge = &self.edges[edge_idx];
    //         if !visited[edge.to] && self.residual_capacity(edge_idx) > 0 {
    //             if self.dfs_path(edge.to, sink, visited, path) {
    //                 return true;
    //             }
    //         }
    //     }

    //     path.pop();
    //     false
    // }

    fn path_capacity(&self, path: &[usize]) -> i32 {
        let mut min_capacity = i32::MAX;

        for window in path.windows(2) {
            let u = window[0];
            let v = window[1];

            for &edge_idx in &self.graph[u] {
                if self.edges[edge_idx].to == v {
                    let residual = self.residual_capacity(edge_idx);
                    if residual > 0 {
                        min_capacity = min_capacity.min(residual);
                        break;
                    }
                }
            }
        }
        min_capacity
    }

    fn augment_path(&mut self, path: &[usize], flow: i32) {
        for window in path.windows(2) {
            let u = window[0];
            let v = window[1];

            for &edge_idx in &self.graph[u] {
                if self.edges[edge_idx].to == v && self.residual_capacity(edge_idx) > 0 {
                    self.edges[edge_idx].flow += flow;
                    self.edges[edge_idx ^ 1].flow -= flow;
                    break;
                }
            }
        }
    }
}
