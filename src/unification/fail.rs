use crate::term::*;
use crate::tree::*;
use crate::unification::unification_utils::*;

//Prüft ob zwei Terme unterschiedliche rigide Köpfe hat
pub fn is_fail(lhs: &Term, rhs: &Term) -> bool {
    print!("Fail?");
    let (_, head1, _) = flatten_hnf(lhs);
    let (_, head2, _) = flatten_hnf(rhs);
    is_rigid(&head1) && is_rigid(&head2) && head1 != head2
}

//Erzeugt den fehlgeschlagenen State
pub fn apply_fail() -> Vec<State> {
    vec![State::fail()]
}
