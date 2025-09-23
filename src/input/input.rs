use crate::counter::read_config;
use crate::input::{
    commands::{Command, read_command},
    printer::print_solutions,
    reader::{create_variable, read_constraints},
};
use crate::json_utils::add_variable;
use crate::term::Variable;
use crate::tree::{State, unify_stream};
use std::io;

// Hauptloop: Variablen verwalten und Unifikation starten
pub fn run_loop(all_var: &mut Vec<Variable>) -> io::Result<()> {
    loop {
        match read_command()? {
            Command::Stop => {
                println!("Programm wird beendet.");
                std::process::exit(0);
            }
            Command::Next => handle_next(all_var)?,
            Command::New(name) => handle_new(all_var, name)?,
            Command::Delete(name) => handle_delete(all_var, &name),
            Command::Invalid(txt) => eprintln!("Ungültig: '{}'", txt),
        }
    }
}

// Fügt neue Variablen hinzu, liest Constraints, konfiguriert und startet die Unifikation
fn handle_next(all_var: &mut Vec<Variable>) -> io::Result<()> {
    add_variable("variables.json", all_var)?;
    let constraints = read_constraints(all_var)?;
    let initial = State::new(constraints);
    println!("\n--- Binding-Limits konfigurieren ---");
    let config = read_config()?;
    println!("Eingestellte Config: {:?}\n", config);

    let stream = unify_stream(&initial, &config);
    print_solutions(stream);
    Ok(())
}

// Erstellt eine neue Variable, validiert und speichert sie, falls sie noch nicht existiert
fn handle_new(all_var: &mut Vec<Variable>, name: String) -> io::Result<()> {
    if all_var.iter().any(|v| v.name == name) {
        eprintln!("Warnung: '{}' existiert schon.", name);
    } else {
        match create_variable(name.clone()) {
            Ok(var) => {
                println!("Variable gespeichert: {:?}", var);
                all_var.push(var);
            }
            Err(err) => eprintln!("{}", err),
        }
    }
    Ok(())
}

// Löscht eine Variable mit dem gegebenen Namen
fn handle_delete(all_var: &mut Vec<Variable>, name: &str) {
    if let Some(idx) = all_var.iter().position(|v| v.name == name) {
        let removed = all_var.remove(idx);
        println!("Gelöscht: {}", removed);
    } else {
        println!("Keine Variable namens '{}'", name);
    }
}
