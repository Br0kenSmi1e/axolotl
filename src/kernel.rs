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

pub fn eval(env: &Env, term: &TermRef) -> ValueRef {
    match term.as_ref() {
        Term::Var(i) => {
            let idx = env.len() - 1 - *i;
            env[idx].clone()
        }
        Term::Lambda(body) => Rc::new(Value::VLambda(Closure {
            env: env.clone(),
            body: body.clone(),
        })),
        Term::Apply(func, arg) => call(&eval(env, func), eval(env, arg)),
        Term::Pi(domain, codomain) => Rc::new(Value::VPi(
            eval(env, domain),
            Closure {
                env: env.clone(),
                body: codomain.clone(),
            },
        )),
        Term::Type(level) => Rc::new(Value::VType(*level)),
    }
}

pub fn quote(depth: usize, value: &ValueRef) -> TermRef {
    match value.as_ref() {
        Value::VLambda(_) => {
            let fresh_var = Rc::new(Value::VNeutral(depth, Vec::new()));
            let body_val = call(value, fresh_var);
            lambda(quote(depth + 1, &body_val))
        }
        Value::VPi(domain, closure) => {
            let fresh_var = Rc::new(Value::VNeutral(depth, Vec::new()));
            let codomain_val = call_closure(&closure, fresh_var.clone());
            pi(quote(depth, domain), quote(depth + 1, &codomain_val))
        }
        Value::VType(level) => universe(*level),
        Value::VNeutral(level, args) => {
            let mut term = var(depth - 1 - *level);
            for arg in args {
                term = apply(term, quote(depth, arg));
            }
            term
        }
    }
}

pub fn normalize(term: &TermRef) -> TermRef {
    quote(0, &eval(&Vec::new(), term))
}

fn call(func: &ValueRef, arg: ValueRef) -> ValueRef {
    match func.as_ref() {
        Value::VLambda(closure) => call_closure(&closure, arg),
        Value::VNeutral(level, args) => {
            let mut args = args.clone();
            args.push(arg);
            Rc::new(Value::VNeutral(*level, args))
        }
        other => panic!("cannot call a non-function value: {other:?}"),
    }
}

fn call_closure(closure: &Closure, arg: ValueRef) -> ValueRef {
    let mut env = closure.env.clone();
    env.push(arg);
    eval(&env, &closure.body)
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

fn fresh(level: usize) -> ValueRef {
    Rc::new(Value::VNeutral(level, Vec::new()))
}

pub fn infer(ctx: &Ctx, env: &Env, term: &TermRef) -> Result<ValueRef, TypeError> {
    match term.as_ref() {
        Term::Var(i) => {
            let idx = ctx.len() - 1 - *i;
            ctx.get(idx).cloned().ok_or(TypeError::UnboundVar)
        }
        Term::Lambda(_) => Err(TypeError::CannotInferLambda),
        Term::Apply(func, arg) => {
            let func_ty = infer(ctx, env, func)?;
            match func_ty.as_ref() {
                Value::VPi(domain, closure) => {
                    check(ctx, env, arg, domain)?;
                    let arg_val = eval(env, arg);
                    Ok(call_closure(closure, arg_val))
                }
                _ => Err(TypeError::NotAFunction),
            }
        }
        Term::Pi(domain, codomain) => {
            let domain_uni = infer_universe(ctx, env, domain)?;
            let domain_val = eval(env, domain);
            let mut ctx = ctx.clone();
            ctx.push(domain_val);
            let mut env = env.clone();
            env.push(fresh(env.len()));
            let codomain_uni = infer_universe(&ctx, &env, codomain)?;
            Ok(Rc::new(Value::VType(domain_uni.max(codomain_uni))))
        }
        Term::Type(level) => Ok(Rc::new(Value::VType(level + 1))),
    }
}

pub fn check(ctx: &Ctx, env: &Env, term: &TermRef, ty: &ValueRef) -> Result<(), TypeError> {
    match (term.as_ref(), ty.as_ref()) {
        (Term::Lambda(body), Value::VPi(domain, closure)) => {
            let fresh_var = fresh(env.len());
            let body_ty = call_closure(closure, fresh_var.clone());
            let mut ctx = ctx.clone();
            ctx.push(domain.clone());
            let mut env = env.clone();
            env.push(fresh_var);
            check(&ctx, &env, body, &body_ty)
        }
        (Term::Lambda(_), _) => Err(TypeError::ExpectedFunction),
        _ => {
            let inferred = infer(ctx, env, term)?;
            if conv(env.len(), &inferred, ty) {
                Ok(())
            } else {
                Err(TypeError::Mismatch(
                    quote(env.len(), &inferred),
                    quote(env.len(), ty),
                ))
            }
        }
    }
}

pub fn conv(depth: usize, a: &ValueRef, b: &ValueRef) -> bool {
    quote(depth, a) == quote(depth, b)
}

pub fn certify(statement: &TermRef, proof: &TermRef) -> Result<(), TypeError> {
    let ctx = Vec::new();
    let env = Vec::new();
    infer_universe(&ctx, &env, statement)?;
    let statement_val = eval(&env, statement);
    check(&ctx, &env, proof, &statement_val)
}

fn infer_universe(ctx: &Ctx, env: &Env, term: &TermRef) -> Result<usize, TypeError> {
    match infer(ctx, env, term)?.as_ref() {
        Value::VType(level) => Ok(*level),
        _ => Err(TypeError::ExpectedUniverse),
    }
}
