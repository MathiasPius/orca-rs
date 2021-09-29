# dependency-graph
Dependency-graph is a minimal library, exposing a `DependencyGraph` structure and a single `Node` trait.

To use the library, simply implement the `Node` trait's two functions for the object you wish to.

# Example
In this example, we'll be using `dependency-graph` to resolve dependencies between our `Package` structs. We'll be using a custom `Dependency` type, because we want to include Semantic Versioning constraints in our dependencies. That way we'll be able to say that our package `mypackage` depends on version 2.0 of some package `some-library` for instance.

Our `Package` and `Dependency` structs use the `semver::Version` and `semver::VersionReq` types to define the versions of the `Packages` (such as 1.2.3-beta.4) and the dependency requirements such as `>=2.0`. See the [semver](https://crates.io/crates/semver) crate for more information.

First we define the `Package` struct:
```rust
struct Package {
    name: &'static str,
    version: Version,
    dependencies: Vec<Dependency>,
}
```
Where `Dependency` is:
```rust
struct Dependency {
    name: &'static str,
    version: VersionReq,
}

```

Implementing the `Node` trait for our `Package` is pretty simple:
```rust
impl Node for Package {
    type DependencyType = Dependency;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies[..]
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        // Check that name is an exact match, and that the dependency
        // requirements are fulfilled by our own version
        self.name == dependency.name && dependency.version.matches(&self.version)
    }
}
```

Let's define some packages and dependencies:
```rust
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
]
```

Now that we've defined all our packages as well as how dependencies between them are resolved (by implementing `Node`), we can build a `DependencyGraph` from it and traverse it, knowing that dependencies will always be visited before the packages that depend on them. 

In our case, we would expect the `base` package to be first (since `derived` depends on it), then `derived` second, since `second_order` depends on it in turn, and therefore cannot be resolved until `derived` has been. And then finally `second_order`, since all its dependencies have now been resolved.

```rust
let graph = DependencyGraph::from(&packages[..]);
for package in graph {
    match package {
        // Print out the package name so we can verify the order in the console
        Step::Resolved(package) => println!("Building {}!", package.name),

        // Since we know that all our Packages only have internal references to each other,
        // we can safely ignore any Unresolved steps in the graph.
        //
        // If for example `second_order` required some unknown package `external_package`,
        // iterating over our graph would yield that as a Step::Unresolved *before* 
        // the `second_order` package.
        Step::Unresolved(_) => unreachable!()
    }
}
```

If we run the above code, we can verify that they did indeed build in the right order:
```
Building base!
Building derived!
Building second_order!
```