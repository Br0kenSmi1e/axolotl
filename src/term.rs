use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Term {
    Var(usize),
    Lam(TermRef),
    App(TermRef, TermRef),
}

pub type TermRef = Rc<Term>;

pub fn var(index: usize) -> TermRef {
    Rc::new(Term::Var(index))
}

pub fn lam(body: TermRef) -> TermRef {
    Rc::new(Term::Lam(body))
}

pub fn app(func: TermRef, arg: TermRef) -> TermRef {
    Rc::new(Term::App(func, arg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_identity_twice() {
        let t1 = lam(var(0));
        let t2 = lam(var(0));
        assert_eq!(t1, t2);
    }

    #[test]
    fn build_const_twice() {
        let t1 = lam(lam(var(1)));
        let t2 = lam(lam(var(1)));
        assert_eq!(t1, t2);
    }

    #[test]
    fn diff_var_binders() {
        let t1 = lam(lam(var(0)));
        let t2 = lam(lam(var(1)));
        assert_ne!(t1, t2);
    }

    #[test]
    fn build_s_comb_twice() {
        let s1 = lam(lam(lam(app(app(var(2), var(0)), app(var(1), var(0))))));
        let s2 = lam(lam(lam(app(app(var(2), var(0)), app(var(1), var(0))))));
        assert_eq!(s1, s2);
    }
}
