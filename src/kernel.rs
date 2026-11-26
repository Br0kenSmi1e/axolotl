use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Term {
    Var(usize),
    Lambda(TermRef),
    Apply(TermRef, TermRef),
    Pi(TermRef, TermRef),
    Type(usize),
}

pub type TermRef = Rc<Term>;

pub fn var(index: usize) -> TermRef {
    Rc::new(Term::Var(index))
}

pub fn lambda(body: TermRef) -> TermRef {
    Rc::new(Term::Lambda(body))
}

pub fn apply(func: TermRef, arg: TermRef) -> TermRef {
    Rc::new(Term::Apply(func, arg))
}

pub fn pi(domain: TermRef, codomain: TermRef) -> TermRef {
    Rc::new(Term::Pi(domain, codomain))
}

pub fn universe(level: usize) -> TermRef {
    Rc::new(Term::Type(level))
}

#[derive(Debug)]
pub enum Value {
    VLambda(Closure),
    VPi(ValueRef, Closure),
    VType(usize),
    VNeutral(usize, Vec<ValueRef>),
}

pub type ValueRef = Rc<Value>;

pub type Env = Vec<ValueRef>;

#[derive(Debug)]
pub struct Closure {
    pub env: Env,
    pub body: TermRef,
}

pub fn vlambda(closure: Closure) -> ValueRef {
    Rc::new(Value::VLambda(closure))
}

pub fn vpi(domain: ValueRef, codomain: Closure) -> ValueRef {
    Rc::new(Value::VPi(domain, codomain))
}

pub fn vtype(level: usize) -> ValueRef {
    Rc::new(Value::VType(level))
}

pub fn vneutral(level: usize, spine: Vec<ValueRef>) -> ValueRef {
    Rc::new(Value::VNeutral(level, spine))
}

pub fn fresh(level: usize) -> ValueRef {
    vneutral(level, Vec::new())
}

pub fn capture(env: &Env, body: &TermRef) -> Closure {
    Closure {
        env: env.clone(),
        body: body.clone(),
    }
}

pub fn normalize(term: &TermRef) -> TermRef {
    readback(0, &eval(&Vec::new(), term))
}

pub fn eval(env: &Env, term: &TermRef) -> ValueRef {
    match term.as_ref() {
        Term::Var(i) => env[env.len() - 1 - *i].clone(),
        Term::Lambda(body) => vlambda(capture(env, body)),
        Term::Apply(func, arg) => vapply(&eval(env, func), eval(env, arg)),
        Term::Pi(domain, codomain) => vpi(eval(env, domain), capture(env, codomain)),
        Term::Type(level) => vtype(*level),
    }
}

pub fn readback(depth: usize, value: &ValueRef) -> TermRef {
    match value.as_ref() {
        Value::VLambda(closure) => lambda(readback(depth + 1, &capply(closure, fresh(depth)))),
        Value::VPi(domain, codomain) => pi(
            readback(depth, domain),
            readback(depth + 1, &capply(codomain, fresh(depth))),
        ),
        Value::VType(level) => universe(*level),
        Value::VNeutral(level, spine) => spine.iter().fold(var(depth - 1 - *level), |term, arg| {
            apply(term, readback(depth, arg))
        }),
    }
}

fn vapply(func: &ValueRef, arg: ValueRef) -> ValueRef {
    match func.as_ref() {
        Value::VLambda(closure) => capply(closure, arg),
        Value::VNeutral(level, spine) => vneutral(*level, extend(spine, arg)),
        other => panic!("cannot call a non-function value: {other:?}"),
    }
}

fn capply(closure: &Closure, arg: ValueRef) -> ValueRef {
    eval(&extend(&closure.env, arg), &closure.body)
}

pub type Ctx = Vec<ValueRef>;

#[derive(Debug, PartialEq)]
pub enum TypeError {
    UnboundVar,
    NotAFunction,
    ExpectedUniverse,
    ExpectedFunction,
    CannotInferLambda,
    Mismatch(TermRef, TermRef),
}

pub fn infer(ctx: &Ctx, env: &Env, term: &TermRef) -> Result<ValueRef, TypeError> {
    match term.as_ref() {
        Term::Var(i) => ctx
            .get(ctx.len() - 1 - *i)
            .cloned()
            .ok_or(TypeError::UnboundVar),
        Term::Lambda(_) => Err(TypeError::CannotInferLambda),
        Term::Apply(func, arg) => infer(ctx, env, func).and_then(|ty| match ty.as_ref() {
            Value::VPi(domain, codomain) => {
                check(ctx, env, arg, domain).map(|_| capply(codomain, eval(env, arg)))
            }
            _ => Err(TypeError::NotAFunction),
        }),
        Term::Pi(domain, codomain) => {
            let domain_uni = infer_universe(ctx, env, domain)?;
            let codomain_uni = infer_universe(
                &extend(&ctx, eval(env, domain)),
                &extend(&env, fresh(env.len())),
                codomain,
            )?;
            Ok(vtype(domain_uni.max(codomain_uni)))
        }
        Term::Type(level) => Ok(vtype(level + 1)),
    }
}

pub fn check(ctx: &Ctx, env: &Env, term: &TermRef, ty: &ValueRef) -> Result<(), TypeError> {
    match (term.as_ref(), ty.as_ref()) {
        (Term::Lambda(body), Value::VPi(domain, codomain)) => {
            let fresh_var = fresh(env.len());
            let body_ty = capply(codomain, fresh_var.clone());
            check(
                &extend(&ctx, domain.clone()),
                &extend(&env, fresh_var),
                body,
                &body_ty,
            )
        }
        (Term::Lambda(_), _) => Err(TypeError::ExpectedFunction),
        _ => infer(ctx, env, term).and_then(|inferred| {
            conv(env.len(), &inferred, ty).then_some(()).ok_or_else(|| {
                TypeError::Mismatch(readback(env.len(), &inferred), readback(env.len(), ty))
            })
        }),
    }
}

pub fn conv(depth: usize, lhs: &ValueRef, rhs: &ValueRef) -> bool {
    readback(depth, lhs) == readback(depth, rhs)
}

pub fn certify(statement: &TermRef, proof: &TermRef) -> Result<(), TypeError> {
    let ctx = Vec::new();
    let env = Vec::new();
    infer_universe(&ctx, &env, statement)
        .and_then(|_| check(&ctx, &env, proof, &eval(&env, statement)))
}

fn infer_universe(ctx: &Ctx, env: &Env, term: &TermRef) -> Result<usize, TypeError> {
    match infer(ctx, env, term)?.as_ref() {
        Value::VType(level) => Ok(*level),
        _ => Err(TypeError::ExpectedUniverse),
    }
}

fn extend(vec: &Vec<ValueRef>, item: ValueRef) -> Vec<ValueRef> {
    vec.iter().cloned().chain(std::iter::once(item)).collect()
}
