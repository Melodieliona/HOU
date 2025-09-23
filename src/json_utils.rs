use serde::de::DeserializeOwned;
use std::fs::OpenOptions;
use std::io::*;

use crate::term::Variable;

//Löscht alle bisherigen Variablen in der JSON Datei
pub fn clear_variables(path: &str) {
    let file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Fehler beim Öffnen der Datei: {}", e);
            return;
        }
    };

    let writer = BufWriter::new(file);

    if let Err(e) = serde_json::to_writer_pretty(writer, &Vec::<Variable>::new()) {
        eprintln!("Fehler beim Schreiben: {}", e);
    }
}

//Liest die Datei
pub fn read_json<T: DeserializeOwned>(path: &str) -> Result<T> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(data)
}

//Fügt neue Variable hinzu
pub fn add_variable(path: &str, vars: &Vec<Variable>) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, vars)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}

//Gibt Variable anhand des Namens aus
pub fn get_var(name: &String, all_var: &[Variable]) -> Result<Variable> {
    all_var
        .iter()
        .find(|var| var.name == *name)
        .cloned()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Variabel konnte nicht gefunden werden"))
}
