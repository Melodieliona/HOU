use crate::counter::*;
use crate::term::*;
use crate::unification::bind::oracle::*;
use crate::unification::*;
use std::collections::*;
use std::fmt;
use std::rc::Rc;

type Constraints = Vec<Constraint>;

//Speichert Substitutionen als verkettete Liste
#[derive(Clone, Debug)]
pub struct SubstEntry {
    var: String,
    val: Term,
    prev: Option<Rc<SubstEntry>>,
}

// Repräsentiert eine persistent verkettete Substitution
#[derive(Clone, Debug)]
pub struct PersistentSubst(pub Option<Rc<SubstEntry>>);

impl PersistentSubst {
    // Erzeugt eine leere Substitution
    pub fn new() -> Self {
        PersistentSubst(None)
    }
    // Überprüft, ob die Variable in den bisherigen Substitutionen enthalten ist
    pub fn lookup(&self, var: &String) -> Option<Term> {
        let mut cur = self.0.clone();
        while let Some(entry) = cur {
            if entry.var == *var {
                return Some(entry.val.clone());
            }
            cur = entry.prev.clone();
        }
        None
    }

    // Fügt eine neue Bindung am Kopf hinzu, falls nicht schon identisch vorhanden
    pub fn push(self, var: String, val: Term) -> Self {
        if let Some(old) = self.lookup(&var) {
            if old == val {
                return self;
            }
        }
        let head = Rc::new(SubstEntry {
            var,
            val,
            prev: self.0,
        });
        PersistentSubst(Some(head))
    }

    // Wandelt die verkettete Substitution in eine HashMap um
    pub fn to_hashmap(&self) -> HashMap<String, Term> {
        let mut map = HashMap::new();
        let mut cur = self.0.clone();
        while let Some(entry) = cur {
            map.insert(entry.var.clone(), entry.val.clone());
            cur = entry.prev.clone();
        }
        map
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub constraints: Vec<Constraint>,
    pub subst: PersistentSubst,
    pub failed: bool,
    pub binding_counts: BindingCounts,
    pub parent: Option<Rc<State>>,
    pub step: Step,
}
#[derive(Clone, Debug)]
pub enum Step {
    Delete,
    Dereference,
    Decompose,
    NormalizeBeta,
    NormalizeAn,
    Fail,
    Start,
    Oracle,
    Identification,
    Imitation,
    HsProjection,
    Elimination,
}

impl State {
    // Erstellt einen neuen State mit Start-Constraints und leerer Subst
    pub fn new(constraints: Constraints) -> Self {
        State {
            constraints,
            subst: PersistentSubst::new(),
            failed: false,
            binding_counts: BindingCounts::new(),
            parent: None,
            step: Step::Start,
        }
    }

    // Erzeugt einen Nachfolge-State mit neuer Subst und aktuellen Counts
    pub fn with_subst_and_count(
        &self,
        constraints: Vec<Constraint>,
        subst: PersistentSubst,
        step: Step,
    ) -> Self {
        State {
            constraints,
            subst,
            failed: false,
            binding_counts: self.binding_counts.clone(),
            parent: Some(Rc::new(self.clone())),
            step,
        }
    }

    //Erzeugt einen fehlgeschlagenen State
    pub fn fail() -> Self {
        State {
            constraints: vec![],
            subst: PersistentSubst::new(),
            failed: true,
            binding_counts: BindingCounts::new(),
            parent: None,
            step: Step::Fail,
        }
    }

    // Prüft, ob alle Constraints gelöst und kein Fail-State vorhanden ist
    fn is_solved(&self) -> bool {
        self.constraints.is_empty() && !self.failed
    }

    // Versucht eine Bindung anzuwenden oder liefert einen fehlgeschlagenen State
    pub fn try_binding(
        &self,
        kind: BindingKind,
        count: usize,
        constraint: &Constraint,
        new_subst: PersistentSubst,
        config: &Config,
        step: Step,
    ) -> State {
        if self.binding_counts.would_exceed(kind, count, config) {
            if let Some(oracle_state) = oracle(constraint, self, config) {
                return oracle_state;
            }
            return State::fail();
        } else {
            let mut st = self.with_subst_and_count(vec![constraint.clone()], new_subst, step);
            st.binding_counts.add_count(kind, count);
            st
        }
    }
}

pub struct Dovetail<I> {
    streams: VecDeque<I>,
}

impl<I> Dovetail<I> {
    // Baut eine neue Dovetail-Queue aus den gegebenen Iteratoren
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

    // Liefert im Round-Robin-Stil das nächste Element aus allen Streams
    fn next(&mut self) -> Option<State> {
        while let Some(mut it) = self.streams.pop_front() {
            if let Some(state) = it.next() {
                self.streams.push_back(it);
                return Some(state);
            }
        }
        None
    }
}

// Baut einen lazily-evaluierenden Unifikations-Iterator für state
pub fn unify_stream(state: &State, config: &Config) -> Box<dyn Iterator<Item = State>> {
    //  Wenn keine Constraints mehr offen sind, liefert einen einzigen State
    if state.constraints.is_empty() {
        if state.is_solved() {
            return Box::new(std::iter::once(state.clone()));
        } else {
            return Box::new(std::iter::empty());
        }
    }

    // Nimm den nächsten Constraint heraus und löst ihn
    let mut rest = state.constraints.clone();
    let head = rest.remove(0);

    // Wende alle Unifizierungsregeln an -> Vec<State>
    let succs: Vec<State> = unification::apply_unify_rules(head.clone(), &state, config)
        .into_iter()
        .filter(|s| !s.failed)
        .collect();

    //  Für jeden Nachfolger rekursiv weiterführen
    let sub_iters: Vec<Box<dyn Iterator<Item = State>>> = succs
        .into_iter()
        .map(move |mut st| {
            st.constraints.extend(rest.clone());
            Box::new(unify_stream(&st, config)) as Box<dyn Iterator<Item = State>>
        })
        .collect();

    Box::new(Dovetail::new(sub_iters))
}

impl fmt::Display for PersistentSubst {
    // Formatiert die Substitution als sortierte Liste
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
    // Gibt nur die Substitution aus
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.subst)
    }
}
