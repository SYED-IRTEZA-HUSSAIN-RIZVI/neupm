//! Resolution engine: recursively resolves a package and all its transitive
//! dependencies into a complete, deduplicated installation plan.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Context, Result};
use reqwest::Client;

use crate::core::package::PackageMetadata;
use crate::registry::npm::adapter::NpmRegistryAdapter;
use crate::resolver::dependency_graph::DependencyGraph;
use crate::resolver::version_selector::select_version;

/// A single resolved package entry in the installation plan.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub metadata: PackageMetadata,
    /// Whether this is a direct (user-requested) dependency.
    pub is_direct: bool,
}

/// The result of resolving a package and its transitive dependencies.
#[derive(Debug)]
pub struct ResolutionResult {
    /// Complete set of packages to install, in dependency order.
    pub packages: Vec<ResolvedPackage>,
    /// Dependency graph (can be used for lockfile generation).
    pub graph: DependencyGraph,
}

/// Engine for resolving packages and their transitive dependencies.
pub struct ResolutionEngine<'a> {
    client: &'a Client,
    adapter: NpmRegistryAdapter,
}

impl<'a> ResolutionEngine<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
            adapter: NpmRegistryAdapter::default(),
        }
    }

    /// Resolve a single package specification and all its transitive runtime
    /// dependencies into a complete `ResolutionResult`.
    ///
    /// Uses BFS to traverse the dependency tree. Packages already present in
    /// the graph (by name) are not re-fetched.
    pub async fn resolve(
        &self,
        package_name: &str,
        version_req: Option<&str>,
    ) -> Result<ResolutionResult> {
        let mut graph = DependencyGraph::new();
        // Queue: (name, version_requirement, is_direct)
        let mut queue: VecDeque<(String, Option<String>, bool)> = VecDeque::new();
        queue.push_back((
            package_name.to_string(),
            version_req.map(|s| s.to_string()),
            true,
        ));

        // Track which packages we've started resolving to avoid re-queuing.
        let mut enqueued: HashSet<String> = HashSet::new();
        enqueued.insert(package_name.to_string());

        // Track direct vs transitive.
        let mut direct_set: HashSet<String> = HashSet::new();
        direct_set.insert(package_name.to_string());

        // Map of name → resolved metadata for the BFS.
        let mut resolved_meta: HashMap<String, PackageMetadata> = HashMap::new();

        while let Some((name, req, _is_direct)) = queue.pop_front() {
            if graph.contains(&name) {
                continue;
            }

            // Fetch raw npm metadata.
            let raw_meta = self
                .adapter
                .resolve_raw(self.client, &name)
                .await
                .with_context(|| format!("Failed to fetch metadata for '{name}'"))?;

            // Select the best matching version.
            let version = select_version(&raw_meta, req.as_deref())
                .with_context(|| format!("Version resolution failed for '{name}'"))?;

            // Extract full package metadata for the resolved version.
            let meta = crate::registry::npm::metadata::extract_version_metadata(
                &name, &raw_meta, &version,
            )
            .with_context(|| format!("Failed to extract metadata for '{name}@{version}'"))?;

            // Enqueue runtime dependencies.
            for (dep_name, dep_range) in &meta.dependencies {
                if !enqueued.contains(dep_name) {
                    enqueued.insert(dep_name.clone());
                    queue.push_back((dep_name.clone(), Some(dep_range.clone()), false));
                }
            }

            // Enqueue optional dependencies (tolerate individual failures).
            for (dep_name, dep_range) in &meta.optional_dependencies {
                if !enqueued.contains(dep_name) {
                    enqueued.insert(dep_name.clone());
                    queue.push_back((dep_name.clone(), Some(dep_range.clone()), false));
                }
            }

            resolved_meta.insert(name.clone(), meta.clone());
            graph.insert(&meta);
        }

        // Topological sort for installation order.
        let ordered_names = graph.installation_order()?;

        let packages = ordered_names
            .iter()
            .filter_map(|name| {
                resolved_meta.get(name).map(|meta| ResolvedPackage {
                    metadata: meta.clone(),
                    is_direct: direct_set.contains(name),
                })
            })
            .collect();

        Ok(ResolutionResult { packages, graph })
    }
}
