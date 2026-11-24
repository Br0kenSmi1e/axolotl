use crate::term::TermRef;
use std::rc::Rc;

#[derive(Debug)]
pub enum Value {
    VLam(Closure),
    VNeutral(usize),
}

pub type ValueRef = Rc<Value>;

pub type Env = Vec<ValueRef>;

#[derive(Debug)]
pub struct Closure {
    pub env: Env,
    pub body: TermRef,
}
