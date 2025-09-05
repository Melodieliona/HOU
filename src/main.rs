mod a_renaming;
mod parser;
mod term;
mod tree;
mod unification;

use parser::*;
use tree::*;

use crate::tree::State;

fn main() {
    //Erzeuge Terme
    let constraint = parse("F:Nat->Nat a:Nat ?= g:Nat->Nat F:Nat a:Nat").unwrap();

    //Erzeuge Baum zum speichern und abrufen
    let initial_state = State::new(vec![constraint]);
    let mut stream = unify_stream(initial_state.constraints, initial_state.subst);
    // Ausgabe
    for (i, sol) in stream.enumerate() {
        println!("Lösung #{}:\n{}", i + 1, sol.subst);
    }
}
