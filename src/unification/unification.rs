use crate::counter::Config;

use crate::term::*;
use crate::tree::State;
use crate::unification::bind::oracle;
use crate::unification::*;

// Wendet eine Unifikationsregeln auf einen einzelnen Constraint an
// und liefert eine Liste von Folge-States zurück.
pub fn apply_unify_rules(constraint: Constraint, state: &State, config: &Config) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint.clone();
    if let Some(fallback) = oracle::oracle(&constraint, state, config) {
        return vec![fallback];
    }
    println!("▶ apply_unify_rules: LHS = {}, RHS = {}", lhs, rhs);
    println!("\n Subst: {}", state.subst);
    println!("\n Counter: {:?}", state.binding_counts);

    //Normalize an: ({λxm.s ? = λyn.t}⊎E,σ) −→ ({λxm.s ? = λxm.t′xn+1...xm}⊎E,σ)
    if normalize_an::is_normalizable_an(&lhs, &rhs) {
        println!("Normalize an:");
        return normalize_an::apply_normalize_an(constraint.clone(), state);
    }
    //Normalize ß : ({λx.s ? = λx.t} ⊎E,σ) −→ ({λx.s↓h ? = λx.t↓h}⊎E,σ)
    if normalize_beta::is_normalizable_beta(&lhs, &rhs) {
        println!("Yes! Normalize ß!");
        return normalize_beta::apply_normalize_beta(constraint.clone(), state);
    }
    //Dereference : ({λx.F s ? = λx.t}⊎E,σ) −→ ({λx.(σF)s ? = λx.t}⊎E,σ)
    if dereference::is_dereference(&lhs, &rhs, state) {
        println!("Yes! Derefernce!");
        return dereference::apply_dereference(constraint.clone(), state);
    }
    //Fail        :  ({λx.asm ? = λx.btn}⊎E,σ) −→ ⊥
    if fail::is_fail(&lhs, &rhs) {
        println!("failed");
        return fail::apply_fail();
    }
    //Delete      : ({s ? = s}⊎E,σ) −→ (E,σ)
    if delete::is_deletable(&lhs, &rhs) {
        println!("Deletable!");
        return delete::apply_delete(&state);
    }
    if decompose::is_decomposable(&lhs, &rhs) {
        println!("Decompose + Bind:");

        // 1. Decompose-States
        let mut next = decompose::apply_decompose(constraint.clone(), state);

        // 2. Bind-States
        //    (auf das gleiche Constraint)
        let mut bind_states = bind::bind::apply_bind(constraint.clone(), state, config);

        // 3. Kombiniere beide Ergebnisse
        next.append(&mut bind_states);

        return next;
    }

    println!("Bind:");
    bind::bind::apply_bind(constraint, state, config)
    /*  //Decompose   : ({λx.asm ? = λx.atm}⊎E,σ) −→ ({s1 ? = t1,...,sm ? = tm}⊎E,σ)
    if decompose::is_decomposable(&lhs, &rhs) {
        println!("Yes! decomposable!");
        return decompose::apply_decompose(constraint, state);
    }

    //Bind        : ({ s ? = t}⊎E,σ) −→ ({s ? = t}⊎E,ϱσ)
    println!("Apply Bind");
    return bind::bind::apply_bind(constraint, state, config); */
}
