use super::spec::{BuildSpec, DependencyDeclaration};
use dependency_graph::Node;

impl Node for BuildSpec {
    type DependencyType = DependencyDeclaration;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies[..]
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        dependency.name == self.name && dependency.version.matches(&self.version)
    }
}

#[cfg(test)]
mod tests {

    use crate::build::spec::{BuildSpec, DependencyDeclaration};
    use dependency_graph::{DependencyGraph, Step};
    use semver::{BuildMetadata, Prerelease};

    #[test]
    fn test_dependencies_synchronous() {
        let build = build_test_graph();
        let graph = DependencyGraph::from(&build[..]);

        for node in graph {
            match node {
                Step::Resolved(build) => println!("build: {:?}", build),
                Step::Unresolved(lookup) => println!("lookup: {:?}", lookup),
            }
        }
    }

    #[test]
    fn test_generate_dependency_graph() {
        DependencyGraph::from(&build_test_graph()[..]);
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
