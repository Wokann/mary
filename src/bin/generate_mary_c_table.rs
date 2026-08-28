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

fn slot(out: &mut dyn Write, entry: &Entry) -> io::Result<()> {
    match &entry.name {
        Some(name) => writeln!(out, "    {name},"),
        None => writeln!(out, "    NULL,"),
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
        return Err("usage: generate_mary_c_table LEGACY_FOMT LEGACY_MFOMT OUTPUT".into());
    }
    let boy = entries(Path::new(&args[1]))?;
    let girl = entries(Path::new(&args[2]))?;

    let mut dp = vec![vec![0usize; girl.len() + 1]; boy.len() + 1];
    for i in (0..boy.len()).rev() {
        for j in (0..girl.len()).rev() {
            dp[i][j] = if boy[i].name.is_some() && boy[i] == girl[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut pairs = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < boy.len() && j < girl.len() {
        if boy[i].name.is_some() && boy[i] == girl[j] {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs.push((boy.len(), girl.len()));

    let mut out = Vec::new();
    writeln!(
        out,
        "/* Shared ordered callable IDs. Select one MARY_* target in the .mary.c source. */"
    )?;
    writeln!(
        out,
        "#if !(defined(MARY_FOMT_US) || defined(MARY_MFOMT_US) || defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))\n#error Select exactly one MARY_* target\n#endif"
    )?;
    writeln!(
        out,
        "#if (defined(MARY_FOMT_US) && (defined(MARY_MFOMT_US) || defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))) || (defined(MARY_MFOMT_US) && (defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))) || (defined(MARY_FOMT_JP) && defined(MARY_MFOMT_JP))\n#error Select exactly one MARY_* target\n#endif"
    )?;
    writeln!(
        out,
        "#if defined(MARY_FOMT_US)\n#define MARY_FOMT\n#define MARY_US\n#elif defined(MARY_MFOMT_US)\n#define MARY_MFOMT\n#define MARY_US\n#elif defined(MARY_FOMT_JP)\n#define MARY_FOMT\n#define MARY_JP\n#elif defined(MARY_MFOMT_JP)\n#define MARY_MFOMT\n#define MARY_JP\n#endif"
    )?;
    writeln!(out, "mary_callable_table\n{{")?;
    let (mut bi, mut gi) = (0, 0);
    for (bc, gc) in pairs {
        if bi < bc || gi < gc {
            if boy[bi..bc] == girl[gi..gc] {
                for entry in &boy[bi..bc] {
                    slot(&mut out, entry)?;
                }
            } else if bi == bc {
                writeln!(out, "#if defined(MARY_MFOMT)")?;
                for entry in &girl[gi..gc] {
                    slot(&mut out, entry)?;
                }
                writeln!(out, "#endif")?;
            } else if gi == gc {
                writeln!(out, "#if defined(MARY_FOMT)")?;
                for entry in &boy[bi..bc] {
                    slot(&mut out, entry)?;
                }
                writeln!(out, "#endif")?;
            } else {
                writeln!(out, "#if defined(MARY_FOMT)")?;
                for entry in &boy[bi..bc] {
                    slot(&mut out, entry)?;
                }
                writeln!(out, "#elif defined(MARY_MFOMT)")?;
                for entry in &girl[gi..gc] {
                    slot(&mut out, entry)?;
                }
                writeln!(out, "#endif")?;
            }
        }
        if bc < boy.len() {
            slot(&mut out, &boy[bc])?;
            bi = bc + 1;
            gi = gc + 1;
        }
    }
    writeln!(out, "}};\n")?;

    let mut prototypes = BTreeMap::new();
    for entry in boy.iter().chain(&girl) {
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
