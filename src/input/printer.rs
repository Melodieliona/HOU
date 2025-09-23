use crate::tree::State;
use std::collections::HashSet;
use std::rc::Rc;

/// Druckt alle Lösungen in umgekehrter Schrittfolge
pub fn print_solutions(stream: impl Iterator<Item = State>) {
    let mut seen = HashSet::new();
    // 1) Fertige, eindeutige Lösungen sammeln
    let solutions: Vec<State> = stream
        .filter(|st| st.constraints.is_empty() && !st.failed) // nur Blätter
        .filter_map(|sol| {
            // kanonische Repräsentation ohne trailing newlines
            let repr = format!("{}", sol.subst).trim_end().to_string();
            // nur die erste Instanz jeder Repr. behalten
            if seen.insert(repr) { Some(sol) } else { None }
        })
        .collect();
    for (i, sol) in solutions.iter().enumerate() {
        println!("Lösung #{}:\n{}", i + 1, sol.subst);

        // Alle States einsammeln
        let mut stack: Vec<Rc<State>> = Vec::new();
        let mut current = Rc::new(sol.clone());
        loop {
            stack.push(current.clone());
            if let Some(parent) = current.parent.clone() {
                current = parent;
            } else {
                break;
            }
        }

        // Schritte rückwärts ausgeben
        for (step, state_rc) in stack.into_iter().rev().enumerate() {
            println!("Schritt {}:", step);
            if state_rc.constraints.is_empty() {
                println!("  (alle Constraints gelöst)");
            } else {
                for c in &state_rc.constraints {
                    println!("  • {}", c);
                }
            }
            for line in format!("{}", state_rc.subst).lines() {
                println!("    {}", line);
            }
        }
    }
}
