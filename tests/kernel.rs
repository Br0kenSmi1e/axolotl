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

// ==== Type System ====

#[test]
fn pi_roundtrip() {
    let ty = pi(universe(0), pi(var(0), var(1)));
    assert_eq!(normalize(&ty), ty);
}

#[test]
fn type_level_beta_reduction() {
    let id_ty = pi(universe(0), pi(var(0), var(1)));
    let redex = apply(lambda(var(0)), id_ty.clone());
    assert_eq!(normalize(&redex), id_ty);
}

#[test]
fn dependent_pi_roundtrips() {
    let ty = pi(
        universe(0),
        pi(
            pi(var(0), universe(0)),
            pi(var(1), pi(apply(var(1), var(0)), apply(var(2), var(1)))),
        ),
    );
    assert_eq!(normalize(&ty), ty);
}

// ==== Type Checker ====

#[test]
fn identity_certifies() {
    let id_ty = pi(universe(0), pi(var(0), var(1)));
    let id = lambda(lambda(var(0)));
    assert!(certify(&id_ty, &id).is_ok());
}

// ==== the Leibniz equality ====

fn eq() -> TermRef {
    lambda(lambda(lambda(pi(
        pi(var(2), universe(0)),
        pi(apply(var(0), var(2)), apply(var(1), var(2))),
    ))))
}

fn eq_ty() -> TermRef {
    pi(universe(0), pi(var(0), pi(var(1), universe(1))))
}

fn eq_of(ty: TermRef, lhs: TermRef, rhs: TermRef) -> TermRef {
    apply(apply(apply(ann(eq(), eq_ty()), ty), lhs), rhs)
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
                        eq_of(var(3), var(2), var(1)),
                        pi(eq_of(var(4), var(2), var(1)), eq_of(var(5), var(4), var(2))),
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
