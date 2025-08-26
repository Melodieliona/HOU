use crate::term::*;
use crate::tree::*;
//TODO: Testen, da o noch leer ist und bind gebaut werden muss

//Dereference ({
// λx.F s ? = λx.t}⊎E,σ) −→ ({λx.(σF)s ? = λx.t}⊎E,σ)
//where none of the previous transitions apply and F is mapped by σ
//1. Selbe lambda auf beiden Seiten
//2. Zerteilen und überprüfen, ob erstes Zeichen des Terms in σ.
//3. Falls ja ersetzen bzw. einfügen
pub fn is_dereference(lhs: &Term, rhs: &Term, subst: &PersistentSubst) -> bool {
    let (binders_l, core_l) = strip_abs(lhs);
    let (binders_r, _core_r) = strip_abs(rhs);
    //identisch und nicht leer
    if binders_l.is_empty() || binders_l != binders_r {
        return false;
    }
    //Erstes Zeichen aus dem Term holen
    let (head, _) = split_app(core_l);
    // Head muss FVar sein und in σ gemappt sein
    if let Term::FVar(f, _ty) = head {
        subst.to_hashmap().contains_key(f)
    } else {
        false
    }
}

//Liste der Binder und Restterm
fn strip_abs(term: &Term) -> (Vec<(String, Type)>, &Term) {
    let mut binders = Vec::new();
    let mut term = term;
    while let Term::Abs(param, ty, body) = term {
        binders.push((param.clone(), ty.clone()));
        term = body;
    }
    (binders, term)
}

//Ersten Zeichen aus Term
//TODO: schauen ob doppelt, vllt in term hinzufügen
fn split_app(term: &Term) -> (&Term, Vec<&Term>) {
    let mut args = Vec::new();
    let mut term = term;
    while let Term::App(f, arg, _res_ty) = term {
        args.push(arg.as_ref());
        term = f.as_ref();
    }
    args.reverse();
    (term, args)
}

//Wendet DEreference an:
pub fn apply_dereference(constraint: Constraint, subst: &PersistentSubst) -> Vec<State> {
    println!("Dereference-Regel angewendet auf {:?}", constraint);

    let Constraint(lhs, rhs) = constraint;

    // Binder & Kernterm extrahieren
    let (binders, core) = strip_abs(&lhs);
    let (head, args_ref) = split_app(core);

    // Head ist garantiert FVar und in σ enthalten
    if let Term::FVar(f, _ty) = head {
        // Hole σ(F)
        let mapping = subst.to_hashmap();
        let mut new_head = mapping.get(f).unwrap().clone();

        // Wende die alten Argumente an
        let arg_terms: Vec<Term> = args_ref.iter().map(|t| (*t).clone()).collect();
        let applied = build_app(new_head, &arg_terms);

        // Bindings wieder oben drauf
        let new_lhs = wrap_abs(&binders, applied);
        let new_constraint = Constraint(new_lhs, rhs);

        // Erzeuge neuen State mit unveränderter σ
        let next = State::with_subst(vec![new_constraint], subst.clone());
        vec![next]
    } else {
        // Das sollte nie passieren, da is_dereference das prüft
        vec![]
    }
}

//Baut den Term neu auf
fn build_app(head: Term, args: &[Term]) -> Term {
    args.iter().fold(head, |acc, arg| {
        let res_ty = match acc.get_type() {
            Type::Arrow(_ty_in, ty_out) => *ty_out.clone(),
            _ => panic!("Kein Funktionstyp gefunden."),
        };
        Term::App(Box::new(acc), Box::new(arg.clone()), res_ty)
    })
}

//Fügt die lambdas wieder hinzu
fn wrap_abs(binders: &[(String, Type)], body: Term) -> Term {
    binders.iter().rev().fold(body, |acc, (param, ty)| {
        Term::Abs(param.clone(), ty.clone(), Box::new(acc))
    })
}
