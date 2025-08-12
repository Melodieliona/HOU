use crate::term::Term;
use std::collections::{HashMap, HashSet};

/// Erzeugt fortlaufend frische Namen: x → x1, x2, …
struct NameGenerator {
    letters: Vec<String>,
    index: usize,
}

impl NameGenerator {
    fn new() -> Self {
        let letters = ('a'..='z').map(|c| c.to_string()).collect();
        NameGenerator { letters, index: 0 }
    }

    /// Nächster Buchstabe, der noch nicht in `used` ist
    fn fresh(&mut self, used: &HashSet<String>) -> String {
        while self.index < self.letters.len() {
            let cand = self.letters[self.index].clone();
            self.index += 1;
            if !used.contains(&cand) {
                return cand;
            }
        }
        panic!("Keine freien Buchstaben mehr verfügbar");
    }
}
// Kapselt den Zustand fürs Alpha-Renaming:
// env: Mapping alter → neuer Binder-Name
// used: Bereits vergebene Namen (inkl. freier Variablen)
// name_gen: Frische Namens-Erzeugung
pub struct AlphaRenamer {
    env: HashMap<String, String>,
    used: HashSet<String>,
    name_gen: NameGenerator,
}

impl AlphaRenamer {
    // Initialisiert den Renamer und sammelt alle freien Variablen
    pub fn new(term: &Term) -> Self {
        let mut renamer = AlphaRenamer {
            env: HashMap::new(),
            used: HashSet::new(),
            name_gen: NameGenerator::new(),
        };
        renamer.collect_free_vars(term);
        renamer
    }

    // Liefert alle bereits genutzten Namen zurück
    pub fn get_used_names(&self) -> &HashSet<String> {
        &self.used
    }

    // Fügt einen Namen zur `used`-Menge hinzu
    pub fn add_used_name(&mut self, name: String) {
        self.used.insert(name);
    }

    // Entfernt einen Namen aus der `used`-Menge
    pub fn remove_used_name(&mut self, name: &str) {
        self.used.remove(name);
    }

    // Führt das eigentliche Alpha-Renaming durch
    pub fn alpha_rename(&mut self, term: &Term) -> Term {
        self.rename(term)
    }

    // Sammelt alle freien Variablen in `used`
    fn collect_free_vars(&mut self, term: &Term) {
        match term {
            Term::Const(c) => {
                self.used.insert(c.clone());
            }

            Term::FVar(x) => {
                self.used.insert(x.clone());
            }

            Term::BVar(_) => {}

            Term::App(l, r) => {
                self.collect_free_vars(l);
                self.collect_free_vars(r);
            }

            Term::Abs(x, body) => {
                self.collect_free_vars(body);
                self.used.remove(x);
            }
        }
    }

    // Interne Rename-Funktion, rekursiv
    fn rename(&mut self, term: &Term) -> Term {
        match term {
            Term::Const(c) => Term::Const(c.clone()),

            Term::BVar(x) => {
                if let Some(new_name) = self.env.get(x) {
                    Term::BVar(new_name.clone())
                } else {
                    Term::BVar(x.clone())
                }
            }

            Term::FVar(x) => Term::FVar(x.clone()),

            Term::App(l, r) => {
                let l2 = self.rename(l);
                let r2 = self.rename(r);
                Term::App(Box::new(l2), Box::new(r2))
            }

            Term::Abs(x, body) => {
                // Entscheide ob wir einen frischen Namen brauchen
                let new_name = if self.used.contains(x) {
                    self.name_gen.fresh(&self.used)
                } else {
                    x.clone()
                };

                // Aktualisiere Zustand
                self.env.insert(x.clone(), new_name.clone());
                self.used.insert(new_name.clone());

                // Body umbenennen
                let renamed_body = self.rename(body);

                // Scope verlassen – Env-Eintrag zurücknehmen
                self.env.remove(x);

                Term::Abs(new_name, Box::new(renamed_body))
            }
        }
    }
}

// Öffentliche Funktion, die den Renamer verwendet
pub fn alpha_rename(term: &Term) -> Term {
    let mut renamer = AlphaRenamer::new(term);
    renamer.alpha_rename(term)
}
