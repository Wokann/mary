use mary::{
    ast::{Stmt, SwitchCase},
    bytecode::encode_script,
    compiler,
    decompiler::decompile_script_structured,
    pretty_print::PrettyStmts,
};

const DECLARATIONS: &str = r#"
proc 0x002 Mark(value)
func 0x003 Probe(value)
"#;

fn contains_low_level(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Ir(_) | Stmt::JumpNext => true,
        Stmt::If(_, body) | Stmt::DoWhile(_, body) => contains_low_level(body),
        Stmt::IfElse(_, then_body, else_body) => {
            contains_low_level(then_body) || contains_low_level(else_body)
        }
        Stmt::For(elements) => contains_low_level(&elements.3),
        Stmt::Switch(_, cases, _, _) => cases.iter().any(|case| match case {
            SwitchCase::Case(_, body)
            | SwitchCase::Fallthrough(_, body)
            | SwitchCase::Default(body)
            | SwitchCase::DefaultFallthrough(body)
            | SwitchCase::ImplicitDefault(body)
            | SwitchCase::DeadJump(body) => contains_low_level(body),
        }),
        _ => false,
    })
}

fn assert_source_round_trip(name: &str, body: &str) {
    let source = format!("{DECLARATIONS}\nscript 1 Stress\n{{\n{body}\n}}\n");
    let mut parsed = compiler::parse_string(&source)
        .unwrap_or_else(|err| panic!("{name}: initial source failed: {err}"));
    let original = parsed.scripts.remove(0).2;
    let original_bytes = encode_script(&original);

    let decompiled =
        decompile_script_structured(&original, &parsed.const_scope).unwrap_or_else(|err| {
            panic!(
                "{name}: structured decompilation failed: {err}\n{}",
                err.state_at_error()
            )
        });
    assert!(
        !contains_low_level(&decompiled),
        "{name}: decompilation retained IR or jump-next"
    );

    let rebuilt_source = format!(
        "{DECLARATIONS}\nscript 1 Stress\n{{\n{}\n}}\n",
        PrettyStmts::with_indent(&decompiled, 1)
    );
    let mut reparsed = compiler::parse_string(&rebuilt_source).unwrap_or_else(|err| {
        panic!("{name}: decompiled text failed to parse: {err}\n{rebuilt_source}")
    });
    let rebuilt_bytes = encode_script(&reparsed.scripts.remove(0).2);

    assert_eq!(
        original_bytes, rebuilt_bytes,
        "{name}: byte mismatch after source round trip\n{rebuilt_source}"
    );
}

#[test]
fn nested_switches_with_conditional_breaks() {
    assert_source_round_trip(
        "nested_switches_with_conditional_breaks",
        r#"
    var a = 1, b = 2, c = 3
    switch a
    {
        case 1
        {
            switch b
            {
                case 2
                {
                    if Probe(c) == 0
                    {
                        switch c
                        {
                            case 3 { Mark(30) }
                            default { Mark(31) }
                        }
                    }
                    else
                    {
                        break
                    }
                    Mark(20)
                }
                default { Mark(21) }
            }
            Mark(10)
        }
        default { Mark(11) }
    }
"#,
    );
}

#[test]
fn loops_inside_switches_inside_loops() {
    assert_source_round_trip(
        "loops_inside_switches_inside_loops",
        r#"
    var outer = 0, inner = 0
    for var i = 0; i < 3; i++
    {
        switch i
        {
            case 0
            {
                do
                {
                    if Probe(inner)
                    {
                        Mark(inner)
                    }
                    else
                    {
                        Mark(100)
                    }
                    inner++
                } while inner < 2
            }
            case 1
            {
                switch inner
                {
                    case 2 { Mark(200) }
                    default { Mark(201) }
                }
            }
            default { Mark(300) }
        }
        outer += i
    }
"#,
    );
}

#[test]
fn fallthrough_default_and_nested_empty_else() {
    assert_source_round_trip(
        "fallthrough_default_and_nested_empty_else",
        r#"
    var value = 2
    switch value
    {
        case 1 fallthrough
        {
            if Probe(value)
            {
                Mark(1)
            }
        }
        case 2
        {
            if Probe(value) == 2
            {
                if Probe(9)
                {
                    Mark(2)
                }
                else
                {
                }
            }
            else
            {
                Mark(3)
            }
        }
        default { Mark(4) }
    }
"#,
    );
}

#[test]
fn five_level_mixed_nesting() {
    assert_source_round_trip(
        "five_level_mixed_nesting",
        r#"
    var a = 0, b = 0
    do
    {
        if Probe(a)
        {
            switch a
            {
                case 0
                {
                    for var i = 0; i < 2; i++
                    {
                        if Probe(i)
                        {
                            switch b
                            {
                                case 0 { Mark(i) }
                                default { Mark(b) }
                            }
                        }
                        else
                        {
                            Mark(50)
                        }
                    }
                }
                default { Mark(60) }
            }
        }
        else
        {
            Mark(70)
        }
        a++
    } while a < 2
"#,
    );
}

#[test]
fn conditional_break_without_else() {
    assert_source_round_trip(
        "conditional_break_without_else",
        r#"
    var value = 1
    switch value
    {
        case 1
        {
            if Probe(value)
            {
                break
            }
            Mark(1)
        }
        default { Mark(2) }
    }
"#,
    );
}

#[test]
fn generated_eight_level_switch_nesting() {
    let mut body = String::from("    var value = 0\n");
    for depth in 0..8 {
        body.push_str(&format!(
            "{}switch value\n{}{{\n{}case {}\n{}{{\n",
            "    ".repeat(depth + 1),
            "    ".repeat(depth + 1),
            "    ".repeat(depth + 2),
            depth,
            "    ".repeat(depth + 2),
        ));
    }
    body.push_str(&format!("{}Mark(800)\n", "    ".repeat(9)));
    for depth in (0..8).rev() {
        body.push_str(&format!(
            "{} }}\n{}default {{ Mark({}) }}\n{}}}\n",
            "    ".repeat(depth + 2),
            "    ".repeat(depth + 2),
            900 + depth,
            "    ".repeat(depth + 1),
        ));
    }

    assert_source_round_trip("generated_eight_level_switch_nesting", &body);
}
