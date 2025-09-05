pub mod bind;
pub mod hs_projection;
pub mod identification;
pub mod imitation;
pub mod iteration;
pub mod jps_projection;

use crate::term::*;
use crate::tree::*;
use crate::unification::Bind::imitation::is_imitationable;
use Type::Arrow;

pub fn is_bindable(lhs: &Term, rhs: &Term) -> bool {
    if let Term::FVar(name, _) = lhs {
        !occurs_in(name, rhs)
    } else {
        false
    }
}

//weil kein Oracle soll {G ? = f G} verhindern
fn occurs_in(var: &str, term: &Term) -> bool {
    match term {
        Term::FVar(n, _) => n == var,
        Term::BVar(_, _) => false,
        Term::Const(_, _) => false,
        Term::Abs(p, _, b) => p == var || occurs_in(var, &*b),
        Term::App(f, a, _) => occurs_in(var, f) || occurs_in(var, a),
    }
}

pub fn apply_bind(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    let Constraint(lhs, rhs) = constraint.clone();
    //λx muss in allen Fällen auf beiden Seiten gleich sein
    let (binders_l, head_s, args_s) = flatten_hnf(&lhs);
    let (binders_r, head_t, args_t) = flatten_hnf(&rhs);

    if binders_l != binders_r {
        println!("Binder l und r ungleich");
        return Vec::new();
    }
    // 1)  If the constraint is rigid-rigid, P(λx.s ? = λx.t) = .
    if is_rigid(head_s) && is_rigid(head_t) {
        println!("Rigid Rigid");
        println!("   –> Keine Regel gefunden, gebe Vec::new() zurück");
        return Vec::new();
    }
    // 2) If the constraint is flex-rigid, let P(λx.F s ? = λx.at) be
    if is_flex(head_s) && is_rigid(head_t) {
        println!("Flex Rigid");

        let mut new_sub = Vec::new();
        // - an imitation of a for F, if a is some constant g,
        if matches!(*head_t, Term::Const(_, _)) {
            if is_imitationable(head_s, head_t) {
                new_sub.extend(imitation::apply_imitation(constraint.clone(), subst));
                for (i, state) in new_sub.iter().enumerate() {
                    println!("Zweig #{}:\n{}", i + 1, state);
                }
            }
        }

        // - all Huet-style projections for F, if F is not an identification variable.
        if is_non_identification_variable(head_s, subst) {
            new_sub.extend(hs_projection::apply_hs_projection(
                constraint.clone(),
                subst,
            ));
        }
        return new_sub;
        //neue(n) State(s) zurückgeben
    }

    /*    // 3) If the constraint is flex-flex and the heads are different, let P(λx.F s ? = λx.Gt) be

    if is_flex(head_s) && is_flex(head_t) && head_s != head_t {
        println!("Flex Flex");
        let mut new_sub = Vec::new();
        // - all identifications and iterations for both F and G, and
        new_sub.extend(identification::apply_identification());

        new_sub.extend(iteration::apply_iteration());
        // - all JP-style projections for non-identification variables among F and G.
        for p in new_sub {
            if is_non_identification_variable(head_s, p) {
                new_sub.extend(jps_projection::apply_jps_projection());
            }
            if is_non_identification_variable(head_t, p) {
                new_sub.extend(jps_projection::apply_projection());
            }
        }
        //neuen State zurück geben
    }
    //4) If the constraint is flex-flex and the heads are identical, we distinguish two cases:
    // -if the head is an elimination variable, P(λx.s ? = λx.t) = ;
    // - otherwise, let P(λx.F s ? = λx.F t) be all iterations for F at arguments of functional
    // type (->) and all eliminations for F.
    if is_flex(head_s) && is_flex(head_t) && head_s == head_t {
        println!("Flex FLex und selber head");
        if is_elimination_variable(&args_s) {
            return Vec::new();
        }

        let mut new_sub = Vec::new();
        for (i, arg) in args_s.iter().enumerate() {
            if matches!(arg.get_type(), Type::Arrow(_, _)) {
                new_sub.extend(iteration::apply_iteration());
            }
        }
    } */
    Vec::new()
}

fn is_flex(t: &Term) -> bool {
    matches!(t, Term::FVar(_, _))
}

fn is_rigid(t: &Term) -> bool {
    matches!(t, Term::Const(_, _) | Term::BVar(_, _))
}

//Term-> Head und Body
fn deconstruct_app(mut t: Term) -> (Term, Vec<Term>) {
    let mut args = Vec::new();
    while let Term::App(fun, arg, _) = t {
        args.push(*arg);
        t = *fun;
    }
    args.reverse();
    (t, args)
}

// Baut aus jedem ρ ∈ P(s ?= t) einen neuen State mit ρ ∘ σ
fn wrap_states(
    constraint: Constraint,
    sub: &PersistentSubst,
    new_sub: Vec<PersistentSubst>,
) -> Vec<State> {
    new_sub
        .into_iter()
        .map(|p| {
            let subp = compose_substs(sub, &p);
            State::with_subst(vec![constraint.clone()], subp)
        })
        .collect()
}
//TODO: Platz für gemeinsamen Zugrif finden. Vllt mod in unification etc umziehen oder nach Term
// und in mod funktionen für alle tun?
//Funktion, die vector von binder namen, head, vector von argumenten zurück gibt
pub fn flatten_hnf(term: &Term) -> (Vec<&String>, &Term, Vec<&Term>) {
    // sammelt binder, gibt rest in cur aus
    let mut binders = Vec::new();
    let mut cur = term;
    while let Term::Abs(param, _ty, body) = cur {
        binders.push(param);
        cur = body;
    }

    // Kopf + Argumente
    let mut args = Vec::new();
    let mut head = cur;
    while let Term::App(fun, arg, _res_ty) = head {
        args.push(arg.as_ref());
        head = fun.as_ref();
    }
    args.reverse();

    (binders, head, args)
}

fn is_non_identification_variable(var: &Term, new_sub: &PersistentSubst) -> bool {
    if let Term::FVar(name, _) = var {
        println!("Keine Identifikations Variable");
        let bound = new_sub.to_hashmap();
        !bound.contains_key(name)
    } else {
        println!("Identifikations Variable");
        false
    }
}

fn compose_substs(sub: &PersistentSubst, new_sub: &PersistentSubst) -> PersistentSubst {
    // 1) Alte σ in HashMap
    let mut map = sub.to_hashmap();
    // 2) Neue ρ überschreibt
    for (v, t) in new_sub.to_hashmap() {
        map.insert(v, t);
    }
    // 3) Rückwärts in PersistentSubst gießen, so dass
    //    spätere Bindungen (aus new_sub) oben liegen.
    let mut acc = PersistentSubst::new();
    for (var, val) in map.into_iter() {
        acc = acc.push(var, val);
    }
    acc
}

fn is_elimination_variable(args: &[&Term]) -> bool {
    // alle Argumente dürfen keinen Arrow-Typ besitzen
    args.iter()
        .all(|t| !matches!(t.get_type(), Type::Arrow(_, _)))
}
