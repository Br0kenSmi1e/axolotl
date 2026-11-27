use crate::kernel::{TermRef, ann, apply, lambda, pi, universe, var};

#[derive(Debug)]
pub enum STerm {
    SVar(String),
    SLambda(String, Box<STerm>),
    SApply(Box<STerm>, Box<STerm>),
    SPi(String, Box<STerm>, Box<STerm>),
    SType(usize),
    SAnn(Box<STerm>, Box<STerm>),
}

#[derive(Debug)]
pub enum SDecl {
    Axiom {
        name: String,
        ty: STerm,
    },
    Def {
        name: String,
        ty: STerm,
        body: STerm,
    },
    Thm {
        name: String,
        ty: STerm,
        proof: STerm,
    },
}

pub struct ParserError {
    pub message: String,
    pub position: usize,
}

pub trait Parser {
    fn parse(&self, source: &str) -> Result<Vec<SDecl>, ParserError>;
}

#[derive(Debug)]
pub enum ResolveError {
    UnknownName(String),
}

pub fn resolve(scope: &mut Vec<String>, sterm: &STerm) -> Result<TermRef, ResolveError> {
    match sterm {
        STerm::SVar(name) => scope
            .iter()
            .rposition(|n| n == name)
            .map(var)
            .ok_or_else(|| ResolveError::UnknownName(name.clone())),
        STerm::SLambda(name, body) => {
            scope.push(name.clone());
            let body = resolve(scope, body)?;
            scope.pop();
            Ok(lambda(body))
        }
        STerm::SApply(func, arg) => Ok(apply(resolve(scope, func)?, resolve(scope, arg)?)),
        STerm::SPi(name, domain, codomain) => {
            let domain = resolve(scope, domain)?;
            scope.push(name.clone());
            let codomain = resolve(scope, codomain)?;
            scope.pop();
            Ok(pi(domain, codomain))
        }
        STerm::SType(level) => Ok(universe(*level)),
        STerm::SAnn(term, ty) => Ok(ann(resolve(scope, term)?, resolve(scope, ty)?)),
    }
}
