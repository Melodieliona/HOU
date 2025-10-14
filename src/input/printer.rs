use crate::term::{Term, Variable};
use crate::tree::State;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Result, Write};
use std::rc::Rc;

// Druckt alle Lösungen in umgekehrter Schrittfolge
pub fn print_solutions(stream: impl Iterator<Item = State>) -> Result<()> {
    let mut seen = HashSet::new();
    let mut seen_vars: HashSet<String> = HashSet::new();
    let mut out = BufWriter::new(File::create("solutions")?);

    // Fertige, eindeutige Lösungen sammeln
    let solutions: Vec<State> = stream
        .filter(|st| st.constraints.is_empty() && !st.failed) // nur Blätter
        .filter_map(|sol| {
            let repr = format!("{}", sol.subst).trim_end().to_string();
            if seen.insert(repr) { Some(sol) } else { None }
        })
        .collect();

    if solutions.is_empty() {
        println!("Es konnten keine Lösungen gefunden werden");
        writeln!(out, "Es konnten keine Lösungen gefunden werden")?;
    }

    for (i, sol) in solutions.iter().enumerate() {
        println!("Lösung #{}:\n{}", i + 1, sol.subst);
        writeln!(out, "Lösung #{}:\n{}", i + 1, sol.subst)?;

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
            println!("Schritt {} ({:?}):", step, state_rc.step);
            writeln!(out, "Schritt {} ({:?}):", step, state_rc.step)?;
            if state_rc.constraints.is_empty() {
                println!("  Alle Constraints gelöst");
                writeln!(out, "  Alle Constraints gelöst")?;
            } else {
                for c in &state_rc.constraints {
                    println!("  • {}", c);
                    writeln!(out, "  • {}", c)?;
                }
            }
            for line in format!("{}", state_rc.subst).lines() {
                println!("    {}", line);
                writeln!(out, "    {}", line)?;
            }
            // Variablen aus Constraints und Substitutionen sammeln
            let mut current_vars: HashMap<String, Variable> = HashMap::new();

            for c in &state_rc.constraints {
                collect_variables(&c.0, &mut current_vars);
                collect_variables(&c.1, &mut current_vars);
            }

            let mapping = state_rc.subst.to_hashmap();
            for term in mapping.values() {
                collect_variables(term, &mut current_vars);
            }

            // Neue Variablen erkennen und ausgeben
            for (name, var) in current_vars {
                if !seen_vars.contains(&name) && name != "x" && name != "y" {
                    println!(
                        "  Neue Variable: {} ({:?}, {},  ({:?}))",
                        var.name, var.term_kind, var.ty, var.var
                    );
                    writeln!(
                        out,
                        "  Neue Variable: {} ({:?}, {},  ({:?}))",
                        var.name, var.term_kind, var.ty, var.var
                    )?;
                    seen_vars.insert(name);
                }
            }
            writeln!(out)?;
        }
    }
    Ok(())
}

fn collect_variables(term: &Term, vars: &mut HashMap<String, Variable>) {
    match term {
        Term::Var(v) => {
            vars.insert(v.name.clone(), v.clone());
        }
        Term::Abs { param, body } => {
            vars.insert(param.name.clone(), param.clone());
            collect_variables(body, vars);
        }
        Term::App { func, arg, .. } => {
            collect_variables(func, vars);
            collect_variables(arg, vars);
        }
    }
}
