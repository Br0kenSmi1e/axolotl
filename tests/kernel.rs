use axolotl::kernel::*;

// ==== the Leibniz equality ====

fn eq() -> TermRef {
    lambda(lambda(lambda(pi(
        pi(var(2), universe(0)),
        pi(apply(var(0), var(2)), apply(var(1), var(2))),
    ))))
}

fn eq0_ty() -> TermRef {
    pi(universe(0), pi(var(0), pi(var(1), universe(1))))
}

fn eq0_of(ty: TermRef, lhs: TermRef, rhs: TermRef) -> TermRef {
    apply(apply(apply(ann(eq(), eq0_ty()), ty), lhs), rhs)
}

#[test]
fn equality_is_transitive() {
    let statement = pi(
        universe(0),
        pi(
            var(0),
            pi(
                var(1),
                pi(
                    var(2),
                    pi(
                        eq0_of(var(3), var(2), var(1)),
                        pi(
                            eq0_of(var(4), var(2), var(1)),
                            eq0_of(var(5), var(4), var(2)),
                        ),
                    ),
                ),
            ),
        ),
    );
    let proof = lambda(lambda(lambda(lambda(lambda(lambda(lambda(lambda(
        apply(apply(var(2), var(1)), apply(apply(var(3), var(1)), var(0))),
    ))))))));
    assert_eq!(certify(&statement, &proof), Ok(()));
}

// ==== Church Numbers ====

fn zero() -> TermRef {
    lambda(lambda(lambda(var(0))))
}

fn succ(n: TermRef) -> TermRef {
    lambda(lambda(lambda(apply(
        var(1),
        apply(apply(apply(ann(n, nat()), var(2)), var(1)), var(0)),
    ))))
}

fn church(n: u64) -> TermRef {
    match n {
        0 => zero(),
        _ => succ(church(n - 1)),
    }
}

fn plus() -> TermRef {
    lambda(lambda(lambda(lambda(lambda(apply(
        apply(apply(var(4), var(2)), var(1)),
        apply(apply(apply(var(3), var(2)), var(1)), var(0)),
    ))))))
}

fn plus_ty() -> TermRef {
    pi(nat(), pi(nat(), nat()))
}

fn nat() -> TermRef {
    pi(universe(0), pi(pi(var(0), var(1)), pi(var(1), var(2))))
}

fn eq1_ty() -> TermRef {
    pi(universe(1), pi(var(0), pi(var(1), universe(1))))
}

fn eq1_of(a: TermRef, x: TermRef, y: TermRef) -> TermRef {
    apply(apply(apply(ann(eq(), eq1_ty()), a), x), y)
}

#[test]
fn two_plus_two_is_four() {
    assert_eq!(
        normalize(&apply(apply(plus(), church(2)), church(2))),
        normalize(&church(4))
    );
}

#[test]
fn certifies_two_plus_two_equals_four() {
    let stmt = eq1_of(
        nat(),
        apply(apply(ann(plus(), plus_ty()), church(2)), church(2)),
        church(4),
    );
    let refl_ty = pi(universe(1), pi(var(0), eq1_of(var(1), var(0), var(0))));
    let pf = apply(
        apply(ann(lambda(lambda(lambda(lambda(var(0))))), refl_ty), nat()),
        church(4),
    );
    assert_eq!(certify(&stmt, &pf), Ok(()));
}
