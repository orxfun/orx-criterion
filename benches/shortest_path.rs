use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Data, Experiment, Variant};
use orx_priority_queue::{DaryHeapOfIndices, PriorityQueue, PriorityQueueDecKey};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// setup

fn new_graph(num_nodes: usize, connectivity_perc: usize) -> Vec<Vec<Edge>> {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let out_degree = num_nodes * connectivity_perc / 100;
    let out_degree_range = || out_degree.saturating_sub(100)..(out_degree + 100);
    (0..num_nodes)
        .map(|i| {
            let num_edges = rng.random_range(out_degree_range());
            (0..num_edges)
                .filter_map(|_| {
                    let j = rng.random_range(0..num_nodes);
                    match i == j {
                        true => None,
                        false => Some(Edge {
                            node: j,
                            cost: rng.random_range(100..10000),
                        }),
                    }
                })
                .collect()
        })
        .collect()
}

struct Edge {
    node: usize,
    cost: usize,
}

fn shortest_path<const D: usize>(
    adj_list: &[Vec<Edge>],
    start: usize,
    goal: usize,
) -> Option<usize> {
    let num_nodes = adj_list.len();
    let mut heap = DaryHeapOfIndices::<_, _, D>::with_index_bound(num_nodes);

    // We're at `start`, with a zero cost
    heap.push(start, 0);

    // Examine the frontier with lower cost nodes first (min-heap)
    while let Some((position, cost)) = heap.pop() {
        // Alternatively we could have continued to find all shortest paths
        if position == goal {
            return Some(cost);
        }

        // For each node we can reach, see if we can find a way with
        // a lower cost going through this node
        for edge in &adj_list[position] {
            _ = heap.try_decrease_key_or_push(&edge.node, cost + edge.cost);
        }
    }

    // Goal not reachable
    None
}

// data

struct GraphSettings {
    num_nodes: usize,
    connectivity_perc: usize,
}

impl Data for GraphSettings {
    fn factor_names() -> Vec<&'static str> {
        vec!["num_nodes", "connectivity"]
    }

    fn factor_values(&self) -> Vec<String> {
        vec![
            self.num_nodes.to_string(),
            format!("{}%", self.connectivity_perc),
        ]
    }
}

// variants

struct HeapWidth(usize);

impl Variant for HeapWidth {
    fn param_names() -> Vec<&'static str> {
        vec!["heap-width"]
    }

    fn param_values(&self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

// experiment

struct ShortestPathExp;

impl Experiment for ShortestPathExp {
    type Data = GraphSettings;

    type Variant = HeapWidth;

    type Input = Vec<Vec<Edge>>;

    type Output = Option<usize>;

    fn input(data: &Self::Data) -> Self::Input {
        new_graph(data.num_nodes, data.connectivity_perc)
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        let (s, t) = (0, input.len() - 1);
        match variant.0 {
            2 => shortest_path::<2>(input, s, t),
            3 => shortest_path::<3>(input, s, t),
            4 => shortest_path::<4>(input, s, t),
            8 => shortest_path::<8>(input, s, t),
            16 => shortest_path::<16>(input, s, t),
            32 => shortest_path::<32>(input, s, t),
            64 => shortest_path::<64>(input, s, t),
            512 => shortest_path::<512>(input, s, t),
            _ => panic!("Not handled heap width"),
        }
    }

    fn expected_output(_: &Self::Data, input: &Self::Input) -> Option<Self::Output> {
        let (s, t) = (0, input.len() - 1);
        Some(shortest_path::<2>(input, s, t))
    }
}

fn run(c: &mut Criterion) {
    let num_nodes = [1 << 12, 1 << 13];
    let connectivity = [2, 100];
    let data: Vec<_> = num_nodes
        .iter()
        .copied()
        .flat_map(|num_nodes| {
            connectivity.map(|connectivity_perc| GraphSettings {
                num_nodes,
                connectivity_perc,
            })
        })
        .collect();

    let variants = [HeapWidth(2), HeapWidth(4), HeapWidth(512)];

    ShortestPathExp::bench(c, "shortest_path", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
