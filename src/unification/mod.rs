pub mod Bind;
pub mod decompose;
pub mod delete;
pub mod dereference;
pub mod fail;
pub mod normalize_an;
pub mod normalize_beta;
pub mod succeed;

use crate::term::Constraint;
use crate::tree::{PersistentSubst, State};

// Wendet alle Unifikationsregeln auf eine einzelne Constraint an
// und liefert eine Liste von Folge-States zurück.
pub fn apply_unify_rules(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    //let mut results = Vec::new();
    let Constraint(lhs, rhs) = constraint.clone();
    let left = &lhs;
    let right = &rhs;

    //Succeed
    if let Some(state) = succeed::apply_succeed(constraint.clone(), subst) {
        return vec![state];
    }
    //Normalize an: ({λxm.s ? = λyn.t}⊎E,σ) −→ ({λxm.s ? = λxm.t′xn+1...xm}⊎E,σ)
    //Normalize ß : ({λx.s ? = λx.t} ⊎E,σ) −→ ({λx.s↓h ? = λx.t↓h}⊎E,σ)
    //Dereference : ({λx.F s ? = λx.t}⊎E,σ) −→ ({λx.(σF)s ? = λx.t}⊎E,σ)
    //Fail        :  ({λx.asm ? = λx.btn}⊎E,σ) −→ ⊥
    //Delete      : ({s ? = s}⊎E,σ) −→ (E,σ)
    if left == right {
        return delete::apply_delete(constraint, subst);
    }
    //Decompose   : ({λx.asm ? = λx.atm}⊎E,σ) −→ ({s1 ? = t1,...,sm ? = tm}⊎E,σ)
    if let Some((binder, s_args, t_args)) = decompose::can_decompose(left, right) {
        println!(" Decomposable:");
        println!("Binder: {}", binder);

        return decompose::apply_decompose(binder, s_args, t_args, subst);
    }

    //Bind        :({s ? = t}⊎E,σ) −→ ({s ? = t}⊎E,ϱσ)*/
    Vec::new()
}
