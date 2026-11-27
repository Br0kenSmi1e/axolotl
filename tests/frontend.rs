use axolotl::frontend::*;
use axolotl::kernel::*;

#[test]
fn build_surface_tree() {
    let tree = STerm::SApply(
        Box::new(STerm::SVar("succ".to_string())),
        Box::new(STerm::SVar("zero".to_string())),
    );
    println!("{tree:?}");
}

#[test]
fn resolve_identity() {
    let sterm = STerm::SLambda("x".to_string(), Box::new(STerm::SVar("x".to_string())));
    let mut scope = Vec::new();
    assert_eq!(resolve(&mut scope, &sterm).unwrap(), lambda(var(0)));
}

#[test]
fn drive_end_to_end() {
    let decls = vec![
        // -- Axiom arm: type + constant, joins as neutrals
        SDecl::Axiom {
            name: "Nat".into(),
            ty: STerm::SType(0),
        },
        SDecl::Axiom {
            name: "zero".into(),
            ty: STerm::SVar("Nat".into()),
        },
        SDecl::Axiom {
            name: "succ".into(),
            ty: STerm::SPi(
                "n".into(),
                Box::new(STerm::SVar("Nat".into())),
                Box::new(STerm::SVar("Nat".into())),
            ),
        },
        // -- Def arm: derived constant, joins as a real value
        SDecl::Def {
            name: "one".into(),
            ty: STerm::SVar("Nat".into()),
            body: STerm::SApply(
                Box::new(STerm::SVar("succ".into())), // references the succ axiom
                Box::new(STerm::SVar("zero".into())), // references the zero axiom
            ),
        },
        // -- Thm arm: theorem referencing the shared type, joins opaque
        SDecl::Thm {
            name: "id_nat".into(),
            ty: STerm::SPi(
                "x".into(),
                Box::new(STerm::SVar("Nat".into())),
                Box::new(STerm::SVar("Nat".into())),
            ),
            proof: STerm::SLambda("x".into(), Box::new(STerm::SVar("x".into()))),
        },
    ];

    let db = drive(&decls).unwrap(); // must succeed

    // 1. every declaration accumulated; lockstep holds
    assert_eq!(db.ctx.len(), 5);
    assert_eq!(db.env.len(), 5);
    assert_eq!(db.scope.len(), 5);

    // 2. names in declaration order
    assert_eq!(db.scope[0], "Nat");
    assert_eq!(db.scope[3], "one");
    assert_eq!(db.scope[4], "id_nat");

    // 3. opacity knob: axioms & theorem are bare probes; the def is computed
    assert!(matches!(db.env[0].as_ref(), Value::VNeutral(_, spine) if spine.is_empty()));
    assert!(matches!(db.env[2].as_ref(), Value::VNeutral(_, spine) if spine.is_empty()));
    assert!(matches!(db.env[3].as_ref(), Value::VNeutral(_, spine) if spine.len() == 1));
    assert!(matches!(db.env[4].as_ref(), Value::VNeutral(_, spine) if spine.is_empty()));
}

#[test]
fn inner_binder_shadows_outer() {
    // fn(x){ fn(y){ x } } → Lambda(Lambda(Var(1)))
    // x is 1 binder up from the body — would have caught the rposition bug
    let sterm = STerm::SLambda(
        "x".into(),
        Box::new(STerm::SLambda(
            "y".into(),
            Box::new(STerm::SVar("x".into())),
        )),
    );
    assert_eq!(
        resolve(&mut Vec::new(), &sterm).unwrap(),
        lambda(lambda(var(1)))
    );
}

#[test]
fn pi_domain_resolves_before_binder_enters() {
    // scope has outer "A"; fn(x: A) -> x
    // domain A is resolved BEFORE pushing x: A = var(0) in scope ["A"]
    // codomain x resolved under ["A","x"]: x = var(0)
    let mut scope = vec!["A".to_string()];
    let sterm = STerm::SPi(
        "x".into(),
        Box::new(STerm::SVar("A".into())),
        Box::new(STerm::SVar("x".into())),
    );
    assert_eq!(resolve(&mut scope, &sterm).unwrap(), pi(var(0), var(0)));
}
