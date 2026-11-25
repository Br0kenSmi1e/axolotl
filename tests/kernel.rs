use axolotl::kernel::*;

#[test]
fn skk_is_identity() {
    let s = lam(lam(lam(app(app(var(2), var(0)), app(var(1), var(0))))));
    let k = lam(lam(var(1)));
    assert_eq!(normalize(&app(app(s, k.clone()), k)), lam(var(0)));
}

// ==== Church Numbers ====

fn zero() -> TermRef {
    lam(lam(var(0)))
}

fn succ(n: TermRef) -> TermRef {
    lam(lam(app(var(1), app(app(n, var(1)), var(0)))))
}

fn church(n: u64) -> TermRef {
    match n {
        0 => zero(),
        _ => succ(church(n - 1)),
    }
}

fn plus() -> TermRef {
    lam(lam(lam(lam(app(
        app(var(3), var(1)),
        app(app(var(2), var(1)), var(0)),
    )))))
}

#[test]
fn two_plus_two_is_four() {
    assert_eq!(
        normalize(&app(app(plus(), church(2)), church(2))),
        normalize(&church(4))
    );
}
