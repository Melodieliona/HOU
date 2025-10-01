mod counter;
mod input;
mod json_utils;
mod parser;
mod term;
mod tree;
mod unification;

use crate::input::reader::load_previous_session;
use std::io;

fn main() -> io::Result<()> {
    let mut all_var = load_previous_session()?;
    input::input::run_loop(&mut all_var)?;
    Ok(())
}
