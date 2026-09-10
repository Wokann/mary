use std::{
    fs::{self, File},
    io::{self, stdin, stdout, BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

use clap::{Parser, Subcommand};
use thiserror::Error;

use mary::{
    ast::Stmt,
    bytecode::{self, DecodeError, EncodeError},
    charmap::{Charmap, CharmapError},
    compiler::ScriptError,
    decompiler::DecompileError,
    ir::{IntValue, Script},
    pretty_print::{PrettyStmts, PrettyStringLit},
};

#[derive(Debug, Error)]
enum Error {
    #[error("Script Error: {0}")]
    ScriptError(#[from] ScriptError),

    #[error("Lib script error: {0}")]
    LibraryScript(ScriptError),

    #[error("Mary-C callable table error: {0}")]
    MaryTable(#[from] mary::mary_c::MaryCError),

    #[error("Mary-C constant header error: {0}")]
    MaryConstants(#[from] mary::mary_c::ConstantHeaderError),

    #[error("Mary-C script table error: {0}")]
    MaryScriptTable(#[from] mary::mary_c::ScriptTableError),

    #[error("Mary-C text-name table error: {0}")]
    MaryTextNames(#[from] mary::mary_c::TextNameTableError),

    #[error("Mary-C symbol mismatch: {0}")]
    MarySymbolMismatch(String),

    #[error("Mary-C ROM target mismatch: selected {selected}, detected {detected:?}")]
    MaryTargetMismatch {
        selected: String,
        detected: mary::utility::rom_info::FomtVariant,
    },

    #[error("Mary-C source error: {0}")]
    MaryScript(#[from] mary::mary_c::MaryScriptError),

    #[error("Mary-C output error: {0}")]
    MaryOutput(#[from] mary::mary_c::PrettyCError),

    #[error("Decode error: {0}")]
    DecodeFailed(#[from] DecodeError),

    #[error("Encode error: {0}")]
    EncodeFailed(#[from] EncodeError),

    #[error("Decompile error: {0}")]
    DecompileFailed(#[from] DecompileError),

    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("CLI Error: {0}")]
    Cli(&'static str),

    #[error("Character map error: {0}")]
    CharmapError(#[from] CharmapError),
}

fn parse_cli_offset(value: &str) -> Result<usize, String> {
    let (digits, radix) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map_or((value, 10), |digits| (digits, 16));
    if digits.is_empty() {
        return Err("offset must contain at least one digit".into());
    }
    usize::from_str_radix(digits, radix)
        .map_err(|_| format!("'{value}' is not a valid decimal or hexadecimal offset"))
}

#[derive(Subcommand)]
enum Command {
    Compile {
        /// Input script to compile (default: stdin)
        input: Option<PathBuf>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output as binary rather than C (limited to one script)
        #[arg(long, conflicts_with = "print_ir")]
        binary: bool,

        /// Print compiled IR as comment in output
        #[arg(long)]
        print_ir: bool,

        /// Byte-to-Unicode map used for localized string literals
        #[arg(long)]
        charmap: Option<PathBuf>,

        /// Parse canonical .mary.c source
        #[arg(long)]
        mary_c: bool,

        /// Ordered callable declarations (.mary.h)
        #[arg(long, requires = "mary_c")]
        library: Option<PathBuf>,

        /// Ordered script slot table (.mary.h)
        #[arg(long, requires = "mary_c")]
        script_table: Option<PathBuf>,

        /// Select a target macro (for example MARY_FOMT_JP)
        #[arg(short = 'D', requires = "mary_c")]
        defines: Vec<String>,
    },

    Decompile {
        /// Input binary
        input_binary: PathBuf,

        /// Input library script
        input_library: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Script ID (do not use with --offset!)
        #[arg(long)]
        script_id: Option<usize>,

        /// Script offset in binary (do not use with --script-id!)
        #[arg(long, value_parser = parse_cli_offset)]
        offset: Option<usize>,

        /// Print decoded IR as comment in output
        #[arg(long)]
        print_ir: bool,

        /// Byte-to-Unicode map used for localized string literals
        #[arg(long)]
        charmap: Option<PathBuf>,

        /// Decompile every pointer-table slot into the output directory
        #[arg(long)]
        all: bool,

        /// Print canonical .mary.c source
        #[arg(long)]
        mary_c: bool,

        /// Ordered script slot table used for semantic names
        #[arg(long, requires = "mary_c")]
        script_table: Option<PathBuf>,

        /// Optional decompiler-only script/text symbol database (.mary.sym)
        #[arg(long = "symbols", visible_alias = "text-names", requires = "mary_c")]
        text_names: Option<PathBuf>,

        /// Select a target macro (for example MARY_FOMT_JP)
        #[arg(short = 'D', requires = "mary_c")]
        defines: Vec<String>,
    },
}

fn load_charmap(
    path: Option<PathBuf>,
    options: Option<&mary::mary_c::Options>,
) -> Result<Option<Arc<Charmap>>, Error> {
    path.map(|path| {
        let source = fs::read_to_string(path)?;
        let map = match options {
            Some(options) => {
                let active_source = mary::charmap::preprocess_conditionals(&source, options)
                    .map_err(|error| {
                        Error::MaryTable(mary::mary_c::MaryCError::Preprocess(error))
                    })?;
                Charmap::parse(&active_source)?
            }
            None => Charmap::parse(&source)?,
        };
        Ok(Arc::new(map))
    })
    .transpose()
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn print_compiled_scripts_in_c<W: io::Write>(
    w: &mut W,
    scripts: Vec<(IntValue, String, Script)>,
    pretty_bytecode: bool,
) -> Result<(), Error> {
    use mary::bytecode::try_encode_script;

    for (_, name, script) in &scripts {
        let bytecode = try_encode_script(script)?;

        // print bytecode as C

        if pretty_bytecode {
            write_ir_as_comment(w, script)?;
            writeln!(w)?;
        }

        let bytes = bytecode.len();

        writeln!(w, "// {name}: length of {bytes} bytes (+ padding)")?;
        write!(w, "unsigned int const {name}[] = {{")?;

        for i in 0..bytecode.len().div_ceil(4) {
            let val = u32::from_le_bytes([
                bytecode[i * 4],
                *bytecode.get(i * 4 + 1).unwrap_or(&0),
                *bytecode.get(i * 4 + 2).unwrap_or(&0),
                *bytecode.get(i * 4 + 3).unwrap_or(&0),
            ]);

            if i % 8 == 0 {
                write!(w, "\n   ")?;
            }

            write!(w, " 0x{val:08X},")?;
        }

        writeln!(w, "\n}};")?;
    }

    Ok(())
}

fn write_ir_as_comment<W: io::Write>(w: &mut W, script: &Script) -> Result<(), io::Error> {
    use mary::ir::Ins;

    writeln!(w, "/*")?;

    writeln!(w, "Script instructions (IR):")?;

    for instruction in &script.instructions {
        match instruction {
            Ins::Label(_) | Ins::Case(_, _) => {
                writeln!(w, "  {0:?}", instruction)?;
            }

            _ => {
                writeln!(w, "    {0:?}", instruction)?;
            }
        }
    }

    writeln!(w)?;
    writeln!(w, "Script strings (IR):")?;

    for i in 0..script.strings.len() {
        let pretty_string_lit = PrettyStringLit::new(&script.strings[i]);
        writeln!(w, "    {i}: {pretty_string_lit}")?;
    }

    writeln!(w, "*/")
}

fn main_error() -> Result<(), Error> {
    let args = Args::parse();

    match args.command {
        Command::Compile {
            input,
            output,
            print_ir,
            binary,
            charmap,
            mary_c,
            library,
            script_table,
            defines,
        } => {
            use mary::compiler::parse_string;

            let input_base = input
                .as_ref()
                .and_then(|path| path.parent())
                .map(PathBuf::from);
            let mut code = match input {
                Some(path) => fs::read_to_string(path)?,
                None => io::read_to_string(stdin().lock())?,
            };

            let mary_options = mary_c
                .then(|| mary_c_options_from_source(defines.clone(), &code))
                .transpose()?;
            let charmap = load_charmap(charmap, mary_options.as_ref())?;
            let scripts = if mary_c {
                let options = mary_options.expect("Mary-C options prepared above");
                let includes = extract_mary_c_includes(&mut code)?;
                let resolve = |path: PathBuf| {
                    if path.is_absolute() {
                        path
                    } else {
                        input_base.clone().unwrap_or_default().join(path)
                    }
                };
                let mut included_library = None;
                let mut included_scripts = None;
                let mut included_constants = None;
                for include in includes {
                    let path = resolve(include);
                    let source = fs::read_to_string(&path)?;
                    if source.contains("mary_callable_table") {
                        included_library = Some(path);
                    } else if source.contains("mary_script_table") {
                        included_scripts = Some(path);
                    } else if source.contains("typedef int ") && source.contains("#define") {
                        included_constants = Some(path);
                    } else {
                        return Err(Error::Cli(
                            "Mary-C include is neither a callable table, script table, nor constant header",
                        ));
                    }
                }
                let library = library
                    .map(resolve)
                    .or(included_library)
                    .ok_or(Error::Cli("Mary-C requires a callable-table .mary.h"))?;
                let script_table = script_table
                    .map(resolve)
                    .or(included_scripts)
                    .ok_or(Error::Cli("Mary-C requires a script-table .mary.h"))?;
                let callable_source = fs::read_to_string(library)?;
                let table_source = fs::read_to_string(script_table)?;
                let constant_scope = included_constants
                    .map(fs::read_to_string)
                    .transpose()?
                    .map(|source| mary::mary_c::parse_constant_header(&source, &options))
                    .transpose()?
                    .unwrap_or_default();
                let callables = mary::mary_c::parse_callable_table_with_scope(
                    &callable_source,
                    &options,
                    &constant_scope,
                )?;
                let scripts = mary::mary_c::parse_script_table(&table_source, &options)?;
                match charmap.as_deref() {
                    Some(charmap) => {
                        mary::mary_c::parse_named_scripts_with_charmap(
                            &code,
                            &options,
                            &callables.scope,
                            &scripts,
                            charmap,
                        )?
                        .scripts
                    }
                    None => {
                        mary::mary_c::parse_named_scripts(
                            &code,
                            &options,
                            &callables.scope,
                            &scripts,
                        )?
                        .scripts
                    }
                }
            } else {
                let parse_result = match charmap {
                    Some(charmap) => mary::compiler::parse_string_with_charmap(&code, charmap)?,
                    None => parse_string(&code)?,
                };
                parse_result.scripts
            };

            if binary {
                if scripts.len() != 1 {
                    Err(Error::Cli(
                        "In binary output mode, only and exactly one (1) script can be defined",
                    ))
                } else {
                    let script = &scripts[0].2;
                    let bytecode = bytecode::try_encode_script(script)?;

                    match output {
                        Some(out_path) => {
                            fs::write(out_path, bytecode)?;
                        }

                        None => {
                            stdout().write_all(&bytecode)?;
                        }
                    }

                    Ok(())
                }
            } else {
                if let Some(path) = output {
                    let output = File::create(path)?;
                    let mut buf_write = BufWriter::new(output);
                    print_compiled_scripts_in_c(&mut buf_write, scripts, print_ir)?;
                    buf_write.flush()?;
                } else {
                    print_compiled_scripts_in_c(&mut stdout().lock(), scripts, print_ir)?;
                }

                Ok(())
            }
        }

        Command::Decompile {
            input_binary,
            input_library,
            output,
            script_id,
            offset,
            print_ir,
            charmap,
            all,
            mary_c,
            script_table,
            text_names,
            defines,
        } => {
            use mary::{compiler::parse_string, decompiler::decompile_script, utility::rom_info};

            let rom = fs::read(input_binary)?;
            let mary_options = mary_c_options(defines)?;
            let charmap = load_charmap(charmap, mary_c.then_some(&mary_options))?;
            let mary_target = selected_mary_target(&mary_options);
            if mary_c && mary_target.is_none() {
                return Err(Error::Cli(
                    "Mary-C decompile requires exactly one MARY_* target",
                ));
            }
            if mary_c && (all || script_id.is_some()) {
                let detected = rom_info::identify_rom(&rom).ok_or(Error::Cli(
                    "ROM header does not match a supported Mary-C target",
                ))?;
                let selected = mary_target.expect("Mary-C target checked above");
                if mary_target_variant(selected) != Some(detected) {
                    return Err(Error::MaryTargetMismatch {
                        selected: selected.to_owned(),
                        detected,
                    });
                }
            }
            let mary_headers = mary_target.map(mary_header_names);
            let mut script_table_include = script_table
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned());
            let input_scripts = script_table
                .map(fs::read_to_string)
                .transpose()?
                .map(|source| mary::mary_c::parse_script_table(&source, &mary_options))
                .transpose()?;
            let text_names = text_names
                .map(fs::read_to_string)
                .transpose()?
                .map(|source| mary::mary_c::parse_text_name_table(&source, &mary_options))
                .transpose()?;
            let symbol_scripts = text_names
                .as_ref()
                .map(mary::mary_c::ScriptSymbols::script_table)
                .transpose()?;
            let named_scripts = symbol_scripts.as_ref().or(input_scripts.as_ref());
            let constants_path = input_library
                .parent()
                .zip(mary_headers)
                .map(|(directory, headers)| directory.join(headers.constants))
                .filter(|path| path.is_file());
            let mut local_single_headers = false;
            if mary_c && !all {
                if let Some(output_path) = output.as_ref() {
                    let directory = output_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new("."));
                    fs::create_dir_all(directory)?;
                    let headers = mary_headers.expect("Mary-C target checked above");
                    fs::write(directory.join(headers.callables), fs::read(&input_library)?)?;
                    if let Some(scripts) = named_scripts {
                        fs::write(
                            directory.join(headers.scripts),
                            render_script_table(scripts),
                        )?;
                        script_table_include = Some(headers.scripts.into());
                    }
                    if let Some(constants) = constants_path.as_ref() {
                        fs::write(directory.join(headers.constants), fs::read(constants)?)?;
                    }
                    local_single_headers = true;
                }
            }

            if all {
                if script_id.is_some() || offset.is_some() {
                    return Err(Error::Cli(
                        "--all cannot be combined with --script-id or --offset",
                    ));
                }
                let output_dir = output.ok_or(Error::Cli(
                    "--all requires --output to name an output directory",
                ))?;
                decompile_all_scripts(
                    &rom,
                    &input_library,
                    &output_dir,
                    charmap.as_deref(),
                    print_ir,
                    mary_c,
                    &mary_options,
                    text_names.as_ref(),
                    mary_target,
                    named_scripts,
                )?;
                return Ok(());
            }

            let event_script_id;
            let mut event_script_name;

            let script = match (script_id, offset) {
                (Some(script_id), None) => {
                    let script_table = rom_info::get_script_table(&rom)?;

                    event_script_id = script_id;
                    event_script_name = named_scripts
                        .and_then(|table| table.name(script_id))
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("EventScript_{script_id:04}"));

                    let entry = script_table
                        .iter()
                        .find(|entry| entry.id() == script_id)
                        .ok_or(Error::Cli("Script ID is outside the pointer table"))?;
                    match entry {
                        rom_info::ScriptTableEntry::Script { data, backing, .. } => {
                            bytecode::decode_script_with_backing(data, backing)?
                        }
                        rom_info::ScriptTableEntry::Empty { .. } => {
                            if mary_c {
                                let library_include = if local_single_headers {
                                    std::borrow::Cow::Borrowed(mary_headers.unwrap().callables)
                                } else {
                                    input_library.to_string_lossy()
                                };
                                let constants_include = constants_path.as_ref().map(|path| {
                                    if local_single_headers {
                                        std::borrow::Cow::Borrowed(mary_headers.unwrap().constants)
                                    } else {
                                        path.to_string_lossy()
                                    }
                                });
                                print_empty_single_mary_c_slot(
                                    output,
                                    event_script_id,
                                    mary_target.unwrap_or("MARY_TARGET_REQUIRED"),
                                    &library_include,
                                    script_table_include.as_deref(),
                                    constants_include.as_deref(),
                                )?;
                            } else {
                                print_empty_script_slot(
                                    output,
                                    event_script_id,
                                    &event_script_name,
                                    &input_library.to_string_lossy(),
                                )?;
                            }
                            return Ok(());
                        }
                    }
                }

                (None, Some(offset)) => {
                    let offset = offset & 0x01FFFFFF;

                    event_script_id = 0;
                    event_script_name = format!("EventScript_{0:08X}", offset + 0x08000000);

                    let input = rom
                        .get(offset..)
                        .ok_or(Error::Cli("Offset is outside the input binary"))?;
                    bytecode::decode_script(&mut &input[..])?
                }

                (None, None) => {
                    event_script_id = 0;
                    event_script_name = "YourEventScriptNameHere".to_string();

                    bytecode::decode_script(&mut &rom[..])?
                }

                _ => {
                    return Err(Error::Cli(
                        "Expected at most one (1) of script ID or offset",
                    ))
                }
            };

            let library_code = fs::read_to_string(&input_library)?;

            let mut library_scope = if mary_c {
                let constants = constants_path
                    .as_ref()
                    .map(fs::read_to_string)
                    .transpose()?
                    .map(|source| mary::mary_c::parse_constant_header(&source, &mary_options))
                    .transpose()?
                    .unwrap_or_default();
                mary::mary_c::parse_callable_table_with_scope(
                    &library_code,
                    &mary_options,
                    &constants,
                )?
                .scope
            } else {
                match parse_string(&library_code) {
                    Ok(parse_result) => parse_result.const_scope,
                    Err(err) => return Err(Error::LibraryScript(err)),
                }
            };
            if mary_c {
                if let Some(scripts) = named_scripts {
                    scripts.add_constants(&mut library_scope);
                }
            }

            if let Some(names) = text_names.as_ref() {
                if let Some(name) = names.script_name(event_script_id) {
                    event_script_name = name.to_owned();
                }
            }

            let stmts = match if mary_c {
                let overrides = text_names
                    .as_ref()
                    .map(|table| table.names(event_script_id, script.strings.len()));
                let local_types = text_names
                    .as_ref()
                    .map(|table| table.local_types(event_script_id, &library_scope))
                    .transpose()
                    .map_err(Error::MarySymbolMismatch)?
                    .unwrap_or_default();
                match overrides.as_deref() {
                    Some(names) => mary::decompiler::decompile_script_with_metadata(
                        &script,
                        &library_scope,
                        &event_script_name,
                        names,
                        &local_types,
                    ),
                    None => mary::decompiler::decompile_script_named(
                        &script,
                        &library_scope,
                        &event_script_name,
                    ),
                }
            } else {
                decompile_script(&script, &library_scope)
            } {
                Ok(stmts) => stmts,

                Err(error) => {
                    eprintln!("{0}", error.state_at_error());
                    return Err(Error::DecompileFailed(DecompileError::from(error)));
                }
            };

            let library_path_for_include = if local_single_headers {
                std::borrow::Cow::Borrowed(mary_headers.unwrap().callables)
            } else {
                input_library.to_string_lossy()
            };
            let constants_path_for_include = constants_path.as_ref().map(|path| {
                if local_single_headers {
                    std::borrow::Cow::Borrowed(mary_headers.unwrap().constants)
                } else {
                    path.to_string_lossy()
                }
            });

            match output {
                Some(path) => {
                    // print to file

                    let output = File::create(path)?;
                    let mut buf_write = BufWriter::new(output);

                    print_decompiled_script(
                        &mut buf_write,
                        event_script_id,
                        event_script_name,
                        stmts,
                        &library_path_for_include,
                        print_ir.then_some(&script),
                        charmap.as_deref(),
                        mary_c,
                        mary_target,
                        script_table_include.as_deref(),
                        constants_path_for_include.as_deref(),
                    )?;

                    buf_write.flush()?;
                    Ok(())
                }

                None => {
                    // print to stdout

                    print_decompiled_script(
                        &mut stdout().lock(),
                        event_script_id,
                        event_script_name,
                        stmts,
                        &library_path_for_include,
                        print_ir.then_some(&script),
                        charmap.as_deref(),
                        mary_c,
                        mary_target,
                        script_table_include.as_deref(),
                        constants_path_for_include.as_deref(),
                    )?;

                    Ok(())
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn decompile_all_scripts(
    rom: &[u8],
    input_library: &PathBuf,
    output_dir: &PathBuf,
    charmap: Option<&Charmap>,
    print_ir: bool,
    mary_c: bool,
    mary_options: &mary::mary_c::Options,
    text_names: Option<&mary::mary_c::ScriptSymbols>,
    mary_target: Option<&str>,
    named_scripts: Option<&mary::mary_c::ScriptTable>,
) -> Result<(), Error> {
    use mary::{compiler::parse_string, decompiler::decompile_script, utility::rom_info};

    let library_code = fs::read_to_string(input_library)?;
    let headers = mary_target.map(mary_header_names);
    let constants_path = input_library
        .parent()
        .zip(headers)
        .map(|(directory, headers)| directory.join(headers.constants))
        .filter(|path| path.is_file());
    let mut library_scope = if mary_c {
        let constants = constants_path
            .as_ref()
            .map(fs::read_to_string)
            .transpose()?
            .map(|source| mary::mary_c::parse_constant_header(&source, mary_options))
            .transpose()?
            .unwrap_or_default();
        mary::mary_c::parse_callable_table_with_scope(&library_code, mary_options, &constants)?
            .scope
    } else {
        match parse_string(&library_code) {
            Ok(parse_result) => parse_result.const_scope,
            Err(err) => return Err(Error::LibraryScript(err)),
        }
    };
    if mary_c {
        if let Some(scripts) = named_scripts {
            scripts.add_constants(&mut library_scope);
        }
    }
    fs::create_dir_all(output_dir)?;
    if mary_c {
        let headers = headers.expect("Mary-C target checked by caller");
        fs::write(output_dir.join(headers.callables), &library_code)?;
        if let Some(constants) = constants_path.as_ref() {
            fs::write(output_dir.join(headers.constants), fs::read(constants)?)?;
        }
    }
    let library_path_for_include = if mary_c {
        std::borrow::Cow::Borrowed(headers.unwrap().callables)
    } else {
        input_library.to_string_lossy()
    };

    let entries = rom_info::get_script_table(rom)?;
    if let Some(scripts) = named_scripts {
        if scripts.slots().len() != entries.len() {
            return Err(Error::MarySymbolMismatch(format!(
                "script table has {} slots but ROM has {}",
                scripts.slots().len(),
                entries.len()
            )));
        }
        for (slot, entry) in scripts.slots().iter().zip(&entries) {
            let symbol_present = matches!(slot, mary::mary_c::ScriptSlot::Script(_));
            let rom_present = matches!(entry, rom_info::ScriptTableEntry::Script { .. });
            if symbol_present != rom_present {
                return Err(Error::MarySymbolMismatch(format!(
                    "script slot {} has different NULL state in symbols and ROM",
                    entry.id()
                )));
            }
        }
    }
    if mary_c {
        let mut table = String::from("mary_script_table\n{\n");
        for entry in &entries {
            match entry {
                rom_info::ScriptTableEntry::Empty { id, .. } => {
                    table.push_str(&format!("    /* 0x{id:04X} */ NULL,\n"));
                }
                rom_info::ScriptTableEntry::Script { id, .. } => {
                    let base_name = named_scripts
                        .and_then(|scripts| scripts.name(*id))
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("EventScript_{id:04}"));
                    let name = text_names
                        .and_then(|symbols| symbols.script_name(*id))
                        .unwrap_or(&base_name);
                    table.push_str(&format!("    /* 0x{id:04X} */ {name},\n"));
                }
            }
        }
        table.push_str("};\n");
        fs::write(output_dir.join(headers.unwrap().scripts), table)?;
    }

    for entry in entries {
        let script_id = entry.id();
        let mut script_name = named_scripts
            .and_then(|table| table.name(script_id))
            .map(str::to_owned)
            .unwrap_or_else(|| format!("EventScript_{script_id:04}"));
        if let Some(names) = text_names {
            if let Some(name) = names.script_name(script_id) {
                script_name = name.to_owned();
            }
        }
        let output_path = output_dir.join(if mary_c {
            format!("{script_name}.mary.c")
        } else {
            format!("EventScript_{script_id:04}.mary")
        });

        match entry {
            rom_info::ScriptTableEntry::Empty { .. } if mary_c => {
                print_empty_mary_c_slot(
                    output_path,
                    script_id,
                    mary_target.unwrap_or("MARY_TARGET_REQUIRED"),
                    constants_path.is_some(),
                    headers.unwrap(),
                )?;
            }
            rom_info::ScriptTableEntry::Empty { .. } => print_empty_script_slot(
                Some(output_path),
                script_id,
                &script_name,
                &library_path_for_include,
            )?,
            rom_info::ScriptTableEntry::Script { data, backing, .. } => {
                let script = bytecode::decode_script_with_backing(data, backing)?;
                if let Some(symbols) = text_names {
                    if symbols.text_count(script_id) > script.strings.len() {
                        return Err(Error::MarySymbolMismatch(format!(
                            "script {script_id} defines {} text symbols but RIFF has {} strings",
                            symbols.text_count(script_id),
                            script.strings.len()
                        )));
                    }
                }
                let stmts = match if mary_c {
                    let overrides =
                        text_names.map(|table| table.names(script_id, script.strings.len()));
                    let local_types = text_names
                        .map(|table| table.local_types(script_id, &library_scope))
                        .transpose()
                        .map_err(Error::MarySymbolMismatch)?
                        .unwrap_or_default();
                    match overrides.as_deref() {
                        Some(names) => mary::decompiler::decompile_script_with_metadata(
                            &script,
                            &library_scope,
                            &script_name,
                            names,
                            &local_types,
                        ),
                        None => mary::decompiler::decompile_script_named(
                            &script,
                            &library_scope,
                            &script_name,
                        ),
                    }
                } else {
                    decompile_script(&script, &library_scope)
                } {
                    Ok(stmts) => stmts,
                    Err(error) => {
                        eprintln!("script {script_id}: {}", error.state_at_error());
                        return Err(Error::DecompileFailed(DecompileError::from(error)));
                    }
                };
                let output = File::create(output_path)?;
                let mut writer = BufWriter::new(output);
                print_decompiled_script(
                    &mut writer,
                    script_id,
                    script_name,
                    stmts,
                    &library_path_for_include,
                    print_ir.then_some(&script),
                    charmap,
                    mary_c,
                    mary_target,
                    mary_c.then_some(headers.unwrap().scripts),
                    constants_path.as_ref().map(|_| headers.unwrap().constants),
                )?;
                writer.flush()?;
            }
        }
    }

    Ok(())
}

fn print_empty_mary_c_slot(
    output_path: PathBuf,
    script_id: usize,
    mary_target: &str,
    include_constants: bool,
    headers: MaryHeaderNames,
) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_path)?);
    writeln!(writer, "#define {mary_target}")?;
    if include_constants {
        writeln!(writer, "#include \"{}\"", headers.constants)?;
    }
    writeln!(writer, "#include \"{}\"", headers.callables)?;
    writeln!(writer, "#include \"{}\"", headers.scripts)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "// NULL script pointer-table slot {script_id}; no RIFF body exists."
    )?;
    writeln!(
        writer,
        "// This placeholder intentionally contains no script definition."
    )?;
    writer.flush()
}

fn print_empty_single_mary_c_slot(
    output: Option<PathBuf>,
    script_id: usize,
    mary_target: &str,
    library_include: &str,
    script_table_include: Option<&str>,
    constants_include: Option<&str>,
) -> io::Result<()> {
    fn write_placeholder<W: Write>(
        writer: &mut W,
        script_id: usize,
        mary_target: &str,
        library_include: &str,
        script_table_include: Option<&str>,
        constants_include: Option<&str>,
    ) -> io::Result<()> {
        writeln!(writer, "#define {mary_target}")?;
        if let Some(constants) = constants_include {
            writeln!(writer, "#include \"{constants}\"")?;
        }
        writeln!(writer, "#include \"{library_include}\"")?;
        if let Some(script_table) = script_table_include {
            writeln!(writer, "#include \"{script_table}\"")?;
        }
        writeln!(writer)?;
        writeln!(
            writer,
            "// NULL script pointer-table slot {script_id}; no RIFF body exists."
        )?;
        writeln!(
            writer,
            "// This placeholder intentionally contains no script definition."
        )
    }

    match output {
        Some(path) => {
            let mut writer = BufWriter::new(File::create(path)?);
            write_placeholder(
                &mut writer,
                script_id,
                mary_target,
                library_include,
                script_table_include,
                constants_include,
            )?;
            writer.flush()
        }
        None => write_placeholder(
            &mut stdout().lock(),
            script_id,
            mary_target,
            library_include,
            script_table_include,
            constants_include,
        ),
    }
}

fn print_empty_script_slot(
    output: Option<PathBuf>,
    script_id: usize,
    script_name: &str,
    library_path_for_include: &str,
) -> io::Result<()> {
    fn write_placeholder<W: Write>(
        w: &mut W,
        script_id: usize,
        script_name: &str,
        library_path_for_include: &str,
    ) -> io::Result<()> {
        writeln!(w, "#include \"{library_path_for_include}\"")?;
        writeln!(w)?;
        writeln!(
            w,
            "// Empty script pointer-table slot; no RIFF body exists."
        )?;
        writeln!(w, "// script {script_id} {script_name}")
    }

    match output {
        Some(path) => {
            let mut writer = BufWriter::new(File::create(path)?);
            write_placeholder(
                &mut writer,
                script_id,
                script_name,
                library_path_for_include,
            )?;
            writer.flush()
        }
        None => write_placeholder(
            &mut stdout().lock(),
            script_id,
            script_name,
            library_path_for_include,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn print_decompiled_script<W: io::Write>(
    w: &mut W,
    script_id: usize,
    script_name: String,
    stmts: Vec<Stmt>,
    library_path_for_include: &str,
    print_ir: Option<&Script>,
    charmap: Option<&Charmap>,
    mary_c: bool,
    mary_target: Option<&str>,
    script_table_include: Option<&str>,
    constants_include: Option<&str>,
) -> io::Result<()> {
    if mary_c {
        writeln!(
            w,
            "#define {}",
            mary_target.unwrap_or("MARY_TARGET_REQUIRED")
        )?;
    }
    if let Some(constants) = constants_include {
        writeln!(w, "#include \"{constants}\"")?;
    }
    writeln!(w, "#include \"{0}\"", library_path_for_include)?;
    if let Some(script_table) = script_table_include {
        writeln!(w, "#include \"{script_table}\"")?;
    }
    writeln!(w)?;

    if mary_c {
        let formatted = match charmap {
            Some(charmap) => {
                mary::mary_c::format_named_script_with_charmap(&script_name, &stmts, charmap)
            }
            None => mary::mary_c::format_named_script(&script_name, &stmts),
        };
        let source = formatted
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        write!(w, "{source}")?;
        if let Some(script) = print_ir {
            writeln!(w)?;
            write_ir_as_comment(w, script)?;
        }
        return Ok(());
    }

    writeln!(w, "script {0} {1}", script_id, script_name)?;

    writeln!(w, "{{")?;

    let pretty_stmts = match charmap {
        Some(charmap) => PrettyStmts::with_charmap(&stmts, 1, charmap),
        None => PrettyStmts::with_indent(&stmts, 1),
    };
    writeln!(w, "{pretty_stmts}")?;

    writeln!(w, "}}")?;

    if let Some(script) = print_ir {
        writeln!(w)?;
        write_ir_as_comment(w, script)?;
    }

    Ok(())
}

fn mary_c_options(defines: Vec<String>) -> Result<mary::mary_c::Options, Error> {
    let mut options = mary::mary_c::Options::default();
    for definition in defines {
        options = options
            .define(&definition)
            .map_err(|error| Error::MaryTable(mary::mary_c::MaryCError::Preprocess(error)))?;
    }
    Ok(options)
}

fn mary_c_options_from_source(
    defines: Vec<String>,
    source: &str,
) -> Result<mary::mary_c::Options, Error> {
    let mut options = mary_c_options(defines)?;
    let targets = [
        "MARY_FOMT_JP",
        "MARY_FOMT_US",
        "MARY_FOMT_EU",
        "MARY_FOMT_DE",
        "MARY_MFOMT_JP",
        "MARY_MFOMT_US",
    ];
    for line in source.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("#define") else {
            continue;
        };
        let name = rest.split_whitespace().next().unwrap_or("");
        if targets.contains(&name) {
            options = options
                .define(name)
                .map_err(|error| Error::MaryTable(mary::mary_c::MaryCError::Preprocess(error)))?;
        }
    }
    let selected = targets
        .iter()
        .filter(|name| options.defines.contains_key(**name))
        .count();
    if selected != 1 {
        return Err(Error::Cli(
            "Mary-C source must select exactly one MARY_* ROM target",
        ));
    }
    Ok(options)
}

fn selected_mary_target(options: &mary::mary_c::Options) -> Option<&str> {
    let targets = [
        "MARY_FOMT_JP",
        "MARY_FOMT_US",
        "MARY_FOMT_EU",
        "MARY_FOMT_DE",
        "MARY_MFOMT_JP",
        "MARY_MFOMT_US",
    ];
    let mut selected = targets
        .into_iter()
        .filter(|name| options.defines.contains_key(*name));
    let first = selected.next()?;
    selected.next().is_none().then_some(first)
}

fn mary_target_variant(target: &str) -> Option<mary::utility::rom_info::FomtVariant> {
    use mary::utility::rom_info::FomtVariant;

    match target {
        "MARY_FOMT_JP" => Some(FomtVariant::FomtJp),
        "MARY_FOMT_US" => Some(FomtVariant::FomtUs),
        "MARY_FOMT_EU" => Some(FomtVariant::FomtEu),
        "MARY_FOMT_DE" => Some(FomtVariant::FomtDe),
        "MARY_MFOMT_JP" => Some(FomtVariant::MfomtJp),
        "MARY_MFOMT_US" => Some(FomtVariant::MfomtUs),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct MaryHeaderNames {
    constants: &'static str,
    callables: &'static str,
    scripts: &'static str,
}

fn mary_header_names(target: &str) -> MaryHeaderNames {
    if target.starts_with("MARY_MFOMT_") {
        MaryHeaderNames {
            constants: "mfomt_constants.mary.h",
            callables: "mfomt_callables.mary.h",
            scripts: "mfomt_scripts.mary.h",
        }
    } else {
        MaryHeaderNames {
            constants: "fomt_constants.mary.h",
            callables: "fomt_callables.mary.h",
            scripts: "fomt_scripts.mary.h",
        }
    }
}

fn render_script_table(table: &mary::mary_c::ScriptTable) -> String {
    let mut source = String::from("mary_script_table\n{\n");
    for (id, slot) in table.slots().iter().enumerate() {
        match slot {
            mary::mary_c::ScriptSlot::Empty => {
                source.push_str(&format!("    /* 0x{id:04X} */ NULL,\n"));
            }
            mary::mary_c::ScriptSlot::Script(name) => {
                source.push_str(&format!("    /* 0x{id:04X} */ {name},\n"));
            }
        }
    }
    source.push_str("};\n");
    source
}

fn extract_mary_c_includes(source: &mut String) -> Result<Vec<PathBuf>, Error> {
    let mut includes = Vec::new();
    let mut rebuilt = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("#include") {
            let value = value.trim();
            if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
                return Err(Error::Cli("Mary-C #include must use a quoted path"));
            }
            includes.push(PathBuf::from(&value[1..value.len() - 1]));
            if line.ends_with('\n') {
                rebuilt.push('\n');
            }
        } else {
            rebuilt.push_str(line);
        }
    }
    *source = rebuilt;
    Ok(includes)
}

fn main() -> Result<(), ()> {
    match main_error() {
        Ok(_) => Ok(()),

        Err(err) => {
            eprintln!("{0}", err);
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{mary_target_variant, print_empty_single_mary_c_slot};
    use mary::utility::rom_info::FomtVariant;
    use std::{fs, path::PathBuf};

    #[test]
    fn single_mary_c_null_slot_uses_local_self_contained_headers() {
        let output_dir =
            PathBuf::from("test_failures").join(format!("single_null_slot_{}", std::process::id()));
        fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("EventScript_0000.mary.c");

        print_empty_single_mary_c_slot(
            Some(output.clone()),
            0,
            "MARY_FOMT_US",
            "fomt_callables.mary.h",
            Some("fomt_scripts.mary.h"),
            Some("fomt_constants.mary.h"),
        )
        .unwrap();

        let source = fs::read_to_string(&output).unwrap();
        assert!(source.starts_with("#define MARY_FOMT_US\n"));
        assert!(source.contains("#include \"fomt_constants.mary.h\"\n"));
        assert!(source.contains("#include \"fomt_callables.mary.h\"\n"));
        assert!(source.contains("#include \"fomt_scripts.mary.h\"\n"));
        assert!(source.contains("NULL script pointer-table slot 0"));
        assert!(!source.contains("void EventScript_"));

        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn mary_targets_map_to_rom_variants_in_canonical_order() {
        assert_eq!(
            mary_target_variant("MARY_FOMT_JP"),
            Some(FomtVariant::FomtJp)
        );
        assert_eq!(
            mary_target_variant("MARY_FOMT_US"),
            Some(FomtVariant::FomtUs)
        );
        assert_eq!(
            mary_target_variant("MARY_FOMT_EU"),
            Some(FomtVariant::FomtEu)
        );
        assert_eq!(
            mary_target_variant("MARY_FOMT_DE"),
            Some(FomtVariant::FomtDe)
        );
        assert_eq!(
            mary_target_variant("MARY_MFOMT_JP"),
            Some(FomtVariant::MfomtJp)
        );
        assert_eq!(
            mary_target_variant("MARY_MFOMT_US"),
            Some(FomtVariant::MfomtUs)
        );
        assert_eq!(mary_target_variant("REGION_US"), None);
    }
}
