use crate::kernel::TermRef;
use crate::kernel::{Ctx, Env, TypeError};
use crate::kernel::{ann, apply, lambda, pi, universe, var};
use crate::kernel::{certify_with_ctx, eval, fresh, infer_universe};

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
            .map(|pos| var(scope.len() - 1 - pos))
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

#[derive(Debug)]
pub enum FrontendError {
    Resolve(ResolveError),
    Type(TypeError),
}

impl From<ResolveError> for FrontendError {
    fn from(e: ResolveError) -> Self {
        FrontendError::Resolve(e)
    }
}

impl From<TypeError> for FrontendError {
    fn from(e: TypeError) -> Self {
        FrontendError::Type(e)
    }
}

pub struct Globals {
    pub scope: Vec<String>,
    pub ctx: Ctx,
    pub env: Env,
}

pub fn drive(decls: &[SDecl]) -> Result<Globals, FrontendError> {
    let mut scope: Vec<String> = Vec::new();
    let mut ctx: Ctx = Vec::new();
    let mut env: Env = Vec::new();
    for decl in decls {
        match decl {
            SDecl::Axiom { name, ty } => {
                let ty = resolve(&mut scope, &ty)?;
                infer_universe(&ctx, &env, &ty)?;
                scope.push(name.to_string());
                ctx.push(eval(&env, &ty));
                env.push(fresh(env.len()));
            }
            SDecl::Def { name, ty, body } => {
                let ty = resolve(&mut scope, &ty)?;
                let body = resolve(&mut scope, &body)?;
                certify_with_ctx(&ctx, &env, &ty, &body)?;
                scope.push(name.to_string());
                ctx.push(eval(&env, &ty));
                env.push(eval(&env, &body));
            }
            SDecl::Thm { name, ty, proof } => {
                let ty = resolve(&mut scope, &ty)?;
                let proof = resolve(&mut scope, &proof)?;
                certify_with_ctx(&ctx, &env, &ty, &proof)?;
                scope.push(name.to_string());
                ctx.push(eval(&env, &ty));
                env.push(fresh(env.len()));
            }
        }
    }
    Ok(Globals { scope, ctx, env })
}
