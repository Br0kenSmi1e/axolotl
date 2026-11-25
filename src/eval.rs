use crate::term::{Term, TermRef, app, lam, var};
use crate::value::{Closure, Env, Value, ValueRef};
use std::rc::Rc;

pub fn eval(env: &Env, term: &TermRef) -> ValueRef {
    match term.as_ref() {
        Term::Var(i) => {
            let idx = env.len() - 1 - *i;
            env[idx].clone()
        }
        Term::Lam(body) => Rc::new(Value::VLam(Closure {
            env: env.clone(),
            body: body.clone(),
        })),
        Term::App(func, arg) => apply(&eval(env, func), eval(env, arg)),
    }
}

pub fn quote(depth: usize, value: &ValueRef) -> TermRef {
    match value.as_ref() {
        Value::VLam(_) => {
            let probe = Rc::new(Value::VNeutral(depth, Vec::new()));
            let body_val = apply(value, probe);
            lam(quote(depth + 1, &body_val))
        }
        Value::VNeutral(level, args) => {
            let mut term = var(depth - 1 - *level);
            for arg in args {
                term = app(term, quote(depth, arg));
            }
            term
        }
    }
}

pub fn normalize(term: &TermRef) -> TermRef {
    quote(0, &eval(&Vec::new(), term))
}

fn apply(func: &ValueRef, arg: ValueRef) -> ValueRef {
    match func.as_ref() {
        Value::VLam(closure) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{app, lam, var};

    #[test]
    fn identity_to_identity() {
        let id = lam(var(0));
        let v = eval(&Vec::new(), &app(id.clone(), id));
        match v.as_ref() {
            Value::VLam(c) => assert_eq!(c.body, var(0)),
            _ => panic!("expected a lambda value"),
        }
    }

    #[test]
    fn const_returns_first_arg() {
        let id = lam(var(0));
        let cons = lam(lam(var(1)));
        let expr = app(app(cons, id.clone()), id);
        match eval(&Vec::new(), &expr).as_ref() {
            Value::VLam(c) => {
                assert_eq!(c.body, var(0));
                assert!(c.env.is_empty());
            }
            _ => panic!("expected a lambda value"),
        }
    }

    #[test]
    fn skk_is_identity() {
        let s = lam(lam(lam(app(app(var(2), var(0)), app(var(1), var(0))))));
        let k = lam(lam(var(1)));
        let skk = app(app(s, k.clone()), k);
        assert_eq!(normalize(&skk), lam(var(0)));
    }

    #[test]
    fn cons_id_app() {
        let cons = lam(lam(var(1)));
        let id = lam(var(0));
        assert_eq!(normalize(&app(cons, id)), lam(lam(var(0))));
    }
}
