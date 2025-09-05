use crate::term::*;
use crate::tree::*;
use crate::unification::Bind::flatten_hnf;
use Type::Arrow;

pub fn apply_identification(
    Constraint(lhs, rhs): Constraint,
    subst: &PersistentSubst,
) -> Vec<State> {
    let (_bs_s, head_s, _args_s) = flatten_hnf(&lhs);
    let (_bs_t, head_t, _args_t) = flatten_hnf(&rhs);

    let (f_name, f_ty) = head_s.get_fvar().unwrap();
    let (g_name, g_ty) = head_t.get_fvar().unwrap();

    //Typen zerlegen in Parameter + Ergebnis
    let (alphas_f, beta_f) = f_ty.split_arrow();
    let (alphas_g, beta_g) = f_ty.split_arrow();

    //Selber Ergebnistyp
    if beta_f != beta_g {
        return Vec::new();
    }

    let mut results = Vec::new();
    let new_subst = subst.clone();
    results.push(State::with_subst(Vec::new(), new_subst));
    results
}
