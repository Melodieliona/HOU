pub mod bind;
pub mod hs_projection;
pub mod identification;
pub mod imitation;
pub mod iteration;
pub mod jps_projection;

use std::collections::HashMap;

use crate::term::*;
use crate::tree::*;

pub fn is_bindable(lhs: &Term, rhs: &Term) -> bool {
    if let Term::FVar(name, _) = lhs {
        !occurs_in(name, rhs)
    } else {
        false
    }
}

//weil kein Oracle soll {G ? = f G} verhindern
fn occurs_in(var: &str, term: &Term) -> bool {
    match term {
        Term::FVar(n, _) => n == var,
        Term::BVar(_, _) => false,
        Term::Const(_, _) => false,
        Term::Abs(p, _, b) => p == var || occurs_in(var, &*b),
        Term::App(f, a, _) => occurs_in(var, f) || occurs_in(var, a),
    }
}

pub fn apply_bind(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint.clone();
    if let Term::FVar(name, _) = lhs {
        let new_subst = subst.clone().push(name.clone(), rhs.clone());
        let st = State::with_subst(vec![constraint], new_subst);
        vec![st]
    } else {
        Vec::new()
    }
}
