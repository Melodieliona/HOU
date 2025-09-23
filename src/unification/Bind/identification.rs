use crate::counter::*;
use crate::term::*;
use crate::tree::*;
use crate::unification::unification_utils::*;
use Type::Arrow;
pub fn apply_identification(constraint: &Constraint, state: &State, config: &Config) -> Vec<State> {
    print!("apply_id");
    let Constraint(lhs, rhs) = &constraint;
    let mut r#gen = init_fresh_gen(vec![lhs, rhs]);

    let (_bs_s, head_s, _args_s) = flatten_hnf(&lhs);
    let (_bs_t, head_t, _args_t) = flatten_hnf(&rhs);
    let (f_name, f_ty) = head_s.get_fvar().unwrap();
    let (g_name, g_ty) = head_t.get_fvar().unwrap();
    //Typen zerlegen in Parameter + Ergebnis
    let (alphas_f, beta_f) = f_ty.split_arrow();
    let (alphas_g, beta_g) = g_ty.split_arrow();
    //Selber Ergebnistyp
    if beta_f != beta_g {
        return Vec::new();
    }
    //gebundene Variablen bauen
    let xs = build_bound_vars(&alphas_f, "x");
    let ys = build_bound_vars(&alphas_g, "y");
    // Fi- und Gj-Funktionen bauen
    let fi = build_fi_terms(&alphas_f, &alphas_g, &mut r#gen, f_name);
    let gj = build_gj_terms(&alphas_f, &alphas_g, g_name, &mut r#gen);

    let h_term = build_h_term(&alphas_f, &alphas_g, beta_f.clone(), &mut r#gen);

    let f_lam = build_f_lambda(&h_term, &xs, &fi);
    let g_lam = build_g_lambda(&h_term, &ys, &gj);

    // Substitution und State erzeugen
    let new_subst = state
        .subst
        .clone()
        .push(f_name.clone(), f_lam)
        .push(g_name.clone(), g_lam);

    let new_state = try_binding(
        state,
        BindingKind::Identification,
        1,
        constraint,
        new_subst,
        config,
    );

    vec![new_state]
}

//Erzeugt Fi für jede Typkomponente mit dem korrekten Funktionstyp
pub fn build_fi_terms(
    alphas_f: &[Type],
    alphas_g: &[Type],
    r#gen: &mut FreshNameGen,
    f_name: &str,
) -> Vec<Term> {
    alphas_g
        .iter()
        .enumerate()
        .map(|(_i, gamma_ty)| {
            // Fi : α₁→…→αₙ→γᵢ
            let fi_ty = alphas_f.iter().rev().fold(gamma_ty.clone(), |acc, a| {
                Arrow(Box::new(a.clone()), Box::new(acc))
            });
            let base = f_name.trim_end_matches(|c: char| c.is_ascii_digit());
            let fi_var = Variable {
                name: r#gen.fresh(base),
                term_kind: TermKind::FVar,
                ty: fi_ty,
                var: Var::Basic,
            };
            Term::Var(fi_var)
        })
        .collect()
}

//Erzeugt Gi für jede Typkomponente mit dem korrekten Funktionstyp
fn build_gj_terms(
    alphas_f: &[Type],
    alphas_g: &[Type],
    g_name: &str,
    r#gen: &mut FreshNameGen,
) -> Vec<Term> {
    alphas_f
        .iter()
        .enumerate()
        .map(|(_j, alpha_ty)| {
            let gj_ty = alphas_g.iter().rev().fold(alpha_ty.clone(), |acc, g| {
                Arrow(Box::new(g.clone()), Box::new(acc))
            });
            let base = g_name.trim_end_matches(|c: char| c.is_ascii_digit());

            let gj_var = Variable {
                name: r#gen.fresh(base),
                term_kind: TermKind::FVar,
                ty: gj_ty,
                var: Var::Basic,
            };
            Term::Var(gj_var)
        })
        .collect()
}

//Erzeugt H mit dem zusammengesetzten Typ von F und G
fn build_h_term(
    alphas_f: &[Type],
    alphas_g: &[Type],
    beta: Type,
    r#gen: &mut FreshNameGen,
) -> Term {
    let mut h_ty = beta;
    for g in alphas_g.iter().rev() {
        h_ty = Arrow(Box::new(g.clone()), Box::new(h_ty));
    }
    for a in alphas_f.iter().rev() {
        h_ty = Arrow(Box::new(a.clone()), Box::new(h_ty));
    }
    let h_var = Variable {
        name: r#gen.fresh("H"),
        term_kind: TermKind::FVar,
        ty: h_ty,
        var: Var::Identification,
    };
    Term::Var(h_var)
}

// Baut λx1...xn. H x1..xn (F1 x1...xn)..(Fm x1..xn)
fn build_f_lambda(h: &Term, xs: &[Variable], fi: &[Term]) -> Term {
    let xs_terms: Vec<Term> = xs.iter().cloned().map(Term::Var).collect();
    let app_fis = fi
        .iter()
        .map(|fi_term| apply_with(fi_term.clone(), &xs_terms, |t| t.clone()))
        .collect::<Vec<_>>();
    let body = apply_with(h.clone(), &xs_terms, |t| t.clone());
    let body = apply_with(body, &app_fis, |t| t.clone());
    wrap_with_abstractions(&body, &xs.to_vec())
}

// Baut λy1..ym. H (G1 y1..ym)..(Gn y1..ym) y1..ym
fn build_g_lambda(h: &Term, ys: &[Variable], gj: &[Term]) -> Term {
    let ys_terms: Vec<Term> = ys.iter().cloned().map(Term::Var).collect();
    let app_gjs = gj
        .iter()
        .map(|gj_term| apply_with(gj_term.clone(), &ys_terms, |t| t.clone()))
        .collect::<Vec<_>>();
    let body = apply_with(h.clone(), &app_gjs, |t| t.clone());
    let body = apply_with(body, &ys_terms, |t| t.clone());
    wrap_with_abstractions(&body, &ys.to_vec())
}
