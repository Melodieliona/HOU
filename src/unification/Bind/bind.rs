use crate::counter::*;
use crate::term::*;
use crate::tree::*;
use crate::unification::bind::*;
use crate::unification::unification_utils::*;

pub fn apply_bind(constraint: Constraint, state: &State, config: &Config) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint.clone();
    //λx muss in allen Fällen auf beiden Seiten gleich sein
    let (binders_l, head_s, _args_s) = flatten_hnf(&lhs);
    let (binders_r, head_t, _args_t) = flatten_hnf(&rhs);

    if binders_l != binders_r {
        return Vec::new();
    }
    // 1)  If the constraint is rigid-rigid, P(λx.s ? = λx.t) = ._----------------------------------------------------------------------------------------------
    if is_rigid(&head_s) && is_rigid(&head_t) {
        return vec![];
    }
    // 2) If the constraint is flex-rigid, let P(λx.F s ? = λx.at) be--------------------------------------------------------------------------------------------
    if is_flex_rigid(&head_s, &head_t) {
        let mut new_states = Vec::new();
        if matches!(
            (head_t.get_termkind(), head_s.get_termkind()),
            (TermKind::Const, _) | (_, TermKind::Const)
        ) {
            if imitation::is_imitationable(&head_s, &head_t) {
                new_states.extend(imitation::apply_imitation(&constraint, state, config));
            }
        }

        // - all Huet-style projections for F, if F is not an identification variable.
        if is_non_identification_var(&head_s, &state.subst) {
            new_states.extend(hs_projection::apply_hs_projection(
                &lhs,
                state,
                config,
                &constraint,
            ));
        }

        return new_states;
    }
    //3)---------------------------------------------------------------------------------------------------------------------------------------------------------
    if is_flex(&head_s) && is_flex(&head_t) && head_s != head_t {
        let mut new_states = Vec::new();
        new_states.extend(identification::apply_identification(
            &constraint,
            state,
            config,
        ));
        if is_non_identification_var(&head_s, &state.subst) {
            new_states.extend(hs_projection::apply_hs_projection(
                &lhs,
                state,
                config,
                &constraint,
            ));
        }
        if is_non_identification_var(&head_t, &state.subst) {
            new_states.extend(hs_projection::apply_hs_projection(
                &rhs,
                state,
                config,
                &constraint,
            ));
        }

        return new_states;
    }
    //4)----------------------------------------------------------------------------------------------------------------------------------------------------------
    if is_flex(&head_s) && is_flex(&head_t) && head_s == head_t {
        let mut results = Vec::new();
        if !is_elimination_variable(&head_s) {
            results.extend(elimination::apply_elimination(
                &lhs,
                state,
                config,
                &constraint,
            ));
            results.extend(elimination::apply_elimination(
                &rhs,
                state,
                config,
                &constraint,
            ));
        }
        return results;
    }
    vec![State::fail()]
}

// Prüft ob eine Seite flexibel und die andere rigide ist.
fn is_flex_rigid(s: &Term, t: &Term) -> bool {
    (is_flex(s) && is_rigid(t)) || (is_rigid(s) && is_flex(t))
}
