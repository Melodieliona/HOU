use crate::term::*;

// Zerlegt einen Term der Form `a s1 … sm` in `(head, [s1, …, sm])`.
// Falls kein Application-Konstrukt vorliegt, liefert es `(term, [])`.
fn head_and_args(term: &Term) -> (&Term, Vec<&Term>) {
    let mut args = Vec::new();
    let mut cur = term;
    // so lange `cur` eine Applikation ist, schraube ab und speichere das Argument
    while let Term::App(fun, arg) = cur {
        args.push(&**arg);
        cur = &**fun;
    }
    (cur, args)
}

// Ein Term ist in Kopfnormalform, wenn er
// Abs(x, body) ist und `body` die Form `a s1 … sm` hat,
// wobei `a` FVar, BVar oder Const ist.
fn is_hnf(term: &Term) -> bool {
    match term {
        Term::Abs(_, body) => {
            let (head, _) = head_and_args(body);
            matches!(head, Term::FVar(_) | Term::BVar(_) | Term::Const(_))
        }
        _ => false,
    }
}

//Prüft, ob beide Terme in hnf sind, denselben Binder x haben und denselben Head a.
// Liefert dann die Argumentlisten ([s1,…,sm], [t1,…,tm]) zurück.
pub fn can_decompose<'a>(
    t1: &'a Term,
    t2: &'a Term,
) -> Option<(String, Vec<&'a Term>, Vec<&'a Term>)> {
    if let (Term::Abs(x1, body1), Term::Abs(x2, body2)) = (t1, t2) {
        if x1 == x2 && is_hnf(t1) && is_hnf(t2) {
            let (head1, args1) = head_and_args(body1);
            let (head2, args2) = head_and_args(body2);
            // Head-Vergleich über PartialEq
            if head1 == head2 {
                return Some((x1.clone(), args1, args2));
            }
        }
    }
    None
}
