use crate::counter::*;
use crate::term::*;
use crate::tree::*;
use crate::unification::unification_utils::*;
use Type::Arrow;

//Testet, ob Imitation eingesetzt werden kann
pub fn is_imitationable(head_s: &Term, head_t: &Term) -> bool {
    let (_term_f, beta_f) = Type::split_arrow(&head_s.get_type());
    let (_term_g, beta_g) = Type::split_arrow(&head_t.get_type());
    beta_f == beta_g
}

pub fn apply_imitation(constraint: &Constraint, state: &State, config: &Config) -> Vec<State> {
    println!("apply imitation");
    let Constraint(lhs, rhs) = constraint;
    let mut r#gen = init_fresh_gen(vec![lhs, rhs]);

    let (_bs_s, mut head_s, _args_s) = flatten_hnf(&lhs);
    let (_bs_t, mut head_t, _args_t) = flatten_hnf(&rhs);

    //Seitenwechsel falls rhs Flex und Lhs rigid
    maybe_swap_sides(&mut head_s, &mut head_t);

    // Kopfvariablen extrahieren
    let (f_name, f_ty) = extract_fvar(&head_s, "apply_imitation: linker Kopf ist keine FVar");
    let (g_name, g_ty) = extract_const(&head_t, "apply_imitation: rechter Kopf ist keine Const");

    // Parameterlisten (Typen) extrahieren
    let (term_f, _res_f) = f_ty.split_arrow();
    let (term_g, _res_g) = g_ty.split_arrow();

    // gebundene Variablen erzeugen
    let xs = build_bound_vars(&term_f, "x");

    // Fi-Anwendungen bauen
    let fi_apps = build_fi_apps(&f_name, &term_f, &term_g, &xs, &mut r#gen);

    // g als Term (Kopf) und Körper zusammensetzen
    let g_head_term = make_const_term(&g_name, &g_ty);
    let body = apply_args_to_head(g_head_term, fi_apps);

    // lambda über xs (oder body selbst, falls leer)
    let lam = wrap_lambda_over_xs(body, &xs);

    // neue Substitution und neuer State
    let new_subst = state.subst.clone().push(f_name.clone(), lam);
    let new_state = try_binding(
        state,
        BindingKind::Imitation,
        1,
        constraint,
        new_subst.clone(),
        config,
    );
    vec![new_state]
}
// Falls linker Kopf konstant ist, tausche die Seiten damit linker Kopf flexibel wird
fn maybe_swap_sides(left: &mut Term, right: &mut Term) {
    if left.get_termkind() == TermKind::Const {
        std::mem::swap(left, right);
    }
}

// Extrahiere F-Variable aus Term oder panic mit Nachricht
fn extract_fvar(term: &Term, panic_msg: &str) -> (String, Type) {
    let v = term
        .get_var()
        .filter(|v| v.term_kind == TermKind::FVar)
        .expect(panic_msg);
    (v.name.clone(), v.ty.clone())
}

// Extrahiere Const-Variable aus Term oder panic mit Nachricht
fn extract_const(term: &Term, panic_msg: &str) -> (String, Type) {
    let v = term
        .get_var()
        .filter(|v| v.term_kind == TermKind::Const)
        .expect(panic_msg);
    (v.name.clone(), v.ty.clone())
}

// Baut Fi-Anwendungen: Fi x1 ... xn für jedes benötigte Fi
fn build_fi_apps(
    f_name: &str,
    term_f: &[Type],
    term_g: &[Type],
    xs: &[Variable],
    r#gen: &mut FreshNameGen,
) -> Vec<Term> {
    let mut apps = Vec::new();

    for i in 0..term_g.len() {
        let mut term = Term::Var(Variable {
            name: r#gen.fresh(f_name),
            term_kind: TermKind::FVar,
            ty: {
                let fi_ty = term_f.iter().rev().fold(term_g[i].clone(), |acc, a| {
                    Arrow(Box::new(a.clone()), Box::new(acc))
                });
                fi_ty
            },
            var: Var::Basic,
        });

        for x in xs {
            if let Type::Arrow(dom, cod) = term.get_type().clone() {
                assert_eq!(*dom, x.ty);
                term = Term::App {
                    func: Box::new(term),
                    arg: Box::new(Term::Var(x.clone())),
                    result_ty: *cod,
                };
            }
        }
        apps.push(term);
    }

    apps
}

// Erzeugt Term::Var für eine Konstante mit Typ
fn make_const_term(name: &str, ty: &Type) -> Term {
    Term::Var(Variable {
        name: name.to_string(),
        term_kind: TermKind::Const,
        ty: ty.clone(),
        var: Var::Basic,
    })
}

// Wendet die Liste von Argument-Terms nacheinander auf den Kopf an
fn apply_args_to_head(mut head: Term, args: Vec<Term>) -> Term {
    println!("body = {}", head);
    for arg in args {
        if let Type::Arrow(dom, cod) = head.get_type() {
            assert_eq!(*dom, arg.get_type());
            head = Term::App {
                func: Box::new(head),
                arg: Box::new(arg),
                result_ty: *cod,
            };
        }
    }
    head
}

// Falls xs leer, gib body zurück, sonst wrappe mit Lambdas über xs
fn wrap_lambda_over_xs(body: Term, xs: &Vec<Variable>) -> Term {
    if xs.is_empty() {
        body
    } else {
        wrap_with_abstractions(&body, &xs)
    }
}
