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

// Wendet eine Unifikationsregeln auf einen einzelnen Constraint an
// und liefert eine Liste von Folge-States zurück.
pub fn apply_unify_rules(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint.clone();
    println!("▶ apply_unify_rules: LHS = {:?}, RHS = {:?}", lhs, rhs);

    //Normalize an: ({λxm.s ? = λyn.t}⊎E,σ) −→ ({λxm.s ? = λxm.t′xn+1...xm}⊎E,σ)
    if normalize_an::is_normalizable_an(&lhs, &rhs) {
        println!("Yes! Normalize an!");
        return normalize_an::apply_normalize_an(constraint.clone(), subst);
    }
    //Normalize ß : ({λx.s ? = λx.t} ⊎E,σ) −→ ({λx.s↓h ? = λx.t↓h}⊎E,σ)
    if normalize_beta::is_normalizable_beta(&lhs, &rhs) {
        println!("Yes! Normalize an!");
        return normalize_beta::apply_normalize_beta(constraint.clone(), subst);
    }
    //Dereference : ({λx.F s ? = λx.t}⊎E,σ) −→ ({λx.(σF)s ? = λx.t}⊎E,σ)
    //Fail        :  ({λx.asm ? = λx.btn}⊎E,σ) −→ ⊥
    //Delete      : ({s ? = s}⊎E,σ) −→ (E,σ)
    if delete::is_deletable(&lhs, &rhs) {
        println!("Yes! deletable!");
        return delete::apply_delete(constraint, subst);
    }
    //Decompose   : ({λx.asm ? = λx.atm}⊎E,σ) −→ ({s1 ? = t1,...,sm ? = tm}⊎E,σ)
    if decompose::is_decomposable(&lhs, &rhs) {
        println!("Yes! decomposable!");
        return decompose::apply_decompose(constraint, subst);
    }
    //Bind        :({s ? = t}⊎E,σ) −→ ({s ? = t}⊎E,ϱσ)*/
    /* mehrere Ausführen: Bei Bind nützlich hier nicht
    // 1. Fall: Bindung einer Variable
    results.extend(bind::apply_bind(constraint.clone(), subst));

    // 2. Fall: Funktoren zerteilen
    results.extend(decompose::apply_decompose(constraint.clone(), subst));

    // 3. Fall: Succeed (Gleichung trivial, z.B. a = a)
    results.extend(succeed::apply_succeed(constraint.clone(), subst));

    // 4. Fall: Unifikation schlägt fehl
    results.extend(fail::apply_fail(constraint, subst)); */

    //  results
    println!("   –> Keine Regel gefunden, gebe Vec::new() zurück");
    Vec::new()
}
//pub use apply_unify_rules;
