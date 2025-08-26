use crate::term::{Constraint, Term};
use crate::tree::{PersistentSubst, State};

// Prüft, ob beide Seiten eine λ-Abstraktion mit demselben Binder x haben
// und mindestens eine Seite nicht in Head-Normal form ist.
pub fn is_normalizable_beta(lhs: &Term, rhs: &Term) -> bool {
    match (lhs, rhs) {
        (Term::Abs(x1, _ty1, s), Term::Abs(x2, _ty2, t)) if x1 == x2 => {
            !is_hnf(&*s) || !is_hnf(&*t)
        }
        _ => false,
    }
}

// Wendet β-Normierung auf den Rumpf an: λx.s ? λx.t → λx.s↓h ? λx.t↓h
pub fn apply_normalize_beta(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint;
    if let (Term::Abs(param, ty, s), Term::Abs(_, _ty2, t)) = (lhs.clone(), rhs.clone()) {
        let s_hnf = head_normalize(*s);
        let t_hnf = head_normalize(*t);
        let new_l = Term::Abs(param.clone(), ty.clone(), Box::new(s_hnf));
        let new_r = Term::Abs(param.clone(), ty, Box::new(t_hnf));
        let st = State::with_subst(vec![Constraint(new_l, new_r)], subst.clone());
        return vec![st];
    }
    vec![]
}

// Ist der Term bereits in Head-Normalform?
fn is_hnf(term: &Term) -> bool {
    match term {
        Term::Abs(_, _, _) => true,
        Term::App(f, _, _) => !matches!(&**f, Term::Abs(_, _, _)),
        _ => true,
    }
}

// Reduziert links-äußerste β-Redexe bis zur HNF
fn head_normalize(mut t: Term) -> Term {
    loop {
        if let Term::App(ref f, ref a, _) = t {
            if let Term::Abs(ref param, _, ref body) = **f {
                // β-Redex: (λparam. body) a
                t = substitute(&*body, &param, &*a);
                continue;
            }
        }
        break;
    }
    t
}

// Einfache Substitution param ↦ arg
fn substitute(term: &Term, param: &str, arg: &Term) -> Term {
    match term {
        Term::BVar(n, ty) if n == param => arg.clone(),
        Term::BVar(n, ty) => Term::BVar(n.clone(), ty.clone()),
        Term::FVar(n, ty) => Term::FVar(n.clone(), ty.clone()),
        Term::Const(n, ty) => Term::Const(n.clone(), ty.clone()),
        Term::App(f, a, ty) => Term::App(
            Box::new(substitute(f, param, arg)),
            Box::new(substitute(a, param, arg)),
            ty.clone(),
        ),
        Term::Abs(p, ty, b) if p == param => Term::Abs(p.clone(), ty.clone(), b.clone()),
        Term::Abs(p, ty, b) => {
            Term::Abs(p.clone(), ty.clone(), Box::new(substitute(b, param, arg)))
        }
    }
}
