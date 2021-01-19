extern crate generoust;

use generoust::giver;

#[giver]
fn single() -> impl Iterator<Item = i64> {
    give!(1);
}

#[test]
fn test_single() {
    assert_eq!(single().collect::<Vec<_>>(), vec![1]);
}
