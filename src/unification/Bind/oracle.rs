use crate::counter::*;
use crate::term::*;
use crate::tree::State;
use crate::unification::unification_utils::{flatten_hnf, init_fresh_gen, wrap_with_abstractions};

// Prüft, ob mindestens ein Binding-Limit erreicht ist
fn at_limit(counts: &crate::counter::BindingCounts, cfg: &Config) -> bool {
    counts.simple_proj >= cfg.max_simple_proj
        || counts.functional_proj >= cfg.max_functional_proj
        || counts.eliminations >= cfg.max_eliminations
        || counts.imitations >= cfg.max_imitations
        || counts.identifications >= cfg.max_identifications
        || counts.total >= cfg.max_total
}

pub fn oracle(Constraint(lhs, rhs): &Constraint, state: &State, cfg: &Config) -> Option<State> {
    if !at_limit(&state.binding_counts, cfg) {
        return None;
    }

    // HNF-Zerlegung
    let (_bs_l, head_l, args_l) = flatten_hnf(lhs);
    let (_bs_r, head_r, args_r) = flatten_hnf(rhs);

    let is_flex_l = matches!(head_l.get_termkind(), TermKind::FVar);
    let is_flex_r = matches!(head_r.get_termkind(), TermKind::FVar);

    //Bei Flex–Flex trivialen Unifier bauen
    if is_flex_l && is_flex_r {
        let (f_name, f_ty) = head_l.get_fvar().unwrap();
        let (g_name, _g_ty) = head_r.get_fvar().unwrap();

        // Ergebnis-Typ beider Flex-Variablen
        let (_, res_ty) = f_ty.split_arrow();

        let mut r#gen = init_fresh_gen(vec![lhs, rhs]);
        let h_var = Variable {
            name: r#gen.fresh("H"),
            term_kind: TermKind::FVar,
            ty: res_ty.clone(),
            var: Var::Basic,
        };
        let h_term = Term::Var(h_var);

        let mk = |args: &[Term]| {
            let binders: Vec<Variable> = args
                .iter()
                .enumerate()
                .map(|(i, a)| Variable {
                    name: format!("x{}", i + 1),
                    term_kind: TermKind::BVar,
                    ty: a.get_type(),
                    var: Var::Basic,
                })
                .collect();
            wrap_with_abstractions(&h_term, &binders)
        };

        let lam_f = mk(&args_l);
        let lam_g = mk(&args_r);

        let sub2 = state
            .subst
            .clone()
            .push(f_name.clone(), lam_f)
            .push(g_name.clone(), lam_g);

        return Some(state.with_subst_and_count(vec![], sub2));
    }

    // Bei FLex Rigid fail
    if (is_flex_l && !is_flex_r) || (!is_flex_l && is_flex_r) {
        return Some(State::fail());
    }

    None
}
