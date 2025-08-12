//Aufbau vom Term
#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    FVar(String),              // freie Variable Großbuchstaben
    BVar(String),              //gebundene Variable Kleinbuchstaben
    Const(String),             // f, g, h
    Abs(String, Box<Term>),    // λ-Abstraktion
    App(Box<Term>, Box<Term>), // Applikation
}

//Getter und Setter FUnktionen für Term
impl Term {
    //Getter
    //FVar
    pub fn get_fvar(&self) -> Option<&String> {
        if let Term::FVar(name) = self {
            Some(name)
        } else {
            None
        }
    }

    //BVar
    pub fn get_bvar(&self) -> Option<&String> {
        if let Term::BVar(name) = self {
            Some(name)
        } else {
            None
        }
    }
    //Const
    pub fn get_const(&self) -> Option<&String> {
        if let Term::Const(name) = self {
            Some(name)
        } else {
            None
        }
    }
    //Abs
    pub fn get_abs(&self) -> Option<(&String, &Term)> {
        if let Term::Abs(param, body) = self {
            Some((param, body))
        } else {
            None
        }
    }
    //App
    pub fn get_app(&self) -> Option<(&Term, &Term)> {
        if let Term::App(func, arg) = self {
            Some((func, arg))
        } else {
            None
        }
    }

    //Setter
    //Fvar
    pub fn set_fvar(&mut self, new_name: String) {
        *self = Term::FVar(new_name);
    }
    //Bvar
    pub fn set_bvar(&mut self, new_name: String) {
        *self = Term::BVar(new_name);
    }
    //Const
    pub fn set_const(&mut self, new_name: String) {
        *self = Term::Const(new_name);
    }
    //Abs
    pub fn set_abs(&mut self, param: String, body: Term) {
        *self = Term::Abs(param, Box::new(body));
    }
    //App
    pub fn set_app(&mut self, func: Term, arg: Term) {
        *self = Term::App(Box::new(func), Box::new(arg));
    }
}

//Aufbau Constraint
#[derive(Clone, Debug, PartialEq)]
pub struct Constraint(pub Term, pub Term);

impl Constraint {
    //Getter links und rechts
    pub fn left_term(&self) -> &Term {
        &self.0
    }
    pub fn right_term(&self) -> &Term {
        &self.1
    }
}
