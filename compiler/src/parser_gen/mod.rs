//! Generated parser module - contains the Pest parser and Rule enum

#[derive(pest_derive::Parser)]
#[grammar = "../lexer/lexer.pest"]
pub struct NovaParser;