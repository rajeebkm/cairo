//! A feedback-vertex-set is a set of vertices whose removal leaves a graph without cycles
//! (<https://en.wikipedia.org/wiki/Feedback_vertex_set>).
//! We use this algorithm to spot the relevant places for adding `get_gas` statements in the
//! resulting Sierra code - there should be a `get_gas` call in every recursive call, or in other
//! words, in any cycle in the function call graph.
//! An efficient algorithm to find the minimum feedback-vertex-set in a directed graph is not known,
//! so here we implement some straight-forward algorithm that guarantees to cover all the cycles in
//! the graph, but doesn't necessarily produce the minimum size of such a set.

use std::collections::HashSet;

use super::graph_node::GraphNode;

#[cfg(test)]
#[path = "feedback_set_test.rs"]
mod feedback_set_test;

/// Context for the feedback-set algorithm.
#[derive(Default)]
struct FeedbackSetAlgoContext<Node: GraphNode> {
    /// The accumulated feedback set so far in the process of the algorithm. In the end of the
    /// algorithm, this is also the result.
    pub feedback_set: HashSet<Node::NodeId>,
    /// Nodes that are currently during the recursion call on them. That is - if one of these is
    /// reached, it indicates it's in some cycle that was not "resolved" yet.
    pub in_flight: HashSet<Node::NodeId>,
    /// The set of nodes that were visited during the algorithm so far.
    pub visited: HashSet<Node::NodeId>,
}
impl<Node: GraphNode> FeedbackSetAlgoContext<Node> {
    fn new() -> Self {
        FeedbackSetAlgoContext {
            feedback_set: HashSet::<Node::NodeId>::new(),
            in_flight: HashSet::<Node::NodeId>::new(),
            visited: HashSet::<Node::NodeId>::new(),
        }
    }
}

/// Calculates a feedback-vertex-set of the graph induced by the given entry points (that is - the
/// graph reachable from the set of entry points).
/// The feedback-vertex-set is not guaranteed to be minimal.
pub fn calc_feedback_set_for_graph<Node: GraphNode>(
    entry_points: &[Node],
) -> HashSet<Node::NodeId> {
    let mut ctx = FeedbackSetAlgoContext::<Node>::new();
    for entry_point in entry_points {
        calc_feedback_set_recursive(&mut ctx, entry_point);
    }
    ctx.feedback_set
}

fn calc_feedback_set_recursive<Node: GraphNode>(
    ctx: &mut FeedbackSetAlgoContext<Node>,
    node: &Node,
) {
    let cur_node_id = node.get_id();
    if ctx.visited.contains(&cur_node_id) {
        return;
    }
    ctx.in_flight.insert(cur_node_id.clone());

    let neighbors = node.get_neighbors();
    for neighbor in neighbors {
        let neighbor_id = neighbor.get_id();
        if ctx.feedback_set.contains(&neighbor_id) {
            continue;
        } else if ctx.in_flight.contains(&neighbor_id) {
            ctx.feedback_set.insert(neighbor_id);
        } else {
            calc_feedback_set_recursive(ctx, &neighbor);
        }
    }
    ctx.in_flight.remove(&cur_node_id);
    ctx.visited.insert(cur_node_id.clone());
}
