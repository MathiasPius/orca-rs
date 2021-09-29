use petgraph::{stable_graph::StableDiGraph, Direction};

pub trait Node {
    type DependencyType;

    fn dependencies(&self) -> &[Self::DependencyType];
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
}

pub enum Step<'a, N: Node> {
    Resolved(&'a N),
    Unresolved(&'a N::DependencyType),
}

impl<'a, N: Node> Step<'a, N> {
    pub fn is_resolved(&self) -> bool {
        match self {
            Step::Resolved(_) => true,
            Step::Unresolved(_) => false,
        }
    }
}

pub struct DependencyGraph<'a, N: Node> {
    graph: StableDiGraph<Step<'a, N>, &'a N::DependencyType>,
}

impl<'a, N> From<&'a [N]> for DependencyGraph<'a, N>
where
    N: Node,
{
    fn from(nodes: &'a [N]) -> Self {
        let mut graph = StableDiGraph::<Step<'a, N>, &'a N::DependencyType>::new();

        // Insert the input nodes into the graph, and record their positions.
        // We'll be adding the edges next, and filling in any unresolved
        // steps we find along the way.
        let nodes: Vec<(_, _)> = nodes
            .iter()
            .map(|node| (node, graph.add_node(Step::Resolved(node))))
            .collect();

        for (node, index) in nodes.iter() {
            for dependency in node.dependencies() {
                // Check to see if we can resolve this dependency internally.
                if let Some((_, dependent)) = nodes.iter().find(|(dep, _)| dep.matches(dependency))
                {
                    // IF we can, just add an edge between the two nodes.
                    graph.add_edge(*index, *dependent, dependency);
                } else {
                    // If not, create a new "Unresolved" node, and create an edge to that.
                    let unresolved = graph.add_node(Step::Unresolved(dependency));
                    graph.add_edge(*index, unresolved, dependency);
                }
            }
        }

        Self { graph }
    }
}

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    pub fn is_internally_resolved(&self) -> bool {
        self.graph.node_weights().all(Step::is_resolved)
    }
}

impl<'a, N> Iterator for DependencyGraph<'a, N>
where
    N: Node,
{
    type Item = Step<'a, N>;

    fn next(&mut self) -> Option<Self::Item> {
        // Returns the first node, which does not have any Outgoing
        // edges, which means it is terminal.
        for index in self.graph.node_indices().rev() {
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

#[cfg(test)]
mod tests {

    use crate::{DependencyGraph, Node, Step};
    use semver::{BuildMetadata, Prerelease, Version, VersionReq};

    #[derive(Debug)]
    struct Package {
        name: &'static str,
        version: Version,
        dependencies: Vec<Dependency>,
    }

    #[derive(Debug)]
    struct Dependency {
        name: &'static str,
        version: VersionReq,
    }

    impl Node for Package {
        type DependencyType = Dependency;

        fn dependencies(&self) -> &[Self::DependencyType] {
            &self.dependencies[..]
        }

        fn matches(&self, dependency: &Self::DependencyType) -> bool {
            self.name == dependency.name && dependency.version.matches(&self.version)
        }
    }

    #[test]
    fn test_dependencies_synchronous() {
        let build = build_test_graph();
        let graph = DependencyGraph::from(&build[..]);

        assert!(!graph.is_internally_resolved());

        for node in graph {
            match node {
                Step::Resolved(build) => println!("build: {:?}", build.name),
                Step::Unresolved(lookup) => println!("lookup: {:?}", lookup.name),
            }
        }
    }

    #[test]
    fn test_generate_dependency_graph() {
        DependencyGraph::from(&build_test_graph()[..]);
    }

    fn build_test_graph() -> Vec<Package> {
        vec![
            Package {
                name: "base",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            Package {
                name: "derived",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "base",
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            Package {
                name: "second_order",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "derived",
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            Package {
                name: "converged",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![
                    Dependency {
                        name: "base",
                        version: ">=1.0.0".parse().unwrap(),
                    },
                    Dependency {
                        name: "derived",
                        version: ">=1.0.0".parse().unwrap(),
                    },
                ],
            },
            Package {
                name: "independent",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            Package {
                name: "external",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "unknown",
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
        ]
    }
}
