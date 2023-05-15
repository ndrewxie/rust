use super::*;

// Quick type conversion function to make tests more compact - nothing important to see here
fn constraint_vars(input: Vec<Vec<usize>>) -> IndexVec<ConstraintIndex, Vec<VarIndex>> {
    input
        .into_iter()
        .map(|constraint| constraint.into_iter().map(|x| VarIndex::new(x)).collect())
        .collect()
}
// Quick type conversion function to make tests more compact - nothing important to see here
fn constraint_cliques(input: Vec<Vec<usize>>) -> IndexVec<CliqueIndex, Vec<ConstraintIndex>> {
    input
        .into_iter()
        .map(|clique| clique.into_iter().map(|x| ConstraintIndex::new(x)).collect())
        .collect()
}
// Quick type conversion function to make tests more compact - nothing important to see here
fn constraint_set<const N: usize>(input: [usize; N]) -> FxIndexSet<ConstraintIndex> {
    FxIndexSet::from_iter(input.into_iter().map(|x| ConstraintIndex::new(x)))
}

#[test]
fn test_gh_issues_example() {
    // Example of constraint that produces this lowering:
    // Bar: 'a.1
    // Bar: 'a.2
    // Bar: 'a.3
    // dedup_solver would map 'a.1 to var 1, 'a.2 to var 2, and 'a.3 to var 3
    let deduped = DedupSolver::dedup(
        constraint_vars(vec![vec![1], vec![2], vec![3]]),
        constraint_cliques(vec![vec![0, 1, 2]]),
        FxIndexSet::default(),
    );
    assert!(
        [constraint_set([0, 1]), constraint_set([0, 2]), constraint_set([1, 2])]
            .contains(&deduped.removed_constraints)
    );
}
#[test]
fn test_noop() {
    // Example of constraints that produces this lowering:
    // Foo: '?0
    // (&'?0 Bar, &'?0 Bar): &'A
    // &'?1 Bar: &'B
    // &'?2 Bar: &'C
    // This is assuming that the constraint_walker indexes '?0 as var 1, '?1 as var 2, etc
    let deduped = DedupSolver::dedup(
        constraint_vars(vec![vec![1], vec![1, 1], vec![2], vec![3]]),
        constraint_cliques(vec![vec![0], vec![1], vec![2], vec![3]]),
        FxIndexSet::default(),
    );
    assert!(deduped.removed_constraints.is_empty());
}
#[test]
fn test_simple() {
    // Example of constraint that produces this lowering:
    // &'?1 Foo: &'A
    // &'?2 Foo: &'A
    // &'?3 Foo: &'B
    // Note how 1 and 2 are grouped into 1 "clique" by the dedup_solver, while 3 is on its own because A =/= B
    let deduped = DedupSolver::dedup(
        constraint_vars(vec![vec![1], vec![2], vec![3]]),
        constraint_cliques(vec![vec![0, 1], vec![2]]),
        FxIndexSet::default(),
    );
    assert!([constraint_set([0]), constraint_set([1])].contains(&deduped.removed_constraints));
}
#[test]
fn test_dependencies() {
    // Example of constraint that produces this lowering:
    // (&'?1 Foo, &'?2 Foo): &'?13
    // (&'?4 Foo, &'?5 Foo): &'?16
    // (&'?1 Foo, &'?2 Foo, &'A Foo): &'?23
    // (&'?4 Foo, &'?5 Foo, &'A Foo): &'?26
    // &'?1: &'?2
    // &'?4: &'?5
    let deduped = DedupSolver::dedup(
        constraint_vars(vec![
            vec![1, 2, 13],
            vec![4, 5, 16],
            vec![1, 2, 23],
            vec![4, 5, 26],
            vec![1, 2],
            vec![4, 5],
        ]),
        constraint_cliques(vec![vec![0, 1], vec![2, 3], vec![4, 5]]),
        FxIndexSet::default(),
    );
    assert!(
        [constraint_set([0, 2, 4]), constraint_set([1, 3, 5])]
            .contains(&deduped.removed_constraints)
    );
}
#[test]
fn test_unremovable_var() {
    fn try_dedup(unremovable_vars: FxIndexSet<VarIndex>) -> FxIndexSet<ConstraintIndex> {
        // Same constraints as `test_dependencies`, but just imagine that all the vars in
        // unremovable_vars are vars the caller can name, and therefore can't be removed
        DedupSolver::dedup(
            constraint_vars(vec![
                vec![1, 2, 13],
                vec![4, 5, 16],
                vec![1, 2, 23],
                vec![4, 5, 26],
                vec![1, 2],
                vec![4, 5],
            ]),
            constraint_cliques(vec![vec![0, 1], vec![2, 3], vec![4, 5]]),
            unremovable_vars,
        )
        .removed_constraints
    }
    assert_eq!(try_dedup(FxIndexSet::from_iter([VarIndex::new(13)])), constraint_set([1, 3, 5]));
    assert_eq!(try_dedup(FxIndexSet::from_iter([VarIndex::new(16)])), constraint_set([0, 2, 4]));
}
#[test]
fn test_dependencies_unresolvable() {
    let deduped = DedupSolver::dedup(
        constraint_vars(vec![
            vec![1, 2, 13],
            vec![4, 5, 16],
            vec![1, 2, 23],
            vec![4, 6, 26],
            vec![1, 2],
            vec![4, 5],
        ]),
        constraint_cliques(vec![vec![0, 1], vec![2, 3], vec![4, 5]]),
        FxIndexSet::default(),
    );
    assert!(deduped.removed_constraints.is_empty());
}
