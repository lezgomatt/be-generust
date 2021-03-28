extern crate be_generust;

use be_generust::giver;

#[giver]
fn with_params(_a: i64, _b: i64, _c: i64) -> impl Iterator<Item = i64> {
    give!(1);
    give!(2);
    give!(3);
}

#[test]
fn test_simple() {
    assert_eq!(with_params(1, 2, 3).collect::<Vec<_>>(), vec![1, 2, 3]);
}
