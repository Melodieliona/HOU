use std::fmt;
//Aufbau vom Term
#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    FVar(String, Type),              // freie Variable Großbuchstaben
    BVar(String, Type),              //gebundene Variable Kleinbuchstaben
    Const(String, Type),             // f, g, h
    Abs(String, Type, Box<Term>),    // λ-Abstraktion
    App(Box<Term>, Box<Term>, Type), // Applikation funktion, argument
}

//Getter und Setter FUnktionen für Term
impl Term {
    //Getter
    //FVar
    pub fn get_fvar(&self) -> Option<(&String, &Type)> {
        if let Term::FVar(name, ty) = self {
            Some((name, ty))
        } else {
            None
        }
    }

    //BVar
    pub fn get_bvar(&self) -> Option<(&String, &Type)> {
        if let Term::BVar(name, ty) = self {
            Some((name, ty))
        } else {
            None
        }
    }
    //Const
    pub fn get_const(&self) -> Option<(&String, &Type)> {
        if let Term::Const(name, ty) = self {
            Some((name, ty))
        } else {
            None
        }
    }
    //Abs
    pub fn get_abs(&self) -> Option<(&String, &Type, &Term)> {
        if let Term::Abs(param, ty, body) = self {
            Some((param, ty, body))
        } else {
            None
        }
    }
    //App
    pub fn get_app(&self) -> Option<(&Term, &Term, &Type)> {
        if let Term::App(func, arg, ty) = self {
            Some((func, arg, ty))
        } else {
            None
        }
    }

    //Setter
    //Fvar
    pub fn set_fvar(&mut self, new_name: String, new_ty: Type) {
        *self = Term::FVar(new_name, new_ty);
    }
    //Bvar
    pub fn set_bvar(&mut self, new_name: String, new_ty: Type) {
        *self = Term::BVar(new_name, new_ty);
    }
    //Const
    pub fn set_const(&mut self, new_name: String, new_ty: Type) {
        *self = Term::Const(new_name, new_ty);
    }
    //Abs
    pub fn set_abs(&mut self, param: String, ty: Type, body: Term) {
        *self = Term::Abs(param, ty, Box::new(body));
    }
    //App
    pub fn set_app(&mut self, func: Term, arg: Term, ty: Type) {
        *self = Term::App(Box::new(func), Box::new(arg), ty);
    }

    pub fn get_type(&self) -> &Type {
        match self {
            Term::App(_, _, ty)
            | Term::Const(_, ty)
            | Term::FVar(_, ty)
            | Term::BVar(_, ty)
            | Term::Abs(_, ty, _) => ty,
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            Term::FVar(name, _) => name,
            Term::BVar(name, _) => name,
            Term::Const(name, _) => name,
            _ => panic!("Term hat keinen Namen"),
        }
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

//Type-Struktur für Bind?
// Fest vordefinierte Basissorten
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BaseType {
    Bool,
    Nat,
    Int,
    Real,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Base(BaseType),
    Arrow(Box<Type>, Box<Type>), // Funktions­typ τ1 → τ2
}
impl Type {
    //zählt arrows
    pub fn split_arrow(&self) -> (Vec<Type>, Type) {
        let mut doms = Vec::new();
        let mut rest = self.clone();
        while let Type::Arrow(dom, cod) = rest {
            doms.push((*dom).clone());
            rest = (*cod).clone();
        }
        (doms, rest)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::FVar(name, _ty) => write!(f, "{}", name),
            Term::BVar(name, _ty) => write!(f, "{}", name),
            Term::Const(c, _ty) => write!(f, "{}", c),

            Term::App(fun, arg, _ty) => {
                write!(f, "{}", fun)?;
                match &**arg {
                    Term::App(_, _, _) | Term::Abs(_, _, _) => write!(f, " ({})", arg),
                    _ => write!(f, " {}", arg),
                }
            }

            Term::Abs(param, _ty, body) => {
                write!(f, "λ{}. {}", param, body)
            }
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Constraint(lhs, rhs) = self;
        write!(f, "{} = {}", lhs, rhs)
    }
}
