//! Dependency graph construction and cycle detection.
//!
//! The dependency graph models packages as nodes and their dependency
//! relationships as directed edges. It supports deterministic topological
//! ordering and cycle detection before any filesystem changes are made.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Result, bail};

use crate::core::package::PackageMetadata;

/// A node in the dependency graph representing one resolved package.
#[derive(Debug, Clone)]
pub struct DepNode {
    /// Package name (e.g., `react` or `@types/node`).
    pub name: String,
    /// Resolved concrete version.
    pub version: String,
    /// Names of direct runtime dependencies.
    pub dep_names: Vec<String>,
}

/// A directed dependency graph. Edges go from dependent → dependency.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Map from package name to its resolved node.
    nodes: HashMap<String, DepNode>,
    /// Insertion order for deterministic output.
    insertion_order: Vec<String>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or overwrite a package node.
    pub fn insert(&mut self, meta: &PackageMetadata) {
        let name = meta.name.clone();
        let node = DepNode {
            name: name.clone(),
            version: meta.version.clone(),
            dep_names: meta.dependencies.keys().cloned().collect(),
        };
        if !self.nodes.contains_key(&name) {
            self.insertion_order.push(name.clone());
        }
        self.nodes.insert(name, node);
    }

    /// Returns `true` if the graph already contains this package.
    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    /// Retrieve a node by package name.
    pub fn get(&self, name: &str) -> Option<&DepNode> {
        self.nodes.get(name)
    }

    /// All package names in the graph.
    pub fn names(&self) -> &[String] {
        &self.insertion_order
    }

    /// Total number of packages in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Produce a list of package names in installation order (topological sort).
    ///
    /// Dependencies appear before the packages that require them.
    /// Returns an error if a dependency cycle is detected.
    pub fn installation_order(&self) -> Result<Vec<String>> {
        // dependents: map from dependency -> list of packages that depend on it
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
        // in_degree: number of dependencies each package depends on (that are in the graph)
        let mut in_degree: HashMap<&str, usize> = HashMap::new();

        for name in &self.insertion_order {
            in_degree.insert(name.as_str(), 0);
        }

        for (name, node) in &self.nodes {
            let mut dep_count = 0;
            for dep in &node.dep_names {
                if self.nodes.contains_key(dep.as_str()) {
                    dep_count += 1;
                    dependents
                        .entry(dep.as_str())
                        .or_default()
                        .push(name.as_str());
                }
            }
            *in_degree.get_mut(name.as_str()).unwrap() = dep_count;
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        // Sort for determinism.
        let mut queue_vec: Vec<&str> = queue.drain(..).collect();
        queue_vec.sort_unstable();
        queue.extend(queue_vec);

        let mut result = Vec::with_capacity(self.nodes.len());
        let mut visited: HashSet<&str> = HashSet::new();

        while let Some(name) = queue.pop_front() {
            if visited.contains(name) {
                continue;
            }
            visited.insert(name);
            result.push(name.to_string());

            if let Some(parents) = dependents.get(name) {
                let mut next: Vec<&str> = Vec::new();
                for &parent in parents {
                    if let Some(deg) = in_degree.get_mut(parent) {
                        *deg -= 1;
                        if *deg == 0 {
                            next.push(parent);
                        }
                    }
                }
                next.sort_unstable();
                queue.extend(next);
            }
        }

        if result.len() != self.nodes.len() {
            bail!(
                "Dependency cycle detected in the package graph. \
                 Packages involved: {:?}",
                self.insertion_order
                    .iter()
                    .filter(|n| !result.contains(n))
                    .collect::<Vec<_>>()
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::package::PackageMetadata;
    use std::collections::HashMap;

    fn make_meta(name: &str, version: &str, deps: &[(&str, &str)]) -> PackageMetadata {
        PackageMetadata {
            name: name.to_string(),
            version: version.to_string(),
            tarball_url: "https://example.com/pkg.tgz".to_string(),
            integrity: "sha512-abc".to_string(),
            dependencies: deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
        }
    }

    #[test]
    fn test_basic_insert_and_lookup() {
        let mut graph = DependencyGraph::new();
        graph.insert(&make_meta("react", "19.0.0", &[]));
        assert!(graph.contains("react"));
        assert_eq!(graph.get("react").unwrap().version, "19.0.0");
    }

    #[test]
    fn test_installation_order_linear_chain() {
        let mut graph = DependencyGraph::new();
        // c → b → a (a has no deps)
        graph.insert(&make_meta("a", "1.0.0", &[]));
        graph.insert(&make_meta("b", "1.0.0", &[("a", "^1.0.0")]));
        graph.insert(&make_meta("c", "1.0.0", &[("b", "^1.0.0")]));

        let order = graph.installation_order().unwrap();
        // 'a' must come before 'b', 'b' before 'c'
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_no_cycle_simple() {
        let mut graph = DependencyGraph::new();
        graph.insert(&make_meta("pkg-a", "1.0.0", &[]));
        graph.insert(&make_meta("pkg-b", "1.0.0", &[("pkg-a", "^1.0.0")]));
        assert!(graph.installation_order().is_ok());
    }

    #[test]
    fn test_independent_packages_deterministic() {
        let mut graph = DependencyGraph::new();
        graph.insert(&make_meta("zebra", "1.0.0", &[]));
        graph.insert(&make_meta("alpha", "1.0.0", &[]));
        graph.insert(&make_meta("mango", "1.0.0", &[]));
        let order = graph.installation_order().unwrap();
        // Should be sorted alphabetically for determinism.
        assert_eq!(order, vec!["alpha", "mango", "zebra"]);
    }
}
