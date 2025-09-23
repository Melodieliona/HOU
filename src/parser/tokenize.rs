use crate::parser::parser::ParseError;

// Tokens für den Parser
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Token {
    Letter(String),
    Dot,
    Comma,
    LParen,
    RParen,
    Eq,
    Lambda,
}

// Wandelt Eingabe in Token um oder gibt InvalidCharacter zurück
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::with_capacity(input.len());
    let mut chars = input.chars().enumerate().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            c if c.is_whitespace() => {}

            '\\' => {
                // erkennen von „\l“
                if let Some(&(_, 'l')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::Lambda);
                } else {
                    return Err(ParseError::InvalidCharacter((i, c)));
                }
            }

            '?' => {
                // erkennen von "?="
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::Eq);
                } else {
                    return Err(ParseError::InvalidCharacter((i, c)));
                }
            }

            'λ' => tokens.push(Token::Lambda),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '.' => tokens.push(Token::Dot),
            ',' => tokens.push(Token::Comma),

            c if c.is_ascii_alphanumeric() => {
                // mehrstellige Buchstaben-/Ziffernfolge sammeln
                let mut s = String::new();
                s.push(c);
                while let Some(&(_, next)) = chars.peek() {
                    if !next.is_whitespace() && next.is_alphanumeric() {
                        chars.next();
                        s.push(next);
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Letter(s));
            }

            _ => {
                return Err(ParseError::InvalidCharacter((i, c)));
            }
        }
    }

    Ok(tokens)
}
