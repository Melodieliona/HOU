use self::ParseError::*;
use self::Term::*;
use self::Token::*;
use crate::a_renaming::*;
use crate::term::*;
use core::fmt;
use std::error::Error;

//Aufbau Token
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Token {
    Lambda,
    Letter(char),
    Dot,
    Comma,
    LParen,
    RParen,
    Eq,
    Colon,
    Arrow, //vllt unnötig
}

//Fehler
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidCharacter((usize, char)),
    InvalidExpression, //wenn bspw kein ( oder ) oder fehlender Parameter nach lambda
    EmptyExpression,   //vllt Position anzeigen
    MissingEq,
    MissingParem,
    LengthWrong((usize, usize)),
    UnexpectedToken(Token),
    UnexpectedEnd,
}
//Fehlerausgabe
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidCharacter((idx, char)) => {
                write!(f, "Ungültiges Zeichen '{}' bei {}", char, idx)
            }
            ParseError::InvalidExpression => write!(f, "The expression is invalid"),
            ParseError::EmptyExpression => write!(f, "The expression is empty"),
            ParseError::MissingEq => write!(f, "Es konnte nur 1 Term gefunden werden. '?='  fehlt"),
            ParseError::MissingParem => write!(f, "Es fehlt eine Klammer!"),
            ParseError::LengthWrong((len1, len2)) => write!(
                f,
                "Term wurde nicht komplett umgewandelt. Erwartete Länge: '{}' Aktuelle Länge: '{}'",
                len1, len2
            ),
            ParseError::UnexpectedToken(t) => write!(f, "Unerwartetes Token: {:?}", t),
            ParseError::UnexpectedEnd => write!(f, "Unerwartetes Ende der Eingabe"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

//Funktion mit der String in Tokens umgewandelt wird oder einen Fehler ausgibt
fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::with_capacity(input.len());
    let mut chars = input.chars().enumerate().peekable();

    //Solange der String nicht leer ist wird er durchlaufen
    while let Some((i, c)) = chars.next() {
        match c {
            c if c.is_whitespace() => {
                continue;
            }
            '/' => {
                if let Some(&(_, 'l')) = chars.peek() {
                    chars.next();
                    tokens.push(Lambda);
                } else {
                    return Err(InvalidCharacter((i, c)));
                }
            }
            '?' => {
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Eq);
                } else {
                    return Err(InvalidCharacter((i, c)));
                }
            }
            'λ' => tokens.push(Lambda),
            '(' => tokens.push(LParen),
            ')' => tokens.push(RParen),
            '.' => tokens.push(Dot),
            ',' => tokens.push(Comma),
            ':' => tokens.push(Colon),
            '-' => {
                if let Some(&(_, '>')) = chars.peek() {
                    chars.next();
                    tokens.push(Arrow);
                } else {
                    return Err(InvalidCharacter((i, c)));
                }
            }
            c if c.is_ascii_alphabetic() => tokens.push(Letter(c)),
            _ => {
                return Err(InvalidCharacter((i, c)));
            }
        }
    }
    if tokens.is_empty() {
        println!("Gebe einen String ein");
        Err(ParseError::EmptyExpression)
    } else {
        Ok(tokens)
    }
}

//TODO: ?= falls links oder recht leer fehlerausgabe

//Funktion die Tokens in einen Constraint umwandelt oder einen Fehler ausgibt
fn parser(tokens: &[Token]) -> Result<Constraint, ParseError> {
    //Finde die Position von '?='
    let eq_pos = tokens
        .iter()
        .position(|t| matches!(t, Eq))
        .ok_or(ParseError::MissingEq)?;

    //Slice vor und nach dem Eq
    let (left_tokens, rest) = tokens.split_at(eq_pos);
    let right_tokens = &rest[1..];

    //Linken Term parsen, mit eigenem pos-Zeiger
    let mut left_pos = 0;
    let left = parse_term(left_tokens, &mut left_pos)?;
    // sicherstellen, dass der gesamte Slice verbraucht wurde
    if left_pos != left_tokens.len() {
        return Err(LengthWrong((left_pos, right_tokens.len())));
    }

    //Rechten Term parsen, mit eigenem pos-Zeiger
    let mut right_pos = 0;
    let right = parse_term(right_tokens, &mut right_pos)?;
    if right_pos != right_tokens.len() {
        return Err(LengthWrong((right_pos, right_tokens.len())));
    }

    Ok(Constraint(left, right))
}

