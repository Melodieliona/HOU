// src/unification/hs_projection.rs

use crate::counter::*;
use crate::term::*;
use crate::tree::*;
use crate::unification::unification_utils::*;
use Type::Arrow;

pub fn apply_hs_projection(
    term: &Term,
    state: &State,
    config: &Config,
    constraint: &Constraint,
) -> Vec<State> {
    // Kopf und gebundene Variablen extrahieren
    let (_bs_s, head_s, _args_s) = flatten_hnf(term);
    let var = match head_s.get_var() {
        Some(v) => v,
        None => return Vec::new(),
    };
    let f_name = &var.name;

    let (alphas, beta) = var.ty.split_arrow();
    if alphas.is_empty() {
        return Vec::new();
    }

    let xs_vars = build_bound_vars(&alphas, "x");
    let xs: Vec<Term> = xs_vars.iter().cloned().map(Term::Var).collect();

    let binders = xs
        .iter()
        .map(|x| x.get_var().unwrap().clone())
        .collect::<Vec<_>>();

    let mut results = Vec::new();

    //  Für jede ai prüfen, ob ai = ys -> b
    for (i, alpha_i) in alphas.iter().enumerate() {
        let (gammas, beta_i) = alpha_i.split_arrow();
        if beta_i != beta {
            continue;
        }

        //  Frische Hilfsvariablen F1…Fm nur lokal als Term::FVar anlegen
        let fm_terms = make_fm_terms(&alphas, &gammas, &f_name, &term);

        //  Jede Fm auf x1…xn applizieren
        let fm_apps: Vec<Term> = fm_terms
            .into_iter()
            .map(|fm| apply_to_args(fm, &xs))
            .collect();

        let body = apply_to_args(xs[i].clone(), &fm_apps);
        let lam = wrap_with_abstractions(&body, &binders);

        let new_subst = state.subst.clone().push(f_name.clone(), lam);
        let kind = if gammas.is_empty() {
            BindingKind::SimpleProjection
        } else {
            BindingKind::FunctionalProjection
        };

        let new_state =
            state.try_binding(kind, 1, constraint, new_subst, config, Step::HsProjection);
        results.push(new_state);
    }
    results
}

fn make_fm_terms(alphas: &[Type], gammas: &[Type], f_name: &str, term: &Term) -> Vec<Term> {
    let mut r#gen = init_fresh_gen(std::iter::once(term));
    gammas
        .iter()
        .map(|gamma_m| {
            let fm_ty = alphas.iter().rev().fold(gamma_m.clone(), |acc, a| {
                Arrow(Box::new(a.clone()), Box::new(acc))
            });
            let fm_name = r#gen.fresh(f_name);
            Term::Var(Variable {
                name: fm_name,
                term_kind: TermKind::FVar,
                ty: fm_ty,
                var: Var::Basic,
            })
        })
        .collect()
}

fn apply_to_args(mut func: Term, args: &[Term]) -> Term {
    for x in args {
        if let Arrow(dom, cod) = func.get_type().clone() {
            assert_eq!(*dom, x.get_type());
            func = Term::App {
                func: Box::new(func),
                arg: Box::new(x.clone()),
                result_ty: *cod,
            };
        }
    }
    func
}
