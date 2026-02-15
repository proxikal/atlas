//! Build order computation tests for phase-08c

use atlas_package::{BuildOrderComputer, Resolution, ResolvedPackage};
use semver::Version;
use std::collections::HashMap;

// Test helper: Create resolution from dependencies
fn make_resolution(packages: Vec<(&str, Vec<&str>)>) -> Resolution {
    let mut resolution = Resolution::new();

    for (name, deps) in packages {
        let package = ResolvedPackage::with_dependencies(
            name.to_string(),
            Version::new(1, 0, 0),
            deps.iter().map(|s| s.to_string()).collect(),
        );
        resolution.add_package(package);
    }

    resolution
}

#[test]
fn test_topological_sort_linear() {
    // pkg1 -> pkg2 -> pkg3 (sequential dependencies)
    let resolution = make_resolution(vec![
        ("pkg1", vec!["pkg2"]),
        ("pkg2", vec!["pkg3"]),
        ("pkg3", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let order = computer.compute_build_order().unwrap();
    assert_eq!(order.len(), 3);

    // pkg3 should come before pkg2, pkg2 before pkg1
    let pkg3_idx = order.iter().position(|p| p == "pkg3").unwrap();
    let pkg2_idx = order.iter().position(|p| p == "pkg2").unwrap();
    let pkg1_idx = order.iter().position(|p| p == "pkg1").unwrap();

    assert!(pkg3_idx < pkg2_idx);
    assert!(pkg2_idx < pkg1_idx);
}

#[test]
fn test_topological_sort_diamond() {
    // root -> left -> bottom
    //      -> right -> bottom
    let resolution = make_resolution(vec![
        ("root", vec!["left", "right"]),
        ("left", vec!["bottom"]),
        ("right", vec!["bottom"]),
        ("bottom", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let order = computer.compute_build_order().unwrap();
    assert_eq!(order.len(), 4);

    // bottom should come before left and right, which should come before root
    let bottom_idx = order.iter().position(|p| p == "bottom").unwrap();
    let left_idx = order.iter().position(|p| p == "left").unwrap();
    let right_idx = order.iter().position(|p| p == "right").unwrap();
    let root_idx = order.iter().position(|p| p == "root").unwrap();

    assert!(bottom_idx < left_idx);
    assert!(bottom_idx < right_idx);
    assert!(left_idx < root_idx);
    assert!(right_idx < root_idx);
}

#[test]
fn test_parallel_build_groups() {
    // Independent packages should be in same group
    let resolution = make_resolution(vec![("pkg1", vec![]), ("pkg2", vec![]), ("pkg3", vec![])]);
    let computer = BuildOrderComputer::new(&resolution);

    let groups = computer.parallel_build_groups().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 3);
}

#[test]
fn test_parallel_build_groups_sequential() {
    // pkg1 -> pkg2 -> pkg3 (must be sequential)
    let resolution = make_resolution(vec![
        ("pkg1", vec!["pkg2"]),
        ("pkg2", vec!["pkg3"]),
        ("pkg3", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let groups = computer.parallel_build_groups().unwrap();
    // Each package must be in its own group
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0], vec!["pkg3"]);
    assert_eq!(groups[1], vec!["pkg2"]);
    assert_eq!(groups[2], vec!["pkg1"]);
}

#[test]
fn test_parallel_build_groups_diamond() {
    // root -> left -> bottom
    //      -> right -> bottom
    // bottom can be first, left+right parallel, then root
    let resolution = make_resolution(vec![
        ("root", vec!["left", "right"]),
        ("left", vec!["bottom"]),
        ("right", vec!["bottom"]),
        ("bottom", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let groups = computer.parallel_build_groups().unwrap();
    assert_eq!(groups.len(), 3);

    // First group: bottom only
    assert_eq!(groups[0], vec!["bottom"]);

    // Second group: left and right (parallel)
    let mut second_group = groups[1].clone();
    second_group.sort();
    assert_eq!(second_group, vec!["left", "right"]);

    // Third group: root only
    assert_eq!(groups[2], vec!["root"]);
}

#[test]
fn test_build_order_empty_graph() {
    let resolution = Resolution::new();
    let computer = BuildOrderComputer::new(&resolution);

    let order = computer.compute_build_order().unwrap();
    assert!(order.is_empty());
    assert!(computer.is_empty());
}

#[test]
fn test_build_order_single_package() {
    let resolution = make_resolution(vec![("pkg1", vec![])]);
    let computer = BuildOrderComputer::new(&resolution);

    let order = computer.compute_build_order().unwrap();
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], "pkg1");
    assert_eq!(computer.package_count(), 1);
}

#[test]
fn test_from_graph() {
    let mut graph = HashMap::new();
    graph.insert("pkg1".to_string(), vec!["pkg2".to_string()]);
    graph.insert("pkg2".to_string(), vec![]);

    let computer = BuildOrderComputer::from_graph(graph);
    assert_eq!(computer.package_count(), 2);

    let order = computer.compute_build_order().unwrap();
    assert_eq!(order.len(), 2);

    // pkg2 before pkg1
    let pkg2_idx = order.iter().position(|p| p == "pkg2").unwrap();
    let pkg1_idx = order.iter().position(|p| p == "pkg1").unwrap();
    assert!(pkg2_idx < pkg1_idx);
}

#[test]
fn test_get_dependencies() {
    let resolution = make_resolution(vec![
        ("pkg1", vec!["pkg2", "pkg3"]),
        ("pkg2", vec![]),
        ("pkg3", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let deps = computer.get_dependencies("pkg1").unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&"pkg2".to_string()));
    assert!(deps.contains(&"pkg3".to_string()));
}

#[test]
fn test_packages() {
    let resolution = make_resolution(vec![("pkg1", vec![]), ("pkg2", vec![]), ("pkg3", vec![])]);
    let computer = BuildOrderComputer::new(&resolution);

    let packages = computer.packages();
    assert_eq!(packages.len(), 3);
    assert_eq!(packages, vec!["pkg1", "pkg2", "pkg3"]);
}

#[test]
fn test_complex_dependency_graph() {
    // More complex graph to test robustness
    //     a
    //    / \
    //   b   c
    //   |\ /|
    //   | X |
    //   |/ \|
    //   d   e
    //    \ /
    //     f
    let resolution = make_resolution(vec![
        ("a", vec!["b", "c"]),
        ("b", vec!["d", "e"]),
        ("c", vec!["d", "e"]),
        ("d", vec!["f"]),
        ("e", vec!["f"]),
        ("f", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let order = computer.compute_build_order().unwrap();
    assert_eq!(order.len(), 6);

    // f must come first
    assert_eq!(order[0], "f");

    // d and e must come after f, before b and c
    let f_idx = 0;
    let d_idx = order.iter().position(|p| p == "d").unwrap();
    let e_idx = order.iter().position(|p| p == "e").unwrap();
    let b_idx = order.iter().position(|p| p == "b").unwrap();
    let c_idx = order.iter().position(|p| p == "c").unwrap();
    let a_idx = order.iter().position(|p| p == "a").unwrap();

    assert!(f_idx < d_idx);
    assert!(f_idx < e_idx);
    assert!(d_idx < b_idx);
    assert!(e_idx < b_idx);
    assert!(d_idx < c_idx);
    assert!(e_idx < c_idx);
    assert!(b_idx < a_idx);
    assert!(c_idx < a_idx);
}

#[test]
fn test_parallel_build_groups_complex() {
    // Same complex graph for parallel groups
    let resolution = make_resolution(vec![
        ("a", vec!["b", "c"]),
        ("b", vec!["d", "e"]),
        ("c", vec!["d", "e"]),
        ("d", vec!["f"]),
        ("e", vec!["f"]),
        ("f", vec![]),
    ]);
    let computer = BuildOrderComputer::new(&resolution);

    let groups = computer.parallel_build_groups().unwrap();

    // Group 1: f only
    assert_eq!(groups[0], vec!["f"]);

    // Group 2: d and e (parallel)
    let mut group2 = groups[1].clone();
    group2.sort();
    assert_eq!(group2, vec!["d", "e"]);

    // Group 3: b and c (parallel)
    let mut group3 = groups[2].clone();
    group3.sort();
    assert_eq!(group3, vec!["b", "c"]);

    // Group 4: a only
    assert_eq!(groups[3], vec!["a"]);
}
