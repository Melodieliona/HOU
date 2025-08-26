use crate::term::*;
use crate::tree::*;

//Prüft ob zwei Terme unterschiedliche rigid Köpfe hat.
//Const oder App mit Const oder gebundenen Variable
fn different_rigid_heads(t1: &Term, t2: &Term) -> bool {
    match (t1, t2) {
        (Term::Const(c1, _), Term::Const(c2, _)) => c1 != c2,
        (Term::App(f1, _, _), Term::App(f2, _, _)) => different_rigid_heads(f1, f2),
        (Term::Abs(_, _, a1), Term::Abs(_, _, a2)) => different_rigid_heads(a1, a2),
        (Term::BVar(b1, _), Term::BVar(b2, _)) => b1 != b2,
        (Term::Const(_, _), Term::BVar(_, _)) | (Term::BVar(_, _), Term::Const(_, _)) => true,
        _ => false,
    }
}

//Prüft Bedingungen
pub fn is_fail(lhs: &Term, rhs: &Term) -> bool {
    different_rigid_heads(lhs, rhs)
}

//Erzeugt den fehlgeschlagenen State
pub fn apply_fail(constraint: Constraint, _subst: &PersistentSubst) -> Vec<State> {
    println!("Fail angewandt auf {:?}", constraint);
    vec![State::fail()]
}
