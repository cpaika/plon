use chrono::{DateTime, Utc};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    pub id: Uuid,
    pub from_task_id: Uuid,
    pub to_task_id: Uuid,
    pub dependency_type: DependencyType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    FinishToStart,   // Default: B starts after A finishes
    StartToStart,    // B starts when A starts
    FinishToFinish,  // B finishes when A finishes
    StartToFinish,   // B finishes when A starts (rare)
}

pub struct DependencyGraph {
    graph: DiGraph<Uuid, DependencyType>,
    node_map: HashMap<Uuid, NodeIndex>,
}

impl Dependency {
    pub fn new(from_task_id: Uuid, to_task_id: Uuid, dependency_type: DependencyType) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_task_id,
            to_task_id,
            dependency_type,
            created_at: Utc::now(),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task_id: Uuid) {
        if !self.node_map.contains_key(&task_id) {
            let node = self.graph.add_node(task_id);
            self.node_map.insert(task_id, node);
        }
    }

    pub fn add_dependency(&mut self, dependency: &Dependency) -> Result<(), String> {
        self.add_task(dependency.from_task_id);
        self.add_task(dependency.to_task_id);

        let from_node = self.node_map[&dependency.from_task_id];
        let to_node = self.node_map[&dependency.to_task_id];

        self.graph.add_edge(from_node, to_node, dependency.dependency_type);

        // Check for cycles
        if self.has_cycle() {
            self.graph.remove_edge(self.graph.find_edge(from_node, to_node).unwrap());
            return Err("Adding this dependency would create a cycle".to_string());
        }

        Ok(())
    }

    pub fn remove_dependency(&mut self, from_task_id: Uuid, to_task_id: Uuid) -> bool {
        if let (Some(&from_node), Some(&to_node)) = 
            (self.node_map.get(&from_task_id), self.node_map.get(&to_task_id))
            && let Some(edge) = self.graph.find_edge(from_node, to_node) {
                self.graph.remove_edge(edge);
                return true;
            }
        false
    }

    pub fn has_cycle(&self) -> bool {
        toposort(&self.graph, None).is_err()
    }

    pub fn topological_sort(&self) -> Result<Vec<Uuid>, String> {
        match toposort(&self.graph, None) {
            Ok(sorted_nodes) => {
                Ok(sorted_nodes
                    .into_iter()
                    .map(|node| self.graph[node])
                    .collect())
            }
            Err(_) => Err("Graph contains a cycle".to_string()),
        }
    }

    pub fn get_dependencies(&self, task_id: Uuid) -> Vec<(Uuid, DependencyType)> {
        if let Some(&node) = self.node_map.get(&task_id) {
            self.graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .map(|edge| {
                    (self.graph[edge.source()], *edge.weight())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_dependents(&self, task_id: Uuid) -> Vec<(Uuid, DependencyType)> {
        if let Some(&node) = self.node_map.get(&task_id) {
            self.graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .map(|edge| {
                    (self.graph[edge.target()], *edge.weight())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_dependencies(&self) -> Vec<Dependency> {
        self.graph
            .edge_indices()
            .map(|edge_idx| {
                let (from_node, to_node) = self.graph.edge_endpoints(edge_idx).unwrap();
                let dependency_type = *self.graph.edge_weight(edge_idx).unwrap();
                Dependency::new(
                    self.graph[from_node],
                    self.graph[to_node],
                    dependency_type,
                )
            })
            .collect()
    }

    pub fn can_start_task(&self, task_id: Uuid, completed_tasks: &HashSet<Uuid>) -> bool {
        let dependencies = self.get_dependencies(task_id);
        
        for (dep_task_id, dep_type) in dependencies {
            match dep_type {
                DependencyType::FinishToStart => {
                    if !completed_tasks.contains(&dep_task_id) {
                        return false;
                    }
                }
                // For other types, would need more complex logic with task states
                _ => {}
            }
        }
        true
    }

    pub fn get_critical_path(&self, task_estimates: &HashMap<Uuid, f32>) -> Vec<Uuid> {
        // Simplified critical path - longest path through the graph
        if let Ok(sorted) = self.topological_sort() {
            let mut distances: HashMap<Uuid, f32> = HashMap::new();
            let mut predecessors: HashMap<Uuid, Option<Uuid>> = HashMap::new();

            for task_id in &sorted {
                let deps = self.get_dependencies(*task_id);
                let max_distance = deps
                    .iter()
                    .map(|(dep_id, _)| {
                        distances.get(dep_id).unwrap_or(&0.0) + 
                        task_estimates.get(dep_id).unwrap_or(&0.0)
                    })
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                distances.insert(*task_id, max_distance);
                
                if let Some((pred_id, _)) = deps.iter().max_by(|(a, _), (b, _)| {
                    let a_dist = distances.get(a).unwrap_or(&0.0) + task_estimates.get(a).unwrap_or(&0.0);
                    let b_dist = distances.get(b).unwrap_or(&0.0) + task_estimates.get(b).unwrap_or(&0.0);
                    a_dist.partial_cmp(&b_dist).unwrap()
                }) {
                    predecessors.insert(*task_id, Some(*pred_id));
                } else {
                    predecessors.insert(*task_id, None);
                }
            }

            // Find the end task with maximum distance
            if let Some(end_task) = sorted.iter().max_by(|a, b| {
                let a_dist = distances.get(a).unwrap_or(&0.0) + task_estimates.get(a).unwrap_or(&0.0);
                let b_dist = distances.get(b).unwrap_or(&0.0) + task_estimates.get(b).unwrap_or(&0.0);
                a_dist.partial_cmp(&b_dist).unwrap()
            }) {
                // Trace back the path
                let mut path = Vec::new();
                let mut current = Some(*end_task);
                
                while let Some(task) = current {
                    path.push(task);
                    current = predecessors.get(&task).and_then(|p| *p);
                }
                
                path.reverse();
                return path;
            }
        }
        
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dependency() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let dep = Dependency::new(from_id, to_id, DependencyType::FinishToStart);
        
        assert_eq!(dep.from_task_id, from_id);
        assert_eq!(dep.to_task_id, to_id);
        assert_eq!(dep.dependency_type, DependencyType::FinishToStart);
    }

    #[test]
    fn test_dependency_graph_basic() {
        let mut graph = DependencyGraph::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();
        
        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);
        
        let dep1 = Dependency::new(task1, task2, DependencyType::FinishToStart);
        let dep2 = Dependency::new(task2, task3, DependencyType::FinishToStart);
        
        assert!(graph.add_dependency(&dep1).is_ok());
        assert!(graph.add_dependency(&dep2).is_ok());
        
        assert!(!graph.has_cycle());
        
        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], task1);
        assert_eq!(sorted[1], task2);
        assert_eq!(sorted[2], task3);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();
        
        let dep1 = Dependency::new(task1, task2, DependencyType::FinishToStart);
        let dep2 = Dependency::new(task2, task3, DependencyType::FinishToStart);
        let dep3 = Dependency::new(task3, task1, DependencyType::FinishToStart); // Creates cycle
        
        assert!(graph.add_dependency(&dep1).is_ok());
        assert!(graph.add_dependency(&dep2).is_ok());
        assert!(graph.add_dependency(&dep3).is_err());
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_get_dependencies_and_dependents() {
        let mut graph = DependencyGraph::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();
        
        graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
        graph.add_dependency(&Dependency::new(task3, task2, DependencyType::StartToStart)).unwrap();
        
        let deps = graph.get_dependencies(task2);
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|(id, _)| *id == task1));
        assert!(deps.iter().any(|(id, _)| *id == task3));
        
        let dependents = graph.get_dependents(task1);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].0, task2);
    }

    #[test]
    fn test_can_start_task() {
        let mut graph = DependencyGraph::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();
        
        graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
        graph.add_dependency(&Dependency::new(task2, task3, DependencyType::FinishToStart)).unwrap();
        
        let mut completed = HashSet::new();
        
        assert!(graph.can_start_task(task1, &completed));
        assert!(!graph.can_start_task(task2, &completed));
        assert!(!graph.can_start_task(task3, &completed));
        
        completed.insert(task1);
        assert!(graph.can_start_task(task2, &completed));
        assert!(!graph.can_start_task(task3, &completed));
        
        completed.insert(task2);
        assert!(graph.can_start_task(task3, &completed));
    }

    #[test]
    fn test_critical_path() {
        let mut graph = DependencyGraph::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();
        let task4 = Uuid::new_v4();
        
        // Create a diamond dependency
        // task1 -> task2 -> task4
        //      \-> task3 ->/
        graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
        graph.add_dependency(&Dependency::new(task1, task3, DependencyType::FinishToStart)).unwrap();
        graph.add_dependency(&Dependency::new(task2, task4, DependencyType::FinishToStart)).unwrap();
        graph.add_dependency(&Dependency::new(task3, task4, DependencyType::FinishToStart)).unwrap();
        
        let mut estimates = HashMap::new();
        estimates.insert(task1, 2.0);
        estimates.insert(task2, 5.0); // Longer path
        estimates.insert(task3, 1.0);
        estimates.insert(task4, 1.0);
        
        let critical_path = graph.get_critical_path(&estimates);
        assert_eq!(critical_path.len(), 3);
        assert_eq!(critical_path[0], task1);
        assert_eq!(critical_path[1], task2);
        assert_eq!(critical_path[2], task4);
    }

    #[test]
    fn test_remove_dependency() {
        let mut graph = DependencyGraph::new();
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        
        graph.add_dependency(&Dependency::new(task1, task2, DependencyType::FinishToStart)).unwrap();
        assert_eq!(graph.get_dependencies(task2).len(), 1);
        
        assert!(graph.remove_dependency(task1, task2));
        assert_eq!(graph.get_dependencies(task2).len(), 0);
        
        assert!(!graph.remove_dependency(task1, task2));
    }
}