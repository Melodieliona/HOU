// src/unification/hs_projection.rs

use crate::term::*;
use crate::tree::*;
use crate::unification::Bind::flatten_hnf;
use Type::Arrow;

pub fn apply_hs_projection(
    Constraint(lhs, rhs): Constraint,
    subst: &PersistentSubst,
) -> Vec<State> {
    // Kopf und gebundene Variablen extrahieren
    let (_bs_s, head_s, _args_s) = flatten_hnf(&lhs);
    let (f_name, f_ty) = head_s.get_fvar().unwrap();

    //  Typ zerlegen in Domänen as und Zieltyp b
    //TODO: alphas und beta statt ty_in ty_out im restlichen Code
    let (alphas, beta) = f_ty.split_arrow();
    let n = alphas.len();
    if n == 0 {
        return Vec::new();
    }

    let mut results = Vec::new();

    //  Für jede ai prüfen, ob ai = ys → b
    for (i, alpha_i) in alphas.iter().enumerate() {
        let (gammas, ty_out) = alpha_i.split_arrow();
        if ty_out != beta {
            continue;
        }

        // Gebundene Variablen x1…xn bauen
        let xs: Vec<Term> = alphas
            .iter()
            .enumerate()
            .map(|(j, ty)| Term::BVar(format!("x{}", j + 1), ty.clone()))
            .collect();

        //  Frische Hilfsvariablen F1…Fm nur lokal als Term::FVar anlegen
        let mut fm_terms = Vec::new();
        for (m, gamma_m) in gammas.iter().enumerate() {
            // Typ Fm : a1→…→an→ym
            let fm_ty = alphas.iter().rev().fold(gamma_m.clone(), |acc, a| {
                Arrow(Box::new(a.clone()), Box::new(acc))
            });
            let fm = Term::FVar(format!("{}{}", f_name, m + 1), fm_ty);
            fm_terms.push(fm);
        }

        //  Jede Fm auf x1…xn applizieren
        let mut fm_apps = Vec::new();
        for fm in fm_terms.iter() {
            let mut t = fm.clone();
            for x in &xs {
                if let Arrow(alphas, beta) = t.get_type().clone() {
                    // dom sollte gleich dem Typ von x sein
                    assert_eq!(*alphas, *x.get_type());
                    t = Term::App(Box::new(t), Box::new(x.clone()), *beta);
                }
            }
            fm_apps.push(t);
        }

        //  Kopfvariable xᵢ auf alle Fⱼ-Anwendungen anwenden
        let mut body = xs[i].clone();
        for arg in fm_apps.into_iter() {
            if let Arrow(alphas, beta) = body.get_type().clone() {
                assert_eq!(*alphas, *arg.get_type());
                body = Term::App(Box::new(body), Box::new(arg), *beta);
            }
        }

        //  λx₁…xₙ. body bauen
        let lam = xs.into_iter().rev().fold(body, |acc, x| {
            Term::Abs(x.get_name().clone(), x.get_type().clone(), Box::new(acc))
        });

        //  Neue Substitution: F ↦ λ… und State erzeugen
        let new_subst = subst.clone().push(f_name.clone(), lam);
        results.push(State::with_subst(Vec::new(), new_subst));
    }

    results
}
