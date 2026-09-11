use mary::{
    bytecode::encode_script,
    decompiler::decompile_script,
    ir::{Ins, JumpId, Script},
    mary_c::{format_script, parse_callable_table, parse_scripts, Options},
};

fn table() -> mary::mary_c::CallableTable {
    parse_callable_table(r#"mary_callable_table(TEST,0){int F(int a);int G(int a,int b);void P(int a);void Text(const char *s);mary_reserved();}"#,&Options::default()).unwrap()
}

#[test]
fn unfamiliar_deep_control_flow_and_calls_are_text_lossless() {
    let source = r#"
mary_script(77) void UnfamiliarStress(void)
{
    int x = G(F(3), G(4, 5)), y;
    y = F(x);
    for (int i = 0; i < 3; i++)
    {
        if (i == 1) { P(i); }
    }
    do
    {
        if (x > 0)
        {
            switch (x)
            {
                case 1:
                    mary_switch_compact (y)
                    {
                        case 7:
                            if (F(y) != 0) { P(y); }
                            break;
                        case 8:
                        case 9:
                            y--;
                        mary_implicit_default:
                            P(0);
                    }
                    break;
                case 2:
                    do { y += F(x); } while (y < 20);
                mary_dead_jump:
                default:
                    P(y);
                    break;
            }
        }
        else { x--; }
    }
    while (x != 0);
    return;
}
"#;
    let table = table();
    let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
    let bytes = encode_script(&first.scripts[0].2);
    let raised = decompile_script(&first.scripts[0].2, &table.scope).unwrap();
    if matches!(raised.as_slice(), [mary::ast::Stmt::Ir(_)]) {
        panic!(
            "structured decompile failed: {}",
            mary::decompiler::decompile_script_structured(&first.scripts[0].2, &table.scope)
                .unwrap_err()
        );
    }
    let printed = format_script(77, "UnfamiliarStress", &raised).unwrap();
    let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
    assert_eq!(bytes, encode_script(&second.scripts[0].2), "{printed}");
}

#[test]
fn switch_case_with_two_terminating_if_else_branches_does_not_fall_through() {
    let source = r#"
mary_script(78) void TerminatingCase(void)
{
    switch (F(0))
    {
        case 0:
            if (F(1))
            {
                break;
            }
            else
            {
                return;
            }
        case 1:
            P(1);
            break;
        default:
            return;
    }
    P(2);
}
"#;
    let table = table();
    let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
    let bytes = encode_script(&first.scripts[0].2);
    let raised = decompile_script(&first.scripts[0].2, &table.scope).unwrap();
    assert!(
        !matches!(raised.as_slice(), [mary::ast::Stmt::Ir(_)]),
        "terminating case fell back to low-level IR"
    );
    let printed = format_script(78, "TerminatingCase", &raised).unwrap();
    let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
    assert_eq!(bytes, encode_script(&second.scripts[0].2), "{printed}");
}

#[test]
fn statement_after_an_all_return_switch_is_handled_losslessly() {
    let source = r#"
mary_script(79) void AllReturnSwitch(void)
{
    switch (F(0))
    {
        case 0:
            return;
        case 1:
            if (F(1))
            {
                return;
            }
            else
            {
                return;
            }
        default:
            return;
    }
    P(2);
}
"#;
    let table = table();
    let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
    let bytes = encode_script(&first.scripts[0].2);
    let raised = decompile_script(&first.scripts[0].2, &table.scope).unwrap();
    assert!(
        !matches!(raised.as_slice(), [mary::ast::Stmt::Ir(_)]),
        "all-return switch fell back to low-level IR"
    );
    let printed = format_script(79, "AllReturnSwitch", &raised).unwrap();
    let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
    assert_eq!(bytes, encode_script(&second.scripts[0].2), "{printed}");
}

#[test]
fn implicit_compact_and_unmatched_switch_exits_remain_distinct() {
    let cases = [
        (
            80,
            "ImplicitReturnSwitch",
            r#"
mary_script(80) void ImplicitReturnSwitch(void)
{
    switch (F(0))
    {
        case 0:
            return;
        mary_implicit_default:
            return;
    }
    P(2);
}
"#,
        ),
        (
            81,
            "CompactImplicitReturnSwitch",
            r#"
mary_script(81) void CompactImplicitReturnSwitch(void)
{
    mary_switch_compact (F(0))
    {
        case 0:
            return;
        mary_implicit_default:
            return;
    }
    P(3);
}
"#,
        ),
        (
            82,
            "UnmatchedSwitchExit",
            r#"
mary_script(82) void UnmatchedSwitchExit(void)
{
    switch (F(0))
    {
        case 0:
            return;
        case 1:
            return;
    }
    P(4);
}
"#,
        ),
        (
            83,
            "LeadingImplicitReturnSwitch",
            r#"
mary_script(83) void LeadingImplicitReturnSwitch(void)
{
    switch (F(0))
    {
        mary_implicit_default:
            return;
        case 0:
            P(5);
            break;
    }
}
"#,
        ),
        (
            84,
            "InterleavedImplicitReturnSwitch",
            r#"
mary_script(84) void InterleavedImplicitReturnSwitch(void)
{
    switch (F(0))
    {
        case 0:
            P(6);
            break;
        mary_implicit_default:
            if (F(8))
            {
                return;
            }
            else
            {
                return;
            }
        case 1:
            P(7);
            break;
    }
}
"#,
        ),
        (
            85,
            "CompactLeadingImplicitReturnSwitch",
            r#"
mary_script(85) void CompactLeadingImplicitReturnSwitch(void)
{
    mary_switch_compact (F(0))
    {
        mary_implicit_default:
            return;
        case 0:
            P(9);
            break;
    }
}
"#,
        ),
        (
            86,
            "CompactInterleavedImplicitReturnSwitch",
            r#"
mary_script(86) void CompactInterleavedImplicitReturnSwitch(void)
{
    mary_switch_compact (F(0))
    {
        case 0:
            P(10);
            break;
        mary_implicit_default:
            if (F(11))
            {
                return;
            }
            else
            {
                return;
            }
        case 1:
            P(12);
            break;
    }
}
"#,
        ),
    ];
    let table = table();
    for (id, name, source) in cases {
        let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
        let bytes = encode_script(&first.scripts[0].2);
        let raised = decompile_script(&first.scripts[0].2, &table.scope).unwrap();
        assert!(
            !matches!(raised.as_slice(), [mary::ast::Stmt::Ir(_)]),
            "{name} fell back to low-level IR"
        );
        let printed = format_script(id, name, &raised).unwrap();
        // A standard-layout implicit default at the physical tail is bytewise
        // equivalent to the layout's mandatory dead jump and canonicalizes
        // to compact + mary_dead_jump. All other placements, and especially
        // compact input, retain the explicit implicit-default identity.
        if id != 80 && name.contains("Implicit") {
            assert!(printed.contains("mary_implicit_default:"), "{printed}");
            assert!(!printed.contains("mary_dead_jump:"), "{printed}");
        }
        let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
        assert_eq!(bytes, encode_script(&second.scripts[0].2), "{printed}");
    }
}

#[test]
fn nested_standard_and_compact_switch_exits_are_lossless() {
    let source = r#"
mary_script(87) void NestedSwitchExitMatrix(void)
{
    switch (F(0))
    {
        case 0:
            mary_switch_compact (F(1))
            {
                case 1:
                    if (F(2))
                    {
                        break;
                    }
                    else
                    {
                        return;
                    }
                mary_implicit_default:
                    return;
            }
            P(13);
            break;
        case 1:
            P(14);
            break;
        default:
            return;
    }
    P(15);
}
"#;
    let table = table();
    let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
    let bytes = encode_script(&first.scripts[0].2);
    let raised = decompile_script(&first.scripts[0].2, &table.scope).unwrap();
    assert!(
        !matches!(raised.as_slice(), [mary::ast::Stmt::Ir(_)]),
        "nested switch matrix fell back to low-level IR"
    );
    let printed = format_script(87, "NestedSwitchExitMatrix", &raised).unwrap();
    assert!(printed.contains("mary_switch_compact"), "{printed}");
    assert!(printed.contains("mary_implicit_default:"), "{printed}");
    let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
    assert_eq!(bytes, encode_script(&second.scripts[0].2), "{printed}");
}

#[test]
fn invalid_constructs_report_the_real_problem() {
    let table = table();
    let cases = [
        (
            "mary_script(1) void X(void){int x=P(1);}",
            "procedures do not yield results",
        ),
        ("mary_script(1) void X(void){P();}", "requires 1"),
        ("mary_script(1) void X(void){Missing(1);}", "not declared"),
        (
            "mary_script(1) void X(void){P(mary_direct_int(-1));}",
            "not declared",
        ),
        (
            "mary_script(1) void X(void){do{break;}while(1);}",
            "loop break is not supported",
        ),
        (
            "mary_script(1) void X(void){const int x;}",
            "requires an initializer",
        ),
        (
            "mary_script(1) void X(void){Text(1);}",
            "requires const char *",
        ),
        ("mary_script(1) void X(void){P(\"wrong\");}", "requires int"),
        (
            "mary_script(1) void X(void){Text(gText_Missing);}",
            "not declared",
        ),
        (
            "mary_script(1) void X(void){P(mary_negated_int());}",
            "requires 1",
        ),
        (
            "mary_script(1) void X(void){P(mary_negated_int(F(1)));}",
            "constant expression",
        ),
        (
            "mary_script(1) void X(void){P(mary_negated_int(\"wrong\"));}",
            "constant integer",
        ),
    ];
    for (source, needle) in cases {
        let error = parse_scripts(source, &Options::default(), &table.scope)
            .unwrap_err()
            .to_string();
        assert!(error.contains(needle), "expected {needle:?} in {error:?}");
    }
}

#[test]
fn documented_unsupported_language_boundary_is_rejected() {
    let table = table();
    let cases = [
        ("while", "mary_script(1) void X(void){while(F(1)){P(1);}}"),
        (
            "continue",
            "mary_script(1) void X(void){for(int i=0;i<3;i++){continue;}}",
        ),
        (
            "goto",
            "mary_script(1) void X(void){goto done;done:return;}",
        ),
        (
            "conditional operator",
            "mary_script(1) void X(void){int x=F(1)?2:3;P(x);}",
        ),
        (
            "comma operator",
            "mary_script(1) void X(void){int x=(F(1),F(2));P(x);}",
        ),
        ("character literal", "mary_script(1) void X(void){P('A');}"),
        ("return value", "mary_script(1) void X(void){return 1;}"),
        ("script parameter", "mary_script(1) void X(int x){P(x);}"),
        (
            "ordinary helper function",
            "int Helper(int x){return x+1;}mary_script(1) void X(void){P(Helper(1));}",
        ),
        ("pointer", "mary_script(1) void X(void){int *p;P(*p);}"),
        (
            "runtime array",
            "mary_script(1) void X(void){int values[2];P(values[0]);}",
        ),
        (
            "structure",
            "mary_script(1) void X(void){struct Pair p;P(p.left);}",
        ),
        ("floating point", "mary_script(1) void X(void){P(1.5);}"),
        ("bitwise and", "mary_script(1) void X(void){P(F(1)&3);}"),
        ("left shift", "mary_script(1) void X(void){P(F(1)<<2);}"),
        ("cast", "mary_script(1) void X(void){P((int)F(1));}"),
        ("sizeof", "mary_script(1) void X(void){P(sizeof(int));}"),
    ];

    for (name, source) in cases {
        assert!(
            parse_scripts(source, &Options::default(), &table.scope).is_err(),
            "documented unsupported construct unexpectedly compiled: {name}"
        );
    }
}

#[test]
fn binary_output_rejects_ir_comments_at_the_cli_boundary() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mary"))
        .args(["compile", "--binary", "--print-ir"])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be used with")
            && stderr.contains("--binary")
            && stderr.contains("--print-ir"),
        "unexpected CLI diagnostic: {stderr}"
    );
}

