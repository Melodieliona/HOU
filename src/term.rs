use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Constraint(pub Term, pub Term);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub name: String,        //F
    pub term_kind: TermKind, // FVar
    pub ty: Type,            // Nat  oder Nat->Nat
    pub var: Var,            //Identification Variable
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum Var {
    Identification,
    Elimination,
    Basic,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum TermKind {
    FVar,
    BVar,
    Const,
    IVar,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Real,
    Nat,
    Bool,
    Arrow(Box<Type>, Box<Type>),
}
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Term {
    #[allow(dead_code)]
    Var(Variable),
    Abs {
        param: Variable,
        body: Box<Term>,
    },
    App {
        func: Box<Term>,
        arg: Box<Term>,
        result_ty: Type,
    },
}

impl Term {
    pub fn get_var(&self) -> Option<&Variable> {
        match self {
            Term::Var(v) => Some(v),
            _ => None,
        }
    }

    // Gibt name und ty zurück, wenn dieser Term eine FVar ist
    pub fn get_fvar(&self) -> Option<(&String, &Type)> {
        if let Term::Var(v) = self {
            if v.term_kind == TermKind::FVar {
                return Some((&v.name, &v.ty));
            }
        }
        None
    }
    // Liefert den Typ dieses Terms
    pub fn get_type(&self) -> Type {
        match self {
            Term::Var(var) => var.ty.clone(),

            Term::App { result_ty, .. } => result_ty.clone(),

            Term::Abs { param, body } => {
                // Pfeiltyp param.ty -> body.get_type()
                let ret = body.get_type();
                Type::Arrow(Box::new(param.ty.clone()), Box::new(ret))
            }
        }
    }

    //Gibt den TermKind des Kopfes des Terms wieder
    pub fn get_termkind(&self) -> TermKind {
        match self {
            Term::Var(var) => var.term_kind.clone(),
            Term::Abs { .. } => TermKind::BVar,
            Term::App { func, .. } => func.get_termkind(),
        }
    }
}
impl Type {
    //zählt arrows
    pub fn split_arrow(&self) -> (Vec<Type>, Type) {
        let mut tys_in = Vec::new();
        let mut rest = self.clone();
        while let Type::Arrow(ty_in, ty_out) = rest {
            tys_in.push((*ty_in).clone());
            rest = *ty_out;
        }
        (tys_in, rest)
    }
}

//------Display---------------------------------------------------------------------------
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Real => write!(f, "Real"),
            Type::Nat => write!(f, "Nat"),
            Type::Bool => write!(f, "Bool"),
            Type::Arrow(a, b) => write!(f, "{}->{}", a, b),
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Var(var) => write!(f, "{}", var),

            Term::Abs { param, body } => {
                write!(f, "λ {}. {}", param, body)
            }

            Term::App { func, arg, .. } => {
                // f a  (einfach Kopf und Argument, ohne zusätzliche Klammern)
                write!(f, "{}", func,)?;
                match &**arg {
                    Term::App {
                        func: _,
                        arg: _,
                        result_ty: _,
                    }
                    | Term::Abs { param: _, body: _ } => write!(f, " ({})", arg),
                    _ => write!(f, " {}", arg),
                }
            }
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Constraint(lhs, rhs) = self;
        write!(f, "{} ?= {}", lhs, rhs)
    }
}
