use crate::term::{Constraint, Term};
use crate::tree::State;
use crate::unification::unification_utils::*;

pub fn is_decomposable(lhs: &Term, rhs: &Term) -> bool {
    println!("Decompose?");
    let (b1, h1, a1) = flatten_hnf(lhs);
    let (b2, h2, a2) = flatten_hnf(rhs);

    b1 == b2 && h1 == h2 && a1.len() == a2.len()
}

// Erzeugt Constraints
pub fn apply_decompose(constraint: Constraint, state: &State) -> Vec<State> {
    println!("Decompose: {}", constraint);
    let subst = &state.subst.clone();
    let Constraint(lhs, rhs) = constraint;
    let (_b, _h, args_l) = flatten_hnf(&lhs);
    let (_b, _h, args_r) = flatten_hnf(&rhs);

    let new_constraints = args_l
        .into_iter()
        .zip(args_r.into_iter())
        .map(|(l, r)| Constraint(l.clone(), r.clone()))
        .collect();

    vec![state.with_subst_and_count(new_constraints, subst.clone())]
}