#[test]
fn arbitrary_binary_offset_outside_the_input_is_a_diagnostic_not_a_panic() {
    let directory = std::path::PathBuf::from("test_failures")
        .join(format!("invalid_offset_{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("short.bin");
    std::fs::write(&input, [0_u8; 4]).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mary"))
        .args([
            "decompile",
            input.to_str().unwrap(),
            "goodies/fomt_callables.mary.h",
            "--offset",
            "0x100",
            "--mary-c",
            "-D",
            "MARY_FOMT_US",
        ])
        .output()
        .unwrap();
    std::fs::remove_dir_all(directory).unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Offset is outside the input binary"),
        "unexpected CLI diagnostic: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "CLI panicked: {stderr}");
}

#[test]
fn mary_c_cli_rejects_unstructured_ir_instead_of_emitting_fake_source() {
    let directory = std::path::PathBuf::from("test_failures")
        .join(format!("strict_mary_c_{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for (name, instructions) in [
        ("stack_underflow", vec![Ins::Add]),
        (
            "residual_jump_next",
            vec![Ins::Jmp(JumpId(0)), Ins::Label(JumpId(0))],
        ),
    ] {
        let input = directory.join(format!("{name}.riff"));
        let output_path = directory.join(format!("{name}.mary.c"));
        std::fs::write(
            &input,
            encode_script(&Script::new(instructions, Vec::new())),
        )
        .unwrap();

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_mary"))
            .args([
                "decompile",
                input.to_str().unwrap(),
                "goodies/fomt_callables.mary.h",
                "--mary-c",
                "-D",
                "MARY_FOMT_US",
                "-o",
                output_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(!output.status.success(), "{name} unexpectedly succeeded");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Decompile error") || stderr.contains("Decompile Error"),
            "unexpected CLI diagnostic for {name}: {stderr}"
        );
        assert!(
            !stderr.contains("panicked"),
            "CLI panicked for {name}: {stderr}"
        );
        assert!(
            !output_path.exists(),
            "Mary-C CLI created source for {name} after structural recovery failed"
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn duplicate_callable_is_rejected() {
    let error = parse_callable_table(
        "mary_callable_table(T,0){int Same(void);void Same(void);}",
        &Options::default(),
    )
    .err()
    .unwrap()
    .to_string();
    assert!(error.contains("more than once"));
}

#[test]
fn mary_nodisc_stack_result_survives_a_rom_shaped_control_flow() {
    let table = table();
    let source="mary_script(3) void StackResult(void){int result;mary_nodisc(result = 1);if(F(2)){result = G(result, 3);}else{P(result);}return;}";
    let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
    let bytes = encode_script(&first.scripts[0].2);
    let raised =
        mary::decompiler::decompile_script_structured(&first.scripts[0].2, &table.scope).unwrap();
    let printed = format_script(3, "StackResult", &raised).unwrap();
    let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
    assert_eq!(bytes, encode_script(&second.scripts[0].2));
}

#[test]
fn switch_break_across_an_inner_loop_is_explicitly_mary_syntax() {
    let table = table();
    let source = "mary_script(4) void CrossLoop(void){int i=0;switch(1){case 1:do{if(F(i)){mary_break_switch;}i++;}while(i<3);P(1);break;default:P(2);break;}}";
    let first = parse_scripts(source, &Options::default(), &table.scope).unwrap();
    let raised =
        mary::decompiler::decompile_script_structured(&first.scripts[0].2, &table.scope).unwrap();
    let printed = format_script(4, "CrossLoop", &raised).unwrap();
    assert!(printed.contains("mary_break_switch;"), "{printed}");
    let second = parse_scripts(&printed, &Options::default(), &table.scope).unwrap();
    assert_eq!(
        encode_script(&first.scripts[0].2),
        encode_script(&second.scripts[0].2)
    );
}

#[test]
fn canonical_tables_texts_and_unfamiliar_nesting_are_lossless() {
    use mary::mary_c::{format_named_script, parse_named_scripts, parse_text_name_table};

    let callables = table();
    let options = Options::default().define("MARY_MFOMT_US").unwrap();
    let symbols = parse_text_name_table(
        r#"
mary_script_symbols {
    NULL,
    Stress {
        gText_Stress_000,
#if defined(MARY_MFOMT)
        gText_Stress_001,
#endif
    },
};
"#,
        &options,
    )
    .unwrap();
    let scripts = symbols.script_table().unwrap();
    let source = r#"
mary_text_table {
    const char gText_Stress_000[] = "A\x05";
    const char gText_Stress_001[] = "B\x05";
};
void Stress(void)
{
    int x = G(F(3), 4);
    do
    {
        switch (x)
        {
            case 1:
                Text(gText_Stress_000);
                break;
            default:
                if (F(x)) { Text(gText_Stress_001); }
                break;
        }
        x--;
    }
    while (x != 0);
}
"#;
    let first = parse_named_scripts(source, &options, &callables.scope, &scripts).unwrap();
    let bytes = encode_script(&first.scripts[0].2);
    let raised =
        mary::decompiler::decompile_script_named(&first.scripts[0].2, &callables.scope, "Stress")
            .unwrap();
    let printed = format_named_script("Stress", &raised).unwrap();
    assert!(printed.contains("mary_text_table"));
    assert!(printed.contains("    const char gText_Stress_000[] ="));
    assert!(!printed.contains("mary_script("));
    let second = parse_named_scripts(&printed, &options, &callables.scope, &scripts).unwrap();
    assert_eq!(bytes, encode_script(&second.scripts[0].2), "{printed}");
}
