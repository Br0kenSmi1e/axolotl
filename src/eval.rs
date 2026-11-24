use crate::term::{Term, TermRef};
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
        Term::App(func, arg) => {
            let func_val = eval(env, func);
            let arg_val = eval(env, arg);
            match func_val.as_ref() {
                Value::VLam(closure) => {
                    let mut env = closure.env.clone();
                    env.push(arg_val);
                    eval(&env, &closure.body)
                }
                Value::VNeutral(_) => panic!("applying neutral: not yet supported"),
            }
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
}
