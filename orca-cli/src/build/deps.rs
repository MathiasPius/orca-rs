use petgraph::{stable_graph::StableGraph, Direction};

use super::spec::{BuildSpec, DependencyDeclaration};

pub trait Node {
    type DependencyType: Dependency;

    fn dependencies(&self) -> Self::DependencyType;
}

pub trait Dependency {
    fn matches(node: &impl Node) -> bool;
}



#[derive(Debug, Clone)]
pub enum DependencyNode<'a> {
    Build(&'a BuildSpec),
    Lookup(&'a DependencyDeclaration),
}

impl<'a> From<&'a BuildSpec> for DependencyNode<'a> {
    fn from(spec: &'a BuildSpec) -> Self {
        DependencyNode::Build(spec)
    }
}

impl<'a> From<&'a DependencyDeclaration> for DependencyNode<'a> {
    fn from(declaration: &'a DependencyDeclaration) -> Self {
        DependencyNode::Lookup(declaration)
    }
}

type InnerGraph<'a> = StableGraph<DependencyNode<'a>, &'a DependencyDeclaration>;
pub struct DependencyGraph<'a> {
    builds: &'a Vec<BuildSpec>,
    graph: InnerGraph<'a>,
}

impl<'a> From<&'a Vec<BuildSpec>> for DependencyGraph<'a> {
    fn from(builds: &'a Vec<BuildSpec>) -> Self {
        let mut graph = InnerGraph::<'a>::new();

        let mut nodes = Vec::new();

        // Populate our graph with all our images
        for spec in builds {
            nodes.push((spec, graph.add_node(DependencyNode::from(spec))));
        }

        for (spec, node) in nodes.iter() {
            for dependency in &spec.dependencies {
                // See if we can find a build in our own build tree that matches the VersionReq
                if let Some((_, dependent_node)) = nodes.iter().find(|(build, _)| {
                    build.name == dependency.name && dependency.version.matches(&build.version)
                }) {
                    // If we can, we will add it as a dependency in our build graph.
                    graph.add_edge(*node, *dependent_node, dependency);
                } else {
                    // If we can't find a build in our own BuildSpec list that matches our dependency,
                    // we'll have to inject an external dependency lookup into the tree.
                    let external_dependency = graph.add_node(DependencyNode::Lookup(dependency));
                    graph.add_edge(*node, external_dependency, dependency);
                }
            }
        }

        DependencyGraph { graph, builds }
    }
}

impl<'a> DependencyGraph<'a> {}

impl<'a> Iterator for DependencyGraph<'a> {
    type Item = DependencyNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for index in self.graph.node_indices() {
            if self
                .graph
                .neighbors_directed(index, Direction::Outgoing)
                .count()
                == 0
            {
                return self.graph.remove_node(index);
            }
        }

        None
    }
}


/*
impl<'a> DependencyGraph<'a> {
    /// The DependencyGraph is consumed, because the resolution mutates
    /// the graph state, and may leave it in an incomplete state.
    pub fn resolve<F: FnMut(BuildSpec)>(mut self, callback: F) {
        let mut build_queue = vec![];

        // Find all nodes in the graph with no Outgoing connections
        // since this means that they do not have any dependencies
        // and therefore can be built immediately.
        for node in self.graph.node_indices() {
            if self.graph.neighbors_directed(node, Direction::Outgoing).count() == 0 {
                build_queue.push(node);
            }
        }

        while let Some(node) = build_queue.pop() {
            debug!("Built {:#?}", self.graph.node_weight(node).unwrap().0);
            // Find all BuildSpecs which depend on this
            let dependents = self.graph.neighbors_directed(node, Direction::Incoming);
            for dependent in dependents {
                // If the dependent only has one dependency (us), schedule it.
                let dependencies: Vec<_> = self.graph
                    .neighbors_directed(dependent, Direction::Outgoing)
                    .collect();
                if dependencies.len() == 1 {
                    println!(
                        "Scheduling dependent image: {:?}",
                        self.graph.node_weight(dependent).unwrap().0
                    );
                    build_queue.push(dependent);
                } else {
                    println!(
                        "Dependent image {:?} still has other dependencies: {:?}",
                        self.graph.node_weight(dependent).unwrap().0,
                        dependencies
                            .iter()
                            .map(|id| self.graph.node_weight(*id).unwrap().0)
                            .collect::<Vec<_>>()
                    );
                }
            }
            self.graph.remove_node(node);
        }
    }
}

impl<'a> From<&'a Vec<BuildSpec>> for DependencyGraph<'a> {
    fn from(builds: &'a Vec<BuildSpec>) -> Self {
        let mut graph = InnerGraph::<'a>::new();

        let mut nodes = Vec::new();

        // Populate our graph with all our images
        for spec in builds {
            nodes.push((spec, graph.add_node((&spec.name, &spec.version))));
        }

        for (spec, node) in nodes.iter() {
            for dependency in &spec.dependencies {
                for (dep_spec, dep_node) in nodes.iter() {
                    if dep_spec.name == dependency.name && dependency.version.matches(&dep_spec.version)
                    {
                        graph.add_edge(*node, *dep_node, (&spec.name, &dependency.version));
                    }
                }
            }
        }

        DependencyGraph {
            graph,
            builds
        }
    }
}
*/

