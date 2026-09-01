//! Frontend for the lossless C-shaped Mary source language.
//!
//! This is deliberately separate from `compiler`: the latter remains the
//! compatibility frontend for the original DSL.

mod preprocessor;
mod pretty;
mod script;
mod script_table;
mod table;
mod text_names;

pub use constants::{parse_constant_header, ConstantHeaderError};
pub use preprocessor::{preprocess, PreprocessError};
pub use pretty::{
    format_named_script, format_named_script_with_charmap, format_script, PrettyCError,
    PrettyCStmts,
};
pub use script::{
    parse_named_scripts, parse_named_scripts_with_charmap, parse_scripts, MaryScriptError,
    ScriptContext,
};
pub use script_table::{parse_script_table, ScriptSlot, ScriptTable, ScriptTableError};
pub use table::parse_callable_table_with_scope;
pub use table::{parse_callable_table, CallableTable, MaryCError};
pub use text_names::{parse_text_name_table, ScriptSymbols, TextNameTableError};

/// Options which select a concrete game/version at preprocessing time.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Object-like definitions, as supplied by repeated command line `-D`s.
    /// A definition without an explicit value has value `1`.
    pub defines: std::collections::HashMap<String, String>,
}

impl Options {
    pub fn define(mut self, definition: &str) -> Result<Self, PreprocessError> {
        let (name, value) = definition.split_once('=').unwrap_or((definition, "1"));
        validate_macro_name(name, 0)?;
        self.defines.insert(name.to_owned(), value.to_owned());
        match name {
            "MARY_FOMT_US" => {
                self.defines.insert("MARY_FOMT".into(), "1".into());
                self.defines.insert("MARY_US".into(), "1".into());
            }
            "MARY_MFOMT_US" => {
                self.defines.insert("MARY_MFOMT".into(), "1".into());
                self.defines.insert("MARY_US".into(), "1".into());
            }
            "MARY_FOMT_JP" => {
                self.defines.insert("MARY_FOMT".into(), "1".into());
                self.defines.insert("MARY_JP".into(), "1".into());
            }
            "MARY_MFOMT_JP" => {
                self.defines.insert("MARY_MFOMT".into(), "1".into());
                self.defines.insert("MARY_JP".into(), "1".into());
            }
            _ => {}
        }
        Ok(self)
    }
}

fn validate_macro_name(name: &str, line: usize) -> Result<(), PreprocessError> {
    let mut chars = name.chars();
    let valid_first = chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic());
    if !valid_first || !chars.all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        return Err(PreprocessError::InvalidMacroName {
            line,
            name: name.to_owned(),
        });
    }
    Ok(())
}
mod constants;