//Term parsen oder Fehler ausgeben
fn parse_term(tokens: &[Token], pos: &mut usize) -> Result<Term, ParseError> {
    // 1) Erst ein einzelnes Atom (Variable, Abs oder (…)) parsen
    let mut node = parse_atom(tokens, pos)?;

    // 2) So lange ein neues Atom folgt, linke Applikation bauen
    while matches!(
        tokens.get(*pos),
        Some(Lambda) | Some(Letter(_)) | Some(LParen)
    ) {
        let rhs = parse_atom(tokens, pos)?;

        // Typ-Konsistenz: Funktion muss Arrow sein
        let func_ty = node.get_type().clone();
        let (ty_in, ty_out) = match func_ty {
            Type::Arrow(i, o) => (i, o),
            _ => return Err(ParseError::InvalidExpression),
        };
        if &*ty_in != rhs.get_type() {
            return Err(ParseError::InvalidExpression);
        }
        node = Term::App(Box::new(node), Box::new(rhs), (*ty_out).clone());
    }

    //TODO: schauen ob unnötig bzw eig in unification
    node = alpha_rename(&node);

    Ok(node)
}

fn parse_atom(tokens: &[Token], pos: &mut usize) -> Result<Term, ParseError> {
    match tokens.get(*pos) {
        // Lambda-Abstraktion
        Some(Lambda) => {
            *pos += 1;

            // Parameterliste sammeln (x:Type,...,dot,body)
            let mut params = Vec::new();
            loop {
                match tokens.get(*pos) {
                    Some(Letter(c)) if c.is_lowercase() && !['f', 'g', 'h'].contains(&c) => {
                        let name = c.to_string();
                        *pos += 1;

                        //: erwartet
                        if tokens.get(*pos) != Some(&Token::Colon) {
                            return Err(ParseError::UnexpectedToken(tokens[*pos].clone()));
                        }
                        *pos += 1;
                        let param_ty = parse_type(tokens, pos)?;
                        params.push((name, param_ty));
                    }
                    Some(Comma) => {
                        *pos += 1; // nächster Parameter folgt
                    }
                    Some(Dot) => {
                        *pos += 1; // Ende der Parameterliste
                        let mut body = parse_term(tokens, pos)?;
                        for (param, param_ty) in params.into_iter().rev() {
                            let ret_ty = body.get_type().clone();
                            body = Term::Abs(
                                param.clone(),
                                Type::Arrow(Box::new(param_ty.clone()), Box::new(ret_ty.clone())),
                                Box::new(body),
                            );
                        }
                        break Ok(body);
                    }
                    Some(t) => {
                        return Err(ParseError::UnexpectedToken(t.clone()));
                    }
                    None => return Err(ParseError::UnexpectedEnd),
                }
            }
        }
        // Variable (gross = frei, klein = bound)
        Some(Letter(c)) => {
            *pos += 1;
            let name = c.to_string();
            if tokens.get(*pos) != Some(&Colon) {
                return Err(ParseError::UnexpectedToken(tokens[*pos].clone()));
            }
            *pos += 1;
            let ty = parse_type(tokens, pos)?;
            if c.is_uppercase() {
                Ok(FVar(name, ty))
            } else if ["f", "h", "g"].contains(&name.as_str()) {
                Ok(Const(name, ty))
            } else {
                Ok(BVar(name, ty))
            }
        }
        // Parenthesierte Sub-Expression
        Some(LParen) => {
            *pos += 1;
            let inner = parse_term(tokens, pos)?;
            // schließende Klammer erwarten
            if tokens.get(*pos) != Some(&RParen) {
                return Err(ParseError::MissingParem);
            }
            *pos += 1;
            Ok(inner)
        }

        _ => Err(ParseError::InvalidExpression),
    }
}

//Öffentliche Funktion die den String parsed und entweder einen Constraint oder Error ausgibt
pub fn parse(input: &str) -> Result<Constraint, ParseError> {
    //Zuerst den String in Token umwandeln
    let tokens = tokenize(input)?;
    //Term Struktur mit Token aufbauen
    parser(&tokens)
}

fn parse_type(tokens: &[Token], pos: &mut usize) -> Result<Type, ParseError> {
    let mut ty = match tokens.get(*pos) {
        Some(LParen) => {
            *pos += 1;
            let inner = parse_type(tokens, pos)?;
            if tokens.get(*pos) != Some(&RParen) {
                return Err(ParseError::MissingParem);
            }
            *pos += 1;
            inner
        }
        Some(Letter(c)) if c.is_ascii_alphabetic() => {
            // BaseType zusammensetzen
            let mut name = c.to_string();
            *pos += 1;
            while let Some(Letter(c2)) = tokens.get(*pos) {
                if c2.is_ascii_alphabetic() {
                    name.push(*c2);
                    *pos += 1;
                } else {
                    break;
                }
            }
            match name.as_str() {
                "Bool" => Type::Base(BaseType::Bool),
                "Nat" => Type::Base(BaseType::Nat),
                "Int" => Type::Base(BaseType::Int),
                "Real" => Type::Base(BaseType::Real),
                other => {
                    return Err(ParseError::UnexpectedToken(Token::Letter(
                        other.chars().next().unwrap(),
                    )));
                }
            }
        }
        _ => return Err(ParseError::InvalidExpression),
    };

    // ->
    if let Some(Arrow) = tokens.get(*pos) {
        *pos += 1;
        let right = parse_type(tokens, pos)?;
        ty = Type::Arrow(Box::new(ty), Box::new(right));
    }

    Ok(ty)
}
