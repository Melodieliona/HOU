use crate::term::{Constraint, Term};
use crate::tree::{PersistentSubst, State};

// Prüft, ob das Constraint trivial ist (s ?= s).
pub fn is_deletable(lhs: &Term, rhs: &Term) -> bool {
    println!("Deletable?");
    lhs == rhs
}

// Entfernt eine triviale Gleichung und liefert eine State mit leerem Constraints-Vektor.
pub fn apply_delete(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    // Wir löschen diesen Constraint, der neue State enthält nur noch den alten Subst-Stand.
    vec![State::with_subst(Vec::new(), subst.clone())]
}
