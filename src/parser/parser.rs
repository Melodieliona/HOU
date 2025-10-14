use crate::json_utils::{get_var, read_json};
use crate::parser::tokenize::{Token, tokenize};
use crate::term::{Constraint, Term, TermKind, Type, Variable};
use core::fmt;
use std::error::Error;
use std::fs;
use std::str::FromStr;

// Liest JSON, tokenisiert und parst zum Constraint
pub fn parse(input: &str) -> Result<Constraint, ParseError> {
    let all_vars: Vec<Variable> =
        read_json("variables.json").map_err(|e| ParseError::JsonError(e.to_string()))?;
    let tokens = tokenize(input)?;
    parse_constraint(&tokens, &all_vars)
}

//Parst den Constraint und überprüft, ob er vollständig bearbeitet wird
fn parse_constraint(tokens: &[Token], all_vars: &[Variable]) -> Result<Constraint, ParseError> {
    // Stelle des '?=' finden
    let eq_pos = tokens
        .iter()
        .position(|t| *t == Token::Eq)
        .ok_or(ParseError::MissingEq)?;

    let (left_tokens, rest) = tokens.split_at(eq_pos);
    let right_tokens = &rest[1..];

    // linken Term parsen
    let mut left_pos = 0;
    let left = parse_term(left_tokens, &mut left_pos, all_vars)?;
    if left_pos != left_tokens.len() {
        return Err(ParseError::LengthMismatch((left_pos, left_tokens.len())));
    }

    // rechten Term parsen
    let mut right_pos = 0;
    let right = parse_term(right_tokens, &mut right_pos, all_vars)?;
    if right_pos != right_tokens.len() {
        return Err(ParseError::LengthMismatch((right_pos, right_tokens.len())));
    }
    // Typprüfung: beide Seiten müssen denselben Typ haben
    let left_ty = left.get_type();
    let right_ty = right.get_type();
    if left_ty != right_ty {
        return Err(ParseError::TypeMismatch {
            expected: left_ty,
            found: right_ty,
        });
    }
    Ok(Constraint(left, right))
}

//Parst einen Term, entscheidet zwischen Lambda und Anwendung
fn parse_term(
    tokens: &[Token],
    pos: &mut usize,
    all_vars: &[Variable],
) -> Result<Term, ParseError> {
    if let Some(Token::Lambda) = tokens.get(*pos) {
        parse_lambda(tokens, pos, all_vars)
    } else {
        parse_application(tokens, pos, all_vars)
    }
}

// Parst Funktions-Anwendungen und prüft Typkonsistenz
fn parse_application(
    tokens: &[Token],
    pos: &mut usize,
    all_vars: &[Variable],
) -> Result<Term, ParseError> {
    // Erstes Atom parsen
    let mut func = parse_atom(tokens, pos, all_vars)?;

    // Solange neues Atom kommt, weitere Applikationen
    while matches!(
        tokens.get(*pos),
        Some(Token::Letter(_)) | Some(Token::LParen)
    ) {
        let arg = parse_atom(tokens, pos, all_vars)?;
        let f_ty = func.get_type();

        func = match f_ty {
            Type::Arrow(boxed_in, boxed_out) => {
                let in_ty = *boxed_in;
                let out_ty = *boxed_out;

                // Typen prüfen
                let a_ty = arg.get_type();
                if in_ty != a_ty {
                    return Err(ParseError::TypeMismatch {
                        expected: in_ty,
                        found: a_ty,
                    });
                }

                Term::App {
                    func: Box::new(func),
                    arg: Box::new(arg),
                    result_ty: out_ty,
                }
            }
            other => return Err(ParseError::ExpectedFunction(other)),
        };
    }

    Ok(func)
}

// Parst eine Klammergruppe oder eine Variable
fn parse_atom(
    tokens: &[Token],
    pos: &mut usize,
    all_vars: &[Variable],
) -> Result<Term, ParseError> {
    match tokens.get(*pos) {
        Some(Token::LParen) => {
            *pos += 1;
            let inner = parse_term(tokens, pos, all_vars)?;
            if tokens.get(*pos) != Some(&Token::RParen) {
                return Err(ParseError::MissingParen);
            }
            *pos += 1;
            Ok(inner)
        }
        Some(Token::Letter(name)) => {
            *pos += 1;
            let var =
                get_var(name, all_vars).map_err(|_| ParseError::UnknownVariable(name.clone()))?;
            Ok(Term::Var(var))
        }
        other => Err(ParseError::UnexpectedToken(other.cloned().unwrap())),
    }
}

