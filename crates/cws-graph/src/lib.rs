use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;
use cws_core::models::{
    Relationship, RelationshipType, GraphData, GraphNode, GraphEdge,
    PromptWithMeta, WorkflowWithSteps
};

pub struct GraphEngine;

impl GraphEngine {
    pub fn new() -> Self {
        Self
    }

    /// Format a collection of Prompts with Meta, Workflows with Steps, and Relationships into Cytoscape.js GraphData format.
    pub fn build_cytoscape_graph(
        &self,
        prompts: &[PromptWithMeta],
        workflows: &[WorkflowWithSteps],
        relationships: &[Relationship],
    ) -> GraphData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Add prompt nodes
        for p in prompts {
            let metadata = serde_json::json!({
                "description": p.prompt.description,
                "tags": p.tags,
                "categories": p.categories,
                "version": p.prompt.current_version,
            });
            nodes.push(GraphNode {
                id: p.prompt.id.to_string(),
                label: p.prompt.title.clone(),
                node_type: "prompt".to_string(),
                metadata,
            });
        }

        // Add workflow nodes
        for w in workflows {
            let metadata = serde_json::json!({
                "description": w.workflow.description,
                "step_count": w.steps.len(),
            });
            nodes.push(GraphNode {
                id: w.workflow.id.to_string(),
                label: w.workflow.title.clone(),
                node_type: "workflow".to_string(),
                metadata,
            });
        }

        // Add relationship edges
        for rel in relationships {
            edges.push(GraphEdge {
                id: rel.id.to_string(),
                source: rel.source_id.to_string(),
                target: rel.target_id.to_string(),
                edge_type: rel.relationship_type.as_str().to_string(),
                weight: rel.weight,
            });
        }

        GraphData { nodes, edges }
    }

    /// Query the neighborhood of a specific node up to `max_depth` (BFS).
    pub fn get_neighborhood(
        &self,
        start_id: Uuid,
        relationships: &[Relationship],
        max_depth: usize,
    ) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Build adjacency list
        let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for rel in relationships {
            adj.entry(rel.source_id).or_default().push(rel.target_id);
            adj.entry(rel.target_id).or_default().push(rel.source_id);
        }

        queue.push_back((start_id, 0));
        visited.insert(start_id);

        while let Some((curr, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(neighbors) = adj.get(&curr) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back((neighbor, depth + 1));
                    }
                }
            }
        }

        visited
    }

    /// Find all connected components in the graph.
    pub fn find_connected_components(
        &self,
        node_ids: &[Uuid],
        relationships: &[Relationship],
    ) -> Vec<Vec<Uuid>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        // Build adjacency list
        let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for rel in relationships {
            adj.entry(rel.source_id).or_default().push(rel.target_id);
            adj.entry(rel.target_id).or_default().push(rel.source_id);
        }

        for &node_id in node_ids {
            if visited.contains(&node_id) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(node_id);
            visited.insert(node_id);

            while let Some(curr) = queue.pop_front() {
                component.push(curr);

                if let Some(neighbors) = adj.get(&curr) {
                    for &neighbor in neighbors {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }

            components.push(component);
        }

        components
    }

    /// Auto-discover relationships based on shared tags and category similarity.
    pub fn discover_similarities(
        &self,
        prompts: &[PromptWithMeta],
        threshold: f32,
    ) -> Vec<Relationship> {
        let mut discoveries = Vec::new();

        for i in 0..prompts.len() {
            for j in (i + 1)..prompts.len() {
                let p1 = &prompts[i];
                let p2 = &prompts[j];

                // Calculate similarity weight
                let mut score = 0.0;

                // Category match
                let cat1 = p1.categories.first();
                let cat2 = p2.categories.first();
                if let (Some(c1), Some(c2)) = (cat1, cat2) {
                    if c1 == c2 && !c1.is_empty() {
                        score += 0.3;
                    }
                }

                // Shared tags
                let t1: HashSet<_> = p1.tags.iter().collect();
                let t2: HashSet<_> = p2.tags.iter().collect();
                let intersection: HashSet<_> = t1.intersection(&t2).collect();
                let union: HashSet<_> = t1.union(&t2).collect();

                if !union.is_empty() {
                    let jaccard = intersection.len() as f32 / union.len() as f32;
                    score += jaccard * 0.7;
                }

                if score >= threshold {
                    discoveries.push(Relationship::new(
                        p1.prompt.id,
                        p2.prompt.id,
                        RelationshipType::Similar,
                        score,
                    ));
                }
            }
        }

        discoveries
    }
}
