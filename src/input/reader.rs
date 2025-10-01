use crate::input::input::run_loop;
use crate::json_utils::{clear, read_json};
use crate::parser::parser::parse;
use crate::term::{Constraint, TermKind, Type, Var, Variable};
use std::io::{self, Write};

// Liest genau einen Constraint ein und prüft ihn
pub fn read_constraints(all_var: &mut Vec<Variable>) -> io::Result<Vec<Constraint>> {
    let mut constraints = Vec::new();

    loop {
        print!("Constraint (Format: lhs ?= rhs) | Stop | Zurück: ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let line = buf.trim();

        if line.eq_ignore_ascii_case("stop") {
            std::process::exit(0);
        }
        if line.eq_ignore_ascii_case("zurück") {
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
    print!("Typ (Real, Nat, Bool, Eigener Typ oder A->B): ");
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

//Fügt neue Typen hinzu
pub fn custom_types() -> io::Result<()> {
    use serde_json;
    use std::{fs, io};

    let raw = fs::read_to_string("types.json")?;
    let mut types: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();

    println!("Gib die Typen ein (einer pro Zeile, leer zum Beenden):");
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if !types.contains(&trimmed.to_string()) {
            types.push(trimmed.to_string());
        }
    }

    let file = fs::File::create("types.json")?;
    serde_json::to_writer_pretty(file, &types)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("Typen wurden gespeichert: {:?}", types);
    Ok(())
}

//Lädt oder verwirft Variablen und Typen aus der vorherigen SItzung
pub fn load_previous_session() -> io::Result<Vec<Variable>> {
    let mut allow_variables = true;
    let mut _types: Vec<String> = if prompt("--- Typen aus vorheriger Sitzung laden?")? {
        read_json("types.json").unwrap_or_else(|e| {
            eprintln!("Fehler beim Lesen types.json: {}", e);
            Vec::new()
        })
    } else {
        clear("types.json");
        allow_variables = false;
        Vec::new()
    };

    if prompt("Möchtest du eigene Typen hinzufügen?")? {
        crate::input::reader::custom_types()?;
        _types = read_json("types.json").unwrap_or_default();
    }

    let vars = if allow_variables && prompt("--- Variablen aus vorheriger Sitzung laden?")? {
        let previous_var = read_json::<Vec<Variable>>("variables.json").unwrap_or_else(|e| {
            eprintln!("Fehler beim Lesen variables.json: {}", e);
            Vec::new()
        });
        println!("{} Variablen übernommen.", previous_var.len());
        previous_var
    } else {
        clear("variables.json");
        println!("Vorherige Variablen verworfen");
        Vec::new()
    };

    Ok(vars)
}

//Antwortmöglichkeiten für load previous session
pub fn prompt(msg: &str) -> io::Result<bool> {
    loop {
        print!("{} (j/n): ", msg);
        io::stdout().flush()?;
        let mut ans = String::new();
        io::stdin().read_line(&mut ans)?;
        match ans.trim().to_lowercase().as_str() {
            "j" | "ja" => return Ok(true),
            "n" | "nein" => return Ok(false),
            _ => println!("Ungültige Eingabe."),
        }
    }
}
