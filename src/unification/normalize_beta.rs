use crate::term::*;
use crate::tree::State;
use crate::unification::unification_utils::*;

// Prüft, ob beide Seiten eine λ-Abstraktion mit demselben Binder x haben
// und mindestens eine Seite nicht in Head-Normal form ist.
pub fn is_normalizable_beta(lhs: &Term, rhs: &Term) -> bool {
    let (bs_l, core_l) = get_abs(lhs);
    let (bs_r, core_r) = get_abs(rhs);
    if bs_l != bs_r {
        return false;
    }
    !is_hnf(core_l) || !is_hnf(core_r)
}

// Wendet β-Normierung auf den Rumpf an: λx.s ? λx.t → λx.s↓h ? λx.t↓h
pub fn apply_normalize_beta(constraint: Constraint, state: &State) -> Vec<State> {
    let subst = &state.subst;

    println!("Normalite_beta");
    let Constraint(lhs, rhs) = constraint;
    //AppLhs
    if let Term::App { func, arg, .. } = lhs.clone() {
        if let Term::Abs { param, body } = *func {
            let reduced = substitute(&*body, &param, &*arg);
            let st =
                state.with_subst_and_count(vec![Constraint(reduced, rhs.clone())], subst.clone());
            return vec![st];
        }
    }
    //App Rhs
    if let Term::App { func, arg, .. } = rhs.clone() {
        if let Term::Abs { param, body } = *func {
            let reduced = substitute(&*body, &param, &*arg);
            let st =
                state.with_subst_and_count(vec![Constraint(lhs.clone(), reduced)], subst.clone());
            return vec![st];
        }
    }
    //Abs
    if let (Term::Abs { param, body: s }, Term::Abs { param: _, body: t }) =
        (lhs.clone(), rhs.clone())
    {
        let s_hnf = head_normalize(*s);
        let t_hnf = head_normalize(*t);
        let new_l = Term::Abs {
            param: param.clone(),
            body: Box::new(s_hnf),
        };
        let new_r = Term::Abs {
            param: param.clone(),
            body: Box::new(t_hnf),
        };
        let new_constr = Constraint(new_l, new_r);
        return vec![state.with_subst_and_count(vec![new_constr], subst.clone())];
    }
    vec![]
}

// Ist der Term bereits in Head-Normalform?
fn is_hnf(term: &Term) -> bool {
    match term {
        Term::Abs { .. } => true,
        Term::App { func, .. } => !matches!(&**func, Term::Abs { .. }),
        _ => true,
    }
}

// Reduziert links β Redexe bis zur HNF
fn head_normalize(mut t: Term) -> Term {
    loop {
        if let Term::App { func, arg, .. } = t.clone() {
            if let Term::Abs { param, body } = *func {
                // β-Redex: (λparam. body) a
                t = substitute(&*body, &param, &*arg);
                continue;
            }
        }
        break;
    }
    t
}

// Einfache Substitution param -> arg
fn substitute(term: &Term, param: &Variable, arg: &Term) -> Term {
    match term {
        // Ersetze gebundene Variable
        Term::Var(v) if v.term_kind == TermKind::BVar && v.name == param.name => arg.clone(),
        // Alle anderen Variablen bleiben
        Term::Var(v) => Term::Var(v.clone()),

        // In einer Abs: stoppe, wenn derselbe Binder wiederkommt
        Term::Abs { param: p, body } if p.name == param.name => Term::Abs {
            param: p.clone(),
            body: body.clone(),
        },
        // In einer Abs rekursiv tiefer
        Term::Abs { param: p, body } => Term::Abs {
            param: p.clone(),
            body: Box::new(substitute(body, param, arg)),
        },

        // In einer App: Funktion und Argument behandeln
        Term::App {
            func,
            arg: a,
            result_ty,
        } => Term::App {
            func: Box::new(substitute(func, param, arg)),
            arg: Box::new(substitute(a, param, arg)),
            result_ty: result_ty.clone(),
        },
    }
}
