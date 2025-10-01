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
        if lower.eq_ignore_ascii_case("weiter") {
            return Ok(Command::Next);
        } else if lower.eq_ignore_ascii_case("stop") {
            return Ok(Command::Stop);
        }
        let mut parts = lower.splitn(2, ' ');
        let command_part = parts.next().unwrap_or("");
        let argument_part = parts.next().unwrap_or("").trim();

        if command_part.eq_ignore_ascii_case("lösche") && !argument_part.is_empty() {
            Ok(Command::Delete(argument_part.into()))
        } else if !lower.is_empty() {
            Ok(Command::New(lower.into()))
        } else {
            Ok(Command::Invalid(input.into()))
        }
    }
}

// Liest eine Zeile von stdin und wandelt sie in ein Command um
pub fn read_command() -> io::Result<Command> {
    print!("Eingabe (Name | Lösche <Name> | Weiter | Stop): ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let cmd = buf.trim().parse().unwrap_or(Command::Invalid(buf.clone()));
    Ok(cmd)
}
