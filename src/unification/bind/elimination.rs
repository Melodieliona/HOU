// src/unification/elimination_for_f.rs

use crate::{
    counter::{BindingKind, Config},
    term::{Constraint, Term, TermKind, Type, Var, Variable},
    tree::{State, Step},
    unification::unification_utils::{build_bound_vars, flatten_hnf, init_fresh_gen},
};
use Type::Arrow;

// Eliminationsregel für F im Flex–Flex-Fall
pub fn apply_elimination(
    term: &Term,
    state: &State,
    config: &Config,
    constraint: &Constraint,
) -> Vec<State> {
    // Kopf und Argumente in HNF extrahieren
    let (_bs_l, head_l, _args_l) = flatten_hnf(&term);
    let (f_name, f_ty) = match head_l.get_fvar() {
        Some((n, t)) => (n, t),
        None => return Vec::new(),
    };

    // Typ F : a1->a2...->b aufsplitten n muss >= 1 sein
    let (alphas, beta) = f_ty.split_arrow();
    let n = alphas.len();
    if n == 0 {
        return Vec::new();
    }

    let mut result_states = Vec::new();
    let indices: Vec<usize> = (0..n).collect();
    for i in 1..=n {
        for seq in combinations(&indices, i) {
            let lam = build_lambda(term, &f_name, &alphas, &beta, &seq);
            let sigma2 = state.subst.clone().push(f_name.clone(), lam);
            let removed_args = n - seq.len();
            let new_state = state.try_binding(
                BindingKind::Elimination,
                removed_args,
                constraint,
                sigma2,
                config,
                Step::Elimination,
            );
            result_states.push(new_state);
        }
    }

    result_states
}

//Alle Kombinationen der Länge k aus indices in steigender Reihenfolge
fn combinations(indices: &[usize], k: usize) -> Vec<Vec<usize>> {
    fn rec(
        input: &[usize],
        k: usize,
        start: usize,
        current: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if current.len() == k {
            out.push(current.clone());
            return;
        }
        for i in start..input.len() {
            current.push(input[i]);
            rec(input, k, i + 1, current, out);
            current.pop();
        }
    }

    let mut ergebnis = Vec::new();
    let mut temp = Vec::new();
    rec(indices, k, 0, &mut temp, &mut ergebnis);
    ergebnis
}

// Baut λx1...xn. G x_{j1}...x_{ji}, inkl. frischer G-Variable
fn build_lambda(term: &Term, f_name: &str, alphas: &[Type], beta: &Type, seq: &[usize]) -> Term {
    let mut r#gen = init_fresh_gen(std::iter::once(term));
    // frischen Namen und Typ für G anlegen
    let base = f_name.trim_end_matches(|c: char| c.is_ascii_digit());
    let g_name = r#gen.fresh(base);
    let mut g_ty = beta.clone();
    for &idx in seq.iter().rev() {
        g_ty = Arrow(Box::new(alphas[idx].clone()), Box::new(g_ty));
    }
    let g_head = Term::Var(Variable {
        name: g_name,
        term_kind: TermKind::FVar,
        ty: g_ty.clone(),
        var: Var::Elimination,
    });

    // Körper G x_{j1}...x_{ji}
    let xs = build_bound_vars(alphas, "x");
    let mut acc = g_head;
    for &i in seq {
        if i < xs.len() - 1 {
            acc = Term::App {
                func: Box::new(acc),
                arg: Box::new(Term::Var(xs[i].clone())),
                result_ty: g_ty.clone(),
            };
        }
    }
    let body = acc;

    // λx1..xn body
    xs.into_iter().rev().fold(body, |acc, x| Term::Abs {
        param: x,
        body: Box::new(acc),
    })
}
