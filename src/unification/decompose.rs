use crate::term::{Constraint, Term};
use crate::tree::{PersistentSubst, State};

//Funktion, die vector von binder namen, head, vector von argumenten zurück gibt
fn flatten_hnf(term: &Term) -> (Vec<&String>, &Term, Vec<&Term>) {
    // sammelt binder, gibt rest in cur aus
    let mut binders = Vec::new();
    let mut cur = term;
    while let Term::Abs(param, _ty, body) = cur {
        binders.push(param);
        cur = body;
    }

    // Kopf + Argumente
    let mut args = Vec::new();
    let mut head = cur;
    while let Term::App(fun, arg, _res_ty) = head {
        args.push(arg.as_ref());
        head = fun.as_ref();
    }
    args.reverse();

    (binders, head, args)
}

pub fn is_decomposable(lhs: &Term, rhs: &Term) -> bool {
    println!("Is decomposable? ");
    let (b1, h1, a1) = flatten_hnf(lhs);
    let (b2, h2, a2) = flatten_hnf(rhs);

    b1 == b2
        && matches!(h1, Term::Const(_, _) | Term::BVar(_, _))
        && matches!(h2, Term::Const(_, _) | Term::BVar(_, _))
        && h1 == h2
        && a1.len() == a2.len()
}

// Erzeugt Constraints
pub fn apply_decompose(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint;
    let (_b, _h, args_l) = flatten_hnf(&lhs);
    let (_b, _h, args_r) = flatten_hnf(&rhs);

    let new_constraints = args_l
        .into_iter()
        .zip(args_r.into_iter())
        .map(|(l, r)| Constraint(l.clone(), r.clone()))
        .collect();

    println!("Decomposed: {:?}", &new_constraints);

    vec![State::with_subst(new_constraints, subst.clone())]
}
