use crate::term::*;
use crate::unification;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
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
    constraints: Constraints,
    subst: PersistentSubst,
    failed: bool,
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
}

// Baumknoten
#[derive(Debug)]
pub struct Node {
    value: State,
    children: Vec<Node>,
}

//Knoten des aktuellen Baumes der auf Unterzustände verweist
impl Node {
    //Getter für node
    pub fn state(&self) -> &State {
        &self.value
    }
    //Erstellt neue Node
    pub fn new(value: State) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }
    //Hängt Kindknoten an
    pub fn add_child(&mut self, c: Node) {
        self.children.push(c)
    }

    // Postorder (bottom-up), ruft callback **nach** den Kindern auf
    pub fn traverse_bottom_up<F>(&self, f: &mut F)
    where
        F: FnMut(&State),
    {
        for c in &self.children {
            c.traverse_bottom_up(f);
        }
        f(&self.value);
    }

    // Postorder nur für Pending State
    pub fn traverse_pending_bottom_up<F>(&self, f: &mut F)
    where
        F: FnMut(&State),
    {
        for c in &self.children {
            c.traverse_pending_bottom_up(f);
        }
        if self.value.status() == Status::Pending {
            f(&self.value);
        }
    }

    /// Postorder nur für Succeed State
    pub fn traverse_succeed_leaves<F>(&self, f: &mut F)
    where
        F: FnMut(&State),
    {
        if self.children.is_empty() {
            if self.value.status() == Status::Succeed {
                f(&self.value);
            }
        } else {
            for c in &self.children {
                c.traverse_succeed_leaves(f);
            }
        }
    }

    // Ruft traverse_succeed_bottom_up auf und druckt jeden Succeed-State
    pub fn print_succeed_states(&self) {
        self.traverse_succeed_leaves(&mut |st| {
            println!("Succeed-State gefunden: {:?}", st);
        });
    }

    // Sammelt alle Pending-States und wählt einen zufällig aus
    fn pick_random_pending(root: &Node) -> Option<State> {
        // Alle Pending-States in einen Vec sammeln
        let mut pendings = Vec::new();
        root.traverse_pending_bottom_up(&mut |st| pendings.push(st.clone()));

        // Mit dem Zufallsgenerator einen auswählen
        let mut rng = thread_rng();
        pendings.choose(&mut rng).cloned()
    }

    // Nur ein Constraint abarbeiten und Kinder erzeugen
    pub fn expand_one(&mut self) {
        let (head, tail) = match self.value.constraints.split_first() {
            Some((first, rest)) => (first.clone(), rest.to_vec()),
            None => return,
        };
        println!("expand_one auf Constraint {}", head);
        // Unifikationsregeln anwenden
        let successors = unification::apply_unify_rules(head.clone(), &self.value.subst);
        // Für jeden neuen State ein Kind anfügen
        self.value.constraints.clear();
        self.value.constraints.shrink_to_fit();

        for mut state in successors {
            //constraints des Parent
            state.constraints.extend(tail.iter().cloned());
            let child = Node::new(state);
            self.children.push(child);
        }
    }

    //Alle Nodes sammeln, die noch Constraints übrig haben
    pub fn collect_pending_nodes(node: &mut Node, out: &mut Vec<*mut Node>) {
        if !node.value.constraints.is_empty() {
            out.push(node as *mut _);
        }
        for child in &mut node.children {
            Node::collect_pending_nodes(child, out);
        }
    }
}