#[cfg(test)]
mod tests {

    use semver::{BuildMetadata, Prerelease};
    use crate::build::{deps::{DependencyGraph, DependencyNode}, spec::{BuildSpec, DependencyDeclaration}};

    #[test]
    fn test_dependencies_synchronous() {
        let build = build_test_graph();
        let graph = DependencyGraph::from(&build);

        for node in graph {
            match node {
                DependencyNode::Build(build) => println!("build: {:?}", build),
                DependencyNode::Lookup(lookup) => println!("lookup: {:?}", lookup),
            }
        }
    }

    #[test]
    fn test_generate_dependency_graph() {
        DependencyGraph::from(&build_test_graph());
    }

    fn build_test_graph() -> Vec<BuildSpec> {
        vec![
            BuildSpec {
                name: "base".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            BuildSpec {
                name: "derived".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![DependencyDeclaration {
                    name: "base".to_string(),
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            BuildSpec {
                name: "second_order".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![DependencyDeclaration {
                    name: "derived".to_string(),
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            BuildSpec {
                name: "converged".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![
                    DependencyDeclaration {
                        name: "base".to_string(),
                        version: ">=1.0.0".parse().unwrap(),
                    },
                    DependencyDeclaration {
                        name: "derived".to_string(),
                        version: ">=1.0.0".parse().unwrap(),
                    },
                ],
            },
            BuildSpec {
                name: "independent".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            BuildSpec {
                name: "external".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![DependencyDeclaration {
                    name: "unknown".to_string(),
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
        ]
    }
}

/*

#[cfg(test)]
mod tests {
    use petgraph::Direction;
    use semver::{BuildMetadata, Prerelease};

    use crate::build::{deps::DependencyGraph, spec::{BuildSpec, DependencyDeclaration}};

    #[test]
    fn test_generate_dependency_graph() {
        let builds = vec![
            BuildSpec {
                name: "base".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            BuildSpec {
                name: "derived".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![DependencyDeclaration {
                    name: "base".to_string(),
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            BuildSpec {
                name: "second_order".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![DependencyDeclaration {
                    name: "derived".to_string(),
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            BuildSpec {
                name: "converged".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![
                    DependencyDeclaration {
                        name: "base".to_string(),
                        version: ">=1.0.0".parse().unwrap(),
                    },
                    DependencyDeclaration {
                        name: "derived".to_string(),
                        version: ">=1.0.0".parse().unwrap(),
                    },
                ],
            },
            BuildSpec {
                name: "independent".to_string(),
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
        ];

        let mut graph = DependencyGraph::from(&builds);

        let mut starts = Vec::new();

        for node in graph.node_indices() {
            if graph.neighbors_directed(node, Direction::Outgoing).count() == 0 {
                // Terminal node
                starts.push(node);
            }
        }

        while let Some(node) = starts.pop() {
            println!("Built {:#?}", graph.node_weight(node).unwrap().0);
            // Find all BuildSpecs which depend on this
            let dependents = graph.neighbors_directed(node, Direction::Incoming);
            for dependent in dependents {
                // If the dependent only has one dependency (us), schedule it.
                let dependencies: Vec<_> = graph
                    .neighbors_directed(dependent, Direction::Outgoing)
                    .collect();
                if dependencies.len() == 1 {
                    println!(
                        "Scheduling dependent image: {:?}",
                        graph.node_weight(dependent).unwrap().0
                    );
                    starts.push(dependent);
                } else {
                    println!(
                        "Dependent image {:?} still has other dependencies: {:?}",
                        graph.node_weight(dependent).unwrap().0,
                        dependencies
                            .iter()
                            .map(|id| graph.node_weight(*id).unwrap().0)
                            .collect::<Vec<_>>()
                    );
                }
            }
            graph.remove_node(node);
        }

        // If everything went well, our graph should be empty, meaning everything was built.
        assert_eq!(graph.node_count(), 0);
    }
}

 */
