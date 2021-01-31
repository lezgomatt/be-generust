extern crate generoust;

use generoust::giver;

#[giver]
fn simple() -> impl Iterator<Item = i64> {
    let w = 1;
    let x = w * w;
    give!(x);
    let mut y = 1;
    y += y;
    give!(y);
    let z: i64 = -3;
    give!(z.abs());
}

#[test]
fn test_simple() {
    assert_eq!(simple().collect::<Vec<_>>(), vec![1, 2, 3]);
}
