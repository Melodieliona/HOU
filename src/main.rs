mod a_renaming;
mod parser;
mod term;
mod tree;
mod unification;

use parser::*;
use rand::{Rng, thread_rng};
use tree::*;

use crate::tree::State;

fn main() {
    //Erzeuge Terme
    let constraint = match parse("h:Nat->Nat(F: Nat-> Nat a: Nat) ?= h: Nat(G: Nat-> Nat b:Nat)") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Parser-Fehler: {}", e);
            std::process::exit(1);
        }
    };
    //Erzeuge Baum zum speichern und abrufen
    let initial_state = State::new(vec![constraint]);
    let mut root = Node::new(initial_state);

    // Randomisierte Expansion bis zur Lösung aller Constraints
    random_unify(&mut root);
    // print_succeed_states() aufrufen .Alle Lösungen ausgeben
    println!("\nGefundene Succeed-States:");
    root.print_succeed_states();
}

fn random_unify(root: &mut Node) {
    let mut rng = thread_rng();
    // Vektor für rohe Zeiger auf pending Nodes
    let mut pending: Vec<*mut Node> = Vec::new();

    loop {
        pending.clear();
        Node::collect_pending_nodes(root, &mut pending);

        println!("Pending-Nodes: {}", pending.len());
        // Abbruch, wenn nichts mehr offen ist
        if pending.is_empty() {
            break;
        }

        // Zufällige Auswahl per Index
        let idx = rng.gen_range(0..pending.len());
        let node_ptr = pending.swap_remove(idx);

        // Einzelnen Schritt expandieren
        unsafe {
            (*node_ptr).expand_one();
        }
    }
}
