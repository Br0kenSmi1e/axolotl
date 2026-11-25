use axolotl::kernel::*;

#[test]
fn skk_is_identity() {
    let s = lambda(lambda(lambda(apply(
        apply(var(2), var(0)),
        apply(var(1), var(0)),
    ))));
    let k = lambda(lambda(var(1)));
    assert_eq!(normalize(&apply(apply(s, k.clone()), k)), lambda(var(0)));
}

// ==== Church Numbers ====

fn zero() -> TermRef {
    lambda(lambda(var(0)))
}

fn succ(n: TermRef) -> TermRef {
    lambda(lambda(apply(var(1), apply(apply(n, var(1)), var(0)))))
}

fn church(n: u64) -> TermRef {
    match n {
        0 => zero(),
        _ => succ(church(n - 1)),
    }
}

fn plus() -> TermRef {
    lambda(lambda(lambda(lambda(apply(
        apply(var(3), var(1)),
        apply(apply(var(2), var(1)), var(0)),
    )))))
}

#[test]
fn two_plus_two_is_four() {
    assert_eq!(
        normalize(&apply(apply(plus(), church(2)), church(2))),
        normalize(&church(4))
    );
}
