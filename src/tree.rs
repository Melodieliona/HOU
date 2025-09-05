use crate::term::*;
use crate::unification::*;
use std::collections::*;
use std::fmt;
use std::rc::Rc;

//Speicherung aller Constraints und Substitutionen
type Constraints = Vec<Constraint>;
type Substitution = HashMap<String, Term>;

//Substitutionen als verkettete Liste
#[derive(Clone, Debug)]
pub struct SubstEntry {
    var: String,
    val: Term,
    //Zeiger auf vorherigen Eintrag
    prev: Option<Rc<SubstEntry>>,
}

//Verkettung von SubstEntry
#[derive(Clone, Debug)]
pub struct PersistentSubst(pub Option<Rc<SubstEntry>>);

impl PersistentSubst {
    /// Erzeugt leere Substitution
    pub fn new() -> Self {
        PersistentSubst(None)
    }

    // Fügt eine neue Bindung an den Kopf
    pub fn push(self, var: String, val: Term) -> Self {
        let head = Rc::new(SubstEntry {
            var,
            val,
            prev: self.0,
        });
        PersistentSubst(Some(head))
    }

    // In eine HashMap umwandeln
    pub fn to_hashmap(&self) -> HashMap<String, Term> {
        let mut map = HashMap::new();
        let mut cur = self.0.clone();
        while let Some(entry) = cur {
            map.insert(entry.var.clone(), entry.val.clone());
            cur = entry.prev.clone();
        }
        map
    }

    pub fn pretty(&self) -> String {
        let mut map = self.to_hashmap();
        let mut names: Vec<String> = map.keys().cloned().collect();
        names.sort();
        let mut out = String::new();
        for name in names {
            if let Some(term) = map.remove(&name) {
                out.push_str(&format!("{} ↦ {}\n", name, term));
            }
        }
        out
    }
}

// Unifikationsstatus
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Succeed,
    Fail,
}

// Ein Unifikationszustand
#[derive(Clone, Debug)]
pub struct State {
    pub constraints: Vec<Constraint>,
    pub subst: PersistentSubst,
    pub failed: bool,
}

impl State {
    //Initialisiert neuen State mit keinen Substitutionen
    pub fn new(constraints: Constraints) -> Self {
        State {
            constraints,
            subst: PersistentSubst::new(),
            failed: false,
        }
    }

    // Wie new, aber erbt statt leerer Substitution die gegebene.
    pub fn with_subst(constraints: Vec<Constraint>, subst: PersistentSubst) -> Self {
        State {
            constraints,
            subst,
            failed: false,
        }
    }

    //Zweig fehlgeschlagen
    pub fn fail() -> Self {
        State {
            constraints: vec![],
            subst: PersistentSubst::new(),
            failed: true,
        }
    }
    //Zweig gelöst
    fn is_solved(&self) -> bool {
        self.constraints.is_empty() && !self.failed
    }
    //Gibt Status aus
    pub fn status(&self) -> Status {
        if self.failed {
            Status::Fail
        } else if self.is_solved() {
            Status::Succeed
        } else {
            Status::Pending
        }
    }
    pub fn pretty_subst(&self) -> String {
        format!("{}", self.subst)
    }
}

pub struct Dovetail<I> {
    streams: VecDeque<I>,
}

impl<I> Dovetail<I> {
    pub fn new(streams: Vec<I>) -> Self {
        Dovetail {
            streams: streams.into_iter().collect(),
        }
    }
}

impl<I> Iterator for Dovetail<I>
where
    I: Iterator<Item = State>,
{
    type Item = State;

    fn next(&mut self) -> Option<State> {
        // Solange noch Streams da sind
        while let Some(mut it) = self.streams.pop_front() {
            if let Some(state) = it.next() {
                // Wenn wir ein Ergebnis haben, requeue den Stream
                self.streams.push_back(it);
                return Some(state);
            }
            // andernfalls: drop den leeren Stream und weiter
        }
        None
    }
}
// Der Kern: baut für eine gegebene Konfiguration (Constraints + σ)
// einen `Iterator<State>` auf, der lazily alle Lösungen liefert.
pub fn unify_stream(
    constraints: Vec<Constraint>,
    subst: PersistentSubst,
) -> Box<dyn Iterator<Item = State>> {
    // 1) Wenn keine Constraints mehr offen sind, yield eine einzige State
    if constraints.is_empty() {
        let solved = State::with_subst(vec![], subst);
        return Box::new(std::iter::once(solved));
    }

    // 2) Nimm die erste Constraint heraus und löse sie
    let mut rest = constraints.clone();
    let head = rest.remove(0);

    // 3) Wende alle Unifizierungsregeln an → Vec<State>
    let succs = apply_unify_rules(head.clone(), &subst);

    // 4) Für jeden Nachfolger: kette seine Rest-Constraints an
    //    und rufe unify_stream rekursiv lazily auf.
    let sub_iters: Vec<Box<dyn Iterator<Item = State>>> = succs
        .into_iter()
        .map(move |mut st| {
            st.constraints.extend(rest.clone());
            Box::new(unify_stream(st.constraints, st.subst.clone()))
                as Box<dyn Iterator<Item = State>>
        })
        .collect();
    // 5) Dovetail über all diese Sub-Iteratoren
    Box::new(Dovetail::new(sub_iters))
}

impl fmt::Display for PersistentSubst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = self.to_hashmap();
        let mut names: Vec<_> = map.keys().cloned().collect();
        names.sort();
        for name in names {
            if let Some(term) = map.remove(&name) {
                writeln!(f, "{} ↦ {}", name, term)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.subst)
    }
}
