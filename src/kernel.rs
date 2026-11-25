use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Term {
    Var(usize),
    Lambda(TermRef),
    Apply(TermRef, TermRef),
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

#[derive(Debug)]
pub enum Value {
    VLambda(Closure),
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
    }
}

pub fn quote(depth: usize, value: &ValueRef) -> TermRef {
    match value.as_ref() {
        Value::VLambda(_) => {
            let probe = Rc::new(Value::VNeutral(depth, Vec::new()));
            let body_val = call(value, probe);
            lambda(quote(depth + 1, &body_val))
        }
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
        Value::VLambda(closure) => {
            let mut env = closure.env.clone();
            env.push(arg);
            eval(&env, &closure.body)
        }
        Value::VNeutral(level, args) => {
            let mut args = args.clone();
            args.push(arg);
            Rc::new(Value::VNeutral(*level, args))
        }
    }
}
