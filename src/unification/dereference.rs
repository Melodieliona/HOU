use crate::term::*;
use crate::tree::*;
use crate::unification::unification_utils::apply_with;
use crate::unification::unification_utils::flatten_hnf;
use crate::unification::unification_utils::wrap_with_abstractions;

//Dereference ({
// λx.F s ? = λx.t}⊎E,σ) −→ ({λx.(σF)s ? = λx.t}⊎E,σ)
pub fn is_dereference(lhs: &Term, rhs: &Term, state: &State) -> bool {
    println!("Dereference?");
    let subst = &state.subst.clone();
    let (binders_l, head_l, _core_l) = flatten_hnf(lhs);
    let (binders_r, head_r, _core_r) = flatten_hnf(rhs);
    //identisch und nicht leer
    if binders_l != binders_r {
        return false;
    }
    // Head_l oder Head_r muss FVar sein und in σ gemappt sein
    match (head_l, head_r) {
        (Term::Var(var), _) | (_, Term::Var(var)) if var.term_kind == TermKind::FVar => {
            let bound = subst.to_hashmap().contains_key(&var.name);
            println!("is_dereference -> {}", bound);
            bound
        }
        _ => false,
    }
}

//Wendet Dereference an:
pub fn apply_dereference(constraint: Constraint, state: &State) -> Vec<State> {
    println!("Dereference: {}", constraint);
    let subst = &state.subst.clone();
    let Constraint(lhs, rhs) = constraint;
    let (_binders_l, _head_l, _core_l) = flatten_hnf(&lhs);
    let (_binders_r, _head_r, _core_r) = flatten_hnf(&rhs);

    //Einmal F und einmal G falls vorhanden
    let new_lhs = dereference(&lhs, &subst);
    let new_rhs = dereference(&rhs, &subst);

    let new_const = Constraint(new_lhs, new_rhs);
    let new_state = state.with_subst_and_count(vec![new_const], subst.clone());
    vec![new_state]
}

fn dereference(lhs: &Term, subst: &PersistentSubst) -> Term {
    //Entferne lambda
    let (binders, head, args) = flatten_hnf(&lhs);

    let Term::Var(Variable {
        name: f,
        term_kind: TermKind::FVar,
        ty: _,
        var: _,
    }) = head
    else {
        return lhs.clone();
    };

    //Hole F->Subst
    let mapping = subst.to_hashmap();

    let new_head = match mapping.get(&f) {
        Some(term) => term.clone(),
        None => {
            return lhs.clone();
        }
    };

    let result = apply_with(new_head, &args, |t| t.clone());

    if !binders.is_empty() {
        let result = wrap_with_abstractions(&result.clone(), &binders.clone());
        return result;
    }
    println!("DereferenceEnd: {}", result);
    result
}
