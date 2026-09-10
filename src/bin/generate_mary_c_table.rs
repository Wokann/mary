use mary::{compiler::parse_string, const_scope::ConstScope, ir::ValueType};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::Path,
};

#[derive(Clone, PartialEq)]
struct Entry {
    name: Option<String>,
    returns: bool,
    params: Vec<ValueType>,
}

fn entries(path: &Path) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
    let parsed = parse_string(&fs::read_to_string(path)?)?;
    Ok(scope_entries(&parsed.const_scope))
}

fn scope_entries(scope: &ConstScope) -> Vec<Entry> {
    let mut declared: Vec<_> = scope
        .callable_map()
        .iter()
        .map(|(n, (id, s))| (id.0, n, s))
        .collect();
    declared.sort_by_key(|entry| entry.0);
    let max = declared.last().map_or(0, |entry| entry.0);
    let mut at = 0;
    let mut out = Vec::new();
    for id in 0..=max {
        if at < declared.len() && declared[at].0 == id {
            let (_, name, shape) = declared[at];
            out.push(Entry {
                name: Some(name.clone()),
                returns: shape.is_func(),
                params: shape.parameter_types().to_vec(),
            });
            at += 1;
        } else {
            out.push(Entry {
                name: None,
                returns: false,
                params: vec![],
            });
        }
    }
    out
}

fn slot(out: &mut dyn Write, id: usize, entry: &Entry) -> io::Result<()> {
    match &entry.name {
        Some(name) => writeln!(out, "    /* 0x{id:03X} */ {name},"),
        None => writeln!(out, "    /* 0x{id:03X} */ NULL,"),
    }
}

fn prototype(out: &mut dyn Write, entry: &Entry) -> io::Result<()> {
    let Some(name) = &entry.name else {
        return Ok(());
    };
    write!(
        out,
        "{} {name}(",
        if entry.returns { "int" } else { "void" }
    )?;
    if entry.params.is_empty() {
        write!(out, "void")?;
    } else {
        for (index, ty) in entry.params.iter().enumerate() {
            if index != 0 {
                write!(out, ", ")?;
            }
            match ty {
                ValueType::String => write!(out, "const char *arg_{}", index + 1)?,
                _ => write!(out, "int arg_{}", index + 1)?,
            }
        }
    }
    writeln!(out, ");")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 4 {
        return Err("usage: generate_mary_c_table LEGACY_LIBRARY fomt|mfomt OUTPUT".into());
    }
    let family = args[2].to_string_lossy();
    let family_upper = match family.as_ref() {
        "fomt" => "FOMT",
        "mfomt" => "MFOMT",
        _ => return Err("family must be fomt or mfomt".into()),
    };
    let family_entries = entries(Path::new(&args[1]))?;

    let mut out = Vec::new();
    writeln!(
        out,
        "/* {family_upper} ordered callable IDs. Region differences use REGION_US/REGION_JP. */"
    )?;
    writeln!(out, "mary_callable_table\n{{")?;
    for (id, entry) in family_entries.iter().enumerate() {
        slot(&mut out, id, entry)?;
    }
    writeln!(out, "}};\n")?;

    let mut prototypes = BTreeMap::new();
    for entry in &family_entries {
        if let Some(name) = &entry.name {
            prototypes
                .entry(name.clone())
                .or_insert_with(|| entry.clone());
        }
    }
    for entry in prototypes.values() {
        prototype(&mut out, entry)?;
    }
    fs::write(&args[3], out)?;
    Ok(())
}
