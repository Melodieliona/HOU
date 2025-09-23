mod counter;
mod input;
mod json_utils;
mod parser;
mod term;
mod tree;
mod unification;

use crate::input::reader::load_previous_variables;
use std::io;

fn main() -> io::Result<()> {
    let mut all_var = load_previous_variables()?;
    input::input::run_loop(&mut all_var)?;
    Ok(())
}
