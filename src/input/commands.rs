use std::io::{self, Write};
use std::str::FromStr;

/// Mögliche Eingabebefehle
#[derive(Debug)]
pub enum Command {
    Delete(String),
    New(String),
    Next,
    Stop,
    Invalid(String),
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let lower = input.trim();
        if lower.eq_ignore_ascii_case("next") {
            Ok(Command::Next)
        } else if lower.eq_ignore_ascii_case("stop") {
            Ok(Command::Stop)
        } else if let Some(rest) = lower.strip_prefix("delete ") {
            Ok(Command::Delete(rest.trim().into()))
        } else if !lower.is_empty() {
            Ok(Command::New(lower.into()))
        } else {
            Ok(Command::Invalid(input.into()))
        }
    }
}

// Liest eine Zeile von stdin und wandelt sie in ein Command um
pub fn read_command() -> io::Result<Command> {
    print!("Eingabe (Name | Delete <Name> | Next | Stop): ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let cmd = buf.trim().parse().unwrap_or(Command::Invalid(buf.clone()));
    Ok(cmd)
}
