use petgraph::{stable_graph::StableGraph};
use semver::{Version, VersionReq};

use super::spec::{BuildSpec};

type DependencyGraph<'a> = StableGraph::<(&'a str, &'a Version), (&'a str, &'a VersionReq)>;

pub fn generate_dependency_graph<'a>(builds: &'a Vec<BuildSpec>) -> DependencyGraph<'a> {
    let mut deps = DependencyGraph::<'a>::new();

    let mut nodes = Vec::new();

    // Populate our graph with all our images
    for spec in builds {
        nodes.push((spec, deps.add_node((&spec.name, &spec.version))));
    }
    
    for (spec, node) in nodes.iter() {
        for dependency in &spec.dependencies {
            for (dep_spec, dep_node) in nodes.iter() {
                if dep_spec.name == dependency.name && dependency.version.matches(&dep_spec.version) {
                    deps.add_edge(*node, *dep_node, (&spec.name, &dependency.version));
                }
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use petgraph::Direction;
    use semver::{BuildMetadata, Prerelease};

    use crate::build::{deps::generate_dependency_graph, spec::{BuildSpec, DependencyDeclaration}};

    #[test]
    fn test_generate_dependency_graph() {
        let builds = vec![
            BuildSpec {
                name: "base".to_string(),
                version: semver::Version { major: 1, minor: 2, patch: 3, pre: Prerelease::new("").unwrap(), build: BuildMetadata::EMPTY },
                dependencies: vec![]
            },

            BuildSpec {
                name: "derived".to_string(),
                version: semver::Version { major: 1, minor: 2, patch: 3, pre: Prerelease::new("").unwrap(), build: BuildMetadata::EMPTY },
                dependencies: vec![DependencyDeclaration {
                    name: "base".to_string(),
                    version: ">=1.0.0".parse().unwrap()
                }]
            },
            BuildSpec {
                name: "second_order".to_string(),
                version: semver::Version { major: 1, minor: 2, patch: 3, pre: Prerelease::new("").unwrap(), build: BuildMetadata::EMPTY },
                dependencies: vec![DependencyDeclaration {
                    name: "derived".to_string(),
                    version: ">=1.0.0".parse().unwrap()
                }]
            },
            BuildSpec {
                name: "converged".to_string(),
                version: semver::Version { major: 1, minor: 2, patch: 3, pre: Prerelease::new("").unwrap(), build: BuildMetadata::EMPTY },
                dependencies: vec![DependencyDeclaration {
                    name: "base".to_string(),
                    version: ">=1.0.0".parse().unwrap()
                },
                DependencyDeclaration {
                    name: "derived".to_string(),
                    version: ">=1.0.0".parse().unwrap()
                }]
            },
            BuildSpec {
                name: "independent".to_string(),
                version: semver::Version { major: 1, minor: 2, patch: 3, pre: Prerelease::new("").unwrap(), build: BuildMetadata::EMPTY },
                dependencies: vec![]
            },
        ];

        let mut graph = generate_dependency_graph(&builds);

        let mut starts = Vec::new();

        for node in graph.node_indices() {
            if graph.neighbors_directed(node, Direction::Outgoing).count() == 0 {
                // Terminal node
                starts.push(node);
            }
        }

        while let Some(node) = starts.pop() {
            // Find all BuildSpecs which depend on this
            let dependents = graph.neighbors_directed(node, Direction::Incoming);
            for dependent in dependents {
                // If the dependent only has one dependency (us), schedule it.
                let dependencies: Vec<_> = graph.neighbors_directed(dependent, Direction::Outgoing).collect();
                if dependencies.len() == 1 {
                    println!("Scheduling dependent image: {:?}", dependent);
                    starts.push(dependent);
                } else {
                    println!("Dependent image {:?} still has other dependencies: {:?}", dependent, dependencies);
                }
            }
            println!("Built {:#?}", node);
            graph.remove_node(node);
        }

        assert_eq!(graph.node_count(), 0);
    }
}