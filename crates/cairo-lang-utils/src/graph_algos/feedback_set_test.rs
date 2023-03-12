use std::collections::HashSet;
use std::hash::Hash;

use itertools::{chain, Itertools};
use test_case::test_case;
use test_log::test;

use crate::graph_algos::feedback_set::calc_feedback_set_for_graph;
use crate::graph_algos::graph_node::GraphNode;

// A node in the graph
#[derive(PartialEq, Eq, Hash, Clone)]
struct IntegerNode {
    id: usize,
    /// The neighbors of each node.
    graph: Vec<Vec<usize>>,
}
impl GraphNode for IntegerNode {
    type NodeId = usize;

    fn get_neighbors(&self) -> Vec<Self> {
        self.graph[self.id]
            .iter()
            .map(|neighbor_id| IntegerNode { id: *neighbor_id, graph: self.graph.clone() })
            .collect()
    }

    fn get_id(&self) -> Self::NodeId {
        self.id
    }
}

#[test]
fn test_list() {
    // Nodes 0 to 9 have only one neighbor (i -> i + 1), and 10 is a leaf.
    let mut graph: Vec<Vec<usize>> = (0..10).map(|id| vec![id + 1]).collect();
    graph.push(vec![]);

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert!(fset.is_empty());
    verify_feedback_set(&graph, fset);
}

#[test]
fn test_cycle() {
    // Each node has only one neighbor. i -> i + 1 for i = 0...8, and 9 -> 0.
    let graph: Vec<Vec<usize>> = (0..10).map(|id| vec![(id + 1) % 10]).collect();

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from([0]));
    verify_feedback_set(&graph, fset);
}

#[test]
fn test_root_points_to_cycle() {
    // 0 to 9 form a cycle.
    let mut graph: Vec<Vec<usize>> = (0..10).map(|id| vec![(id + 1) % 10]).collect();
    // And 10 (the root) has an edge to 0.
    graph.push(/* 10: */ vec![0]);

    // Note 10 is used as a root.
    let entry_points = vec![IntegerNode { id: 10, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from([0]));
    verify_feedback_set(&graph, fset);
}

#[test]
fn test_connected_cycles() {
    // 0 to 4 form one cycle and 5 to 9 form another cycle.
    let mut graph: Vec<Vec<usize>> =
        chain!((0..5).map(|id| vec![(id + 1) % 5]), (0..5).map(|id| vec![5 + (id + 1) % 5]))
            .collect();

    // 4 is connected to 5.
    graph[4].push(5);

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from([0, 5]));
    verify_feedback_set(&graph, fset);
}

#[test]
fn test_mesh() {
    // Each node has edges to all other nodes.
    let graph = (0..5).map(|i| (0..5).filter(|j| *j != i).collect::<Vec<usize>>()).collect_vec();

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from_iter(0..4));
    verify_feedback_set(&graph, fset);
}

// The feedback set depends on the root node we start from (but it's stable given the same root).
#[test_case(0, HashSet::from([0, 3]); "root_0")]
#[test_case(3, HashSet::from([3]); "root_3")]
fn test_tangent_cycles(root: usize, expected_fset: HashSet<usize>) {
    // 0 to 3 form one cycle and 3 to 6 form another cycle. Note 3 is in both.
    // 0 -> 1 -> 2 ->  3  -> 4 -> 6
    // ^______________| ^_________|
    let graph: Vec<Vec<usize>> = chain!(
        (0..3).map(|id| vec![id + 1]),
        // 3:
        vec![vec![0, 4]].into_iter(),
        (0..3).map(|id| vec![3 + (id + 2) % 4])
    )
    .collect();

    let entry_points = vec![IntegerNode { id: root, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, expected_fset);
    verify_feedback_set(&graph, fset);
}

// Test a graph with multiple cycles.
#[test_case(0, HashSet::from([0]); "root_0")]
#[test_case(1, HashSet::from([1, 2]); "root_1")]
#[test_case(2, HashSet::from([2, 3]); "root_2")]
#[test_case(3, HashSet::from([3]); "root_3")]
fn test_multiple_cycles(root: usize, expected_fset: HashSet<usize>) {
    let graph: Vec<Vec<usize>> = vec![
        // 0:
        vec![1, 2],
        // 1:
        vec![2, 3],
        // 2:
        vec![3],
        // 3:
        vec![0],
    ];

    let entry_points = vec![IntegerNode { id: root, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, expected_fset);
    verify_feedback_set(&graph, fset);
}

// Test a graph where the root is in a cycle and also calls another cycle.
#[test]
fn test_root_in_cycle_and_calls_cycle() {
    let graph: Vec<Vec<usize>> = vec![
        // 0:
        vec![0, 1],
        // 1:
        vec![1],
    ];

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from([0, 1]));
    verify_feedback_set(&graph, fset);
}

// Test a "random" complicated graph.
#[test]
fn test_complicated_graph_1() {
    let graph: Vec<Vec<usize>> = vec![
        // 0:
        vec![1],
        // 1:
        vec![2],
        // 2:
        vec![3, 4],
        // 3:
        vec![],
        // 4:
        vec![5, 6],
        // 5:
        vec![0, 2],
        // 6:
        vec![1, 2],
    ];

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from([0, 1, 2]));
    verify_feedback_set(&graph, fset);
}

