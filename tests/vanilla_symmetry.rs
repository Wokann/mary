#[cfg(feature = "test_with_roms")]
mod common;

#[cfg(feature = "test_with_roms")]
mod tests {
    use std::{
        fs, io,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use super::common;
    use mary::compiler::{self, ScriptError};
    use mary::utility::rom_info::FomtVariant;
    use thiserror::Error;

    const FOMT_US_ROM: &str = "rom/fomt.gba";
    const MFOMT_US_ROM: &str = "rom/mfomt.gba";
    const FOMT_JP_ROM: &str = "rom/fomtjp.gba";
    const MFOMT_JP_ROM: &str = "rom/mfomtjp.gba";
    const FOMT_LIBRARY: &str = "goodies/lib_fomt.txt";
    const MFOMT_LIBRARY: &str = "goodies/lib_mfomt.txt";

    fn local_rom_path(path: &str) -> PathBuf {
        let repository_path = Path::new(path);
        if repository_path.is_file() {
            repository_path.to_owned()
        } else {
            Path::new("..").join(path)
        }
    }

    #[derive(Clone, Copy)]
    struct RomCase {
        name: &'static str,
        rom_path: &'static str,
        library_path: &'static str,
        variant: FomtVariant,
        slots: usize,
    }

    const FOMT_US: RomCase = RomCase {
        name: "fomt-us",
        rom_path: FOMT_US_ROM,
        library_path: FOMT_LIBRARY,
        variant: FomtVariant::FomtUs,
        slots: 1329,
    };
    const MFOMT_US: RomCase = RomCase {
        name: "mfomt-us",
        rom_path: MFOMT_US_ROM,
        library_path: MFOMT_LIBRARY,
        variant: FomtVariant::MfomtUs,
        slots: 1416,
    };
    const FOMT_JP: RomCase = RomCase {
        name: "fomt-jp",
        rom_path: FOMT_JP_ROM,
        library_path: FOMT_LIBRARY,
        variant: FomtVariant::FomtJp,
        slots: 1329,
    };
    const MFOMT_JP: RomCase = RomCase {
        name: "mfomt-jp",
        rom_path: MFOMT_JP_ROM,
        library_path: MFOMT_LIBRARY,
        variant: FomtVariant::MfomtJp,
        slots: 1416,
    };

    #[derive(Debug, Error)]
    enum TestFailure {
        #[error("IO Error")]
        Io(#[from] io::Error),

        #[error("byte round trip failed on script {script_id}; dumps: {dump_dir}")]
        ScriptFailed { script_id: usize, dump_dir: PathBuf },

        #[error("Lib script parse failed: {0}")]
        LibraryScript(ScriptError),

        #[error("script {script_id} could not be decoded: {source}")]
        Decode {
            script_id: usize,
            source: mary::bytecode::DecodeError,
        },

        #[error("test charmap could not be parsed: {0}")]
        Charmap(#[from] mary::charmap::CharmapError),

        #[error("Not all scripts could be decompiled or recompiled")]
        DecompileFailure,

        #[error("round-trip worker thread panicked")]
        WorkerPanicked,
    }

    fn validate_table(case: RomCase, rom: &[u8]) -> Result<(), TestFailure> {
        use mary::utility::rom_info::{get_script_table, identify_rom, ScriptTableEntry};

        let table = get_script_table(rom)?;
        assert_eq!(
            identify_rom(rom),
            Some(case.variant),
            "{} variant",
            case.name
        );
        assert_eq!(table.len(), case.slots, "{} slot count", case.name);
        assert!(matches!(
            table.first(),
            Some(ScriptTableEntry::Empty { id: 0 })
        ));
        assert!(table
            .windows(2)
            .all(|pair| pair[1].id() == pair[0].id() + 1));
        Ok(())
    }

    fn reencode_scripts(case: RomCase) -> Result<(), TestFailure> {
        use mary::{bytecode, utility::rom_info};

        let rom = fs::read(local_rom_path(case.rom_path))?;
        validate_table(case, &rom)?;
        let table = rom_info::get_script_table(&rom)?;
        let encoded_scripts: Vec<(usize, &[u8])> = table
            .iter()
            .filter_map(|entry| match entry {
                rom_info::ScriptTableEntry::Empty { .. } => None,
                rom_info::ScriptTableEntry::Script { id, data, .. } => Some((*id, *data)),
            })
            .collect();

        for (script_id, to_decode) in encoded_scripts {
            let script = bytecode::decode_script(&mut &to_decode[..])
                .map_err(|source| TestFailure::Decode { script_id, source })?;
            let reencoded = bytecode::encode_script(&script);

            if to_decode != reencoded {
                let dump_dir = PathBuf::from("test_failures").join(case.name);
                fs::create_dir_all(&dump_dir)?;
                fs::write(
                    dump_dir.join(format!("script_{script_id}_original.dmp")),
                    to_decode,
                )?;
                fs::write(
                    dump_dir.join(format!("script_{script_id}_rebuilt.dmp")),
                    reencoded,
                )?;

                return Err(TestFailure::ScriptFailed {
                    script_id,
                    dump_dir,
                });
            }
        }

        Ok(())
    }

    fn render_decompiled_source(
        library: &str,
        script_id: usize,
        stmts: &[mary::ast::Stmt],
        charmap: &mary::charmap::Charmap,
    ) -> String {
        use mary::pretty_print::PrettyStmts;

        let body = format!("{}", PrettyStmts::with_charmap(stmts, 1, charmap));
        format!(
            "{library}\n\nscript {script_id} EventScript_{script_id}\n{{\n{}\n}}\n",
            body
        )
    }

    fn contains_jump_next(stmts: &[mary::ast::Stmt]) -> bool {
        common::contains_stmt(stmts, &|stmt| matches!(stmt, mary::ast::Stmt::JumpNext))
    }

    fn contains_ir(stmts: &[mary::ast::Stmt]) -> bool {
        common::contains_stmt(stmts, &|stmt| matches!(stmt, mary::ast::Stmt::Ir(_)))
    }

    fn recompile_scripts(case: RomCase) -> Result<(), TestFailure> {
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

        let rom = fs::read(local_rom_path(case.rom_path))?;
        validate_table(case, &rom)?;
        let encoded_scripts: Vec<(usize, &[u8])> = get_script_table(&rom)?
            .into_iter()
            .filter_map(|entry| match entry {
                ScriptTableEntry::Empty { .. } => None,
                ScriptTableEntry::Script { id, data, .. } => Some((id, data)),
            })
            .collect();

        let lib_text = fs::read_to_string(case.library_path)?;
        let charmap_source = fs::read_to_string("charmap.txt")?;
        let charmap = Arc::new(mary::charmap::Charmap::parse(&charmap_source)?);

        let lib_scope = match compiler::parse_string(&lib_text) {
            Ok(parse_context) => parse_context.const_scope,
            Err(err) => return Err(TestFailure::LibraryScript(err)),
        };

        let scripts = encoded_scripts
            .iter()
            .map(|(script_id, bytes)| {
                decode_script(&mut &bytes[..]).map_err(|source| TestFailure::Decode {
                    script_id: *script_id,
                    source,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut results: Vec<FailType> = vec![];
        let mut low_level_ids = vec![];
        let mut jump_next_ids = vec![];

        for ((script_id, to_decode), decoded) in encoded_scripts.iter().zip(&scripts) {
            let script_id = *script_id;

            let decompiled = match decompile_script(decoded, &lib_scope) {
                Ok(decompiled) => decompiled,

                Err(err) => {
                    results.push(FailType::DecompileError(format!("{err}")));
                    continue;
                }
            };

            if contains_ir(&decompiled) {
                low_level_ids.push(script_id);
                if let Err(err) = mary::decompiler::decompile_script_structured(decoded, &lib_scope)
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
            let source = render_decompiled_source(&lib_text, script_id, &decompiled, &charmap);
            let parsed_source = compiler::parse_string_with_charmap(&source, Arc::clone(&charmap));
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

            if *to_decode != reencoded {
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

        for ((script_id, _), result) in encoded_scripts.iter().zip(&results) {
            let script_id = *script_id;
            match result {
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
            encoded_scripts
                .iter()
                .zip(&results)
                .filter_map(|((script_id, _), result)| predicate(result).then_some(*script_id))
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

    fn recompile_scripts_on_large_stack(case: RomCase) -> Result<(), TestFailure> {
        std::thread::Builder::new()
            .name(format!("{}-round-trip", case.name))
            .stack_size(32 * 1024 * 1024)
            .spawn(move || recompile_scripts(case))?
            .join()
            .map_err(|_| TestFailure::WorkerPanicked)?
    }

    macro_rules! rom_round_trip_tests {
        ($(($binary:ident, $source:ident, $case:expr)),+ $(,)?) => {
            $(
                #[test]
                fn $binary() -> Result<(), TestFailure> {
                    reencode_scripts($case)
                }

                #[test]
                fn $source() -> Result<(), TestFailure> {
                    recompile_scripts_on_large_stack($case)
                }
            )+
        };
    }

    rom_round_trip_tests!(
        (script_reencode_fomt, script_recompile_fomt, FOMT_US),
        (script_reencode_mfomt, script_recompile_mfomt, MFOMT_US),
        (script_reencode_fomt_jp, script_recompile_fomt_jp, FOMT_JP),
        (
            script_reencode_mfomt_jp,
            script_recompile_mfomt_jp,
            MFOMT_JP
        ),
    );
}
