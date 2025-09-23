use crate::input::input::run_loop;
use crate::json_utils::{clear_variables, read_json};
use crate::parser::parser::parse;
use crate::term::{Constraint, TermKind, Type, Var, Variable};
use std::io::{self, Write};

// Lädt Variablen aus der letzten Sitzung oder verwirft sie
pub fn load_previous_variables() -> io::Result<Vec<Variable>> {
    println!("\n--- Variablen aus vorheriger Sitzung laden? (y/n) ---");
    print!("Antwort: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    let mut all_var = Vec::new();
    if answer == "y" || answer == "ja" {
        let previous: Vec<Variable> = read_json("variables.json")?;
        println!("{} Variablen übernommen.", previous.len());
        all_var.extend(previous);
    } else if answer == "n" || answer == "nein" {
        clear_variables("variables.json");
        println!("Vorherige Variablen verworfen.");
    } else {
        println!("Ungültige Eingabe.");
        load_previous_variables()?;
    }
    Ok(all_var)
}

// Liest genau einen Constraint ein und prüft ihn
pub fn read_constraints(all_var: &mut Vec<Variable>) -> io::Result<Vec<Constraint>> {
    let mut constraints = Vec::new();

    loop {
        print!("Constraint (Format 'lhs ?= rhs') | stop | return: ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let line = buf.trim();

        if line.eq_ignore_ascii_case("stop") {
            std::process::exit(0);
        }
        if line.eq_ignore_ascii_case("return") {
            run_loop(all_var)?;
            break;
        }

        if check_constraint(line, all_var) {
            match parse(line) {
                Ok(c) => {
                    constraints.push(c);
                    return Ok(constraints);
                }
                Err(e) => {
                    eprintln!("Fehler beim Parsen: {}", e);
                }
            }
        }
    }
    Ok(constraints)
}

// Erstellt eine neue Variable (TermKind + Type) und validiert den Namen
pub fn create_variable(name: String) -> Result<Variable, String> {
    let kind = read_termkind()?;
    validate_name_case(&name, &kind)?;
    let ty = read_type()?;
    Ok(Variable {
        name,
        term_kind: kind,
        ty,
        var: Var::Basic,
    })
}

//Frägt den Nutzer nach TermKind und parst die Eingabe
fn read_termkind() -> Result<TermKind, String> {
    print!("TermKind (FVar, BVar, Const): ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).map_err(|e| e.to_string())?;
    buf.trim().parse::<TermKind>().map_err(|e| format!("{}", e))
}

//Frägt den Nutzer nach dem Typ und parst die Eingabe
fn read_type() -> Result<Type, String> {
    print!("Typ (Real, Nat, Bool oder A->B): ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).map_err(|e| e.to_string())?;
    buf.trim().parse::<Type>().map_err(|e| format!("{}", e))
}

// Validiert, ob der Name dem TermKind entspricht (Groß- vs. Kleinbuchstabe)
fn validate_name_case(name: &str, kind: &TermKind) -> Result<(), String> {
    if let Some(first) = name.chars().next() {
        match kind {
            TermKind::FVar if !first.is_uppercase() => {
                return Err(format!("FVar muss Großbuchstabe sein: {}", name));
            }
            TermKind::BVar | TermKind::Const if !first.is_lowercase() => {
                return Err(format!("{:?} muss Kleinbuchstabe sein: {}", kind, name));
            }
            _ => {}
        }
    }
    Ok(())
}

// Prüft Format lhs ?= rhs und ob alle Variablen definiert sind
fn check_constraint(line: &str, all_var: &[Variable]) -> bool {
    let parts: Vec<&str> = line.split("?=").collect();
    if parts.len() != 2 || parts[0].trim().is_empty() || parts[1].trim().is_empty() {
        println!("Falsches Format");
        return false;
    }

    let line = line.replace("\\l", " ");
    let tokens = line
        .split(|c: char| c.is_whitespace() || "().,λ".contains(c))
        .filter(|s| !s.is_empty() && *s != "?=")
        .collect::<Vec<_>>();

    for token in tokens {
        if all_var.iter().all(|v| v.name != token) {
            println!("Die Variable '{}' ist nicht definiert", token);
            return false;
        }
    }
    true
}
