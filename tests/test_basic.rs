extern crate generoust;

use generoust::giver;

#[giver]
fn one<R>() -> R
where
    R: Iterator<Item = i64>,
{
    give!(1);
}

#[test]
fn test_one() {
    assert_eq!(one().collect::<Vec<_>>(), vec![1]);
}
