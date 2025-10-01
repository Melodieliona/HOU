use std::collections::HashMap;

use crate::term::*;
use crate::tree::State;
use crate::unification::unification_utils::*;

// Prüft, ob beide Seiten λ-Abstraktionen sind mit m ≥ n und entweder unterschiedliche Binder-Namen oder m > n.
pub fn is_normalizable_an(lhs: &Term, rhs: &Term) -> bool {
    let (lhs_vars, _) = collect_lambdas(lhs);
    let (rhs_vars, _) = collect_lambdas(rhs);
    let m = lhs_vars.len();
    let n = rhs_vars.len();

    n > 0 && m >= n && (m > n || lhs_vars != rhs_vars)
}

// Wendet die an-Normierung an:
pub fn apply_normalize_an(constraint: Constraint, state: &State) -> Vec<State> {
    let subst = &state.subst.clone();
    let Constraint(lhs, rhs) = constraint;

    let (lhs_vars, _) = collect_lambdas(&lhs);
    let (rhs_vars, rhs_body) = collect_lambdas(&rhs);
    let mapping = build_variable_mapping(&lhs_vars, &rhs_vars);

    let renamed_body = rename_vars(rhs_body, &mapping);
    let applied_args = apply_with(renamed_body, &lhs_vars[rhs_vars.len()..], |v: &Variable| {
        Term::Var(v.clone())
    });
    let new_rhs = wrap_with_abstractions(&applied_args, &lhs_vars);

    let new_constraint = Constraint(lhs.clone(), new_rhs);
    let new_state = state.with_subst_and_count(vec![new_constraint], subst.clone());
    vec![new_state]
}

//Ersetzt Variablennamen im Term gemäß Mapping
fn rename_vars(term: Term, mapping: &HashMap<String, String>) -> Term {
    match term {
        Term::Var(var) => {
            let new_name = mapping.get(&var.name).cloned().unwrap_or(var.name.clone());
            let new_var = Variable {
                name: new_name,
                term_kind: var.term_kind,
                ty: var.ty,
                var: Var::Basic,
            };
            Term::Var(new_var)
        }
        Term::Abs { param, body } => {
            let new_name = mapping
                .get(&param.name)
                .cloned()
                .unwrap_or(param.name.clone());
            let new_param = Variable {
                name: new_name,
                term_kind: param.term_kind,
                ty: param.ty,
                var: Var::Basic,
            };
            Term::Abs {
                param: new_param,
                body: Box::new(rename_vars(*body, mapping)),
            }
        }
        Term::App {
            func,
            arg,
            result_ty,
        } => Term::App {
            func: Box::new(rename_vars(*func, mapping)),
            arg: Box::new(rename_vars(*arg, mapping)),
            result_ty,
        },
    }
}

// Erstellt ein Mapping von Variablennamen rhs -> lhs
fn build_variable_mapping(lhs_vars: &[Variable], rhs_vars: &[Variable]) -> HashMap<String, String> {
    rhs_vars
        .iter()
        .enumerate()
        .map(|(i, rhs_var)| (rhs_var.name.clone(), lhs_vars[i].name.clone()))
        .collect()
}