// Parst eine kommagetrennte Liste von BVars
fn parse_lambda(
    tokens: &[Token],
    pos: &mut usize,
    all_vars: &[Variable],
) -> Result<Term, ParseError> {
    *pos += 1; // '\' oder 'λ'
    let params = parse_params(tokens, pos, all_vars)?;

    // Dot
    if tokens.get(*pos) != Some(&Token::Dot) {
        return Err(ParseError::UnexpectedToken(Token::Dot));
    }
    *pos += 1;

    let body = parse_term(tokens, pos, all_vars)?;
    let term = params.into_iter().rev().fold(body, |acc, var| Term::Abs {
        param: var.clone(),
        body: Box::new(acc),
    });

    Ok(term)
}

// Liest BVars bis zum Punkt und validiert den TermKind
fn parse_params(
    tokens: &[Token],
    pos: &mut usize,
    all_vars: &[Variable],
) -> Result<Vec<Variable>, ParseError> {
    let mut params = Vec::new();

    loop {
        match tokens.get(*pos) {
            Some(Token::Letter(name)) => {
                *pos += 1;
                let var = get_var(name, all_vars)
                    .map_err(|_| ParseError::UnknownVariable(name.clone()))?;
                if var.term_kind != TermKind::BVar {
                    return Err(ParseError::UnexpectedKind {
                        name: var.name.clone(),
                        expected: TermKind::BVar,
                        found: var.term_kind.clone(),
                    });
                }
                params.push(var);
            }
            other => return Err(ParseError::UnexpectedToken(other.cloned().unwrap())),
        }

        match tokens.get(*pos) {
            Some(Token::Comma) => *pos += 1,
            Some(Token::Dot) => break,
            other => return Err(ParseError::UnexpectedToken(other.cloned().unwrap())),
        }
    }

    Ok(params)
}
//-----------------------------Fehlerausgabe------------------------------------------------------------------

// Fehler während Lexing/Parsing
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidCharacter((usize, char)),
    MissingEq,
    MissingParen,
    LengthMismatch((usize, usize)),
    UnexpectedToken(Token),
    UnknownVariable(String),
    ExpectedFunction(Type),
    UnexpectedKind {
        name: String,
        expected: TermKind,
        found: TermKind,
    },
    TypeMismatch {
        expected: Type,
        found: Type,
    },
    JsonError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError::*;
        match self {
            InvalidCharacter((i, c)) => write!(f, "Ungültiges Zeichen '{}' an Position {}", c, i),
            MissingEq => write!(f, "Fehlender Operator '?='"),
            MissingParen => write!(f, "')' erwartet"),
            UnexpectedToken(tok) => write!(f, "Unerwartetes Token: {:?}", tok),
            LengthMismatch((got, exp)) => write!(
                f,
                "Nicht alle Tokens verbraucht (parsed {}, erwartet {})",
                got, exp
            ),
            UnknownVariable(n) => write!(f, "Variable '{}' nicht gefunden", n),
            ExpectedFunction(t) => write!(f, "Erwartet Pfeil‐Typ, gefunden {:?}", t),
            TypeMismatch { expected, found } => write!(
                f,
                "Typfehler: erwartet {:?}, gefunden {:?}",
                expected, found
            ),
            UnexpectedKind {
                name,
                expected,
                found,
            } => write!(
                f,
                "Ungültiger TermKind für '{}': erwartet {:?}, gefunden {:?}",
                name, expected, found
            ),
            JsonError(msg) => write!(f, "JSON‐Fehler: {}", msg),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

//------------------------------------------------------------------------------------------------------------
// Parsing für TermKind
impl FromStr for TermKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fvar" => Ok(TermKind::FVar),
            "bvar" => Ok(TermKind::BVar),
            "const" => Ok(TermKind::Const),
            other => Err(format!("Unbekannter TermKind: {}", other)),
        }
    }
}

// Parsing für Type
impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();

        // Pfeiltyp A->B erkennen
        if let Some(idx) = s.find("->") {
            let lhs = Self::from_str(&s[..idx])?;
            let rhs: Type = Self::from_str(&s[idx + 2..])?;
            return Ok(Type::Arrow(Box::new(lhs), Box::new(rhs)));
        }

        // Basis-Typen
        match s.to_lowercase().as_str() {
            "real" => Ok(Type::Real),
            "nat" => Ok(Type::Nat),
            "bool" => Ok(Type::Bool),
            _ => {
                let raw = fs::read_to_string("types.json").map_err(|e| e.to_string())?;
                let json: Vec<String> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
                if json.iter().any(|t| t == s) {
                    Ok(Type::Custom(s.to_string()))
                } else {
                    Err(format!("Unbekannter Typ: {}", s))
                }
            }
        }
    }
}
