use super::spec::{BuildSpec, Dependency};
use dependency_graph::Node;

impl Node for BuildSpec {
    type DependencyType = Dependency;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies[..]
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        dependency.name == self.name && dependency.version.matches(&self.version)
    }
}
