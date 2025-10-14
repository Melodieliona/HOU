use crate::counter::Config;

use crate::term::*;
use crate::tree::State;
use crate::unification::bind::oracle;
use crate::unification::*;

// Wendet eine Unifikationsregeln auf einen einzelnen Constraint an
// und liefert eine Liste von Folge-States zurück.
pub fn apply_unify_rules(constraint: Constraint, state: &State, config: &Config) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint.clone();
    println!("Constraint: {}", constraint);
    println!("Subst: {}", state.subst);

    if let Some(fallback) = oracle::oracle(&constraint, state, config) {
        return vec![fallback];
    }

    //Normalize an: ({λxm.s ? = λyn.t}⊎E,σ) −→ ({λxm.s ? = λxm.t′xn+1...xm}⊎E,σ)
    if normalize_an::is_normalizable_an(&lhs, &rhs) {
        return normalize_an::apply_normalize_an(constraint.clone(), state);
    }
    //Normalize ß : ({λx.s ? = λx.t} ⊎E,σ) −→ ({λx.s↓h ? = λx.t↓h}⊎E,σ)
    if normalize_beta::is_normalizable_beta(&lhs, &rhs) {
        return normalize_beta::apply_normalize_beta(constraint.clone(), state);
    }
    //Dereference : ({λx.F s ? = λx.t}⊎E,σ) −→ ({λx.(σF)s ? = λx.t}⊎E,σ)
    if dereference::is_dereference(&lhs, &rhs, state) {
        return dereference::apply_dereference(constraint.clone(), state);
    }
    //Fail        :  ({λx.asm ? = λx.btn}⊎E,σ) −→ ⊥
    if fail::is_fail(&lhs, &rhs) {
        return fail::apply_fail();
    }
    //Delete      : ({s ? = s}⊎E,σ) −→ (E,σ)
    if delete::is_deletable(&lhs, &rhs) {
        return delete::apply_delete(&state);
    }
    if decompose::is_decomposable(&lhs, &rhs) {
        // 1. Decompose-States
        let mut next = decompose::apply_decompose(constraint.clone(), state);

        // 2. Bind-States
        let mut bind_states = bind::bind::apply_bind(constraint.clone(), state, config);
        next.append(&mut bind_states);

        return next;
    }

    bind::bind::apply_bind(constraint, state, config)
}
