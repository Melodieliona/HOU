use std::collections::HashMap;

use crate::term::*;
use crate::tree::{PersistentSubst, State};

// Prüft, ob beide Seiten λ-Abstraktionen sind mit m ≥ n und entweder unterschiedliche Binder-Namen oder m > n.
pub fn is_normalizable_an(lhs: &Term, rhs: &Term) -> bool {
    let (lhs_vars, _) = collect_lambdas(lhs);
    let (rhs_vars, _) = collect_lambdas(rhs);
    let m = lhs_vars.len();
    let n = rhs_vars.len();

    n > 0 && m >= n && (m > n || lhs_vars != rhs_vars)
}

// Wendet die an-Normierung an:
pub fn apply_normalize_an(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint;
    let (lhs_vars, _) = collect_lambdas(&lhs);
    let (rhs_vars, body) = collect_lambdas(&rhs);
    //let m = lhs_vars.len();
    let n = rhs_vars.len();

    // Mapping y -> x
    let mapping: HashMap<_, _> = rhs_vars
        .iter()
        .enumerate()
        .map(|(i, (y, _))| (y.clone(), lhs_vars[i].0.clone()))
        .collect();

    // t' = Body von rhs unter
    let t_prime = rename_vars(body, &mapping);

    // Sei inner = t' x...
    let inner = lhs_vars[n..].iter().fold(t_prime, |acc, (var, var_ty)| {
        let arg = Term::BVar(var.clone(), var_ty.clone());
        let res_ty = match acc.get_type() {
            Type::Arrow(ty_in, ty_out) => *ty_out.clone(),
            _ => panic!("apply_normalize_an: '{:#?}' ist keine Funktion", acc),
        };
        Term::App(Box::new(acc), Box::new(arg), res_ty)
    });

    // Neue rechte Seite
    let new_rhs = lhs_vars.iter().rev().fold(inner, |acc, (var, var_ty)| {
        Term::Abs(var.clone(), var_ty.clone(), Box::new(acc))
    });

    let new_constraint = Constraint(lhs.clone(), new_rhs);
    let st = State::with_subst(vec![new_constraint], subst.clone());
    vec![st]
}

// Extrahiert oberste λ-Binder und den Rumpf
fn collect_lambdas(term: &Term) -> (Vec<(String, Type)>, Term) {
    let mut vars = Vec::new();
    let mut t = term.clone();
    while let Term::Abs(fun, ty, body) = t {
        vars.push((fun.clone(), ty.clone()));
        t = *body;
    }
    (vars, t)
}

fn rename_vars(term: Term, mapping: &HashMap<String, String>) -> Term {
    match term {
        Term::FVar(name, ty) => Term::FVar(name, ty),
        Term::Const(name, ty) => Term::Const(name, ty),
        Term::BVar(name, ty) => {
            if let Some(x) = mapping.get(&name) {
                Term::BVar(x.clone(), ty)
            } else {
                Term::BVar(name, ty)
            }
        }
        Term::App(f, a, ty) => Term::App(
            Box::new(rename_vars(*f, mapping)),
            Box::new(rename_vars(*a, mapping)),
            ty,
        ),
        Term::Abs(p, ty, b) => {
            let new_p = mapping.get(&p).cloned().unwrap_or(p.clone());
            Term::Abs(new_p, ty, Box::new(rename_vars(*b, mapping)))
        }
    }
}
