use crate::term::Term;
use crate::tree::State;

// Prüft, ob das Constraint trivial ist (s ?= s).
pub fn is_deletable(lhs: &Term, rhs: &Term) -> bool {
    println!("Delete?");
    lhs == rhs
}

// Entfernt den Constraint und liefert eine State mit den Substitutionen
pub fn apply_delete(state: &State) -> Vec<State> {
    vec![state.with_subst_and_count(Vec::new(), state.subst.clone())]
}