// Test another "random" complicated graph (same graph, but 0 and 4 have self-edges).
#[test]
fn test_complicated_graph_2() {
    let graph: Vec<Vec<usize>> = vec![
        // 0:
        vec![0, 1],
        // 1:
        vec![2],
        // 2:
        vec![3, 4],
        // 3:
        vec![],
        // 4:
        vec![4, 5, 6],
        // 5:
        vec![0, 2],
        // 6:
        vec![1, 2],
    ];

    let entry_points = vec![IntegerNode { id: 0, graph: graph.clone() }];
    let fset = HashSet::<usize>::from_iter(calc_feedback_set_for_graph(&entry_points));
    assert_eq!(fset, HashSet::from([0, 1, 2, 4]));
    verify_feedback_set(&graph, fset);
}

/// Verifies the feedback set indeed covers all the cycles of the graph. This is done by removing
/// the feedback set from the original graph and verifying the new graph has no cycles.
fn verify_feedback_set(graph: &[Vec<usize>], fset: HashSet<usize>) {
    let new_graph = build_graph_removing_nodes(graph, fset);
    let mut visited = HashSet::new();
    for idx in 0..graph.len() {
        assert!(!contains_cycle(&new_graph, idx, &mut visited));
    }
}

/// Checks for cycles in the induced graph with the given root (that is, the graph, only with the
/// nodes reachable from the given root node).
fn contains_cycle(graph: &[Vec<usize>], root_idx: usize, visited: &mut HashSet<usize>) -> bool {
    let mut in_flight = HashSet::new();
    contains_cycle_recursive(graph, root_idx, &mut in_flight, visited)
}

fn contains_cycle_recursive(
    graph: &[Vec<usize>],
    cur_node_idx: usize,
    in_flight: &mut HashSet<usize>,
    visited: &mut HashSet<usize>,
) -> bool {
    if in_flight.contains(&cur_node_idx) {
        // Cycle found.
        return true;
    }
    if visited.contains(&cur_node_idx) {
        return false;
    }
    in_flight.insert(cur_node_idx);
    let res = graph[cur_node_idx]
        .iter()
        .any(|neighbor| contains_cycle_recursive(graph, *neighbor, in_flight, visited));
    in_flight.remove(&cur_node_idx);
    visited.insert(cur_node_idx);
    res
}

/// Build a new graph by removing nodes from the given graph.
fn build_graph_removing_nodes(
    graph: &[Vec<usize>],
    nodes_to_remove: HashSet<usize>,
) -> Vec<Vec<usize>> {
    let mut new_graph: Vec<Vec<usize>> = Vec::new();
    for (i, node) in graph.iter().enumerate() {
        if nodes_to_remove.contains(&i) {
            new_graph.push(vec![]);
        } else {
            let neighbors = node
                .iter()
                .filter(|neighbor| !nodes_to_remove.contains(neighbor))
                .cloned()
                .collect_vec();
            new_graph.push(neighbors)
        }
    }
    new_graph
}
