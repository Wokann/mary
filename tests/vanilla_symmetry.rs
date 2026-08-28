#[cfg(feature = "test_with_roms")]
mod tests {
    use std::{env, fs, io};

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
        let encoded_scripts = rom_info::get_all_scripts(&rom)?;

        for i in 0..encoded_scripts.len() {
            let to_decode = encoded_scripts[i];
            let script = bytecode::decode_script(&mut &to_decode[..]).unwrap();
            let reencoded = bytecode::encode_script(&script);

            if !(to_decode == reencoded) {
                fs::write(format!("test_failed_base_{0}.dmp", i + 1), to_decode)?;
                fs::write(format!("test_failed_reen_{0}.dmp", i + 1), reencoded)?;

                return Err(TestFailure::ScriptFailed(i + 1));
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

    fn render_decompiled_source(
        library: &str,
        script_id: usize,
        stmts: &[mary::ast::Stmt],
    ) -> String {
        use mary::pretty_print::PrettyStmts;

        format!(
            "{library}\n\nscript {script_id} EventScript_{script_id}\n{{\n{}\n}}\n",
            PrettyStmts::with_indent(stmts, 1)
        )
    }

    fn recompile_scripts(which_rom: &str, which_lib: &str) -> Result<(), TestFailure> {
        use mary::{
            bytecode::{decode_script, encode_script},
            decompiler::decompile_script,
            utility::rom_info::get_all_scripts,
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
        let encoded_scripts = get_all_scripts(&rom)?;

        let lib_text = fs::read_to_string(env::var(which_lib)?)?;

        let lib_scope = match compiler::parse_string(&lib_text) {
            Ok(parse_context) => parse_context.const_scope,
            Err(err) => return Err(TestFailure::LibScriptError(err)),
        };

        let mut scripts = vec![];

        for i in 0..encoded_scripts.len() {
            let to_decode = encoded_scripts[i];
            let decoded = decode_script(&mut &to_decode[..]).unwrap();
            scripts.push(decoded);
        }

        let mut results: Vec<FailType> = vec![];
        let mut low_level_ids = vec![];

        for i in 0..encoded_scripts.len() {
            let to_decode = encoded_scripts[i];
            let decoded = &scripts[i];

            let decompiled = match decompile_script(&decoded, &lib_scope) {
                Ok(decompiled) => decompiled,

                Err(err) => {
                    results.push(FailType::DecompileError(format!("{err}")));
                    continue;
                }
            };

            if matches!(&decompiled[..], [mary::ast::Stmt::Ir(_)]) {
                low_level_ids.push(i + 1);
                if let Err(err) =
                    mary::decompiler::decompile_script_structured(&scripts[i], &lib_scope)
                {
                    println!("structured failure {}: {err}", i + 1);
                }
            }

            // A decompilation is only successful if its printed source can be
            // parsed and compiled again. Compiling the in-memory AST directly
            // would hide pretty-printer/parser information loss.
            let source = render_decompiled_source(&lib_text, i + 1, &decompiled);
            let recompiled = match compiler::parse_string(&source) {
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

        if success_count == scripts.len() {
            println!(
                "100% ({}/{}) strict source round-trip success; low-level IR scripts: {:?}",
                success_count,
                scripts.len(),
                low_level_ids
            );
            return Ok(());
        }

        for i in 0..results.len() {
            match &results[i] {
                FailType::Ok => continue,

                FailType::DecompileError(err) => {
                    println!("Script {0} decompile error: {err}", i + 1)
                }

                FailType::SourceParseError(err) => {
                    println!("Script {0} source parse/compile error: {err}", i + 1)
                }

                FailType::OutputMismatch {
                    first_diff,
                    original_len,
                    reencoded_len,
                } => {
                    println!(
                        "Script {} output mismatch: first byte {:?}, original length {}, rebuilt length {}",
                        i + 1,
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
                .filter_map(|(index, result)| predicate(result).then_some(index + 1))
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

    #[test]
    fn script_recompile_fomt() -> Result<(), TestFailure> {
        /* FOMT US : 0x080F89D4 1328 */

        let which_rom = "FOMT_US_GBA";
        let which_lib = "FOMT_US_MARY_LIB";

        // let script_table_addr = 0x080F89D4;
        // let script_table_size = 1328;

        recompile_scripts(which_rom, which_lib)
    }

    #[test]
    fn script_recompile_mfomt() -> Result<(), TestFailure> {
        /* MFOMT US : 0x081014BC 1415 */

        let which_rom = "MFOMT_US_GBA";
        let which_lib = "MFOMT_US_MARY_LIB";
        // let script_table_addr = 0x081014BC;
        // let script_table_size = 1415;

        recompile_scripts(which_rom, which_lib)
    }
}
