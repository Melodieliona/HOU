use crate::term::*;
use crate::tree::*;

use std::collections::HashSet;

pub struct FreshNameGen {
    used: HashSet<String>,
    counter: usize,
}

impl FreshNameGen {
    pub fn new(existing: impl IntoIterator<Item = String>) -> Self {
        FreshNameGen {
            used: existing.into_iter().collect(),
            counter: 1,
        }
    }

    //generiert neuen Namen
    pub fn fresh(&mut self, prefix: &str) -> String {
        loop {
            let name = format!("{}{}", prefix, self.counter);
            self.counter += 1;
            if self.used.insert(name.clone()) {
                return name;
            }
        }
    }
}

//Initialisiert einen neuen Variablennamen, mithilfe der bereits verwendeten
pub fn init_fresh_gen<'a, I>(terms: I) -> FreshNameGen
where
    I: IntoIterator<Item = &'a Term>,
{
    let mut used = HashSet::new();
    for t in terms {
        collect_names(t, &mut used);
    }
    FreshNameGen::new(used)
}

//Sammelt, alle bereits verwendeten Namen
pub fn collect_names(term: &Term, set: &mut HashSet<String>) {
    match term {
        Term::Var(v) => {
            set.insert(v.name.clone());
        }
        Term::Abs { param, body } => {
            set.insert(param.name.clone());
            collect_names(body, set);
        }
        Term::App { func, arg, .. } => {
            collect_names(func, set);
            collect_names(arg, set);
        }
    }
}

//Ist keine Identifikationsvariable, falls nicht ins Subst enthalten und FVar
pub fn is_non_identification_var(term: &Term, subst: &PersistentSubst) -> bool {
    if let Term::Var(var) = term {
        if var.term_kind == TermKind::FVar {
            let bound = subst.to_hashmap();
            let still_free = !bound.contains_key(&var.name);
            return still_free && var.var != Var::Identification;
        }
    }
    false
}

//Überprüft, ob die Variable nur Basistypen besitzt
pub fn is_elimination_variable(head: &Term) -> bool {
    head.get_var().unwrap().var == Var::Elimination
}

//Überprüft, ob die Variable eine FVar ist
pub fn is_flex(t: &Term) -> bool {
    matches!(t, Term::Var(var) if var.term_kind == TermKind::FVar)
}

//Zeigt ob ein Kopf rigide ist
pub fn is_rigid(t: &Term) -> bool {
    !matches!(t.get_termkind(), TermKind::FVar)
}

// Baut lambda-Abstraktionen um einen Term herum
pub fn wrap_with_abstractions(term: &Term, binders: &Vec<Variable>) -> Term {
    binders
        .iter()
        .rev()
        .fold(term.clone(), |acc, var| Term::Abs {
            param: var.clone(),
            body: Box::new(acc),
        })
}

// Extrahiert alle Lambda-Binder und den verbleibenden Rumpf
pub fn collect_lambdas(term: &Term) -> (Vec<Variable>, Term) {
    let mut binders = Vec::new();
    let mut current = term.clone();
    while let Term::Abs { param, body } = current {
        binders.push(param);
        current = *body;
    }
    (binders, current)
}

//Erzeugt eine Liste von BVars mit durchnummerierten Namen
pub fn build_bound_vars(alphas: &[Type], prefix: &str) -> Vec<Variable> {
    alphas
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let name = format!("{}{}", prefix, i + 1);
            Variable {
                name,
                term_kind: TermKind::BVar,
                ty: ty.clone(),
                var: Var::Basic,
            }
        })
        .collect()
}

//Funktion, die Binder, Head und Argumente zurück gibt
pub fn flatten_hnf(term: &Term) -> (Vec<Variable>, Term, Vec<Term>) {
    // sammelt binder, gibt rest in cur aus
    let mut binders = Vec::new();
    let mut cur = term;
    while let Term::Abs { param, body } = cur {
        binders.push(param.clone());
        cur = body;
    }

    // Kopf + Argumente
    let mut args = Vec::new();
    let mut head = cur.clone();
    while let Term::App { func, arg, .. } = head {
        args.push(arg.as_ref().clone());
        head = func.as_ref().clone();
    }
    args.reverse();

    (binders, head, args)
}

//Wandelt Argumente in einen Term um
pub fn apply_with<A, F>(mut term: Term, args: &[A], to_term: F) -> Term
where
    F: Fn(&A) -> Term,
{
    for a in args {
        let arg_term = to_term(a);

        let result_ty = match term.get_type() {
            Type::Arrow(_, ret) => *ret.clone(),
            other => panic!("apply_with: erwartet Funktions-Typ, fand {}", other),
        };

        term = Term::App {
            func: Box::new(term),
            arg: Box::new(arg_term),
            result_ty,
        };
    }
    term
}

// Extrahiert Lambda Binder aus einem Term
pub fn get_abs(term: &Term) -> (Vec<&Variable>, &Term) {
    let mut binders = Vec::new();
    let mut term = term;
    while let Term::Abs { param, body } = term {
        binders.push(param);
        term = body;
    }
    (binders, term)
}
