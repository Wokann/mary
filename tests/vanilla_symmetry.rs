#[cfg(feature = "test_with_roms")]
mod tests {
    use std::{env, fs, io, sync::Arc};

    use mary::compiler::{self, ScriptError};
    use thiserror::Error;

    #[derive(Debug, Error)]
    enum TestFailure {
        #[error("IO Error")]
        IoError(#[from] io::Error),

        #[error("Environment variable error")]
        EnvVarError(#[from] env::VarError),

        #[error("Test failed on script {0}")]
        ScriptFailed(usize),

        #[error("Lib script parse failed: {0}")]
        LibScriptError(ScriptError),

        #[error("Not all scripts could be decompiled or recompiled")]
        DecompileFailure,
    }

    fn reencode_scripts(which_rom: &str) -> Result<(), TestFailure> {
        use mary::{bytecode, utility::rom_info};

        let rom = fs::read(env::var(which_rom)?)?;
        let table = rom_info::get_script_table(&rom)?;
        assert_eq!(table.first().map(|entry| entry.id()), Some(0));
        match rom_info::identify_rom(&rom).unwrap() {
            rom_info::FomtVariant::FomtUs => {
                assert_eq!(table.len(), 1329);
                assert!(matches!(
                    table.first(),
                    Some(rom_info::ScriptTableEntry::Empty { id: 0 })
                ));
            }
            rom_info::FomtVariant::MfomtUs => {
                assert_eq!(table.len(), 1416);
                assert!(matches!(
                    table.first(),
                    Some(rom_info::ScriptTableEntry::Empty { id: 0 })
                ));
            }
            rom_info::FomtVariant::FomtJp => {
                assert_eq!(table.len(), 1329);
                assert!(matches!(
                    table.first(),
                    Some(rom_info::ScriptTableEntry::Empty { id: 0 })
                ));
            }
            rom_info::FomtVariant::MfomtJp => {
                assert_eq!(table.len(), 1416);
                assert!(matches!(
                    table.first(),
                    Some(rom_info::ScriptTableEntry::Empty { id: 0 })
                ));
            }
        }
        assert!(table
            .windows(2)
            .all(|pair| pair[1].id() == pair[0].id() + 1));
        let encoded_scripts: Vec<(usize, &[u8])> = table
            .iter()
            .filter_map(|entry| match entry {
                rom_info::ScriptTableEntry::Empty { .. } => None,
                rom_info::ScriptTableEntry::Script { id, data, .. } => Some((*id, *data)),
            })
            .collect();

        for i in 0..encoded_scripts.len() {
            let (script_id, to_decode) = encoded_scripts[i];
            let script = bytecode::decode_script(&mut &to_decode[..]).unwrap();
            let reencoded = bytecode::encode_script(&script);

            if !(to_decode == reencoded) {
                fs::write(format!("test_failed_base_{script_id}.dmp"), to_decode)?;
                fs::write(format!("test_failed_reen_{script_id}.dmp"), reencoded)?;

                return Err(TestFailure::ScriptFailed(script_id));
            }
        }

        Ok(())
    }

    #[test]
    fn script_reencode_fomt() -> Result<(), TestFailure> {
        /* FOMT US : 0x080F89D4 1328 */

        let which_rom = "FOMT_US_GBA";
        // let script_table_addr = 0x080F89D4;
        // let script_table_size = 1328;

        reencode_scripts(which_rom)
    }

    #[test]
    fn script_reencode_mfomt() -> Result<(), TestFailure> {
        /* MFOMT US : 0x081014BC 1415 */

        let which_rom = "MFOMT_US_GBA";
        // let script_table_addr = 0x081014BC;
        // let script_table_size = 1415;

        reencode_scripts(which_rom)
    }

    #[test]
    fn script_reencode_fomt_jp() -> Result<(), TestFailure> {
        reencode_scripts("FOMT_JP_GBA")
    }

    #[test]
    fn script_reencode_mfomt_jp() -> Result<(), TestFailure> {
        reencode_scripts("MFOMT_JP_GBA")
    }

    fn render_decompiled_source(
        library: &str,
        script_id: usize,
        stmts: &[mary::ast::Stmt],
        charmap: Option<&mary::charmap::Charmap>,
    ) -> String {
        use mary::pretty_print::PrettyStmts;

        let body = match charmap {
            Some(charmap) => format!("{}", PrettyStmts::with_charmap(stmts, 1, charmap)),
            None => format!("{}", PrettyStmts::with_indent(stmts, 1)),
        };
        format!(
            "{library}\n\nscript {script_id} EventScript_{script_id}\n{{\n{}\n}}\n",
            body
        )
    }

    fn contains_jump_next(stmts: &[mary::ast::Stmt]) -> bool {
        use mary::ast::{Stmt, SwitchCase};

        stmts.iter().any(|stmt| match stmt {
            Stmt::JumpNext => true,
            Stmt::If(_, body) | Stmt::DoWhile(_, body) => contains_jump_next(body),
            Stmt::IfElse(_, then_body, else_body) => {
                contains_jump_next(then_body) || contains_jump_next(else_body)
            }
            Stmt::For(elements) => contains_jump_next(&elements.3),
            Stmt::Switch(_, cases, _, _) => cases.iter().any(|case| match case {
                SwitchCase::Case(_, body)
                | SwitchCase::Fallthrough(_, body)
                | SwitchCase::Default(body)
                | SwitchCase::DefaultFallthrough(body)
                | SwitchCase::ImplicitDefault(body)
                | SwitchCase::DeadJump(body) => contains_jump_next(body),
            }),
            _ => false,
        })
    }

    fn recompile_scripts(
        which_rom: &str,
        which_lib: &str,
        charmap_path: Option<&str>,
    ) -> Result<(), TestFailure> {
        use mary::{
            bytecode::{decode_script, encode_script},
            decompiler::decompile_script,
            utility::rom_info::{get_script_table, ScriptTableEntry},
        };

        #[derive(Debug)]
        enum FailType {
            Ok,
            DecompileError(String),
            SourceParseError(String),
            OutputMismatch {
                first_diff: Option<usize>,
                original_len: usize,
                reencoded_len: usize,
            },
        }

        let rom = fs::read(env::var(which_rom)?)?;
        let encoded_scripts: Vec<(usize, &[u8])> = get_script_table(&rom)?
            .into_iter()
            .filter_map(|entry| match entry {
                ScriptTableEntry::Empty { .. } => None,
                ScriptTableEntry::Script { id, data, .. } => Some((id, data)),
            })
            .collect();

        let lib_text = fs::read_to_string(env::var(which_lib)?)?;
        let charmap = charmap_path
            .map(|path| fs::read_to_string(path))
            .transpose()?
            .map(|source| mary::charmap::Charmap::parse(&source).unwrap())
            .map(Arc::new);

        let lib_scope = match compiler::parse_string(&lib_text) {
            Ok(parse_context) => parse_context.const_scope,
            Err(err) => return Err(TestFailure::LibScriptError(err)),
        };

        let mut scripts = vec![];

        for i in 0..encoded_scripts.len() {
            let to_decode = encoded_scripts[i].1;
            let decoded = decode_script(&mut &to_decode[..]).unwrap();
            scripts.push(decoded);
        }

        let mut results: Vec<FailType> = vec![];
        let mut low_level_ids = vec![];
        let mut jump_next_ids = vec![];

        for i in 0..encoded_scripts.len() {
            let script_id = encoded_scripts[i].0;
            let to_decode = encoded_scripts[i].1;
            let decoded = &scripts[i];

            let decompiled = match decompile_script(&decoded, &lib_scope) {
                Ok(decompiled) => decompiled,

                Err(err) => {
                    results.push(FailType::DecompileError(format!("{err}")));
                    continue;
                }
            };

            if matches!(&decompiled[..], [mary::ast::Stmt::Ir(_)]) {
                low_level_ids.push(script_id);
                if let Err(err) =
                    mary::decompiler::decompile_script_structured(&scripts[i], &lib_scope)
                {
                    println!("structured failure {script_id}: {err}");
                }
            }

            if contains_jump_next(&decompiled) {
                jump_next_ids.push(script_id);
            }

            // A decompilation is only successful if its printed source can be
            // parsed and compiled again. Compiling the in-memory AST directly
            // would hide pretty-printer/parser information loss.
            let source =
                render_decompiled_source(&lib_text, script_id, &decompiled, charmap.as_deref());
            let parsed_source = match &charmap {
                Some(charmap) => compiler::parse_string_with_charmap(&source, Arc::clone(charmap)),
                None => compiler::parse_string(&source),
            };
            let recompiled = match parsed_source {
                Ok(mut parsed) if parsed.scripts.len() == 1 => parsed.scripts.remove(0).2,
                Ok(parsed) => {
                    results.push(FailType::SourceParseError(format!(
                        "expected exactly one script after parsing, got {}",
                        parsed.scripts.len()
                    )));
                    continue;
                }
                Err(err) => {
                    results.push(FailType::SourceParseError(format!("{err}")));
                    continue;
                }
            };

            let reencoded = encode_script(&recompiled);

            if !(to_decode == reencoded) {
                let first_diff = to_decode
                    .iter()
                    .zip(&reencoded)
                    .position(|(original, rebuilt)| original != rebuilt)
                    .or_else(|| {
                        (to_decode.len() != reencoded.len())
                            .then_some(to_decode.len().min(reencoded.len()))
                    });
                results.push(FailType::OutputMismatch {
                    first_diff,
                    original_len: to_decode.len(),
                    reencoded_len: reencoded.len(),
                });
                continue;
            }

            results.push(FailType::Ok);
        }

        let success_count = results.iter().filter(|r| matches!(r, FailType::Ok)).count();

        if success_count == scripts.len() && low_level_ids.is_empty() && jump_next_ids.is_empty() {
            println!(
                "100% ({}/{}) strict source round-trip success; low-level IR scripts: {:?}; jump-next scripts: {:?}",
                success_count,
                scripts.len(),
                low_level_ids,
                jump_next_ids
            );
            return Ok(());
        }

        if !low_level_ids.is_empty() {
            println!(
                "high-level structuring incomplete; low-level IR scripts: {:?}",
                low_level_ids
            );
        }

        if !jump_next_ids.is_empty() {
            println!(
                "high-level structuring incomplete; jump-next scripts: {:?}",
                jump_next_ids
            );
        }

        for i in 0..results.len() {
            let script_id = encoded_scripts[i].0;
            match &results[i] {
                FailType::Ok => continue,

                FailType::DecompileError(err) => {
                    println!("Script {script_id} decompile error: {err}")
                }

                FailType::SourceParseError(err) => {
                    println!("Script {script_id} source parse/compile error: {err}")
                }

                FailType::OutputMismatch {
                    first_diff,
                    original_len,
                    reencoded_len,
                } => {
                    println!(
                        "Script {} output mismatch: first byte {:?}, original length {}, rebuilt length {}",
                        script_id,
                        first_diff,
                        original_len,
                        reencoded_len
                    )
                }
            }
        }

        let ids = |predicate: &dyn Fn(&FailType) -> bool| {
            results
                .iter()
                .enumerate()
                .filter_map(|(index, result)| predicate(result).then_some(encoded_scripts[index].0))
                .collect::<Vec<_>>()
        };

        let decompile_errors = results
            .iter()
            .filter(|r| matches!(r, FailType::DecompileError(_)))
            .count();
        let source_errors = results
            .iter()
            .filter(|r| matches!(r, FailType::SourceParseError(_)))
            .count();
        let mismatches = results
            .iter()
            .filter(|r| matches!(r, FailType::OutputMismatch { .. }))
            .count();

        println!(
            "decompile error IDs: {:?}",
            ids(&|r| matches!(r, FailType::DecompileError(_)))
        );
        println!(
            "source parse/compile error IDs: {:?}",
            ids(&|r| matches!(r, FailType::SourceParseError(_)))
        );
        println!(
            "byte mismatch IDs: {:?}",
            ids(&|r| matches!(r, FailType::OutputMismatch { .. }))
        );

        let success_rate_of_10000 = success_count * 10000 / scripts.len();

        println!(
            "{0}.{1}% ({2}/{3}) strict source round-trip success; decompile errors: {4}, source parse/compile errors: {5}, byte mismatches: {6}",
            success_rate_of_10000 / 100,
            success_rate_of_10000 % 100,
            success_count,
            scripts.len(),
            decompile_errors,
            source_errors,
            mismatches
        );

        Err(TestFailure::DecompileFailure)
    }

    fn recompile_scripts_on_large_stack(
        which_rom: &'static str,
        which_lib: &'static str,
        charmap_path: Option<&'static str>,
    ) -> Result<(), TestFailure> {
        std::thread::Builder::new()
            .name(format!("{which_rom}-round-trip"))
            .stack_size(32 * 1024 * 1024)
            .spawn(move || recompile_scripts(which_rom, which_lib, charmap_path))
            .unwrap()
            .join()
            .unwrap()
    }

    #[test]
    fn script_recompile_fomt() -> Result<(), TestFailure> {
        /* FOMT US : 0x080F89D4 1328 */

        let which_rom = "FOMT_US_GBA";
        let which_lib = "FOMT_US_MARY_LIB";

        // let script_table_addr = 0x080F89D4;
        // let script_table_size = 1328;

        recompile_scripts_on_large_stack(which_rom, which_lib, Some("charmap_jp.txt"))
    }

    #[test]
    fn script_recompile_mfomt() -> Result<(), TestFailure> {
        /* MFOMT US : 0x081014BC 1415 */

        let which_rom = "MFOMT_US_GBA";
        let which_lib = "MFOMT_US_MARY_LIB";
        // let script_table_addr = 0x081014BC;
        // let script_table_size = 1415;

        recompile_scripts_on_large_stack(which_rom, which_lib, Some("charmap_jp.txt"))
    }

    #[test]
    fn script_recompile_fomt_jp() -> Result<(), TestFailure> {
        recompile_scripts_on_large_stack("FOMT_JP_GBA", "FOMT_JP_MARY_LIB", Some("charmap_jp.txt"))
    }

    #[test]
    fn script_recompile_mfomt_jp() -> Result<(), TestFailure> {
        recompile_scripts_on_large_stack(
            "MFOMT_JP_GBA",
            "MFOMT_JP_MARY_LIB",
            Some("charmap_jp.txt"),
        )
    }
}
