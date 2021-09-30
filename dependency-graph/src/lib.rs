use petgraph::{stable_graph::StableDiGraph, Direction};

/// Must be implemented by the type you wish
/// to build a dependency graph for. See the README.md for an example
pub trait Node {
    /// Encodes a dependency relationship. In a Package Manager dependency graph for intance, this might be a (package name, version) tuple.
    /// It might also just be the exact same type as the one that implements the Node trait, in which case `Node::matches` can be implemented through simple equality.
    type DependencyType;

    /// Returns a slice of dependencies for this Node
    fn dependencies(&self) -> &[Self::DependencyType];

    /// Returns true if the `dependency` can be met by us.
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
}

/// Wrapper around dependency graph nodes.
/// Since a graph might have dependencies that cannot be resolved internally,
/// this wrapper is necessary to differentiate between internally resolved and
/// externally (unresolved) dependencies.
/// An Unresolved dependency does not necessarily mean that it *cannot* be resolved,
/// only that no Node within the graph fulfills it.
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

    pub fn as_resolved(&self) -> Option<&N> {
        match self {
            Step::Resolved(node) => Some(node),
            Step::Unresolved(_) => None,
        }
    }

    pub fn as_unresolved(&self) -> Option<&N::DependencyType> {
        match self {
            Step::Resolved(_) => None,
            Step::Unresolved(dependency) => Some(dependency),
        }
    }
}

/// The [`DependencyGraph`] structure builds an internal [Directed Graph](`petgraph::stable_graph::StableDiGraph`), which can then be traversed
/// in an order which ensures that dependent Nodes are visited before their parents.
pub struct DependencyGraph<'a, N: Node> {
    graph: StableDiGraph<Step<'a, N>, &'a N::DependencyType>,
}

/// The only way to build a [`DependencyGraph`] is from a slice of objects implementing [`Node`].
/// The graph references the original items, meaning the objects cannot be modified while
/// the [`DependencyGraph`] holds a reference to them.
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
                    // If we can, just add an edge between the two nodes.
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
    /// True if all graph [`Node`]s have only references to other internal [`Node`]s.
    /// That is, there are no unresolved dependencies between nodes.
    pub fn is_internally_resolvable(&self) -> bool {
        self.graph.node_weights().all(Step::is_resolved)
    }

    /// Get an iterator over unresolved dependencies, without traversing the whole graph.
    /// Useful for doing pre-validation or pre-fetching of external dependencies before
    /// starting to resolve internal dependencies.
    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType> {
        self.graph.node_weights().filter_map(Step::as_unresolved)
    }
}

/// Iterate over the DependencyGraph in an order which ensures dependencies are resolved before each Node is visited.
/// Note: If a `Step::Unresolved` node is returned, it is the caller's responsibility to ensure the dependency is resolved
/// before continuing.
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

        assert!(!graph.is_internally_resolvable());

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

    #[test]
    fn test_internally_resolved() {
        let packages = vec![
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
                    major: 3,
                    minor: 2,
                    patch: 0,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "base",
                    version: "=1.2.3".parse().unwrap(),
                }],
            },
            Package {
                name: "second_order",
                version: semver::Version {
                    major: 11,
                    minor: 2,
                    patch: 4,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "derived",
                    version: ">=3.0.0".parse().unwrap(),
                }],
            },
        ];

        let graph = DependencyGraph::from(&packages[..]);

        for package in graph {
            match package {
                // Print out the package name so we can verify the order ourselves
                Step::Resolved(package) => println!("Building {}!", package.name),

                // Since we know that all our Packages only have internal references to each other,
                // we can safely ignore any Unresolved steps in the graph.
                //
                // If for example `second_order` required some unknown package `external_package`,
                // iterating over our graph would yield that as a Step::Unresolved *before*
                // the `second_order` package.
                Step::Unresolved(_) => unreachable!(),
            }
        }
    }
}
