use std::{fs, path::Path, process::Command};

fn write_fixture(root: &Path) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join("fomt_constants.mary.h"),
        "/* #define marker used by include classification */\ntypedef int MaryScriptId;\ntypedef enum MaryBool\n{\n    FALSE = 0,\n    TRUE = 1,\n} MaryBool;\n",
    )
    .unwrap();
    fs::write(
        root.join("test_callables.mary.h"),
        "mary_callable_table { Talk, };\nvoid Talk(const char *text);\n",
    )
    .unwrap();
    fs::write(
        root.join("test_scripts.mary.h"),
        "mary_script_table { NULL, EventScript_First, EventScript_Second, };\n",
    )
    .unwrap();
    for (name, text) in [("First", "Hello"), ("Second", "World")] {
        let extra = if name == "First" {
            " const char gText_FirstUnused[] = \"Hello\";"
        } else {
            ""
        };
        fs::write(
            root.join(format!("EventScript_{name}.mary.c")),
            format!(
                "#define MARY_FOMT_US\n#include \"fomt_constants.mary.h\"\n#include \"test_callables.mary.h\"\n#include \"test_scripts.mary.h\"\n\nmary_text_table {{ const char gText_{name}[] = \"{text}\";{extra} }};\nvoid EventScript_{name}(void) {{ Talk(gText_{name}); }}\n"
            ),
        )
        .unwrap();
    }
    fs::write(
        root.join("EventScript_Added.mary.c"),
        "#define MARY_FOMT_US\n#include \"fomt_constants.mary.h\"\n#include \"test_callables.mary.h\"\n#include \"test_scripts.mary.h\"\n\nmary_text_table { const char gText_Added[] = \"Same\"; const char gText_AddedUnused[] = \"Same\"; };\nvoid EventScript_Added(void) { Talk(gText_Added); }\n",
    )
    .unwrap();
}

fn bundle_output(root: &Path, output: &Path, layout: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mary"))
        .arg("bundle")
        .arg(root)
        .arg("-o")
        .arg(output)
        .arg("--layout")
        .arg(layout)
        .arg("--library")
        .arg(root.join("test_callables.mary.h"))
        .arg("--script-table")
        .arg(root.join("test_scripts.mary.h"))
        .arg("-D")
        .arg("MARY_FOMT_US")
        .output()
        .unwrap()
}

fn run_bundle(root: &Path, output: &Path, layout: &str) {
    let result = bundle_output(root, output, layout);
    assert!(
        result.status.success(),
        "bundle failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn packed_and_split_bundles_emit_linker_ready_sources() {
    let root = Path::new("test_failures").join(format!("linker_bundle_{}", std::process::id()));
    let source = root.join("source");
    let packed = root.join("packed");
    let split = root.join("split");
    write_fixture(&source);

    run_bundle(&source, &packed, "packed");
    run_bundle(&source, &split, "split");

    let packed_asm = fs::read_to_string(packed.join("scripts.s")).unwrap();
    assert!(packed_asm.contains(".incbin"));
    assert!(packed_asm.contains(".set EventScript_First, gMaryScripts + 0x0"));
    assert!(packed_asm.contains(".set gText_First, EventScript_First + 0x"));
    assert!(packed_asm.contains(".set gText_Second, EventScript_Second + 0x"));
    assert!(packed_asm.contains(".set gText_FirstUnused, EventScript_First + 0x"));
    assert!(!packed_asm.contains("EventScript_Added"));

    let table = fs::read_to_string(packed.join("script_table.s")).unwrap();
    assert!(table.contains(".section .rodata.mary_script_table"));
    assert!(
        table.contains("    .word 0\n    .word EventScript_First\n    .word EventScript_Second\n")
    );
    assert!(!table.contains("EventScript_Added"));
    assert!(!packed.join("scripts.generated.mary.h").exists());

    let dependency = fs::read_to_string(packed.join("scripts.d")).unwrap();
    for input in [
        "EventScript_First.mary.c",
        "EventScript_Second.mary.c",
        "fomt_constants.mary.h",
        "test_callables.mary.h",
        "test_scripts.mary.h",
    ] {
        assert!(dependency.contains(input), "dependency file omits {input}");
    }
    assert!(!dependency.contains("EventScript_Added.mary.c"));

    assert!(packed.join("scripts.bin").is_file());
    assert!(!split.join("scripts.bin").exists());
    let first = fs::read(split.join("riff/EventScript_First.riff")).unwrap();
    let second = fs::read(split.join("riff/EventScript_Second.riff")).unwrap();
    assert!(!split.join("riff/EventScript_Added.riff").exists());
    let mut first_reader = first.as_slice();
    assert_eq!(
        mary::bytecode::decode_script(&mut first_reader)
            .unwrap()
            .strings
            .len(),
        2
    );
    let packed_bytes = fs::read(packed.join("scripts.bin")).unwrap();
    assert_eq!(&packed_bytes[..first.len()], first);
    let second_offset = first.len().div_ceil(4) * 4;
    assert_eq!(
        &packed_bytes[second_offset..second_offset + second.len()],
        second
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn directory_bundle_reports_every_registered_script_without_a_source() {
    let root = Path::new("test_failures").join(format!("linker_missing_{}", std::process::id()));
    let source = root.join("source");
    let output = root.join("output");
    write_fixture(&source);
    fs::write(
        source.join("test_scripts.mary.h"),
        "mary_script_table { NULL, EventScript_First, EventScript_Second, EventScript_Missing, };\n",
    )
    .unwrap();

    let result = bundle_output(&source, &output, "packed");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("0x0003 EventScript_Missing"), "{stderr}");
    assert!(!output.join("scripts.bin").exists());

    fs::remove_dir_all(root).unwrap();
}
