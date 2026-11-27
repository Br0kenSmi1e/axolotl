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
