use crate::term::*;
use crate::tree::{PersistentSubst, State};
use crate::unification::bind::oracle::oracle;
use std::io;
use std::io::Write;

// Zähler für jede Bindungs-Art + Gesamtzahl
#[derive(Clone, Debug)]
pub struct BindingCounts {
    pub simple_proj: usize,
    pub functional_proj: usize,
    pub eliminations: usize,
    pub imitations: usize,
    pub identifications: usize,
    pub total: usize,
}

// Verschiedene Bindungsregeln, die gezählt werden
#[derive(Copy, Clone, Debug)]
pub enum BindingKind {
    SimpleProjection,
    FunctionalProjection,
    Elimination,
    Imitation,
    Identification,
}

// Limits für jede Bindungs-Art und insgesamt
#[derive(Clone, Debug)]
pub struct Config {
    pub max_simple_proj: usize,
    pub max_functional_proj: usize,
    pub max_eliminations: usize,
    pub max_imitations: usize,
    pub max_identifications: usize,
    pub max_total: usize,
}

// Standardgrenzen für alle Bindungsarten und Gesamtzahl
impl Default for Config {
    fn default() -> Self {
        Config {
            max_simple_proj: 10,
            max_functional_proj: 10,
            max_eliminations: 10,
            max_imitations: 10,
            max_identifications: 10,
            max_total: 40,
        }
    }
}

impl BindingCounts {
    // Erzeugt einen neuen Zähler mit allen Werten = 0
    pub fn new() -> Self {
        BindingCounts {
            simple_proj: 0,
            functional_proj: 0,
            eliminations: 0,
            imitations: 0,
            identifications: 0,
            total: 0,
        }
    }

    // Erhöht den Zähler für kind um count und passt total an
    pub fn add_count(&mut self, kind: BindingKind, count: usize) {
        match kind {
            BindingKind::SimpleProjection => self.simple_proj += count,
            BindingKind::FunctionalProjection => self.functional_proj += count,
            BindingKind::Elimination => self.eliminations += count,
            BindingKind::Imitation => self.imitations += count,
            BindingKind::Identification => self.identifications += count,
        }
        self.total += count;
    }

    // Prüft, ob count Schritte von kind die Limits in config überschreiten.
    pub fn would_exceed(&self, kind: BindingKind, count: usize, config: &Config) -> bool {
        let art_over = match kind {
            BindingKind::SimpleProjection => {
                self.simple_proj.saturating_add(count) > config.max_simple_proj
            }
            BindingKind::FunctionalProjection => {
                self.functional_proj.saturating_add(count) > config.max_functional_proj
            }
            BindingKind::Elimination => {
                self.eliminations.saturating_add(count) > config.max_eliminations
            }
            BindingKind::Imitation => self.imitations.saturating_add(count) > config.max_imitations,
            BindingKind::Identification => {
                self.identifications.saturating_add(count) > config.max_identifications
            }
        };
        let total_over = self.total.saturating_add(count) > config.max_total;
        art_over || total_over
    }
}

// Versucht eine Bindung anzuwenden oder liefert einen fehlgeschlagenen State
pub fn try_binding(
    state: &State,
    kind: BindingKind,
    count: usize,
    constraint: &Constraint,
    new_subst: PersistentSubst,
    config: &Config,
) -> State {
    if state.binding_counts.would_exceed(kind, count, config) {
        if let Some(oracle_state) = oracle(constraint, state, config) {
            return oracle_state;
        }
        return State::fail();
    } else {
        let mut st = state.with_subst_and_count(vec![constraint.clone()], new_subst);
        st.binding_counts.add_count(kind, count);
        st
    }
}

// Liest eine Zahl von stdin und verwendet default, wenn die Eingabe leer ist
pub fn read_limit(prompt: &str, default: usize) -> io::Result<usize> {
    loop {
        print!("{} [{}]: ", prompt, default);
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let txt = line.trim();
        if txt.is_empty() {
            return Ok(default);
        }
        if let Ok(n) = txt.parse::<usize>() {
            return Ok(n);
        }
        eprintln!("Ungültige Zahl: '{}'. Bitte erneut eingeben.", txt);
    }
}

// Frägt den Nutzer nach individuellen oder Standard-Grenzen für Bindungen
pub fn read_config() -> io::Result<Config> {
    println!("\n--- Möchtest du die Standardwerte für die Bindungsgrenzen verwenden? (y/n) ---");
    print!("Antwort: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if answer == "y" || answer == "ja" {
        let cfg = Config::default();
        println!("→ Standardwerte übernommen: {:?}", cfg);
        return Ok(cfg);
    }

    println!("→ Bitte individuelle Grenzen eingeben:");

    let default = Config::default();
    let max_simple_proj = read_limit("Max einfache Projektionen", default.max_simple_proj)?;
    let max_functional_proj =
        read_limit("Max funktionale Projektionen", default.max_functional_proj)?;
    let max_eliminations = read_limit("Max Eliminationen", default.max_eliminations)?;
    let max_imitations = read_limit("Max Imitationen", default.max_imitations)?;
    let max_identifications = read_limit("Max Identifikationen", default.max_identifications)?;
    let max_total = read_limit("Max total Bindings", default.max_total)?;

    Ok(Config {
        max_simple_proj,
        max_functional_proj,
        max_eliminations,
        max_imitations,
        max_identifications,
        max_total,
    })
}
