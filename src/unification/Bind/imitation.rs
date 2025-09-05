use std::fmt::format;

use crate::term::*;
use crate::tree::*;
use crate::unification::*;
use Type::Arrow;

pub fn is_imitationable(head_s: &Term, head_t: &Term) -> bool {
    //Endtypen vergleichen
    let (_term_f, ty_out_f) = Type::split_arrow(head_s.get_type());
    let (_term_g, ty_out_g) = Type::split_arrow(head_t.get_type());
    ty_out_f == ty_out_g
}
pub fn apply_imitation(Constraint(lhs, rhs): Constraint, subst: &PersistentSubst) -> Vec<State> {
    println!("apply imitation");
    let (_bs_s, head_s, _args_s) = Bind::flatten_hnf(&lhs);
    let (_bs_t, head_t, _args_t) = Bind::flatten_hnf(&rhs);
    //F → λxn.g(F1xn)...(Fmxn)
    let (f_name, f_ty) = head_s.get_fvar().unwrap();
    let (g_name, g_ty) = head_t
        .get_const()
        .expect("apply_imitation: right head must be Const");

    let (term_f, _) = f_ty.split_arrow();
    let (term_g, _) = g_ty.split_arrow();

    //gebundene Variablen
    let xs: Vec<Term> = term_f
        .iter()
        .enumerate()
        .map(|(i, ty)| Term::BVar(format!("x{}", i + 1), ty.clone()))
        .collect();
    //F1...Fm in newsubst
    let mut fi_apps = Vec::new();
    for (i, g_ty) in term_g.iter().enumerate() {
        let fi_ty = term_f.iter().rev().fold(g_ty.clone(), |acc, a| {
            Arrow(Box::new(a.clone()), Box::new(acc))
        });

        let mut term = Term::FVar(format!("{}{}", f_name, i + 1), fi_ty);

        //Fi+ gebundene Variable
        for x in &xs {
            if let Arrow(dom, cod) = term.get_type().clone() {
                assert_eq!(*dom, *x.get_type());
                term = Term::App(Box::new(term), Box::new(x.clone()), *cod);
            }
        }
        fi_apps.push(term);
    }
    //kopf g auf Fi
    let body = fi_apps
        .into_iter()
        .fold(Term::Const(g_name.clone(), g_ty.clone()), |acc, arg| {
            if let Arrow(dom, ty_out) = acc.get_type().clone() {
                assert_eq!(*dom, *arg.get_type());
                Term::App(Box::new(acc), Box::new(arg), *ty_out)
            } else {
                unreachable!()
            }
        });
    //lambda.body
    let lam = xs.into_iter().rev().fold(body, |acc, x| {
        Term::Abs(x.get_name().clone(), x.get_type().clone(), Box::new(acc))
    });
    //f -> neuer Term in newsubt
    let new_subst = subst.clone().push(f_name.clone(), lam);
    vec![State::with_subst(Vec::new(), new_subst)]
}
