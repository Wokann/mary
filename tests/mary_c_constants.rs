use std::{collections::HashMap, fs};

use mary::{
    bytecode::encode_script,
    decompiler::decompile_script_named,
    ir::Ins,
    mary_c::{
        format_named_script, parse_callable_table_with_scope, parse_constant_header,
        parse_named_scripts, parse_script_table, parse_text_name_table, Options,
    },
};

#[test]
fn tool_experience_requirements_parse_for_all_four_targets() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        parse_constant_header(&header, &options).unwrap();
    }
    for declaration in [
        "TOOL_EXPERIENCE_REQUIRED_FOR_COPPER = 6000",
        "TOOL_EXPERIENCE_REQUIRED_FOR_SILVER = 18000",
        "TOOL_EXPERIENCE_REQUIRED_FOR_GOLD = 36000",
        "TOOL_EXPERIENCE_REQUIRED_FOR_MYSTRILE_OR_MYTHIC = 65535",
    ] {
        assert!(header.contains(declaration), "missing {declaration}");
    }
}

#[test]
fn retired_external_tree_is_not_cited_as_evidence() {
    fn inspect(path: &std::path::Path, forbidden: &str, violations: &mut Vec<String>) {
        if path.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                inspect(&entry.unwrap().path(), forbidden, violations);
            }
            return;
        }
        let extension = path.extension().and_then(|value| value.to_str());
        if !matches!(extension, Some("rs" | "h" | "md" | "sym" | "txt")) {
            return;
        }
        if fs::read_to_string(path)
            .map(|text| text.to_ascii_lowercase().contains(forbidden))
            .unwrap_or(false)
        {
            violations.push(path.display().to_string());
        }
    }

    let forbidden = ["hm", "fomt"].concat();
    let mut violations = Vec::new();
    for root in [
        "src",
        "tests",
        "goodies",
        "docs",
        "README.md",
        "README.zh-CN.md",
    ] {
        inspect(std::path::Path::new(root), &forbidden, &mut violations);
    }
    assert!(
        violations.is_empty(),
        "retired external source tree is still cited in: {violations:?}"
    );
}

#[test]
fn semantic_state_variables_are_typed_or_explicitly_documented() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let typed = header
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("mary_var_type(")
                .and_then(|rest| rest.split(',').next())
        })
        .collect::<Vec<_>>();
    let semantic_markers = [
        "_STATE",
        "_RESULT",
        "_CHOICE",
        "_ACTIVE",
        "_KIND",
        "_STATUS",
        "_PHASE",
        "_MODE",
        "_SELECTION",
        "_WEATHER",
        "_SEASON",
        "_DAY_OF_WEEK",
        "_OUTFIT",
        "_GROWTH_STAGE",
    ];
    let mut untyped = header
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("VAR_") || !line.contains('=') {
                return None;
            }
            let name = line.split_whitespace().next().unwrap();
            if semantic_markers.iter().any(|marker| name.contains(marker)) && !typed.contains(&name)
            {
                Some(name.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    untyped.sort();
    untyped.dedup();
    assert!(
        untyped.is_empty(),
        "semantic state variables lack mary_var_type mappings: {untyped:?}"
    );
}

#[test]
fn facing_direction_symbols_round_trip_on_all_four_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFacing, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestFacing(void) { SetEntityPosition(0, -10, 20, 0); SetEntityFacing(0, 3); SetEntityFacing(1, GetOppositeFacing(1)); CreateFarmHorse(2, 1, 0, 196, 145); if (GetEntityFacing(0) == 2) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestFacing").unwrap();
        let source = format_named_script("TestFacing", &raised).unwrap();
        assert!(
            source.contains("SetEntityPosition(ENTITY_PLAYER, X(-10), Y(20), FACING_DOWN)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("SetEntityFacing(ENTITY_PLAYER, FACING_RIGHT)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GetOppositeFacing(FACING_UP)"),
            "{target}: {source}"
        );
        assert!(source.contains("CreateFarmHorse(2,"), "{target}: {source}");
        assert!(
            source.contains("GetEntityFacing(ENTITY_PLAYER) == FACING_LEFT"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2)
        );
    }
}

#[test]
fn game_state_variable_constants_are_printed_and_round_trip() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestVariable, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestVariable(void) { if (VarGet(1) == 0) { return; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestVariable").unwrap();
    let source = format_named_script("TestVariable", &raised).unwrap();
    assert!(source.contains("VarGet(VAR_SEASON)"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );

    let numeric = parse_named_scripts(
        "void TestVariable(void) { if (VarGet(6) == 0 || VarGet(7) == 1) { return; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestVariable").unwrap();
    let source = format_named_script("TestVariable", &raised).unwrap();
    assert!(source.contains("VarGet(VAR_WEATHER_TODAY)"), "{source}");
    assert!(source.contains("VarGet(VAR_WEATHER_TOMORROW)"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn indirect_game_state_variable_ids_recover_the_current_value_domain() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestIndirectVariable, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestIndirectVariable(void) {\n\
             int variable = 1;\n\
             int copied_variable = variable;\n\
             VarSet(copied_variable, 3);\n\
             if (VarGet(copied_variable) == 3) { TalkClose(); }\n\
             copied_variable = 6;\n\
             VarSet(copied_variable, 2);\n\
             if (VarGet(copied_variable) == 2) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestIndirectVariable",
        )
        .unwrap();
        let source = format_named_script("TestIndirectVariable", &raised).unwrap();
        assert!(
            source.contains("VarSet(var_1, SEASON_WINTER)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("VarGet(var_1) == SEASON_WINTER"),
            "{target}: {source}"
        );
        assert!(
            source.contains("VarSet(var_1, WEATHER_SNOW)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("VarGet(var_1) == WEATHER_SNOW"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn horse_race_result_values_are_printed_symbolically_and_round_trip() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestHorseRaceResult, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestHorseRaceResult(void) { if (VarGet(VAR_SPRING_HORSE_RACE_RESULT) == 1) { VarSet(VAR_FALL_HORSE_RACE_RESULT, 0); } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestHorseRaceResult",
        )
        .unwrap();
        let source = format_named_script("TestHorseRaceResult", &raised).unwrap();
        assert!(
            source.contains("VarGet(VAR_SPRING_HORSE_RACE_RESULT) == HORSE_RACE_RESULT_WON"),
            "{target}: {source}"
        );
        assert!(
            source.contains("VarSet(VAR_FALL_HORSE_RACE_RESULT, HORSE_RACE_RESULT_NOT_WON)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn gender_shifted_choice_and_runtime_event_slots_have_stable_semantic_names() {
    let entries = [
        (86, "VAR_UNKNOWN_SLOT_086"),
        (94, "VAR_UNKNOWN_SLOT_094"),
        (96, "VAR_UNKNOWN_SLOT_096"),
        (98, "VAR_CLIFF_YELLOW_HEART_EVENT_CHOICE"),
        (115, "VAR_UNKNOWN_SLOT_115"),
        (310, "VAR_KAI_RETURN_GREETING_EVENT_STATE"),
        (313, "VAR_GOTZ_WORK_SUSPENDED"),
        (323, "VAR_VAN_INTRODUCTION_EVENT_STATE"),
        (326, "VAR_LOU_OR_RUBY_INTRODUCTION_EVENT_STATE"),
        (
            595,
            "VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING",
        ),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }

    let fomt_entries = [
        (302, "VAR_KAI_RETURN_GREETING_EVENT_STATE"),
        (305, "VAR_GOTZ_WORK_SUSPENDED"),
        (315, "VAR_VAN_INTRODUCTION_EVENT_STATE"),
        (318, "VAR_LOU_OR_RUBY_INTRODUCTION_EVENT_STATE"),
        (
            509,
            "VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING",
        ),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in fomt_entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn farm_facility_and_storage_variables_round_trip_on_all_targets() {
    let variables = [
        (8, "VAR_PLAYER_BIRTHDAY_SEASON"),
        (9, "VAR_PLAYER_BIRTHDAY_DAY"),
        (10, "VAR_PLAYER_STAMINA"),
        (11, "VAR_PLAYER_MAX_STAMINA"),
        (12, "VAR_PLAYER_FATIGUE"),
        (13, "VAR_HAS_MYSTIC_BERRY"),
        (14, "VAR_FARMHOUSE_UPGRADE_LEVEL"),
        (15, "VAR_COOP_UPGRADE_LEVEL"),
        (16, "VAR_BARN_UPGRADE_LEVEL"),
        (17, "VAR_HAS_MOUNTAIN_COTTAGE"),
        (18, "VAR_HAS_TOWN_COTTAGE"),
        (19, "VAR_HAS_SEASIDE_COTTAGE"),
        (20, "VAR_HAS_BATHROOM"),
        (21, "VAR_HAS_REFRIGERATOR"),
        (22, "VAR_HAS_SHELF"),
        (23, "VAR_HAS_KITCHEN"),
        (24, "VAR_HAS_CARPET"),
        (25, "VAR_HAS_LARGE_BED"),
        (26, "VAR_HAS_MIRROR"),
        (27, "VAR_HAS_CLOCK"),
        (28, "VAR_HAS_VASE"),
        (29, "VAR_HAS_RECORD_PLAYER"),
        (30, "VAR_HAS_STOCKING"),
        (31, "VAR_STOCKING_ARTICLE_ID"),
        (32, "VAR_HAS_KITCHEN_KNIFE"),
        (33, "VAR_HAS_KITCHEN_FRYING_PAN"),
        (34, "VAR_HAS_KITCHEN_POT"),
        (35, "VAR_HAS_KITCHEN_MIXER"),
        (36, "VAR_HAS_KITCHEN_WHISK"),
        (37, "VAR_HAS_KITCHEN_ROLLING_PIN"),
        (38, "VAR_HAS_KITCHEN_OVEN"),
        (39, "VAR_HAS_KITCHEN_SEASONING_SET"),
        (40, "VAR_STORED_LUMBER"),
        (41, "VAR_BARN_STORED_FODDER"),
        (42, "VAR_COOP_STORED_CHICKEN_FEED"),
        (43, "VAR_BEEHIVE_HONEY_AVAILABLE"),
        (44, "VAR_RUCKSACK_UPGRADE_LEVEL"),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFarmVariable, };\n", &options).unwrap();

        for (variable_id, symbol) in variables {
            let numeric = parse_named_scripts(
                &format!(
                    "void TestFarmVariable(void) {{ if (VarGet({variable_id}) != 0) {{ return; }} }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestFarmVariable")
                    .unwrap();
            let source = format_named_script("TestFarmVariable", &raised).unwrap();
            assert!(
                source.contains(&format!("VarGet({symbol})")),
                "{target}: {source}"
            );
            let symbolic =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {symbol}"
            );
        }
    }
}

#[test]
fn spouse_marriage_variables_are_gender_specific_and_round_trip() {
    for (target, variable_id, symbol) in [
        ("MARY_FOMT_US", 76, "VAR_KAREN_MARRIAGE_STATE"),
        ("MARY_FOMT_JP", 76, "VAR_KAREN_MARRIAGE_STATE"),
        ("MARY_MFOMT_US", 80, "VAR_RICK_MARRIAGE_STATE"),
        ("MARY_MFOMT_JP", 80, "VAR_RICK_MARRIAGE_STATE"),
        ("MARY_FOMT_US", 127, "VAR_POPURI_KAI_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_JP", 127, "VAR_POPURI_KAI_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_US", 127, "VAR_RICK_KAREN_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_JP", 127, "VAR_RICK_KAREN_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_US", 119, "VAR_RICK_KAREN_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_US", 135, "VAR_ANN_CLIFF_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_US", 143, "VAR_MARY_GRAY_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_US", 151, "VAR_ELLI_DOCTOR_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_JP", 119, "VAR_RICK_KAREN_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_JP", 135, "VAR_ANN_CLIFF_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_JP", 143, "VAR_MARY_GRAY_RIVAL_MARRIAGE_STATE"),
        ("MARY_FOMT_JP", 151, "VAR_ELLI_DOCTOR_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_US", 135, "VAR_POPURI_KAI_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_US", 143, "VAR_ANN_CLIFF_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_US", 151, "VAR_MARY_GRAY_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_US", 159, "VAR_ELLI_DOCTOR_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_JP", 135, "VAR_POPURI_KAI_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_JP", 143, "VAR_ANN_CLIFF_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_JP", 151, "VAR_MARY_GRAY_RIVAL_MARRIAGE_STATE"),
        ("MARY_MFOMT_JP", 159, "VAR_ELLI_DOCTOR_RIVAL_MARRIAGE_STATE"),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestMarriageState, };\n", &options).unwrap();
        let numeric_source = format!(
            "void TestMarriageState(void) {{ if (VarGet({variable_id}) == 2) {{ return; }} }}\n"
        );
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMarriageState")
                .unwrap();
        let source = format_named_script("TestMarriageState", &raised).unwrap();
        assert!(
            source.contains(&format!("VarGet({symbol})")),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn tv_shopping_delivery_variables_follow_gender_specific_tables() {
    let products = [
        "REFRIGERATOR",
        "SHELF",
        "KITCHEN",
        "CARPET",
        "LARGE_BED",
        "KNIFE",
        "FRYING_PAN",
        "POT",
        "MIXER",
        "WHISK",
        "ROLLING_PIN",
        "OVEN",
        "SEASONING_SET",
        "POWER_BERRY",
        "MIRROR",
        "CLOCK",
    ];

    for (target, base_id) in [
        ("MARY_FOMT_US", 154),
        ("MARY_FOMT_JP", 154),
        ("MARY_MFOMT_US", 162),
        ("MARY_MFOMT_JP", 162),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestTVDelivery, };\n", &options).unwrap();

        for (offset, product) in products.iter().enumerate() {
            let variable_id = base_id + offset;
            let symbol = format!("VAR_TV_SHOPPING_{product}_DELIVERY_STATE");
            let numeric = parse_named_scripts(
                &format!(
                    "void TestTVDelivery(void) {{ if (VarGet({variable_id}) == 2) {{ return; }} }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestTVDelivery")
                    .unwrap();
            let source = format_named_script("TestTVDelivery", &raised).unwrap();
            assert!(
                source.contains(&format!("VarGet({symbol})")),
                "{target}: {source}"
            );
            let symbolic = parse_named_scripts(
                &format!(
                    "void TestTVDelivery(void) {{ if (VarGet({symbol}) == TV_SHOPPING_DELIVERY_COMPLETED) {{ return; }} }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {symbol}"
            );
        }
    }
}

#[test]
fn thomas_request_delivery_variable_follows_gender_specific_slots() {
    for (target, variable_id) in [
        ("MARY_FOMT_US", 266),
        ("MARY_FOMT_JP", 266),
        ("MARY_MFOMT_US", 274),
        ("MARY_MFOMT_JP", 274),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestThomasRequest, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestThomasRequest(void) {{ if (VarGet({variable_id}) == 2) {{ return; }} }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestThomasRequest")
                .unwrap();
        let source = format_named_script("TestThomasRequest", &raised).unwrap();
        assert!(
            source.contains("VarGet(VAR_THOMAS_REQUEST_ITEM_DELIVERY_STATE)"),
            "{target}: {source}"
        );
        let symbolic = parse_named_scripts(
            "void TestThomasRequest(void) { if (VarGet(VAR_THOMAS_REQUEST_ITEM_DELIVERY_STATE) == THOMAS_REQUEST_DELIVERY_COMPLETED) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn special_spouse_family_variables_follow_gender_specific_layouts() {
    let fomt = [
        (370, "VAR_HARVEST_GODDESS_PROPOSAL_EVENT_STATE"),
        (371, "VAR_HARVEST_GODDESS_WEDDING_AND_NICKNAME_EVENT_STATE"),
        (
            372,
            "VAR_HARVEST_GODDESS_ANNIVERSARY_AND_BIRTHDAY_EVENT_STATE",
        ),
        (
            373,
            "VAR_HARVEST_GODDESS_FIFTIETH_ANNIVERSARY_GIFT_EVENT_STATE",
        ),
        (374, "VAR_HARVEST_GODDESS_CHILD_BIRTHDAY_EVENT_STATE"),
        (375, "VAR_HARVEST_GODDESS_FAMILY_EVENING_EVENT_STATE"),
        (376, "VAR_HARVEST_GODDESS_PLAYER_BIRTHDAY_EVENT_STATE"),
        (377, "VAR_HARVEST_GODDESS_PREGNANCY_EVENT_STATE"),
        (378, "VAR_HARVEST_GODDESS_CHILDBIRTH_EVENT_STATE"),
        (379, "VAR_HARVEST_GODDESS_CHILD_FIRST_STEPS_EVENT_STATE"),
        (380, "VAR_HARVEST_GODDESS_CHILD_INJURY_EVENT_STATE"),
    ];
    let mfomt = [
        (378, "VAR_KAPPA_PROPOSAL_EVENT_STATE"),
        (379, "VAR_KAPPA_WEDDING_AND_NICKNAME_EVENT_STATE"),
        (380, "VAR_KAPPA_ANNIVERSARY_AND_BIRTHDAY_EVENT_STATE"),
        (381, "VAR_KAPPA_FIFTIETH_ANNIVERSARY_GIFT_EVENT_STATE"),
        (382, "VAR_KAPPA_FAMILY_EVENT_CHILD_STAGE_1_STATE"),
        (383, "VAR_KAPPA_FAMILY_EVENT_CHILD_STAGE_2_STATE"),
        (384, "VAR_KAPPA_FAMILY_EVENT_CHILD_STAGE_3_STATE"),
        (385, "VAR_KAPPA_PREGNANCY_EVENT_STATE"),
        (386, "VAR_KAPPA_CHILDBIRTH_EVENT_STATE"),
        (387, "VAR_KAPPA_CHILD_FIRST_STEPS_EVENT_STATE"),
        (388, "VAR_KAPPA_CHILD_INJURY_EVENT_STATE"),
        (389, "VAR_WON_PROPOSAL_EVENT_STATE"),
        (390, "VAR_WON_WEDDING_AND_NICKNAME_EVENT_STATE"),
        (391, "VAR_WON_ANNIVERSARY_AND_BIRTHDAY_EVENT_STATE"),
        (392, "VAR_WON_FIFTIETH_ANNIVERSARY_GIFT_EVENT_STATE"),
        (393, "VAR_WON_FAMILY_EVENT_CHILD_STAGE_1_STATE"),
        (394, "VAR_WON_FAMILY_EVENT_CHILD_STAGE_2_STATE"),
        (395, "VAR_WON_FAMILY_EVENT_CHILD_STAGE_3_STATE"),
        (396, "VAR_WON_PREGNANCY_EVENT_STATE"),
        (397, "VAR_WON_CHILDBIRTH_EVENT_STATE"),
        (398, "VAR_WON_CHILD_FIRST_STEPS_EVENT_STATE"),
        (399, "VAR_WON_CHILD_INJURY_EVENT_STATE"),
        (400, "VAR_GOURMET_PROPOSAL_EVENT_STATE"),
        (401, "VAR_GOURMET_WEDDING_AND_NICKNAME_EVENT_STATE"),
        (402, "VAR_GOURMET_ANNIVERSARY_AND_BIRTHDAY_EVENT_STATE"),
        (403, "VAR_GOURMET_FIFTIETH_ANNIVERSARY_GIFT_EVENT_STATE"),
        (404, "VAR_GOURMET_FAMILY_EVENT_CHILD_STAGE_1_STATE"),
        (405, "VAR_GOURMET_FAMILY_EVENT_CHILD_STAGE_2_STATE"),
        (406, "VAR_GOURMET_FAMILY_EVENT_CHILD_STAGE_3_STATE"),
        (407, "VAR_GOURMET_PREGNANCY_EVENT_STATE"),
        (408, "VAR_GOURMET_CHILDBIRTH_EVENT_STATE"),
        (409, "VAR_GOURMET_CHILD_FIRST_STEPS_EVENT_STATE"),
        (410, "VAR_GOURMET_CHILD_INJURY_EVENT_STATE"),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let variables = if target.contains("MFOMT") {
            &mfomt[..]
        } else {
            &fomt[..]
        };
        for &(id, symbol) in variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn standard_spouse_family_variables_follow_each_games_physical_order() {
    let fomt_spouses = ["POPURI", "ANN", "ELLI", "KAREN", "MARY"];
    let mfomt_spouses = ["KAI", "CLIFF", "DOCTOR", "RICK", "GRAY"];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let (spouses, base) = if target.contains("MFOMT") {
            (&mfomt_spouses, 178)
        } else {
            (&fomt_spouses, 170)
        };

        for (index, spouse) in spouses.iter().enumerate() {
            for (offset, suffix) in [
                (index * 2, "ANNIVERSARY_EVENT_STATE"),
                (index * 2 + 1, "ANNIVERSARY_MORNING_REMINDER_STATE"),
                (10 + index, "CHILD_BIRTHDAY_EVENT_STATE"),
                (15 + index, "FAMILY_EVENT_CHILD_STAGE_1_STATE"),
                (20 + index, "FAMILY_EVENT_CHILD_STAGE_2_STATE"),
                (25 + index, "PREGNANCY_EVENT_STATE"),
                (30 + index, "CHILDBIRTH_EVENT_STATE"),
                (35 + index, "CHILD_FIRST_STEPS_EVENT_STATE"),
            ] {
                let id = base + offset;
                let symbol = format!("VAR_{spouse}_{suffix}");
                assert_eq!(
                    constants.typed_int_const_name(variable_type, id as i64),
                    Some(symbol.as_str()),
                    "{target}: variable {id}"
                );
            }

            for (offset, suffix) in [
                (40 + index, "SPOUSE_COLLAPSE_EVENT_STATE"),
                (45 + index, "CHILD_INJURY_EVENT_STATE"),
            ] {
                let id = base + offset;
                let symbol = format!("VAR_{spouse}_{suffix}");
                assert_eq!(
                    constants.typed_int_const_name(variable_type, id as i64),
                    Some(symbol.as_str()),
                    "{target}: variable {id}"
                );
            }
        }
    }
}

#[test]
fn horse_lifecycle_event_variables_follow_the_eight_slot_gender_shift() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 228 } else { 220 };
        for (offset, symbol) in [
            (0, "VAR_BARLEY_FOAL_OFFER_EVENT_STATE"),
            (1, "VAR_BARLEY_HORSE_YEAR_EVALUATION_EVENT_STATE"),
            (2, "VAR_DAYS_SINCE_FAILED_HORSE_YEAR_EVALUATION"),
            (3, "VAR_BARLEY_REPLACEMENT_FOAL_EVENT_STATE"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset),
                Some(symbol),
                "{target}: variable {}",
                base + offset
            );
        }
    }
}

#[test]
fn first_shared_town_event_variables_follow_the_eight_slot_gender_shift() {
    let variables = [
        (0, "VAR_FESTIVAL_DAY_ANNOUNCEMENT_EVENT_STATE"),
        (1, "VAR_PLAYER_COLLAPSE_RECOVERY_EVENT_STATE"),
        (2, "VAR_RICK_AND_POPURI_RUSH_TO_SICK_LILLIA_EVENT_STATE"),
        (3, "VAR_RICK_AND_POPURI_SICK_LILLIA_FOLLOWUP_STATE"),
        (4, "VAR_LILLIA_READS_RODS_LETTER_EVENT_STATE"),
        (5, "VAR_RICK_CONFRONTS_KAI_ABOUT_POPURI_EVENT_STATE"),
        (6, "VAR_KAREN_COMFORTS_LONELY_RICK_EVENT_STATE"),
        (7, "VAR_POPURI_ASKS_KAI_FOR_NECKLACE_EVENT_STATE"),
        (8, "VAR_POPURI_PLANS_LILLIA_BIRTHDAY_GIFT_EVENT_STATE"),
        (
            9,
            "VAR_BARLEY_AND_DOUG_DISCUSS_JOANNAS_PHONE_CALL_EVENT_STATE",
        ),
        (12, "VAR_MAY_PHONE_CALL_WITH_JOANNA_EVENT_STATE"),
        (13, "VAR_SAIBARA_VISITS_ELLEN_EVENT_1_STATE"),
        (14, "VAR_SAIBARA_VISITS_ELLEN_EVENT_2_STATE"),
        (15, "VAR_GRAY_AND_KAI_FRIENDSHIP_EVENT_STATE"),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 234 } else { 226 };
        for (offset, symbol) in variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset),
                Some(symbol),
                "{target}: variable {}",
                base + offset
            );
        }
    }
}

#[test]
fn grape_harvest_and_town_event_variables_follow_the_eight_slot_gender_shift() {
    let variables = [
        (0, "VAR_DUKE_GRAPE_HARVEST_INVITATION_EVENT_STATE"),
        (1, "VAR_TOLD_CLIFF_ABOUT_GRAPE_HARVEST_JOB"),
        (2, "VAR_TOLD_GRAY_ABOUT_GRAPE_HARVEST_JOB"),
        (3, "VAR_TOLD_BASIL_ABOUT_GRAPE_HARVEST_JOB"),
        (4, "VAR_TOLD_JEFF_ABOUT_GRAPE_HARVEST_JOB"),
        (5, "VAR_TOLD_CARTER_ABOUT_GRAPE_HARVEST_JOB"),
        (6, "VAR_CLIFF_PERMANENT_WINERY_JOB_EVENT_STATE"),
        (8, "VAR_DUKE_AND_MANNA_MISSING_JUICE_ARGUMENT_EVENT_STATE"),
        (9, "VAR_MANNA_FLATTERS_JEFF_EVENT_STATE"),
        (10, "VAR_BASIL_LETTER_ADVICE_EVENT_STATE"),
        (11, "VAR_BASIL_LETTER_ADVICE_CHOICE"),
        (12, "VAR_BASIL_PUBLISHING_AWARD_EVENT_STATE"),
        (13, "VAR_ANNA_COOKING_LESSONS_INVITATION_EVENT_STATE"),
        (14, "VAR_ANNA_COOKING_LESSONS_ACCEPTED"),
        (15, "VAR_ANNA_COOKING_LESSON_EVENT_STATE"),
        (16, "VAR_ANNA_COOKING_LESSONS_COMPLETED_COUNT"),
        (17, "VAR_MARY_AND_GRAY_BOOK_AND_HEALTH_EVENT_STATE"),
        (18, "VAR_MARY_MARRIED_LIFE_AND_WRITING_EVENT_STATE"),
        (19, "VAR_FARM_INTRODUCTION_AND_SHIPPING_TUTORIAL_STATE"),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 251 } else { 243 };
        for (offset, symbol) in variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset),
                Some(symbol),
                "{target}: variable {}",
                base + offset
            );
        }
    }
}

#[test]
fn request_harris_and_ellen_event_variables_follow_the_eight_slot_gender_shift() {
    let variables = [
        (0, "VAR_HARVEST_GODDESS_ITEM_REQUEST_EVENT_STATE"),
        (1, "VAR_HARVEST_GODDESS_REQUESTED_ITEM_INDEX"),
        (2, "VAR_HARVEST_GODDESS_ITEM_REQUEST_CHOICE"),
        (3, "VAR_THOMAS_REQUEST_ITEM_DELIVERY_STATE"),
        (4, "VAR_HARRIS_AJA_LETTER_ADVICE_EVENT_STATE"),
        (5, "VAR_HARRIS_AJA_LETTER_ADVICE_CHOICE"),
        (6, "VAR_DAYS_SINCE_HARRIS_AJA_LETTER_ADVICE"),
        (7, "VAR_HARRIS_AJA_LETTER_ADVICE_FOLLOWUP_STATE"),
        (8, "VAR_ELLEN_WHITE_FLOWER_LEGEND_EVENT_STATE"),
        (9, "VAR_ELLEN_WHITE_FLOWER_DISCOVERY_EVENT_STATE"),
        (10, "VAR_ELLEN_GRANDFATHER_LETTER_EVENT_STATE"),
        (11, "VAR_ELLEN_KNITS_STOCKING_EVENT_STATE"),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 271 } else { 263 };
        for (offset, symbol) in variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset),
                Some(symbol),
                "{target}: variable {}",
                base + offset
            );
        }
    }
}

#[test]
fn villager_story_event_variables_follow_the_eight_slot_gender_shift() {
    let variables = [
        (0, "VAR_ELLI_PLAYS_WITH_STU_EVENT_STATE"),
        (1, "VAR_WON_DISCOVERS_JEFFS_PAINTING_TALENT_EVENT_STATE"),
        (
            2,
            "VAR_SASHA_TEACHES_JEFF_TO_REFUSE_STORE_CREDIT_EVENT_STATE",
        ),
        (3, "VAR_MANNA_DISCUSSING_AJAS_DEPARTURE_EVENT_STATE"),
        (
            4,
            "VAR_LILLIA_AND_SASHA_REMINISCE_ABOUT_JEFFS_MARRIAGE_EVENT_STATE",
        ),
        (5, "VAR_KAREN_AND_DUKE_DRINKING_CONTEST_EVENT_STATE"),
        (6, "VAR_WON_MEETS_KAREN_EVENT_STATE"),
        (7, "VAR_DOCTOR_DISCUSSING_HIS_FAMILY_PROFESSION_EVENT_STATE"),
        (8, "VAR_JEFF_BLOOD_TYPE_CORRECTION_EVENT_STATE"),
        (9, "VAR_ELLI_NURSING_CAREER_ADVICE_EVENT_STATE"),
        (10, "VAR_ELLI_TREATS_STUS_COLD_EVENT_STATE"),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 285 } else { 277 };
        for (offset, symbol) in variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset),
                Some(symbol),
                "{target}: variable {}",
                base + offset
            );
        }
    }
}

#[test]
fn church_cliff_and_late_town_events_follow_the_eight_slot_gender_shift() {
    let variables = [
        (0, "VAR_CARTER_MYSTERIOUS_VOICE_EVENT_STATE"),
        (1, "VAR_CARTER_OPENS_CHURCH_BACK_DOOR_EVENT_STATE"),
        (2, "VAR_PLAYER_SNEAKS_PAST_SLEEPING_CARTER_EVENT_STATE"),
        (3, "VAR_CLIFF_COLLAPSES_IN_SNOW_EVENT_STATE"),
        (4, "VAR_CLIFF_COLLAPSE_FOLLOWUP_EVENT_STATE"),
        (6, "VAR_CLIFF_LEAVES_MINERAL_TOWN_EVENT_STATE"),
        (7, "VAR_ANN_MOTHERS_DEATH_ANNIVERSARY_EVENT_STATE"),
        (8, "VAR_DOUG_AND_DUKE_ARGUMENT_EVENT_STATE"),
        (9, "VAR_DOUG_AND_DUKE_ARGUMENT_CHOICE"),
        (10, "VAR_ANN_GIVES_DOUG_BIRTHDAY_PRESENT_EVENT_STATE"),
        (
            11,
            "VAR_ANN_AND_CLIFF_SIBLING_COMPARISON_ARGUMENT_EVENT_STATE",
        ),
        (
            12,
            "VAR_POPURI_BRINGS_CUSTOMERS_TO_KAIS_BEACH_CAFE_EVENT_STATE",
        ),
        (13, "VAR_KAI_RETURNS_FOR_SUMMER_EVENT_STATE"),
        (15, "VAR_KAI_LEAVES_AFTER_SUMMER_EVENT_STATE"),
        (16, "VAR_GOTZ_LOSES_MOTIVATION_EVENT_STATE"),
        (18, "VAR_GOTZ_REGAINS_MOTIVATION_EVENT_STATE"),
        (19, "VAR_GOTZ_AND_HARRIS_PATROL_DISCUSSION_EVENT_STATE"),
        (20, "VAR_ZACK_VISITS_SICK_LILLIA_EVENT_STATE"),
        (21, "VAR_ZACK_GIVES_FISHING_ROD_EVENT_STATE"),
        (22, "VAR_ZACK_FISHING_ROD_FOLLOWUP_EVENT_STATE"),
        (23, "VAR_WON_INTRODUCTION_EVENT_STATE"),
        (24, "VAR_WON_APPLE_CHALLENGE_EVENT_STATE"),
        (25, "VAR_WON_VASE_PURCHASE_EVENT_STATE"),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 296 } else { 288 };
        for (offset, symbol) in variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset),
                Some(symbol),
                "{target}: variable {}",
                base + offset
            );
        }
    }
}

#[test]
fn village_girls_cooking_request_is_mfomt_only() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, 342),
            Some("VAR_VILLAGE_GIRLS_COOKING_REQUEST_EVENT_STATE"),
            "{target}"
        );
    }
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_ne!(
            constants.typed_int_const_name(variable_type, 342),
            Some("VAR_VILLAGE_GIRLS_COOKING_REQUEST_EVENT_STATE"),
            "{target}"
        );
    }
}

#[test]
fn first_festival_variables_use_gender_specific_physical_blocks() {
    for (target, rice_cake, horse_invitation, horse_entry, cooking_invitation) in [
        ("MARY_FOMT_US", 384, 391, 392, 395),
        ("MARY_FOMT_JP", 384, 391, 392, 395),
        ("MARY_MFOMT_US", 414, 435, 436, 439),
        ("MARY_MFOMT_JP", 414, 435, 436, 439),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in [
            (rice_cake, "VAR_NEW_YEAR_RICE_CAKE_FESTIVAL_EVENT_STATE"),
            (horse_invitation, "VAR_HORSE_RACE_INVITATION_EVENT_STATE"),
            (horse_entry, "VAR_HORSE_RACE_PLAYER_ENTRY_SELECTED"),
            (
                cooking_invitation,
                "VAR_COOKING_FESTIVAL_INVITATION_EVENT_STATE",
            ),
        ] {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_festival_runtime_fields_follow_verified_calendar_dispatchers() {
    let entries = [
        (393, "VAR_SPRING_HORSE_RACE_FESTIVAL_ACTIVE"),
        (394, "VAR_SPRING_HORSE_RACE_RESULT"),
        (396, "VAR_COOKING_FESTIVAL_ACTIVE"),
        (397, "VAR_COOKING_FESTIVAL_DISH_CATEGORY"),
        (398, "VAR_COOKING_FESTIVAL_COMPLETED"),
        (399, "VAR_COOKING_FESTIVAL_PLAYER_ENTERED"),
        (400, "VAR_COOKING_FESTIVAL_PLAYER_DISH_RATING"),
        (401, "VAR_COOKING_FESTIVAL_PLAYER_DISH_FOOD_ID"),
        (404, "VAR_BEACH_DAY_FESTIVAL_ACTIVE"),
        (405, "VAR_FRISBEE_TOURNAMENT_RESULT"),
        (408, "VAR_CHICKEN_FESTIVAL_ACTIVE"),
        (409, "VAR_CHICKEN_FESTIVAL_RESULT"),
        (414, "VAR_FIREWORKS_FESTIVAL_ACTIVE"),
        (418, "VAR_MUSIC_FESTIVAL_ACTIVE"),
        (420, "VAR_HARVEST_FESTIVAL_ACTIVE"),
        (422, "VAR_HARVEST_FESTIVAL_SESSION_PHASE"),
        (423, "VAR_UNKNOWN_SLOT_423"),
        (428, "VAR_FALL_HORSE_RACE_FESTIVAL_ACTIVE"),
        (429, "VAR_FALL_HORSE_RACE_RESULT"),
        (432, "VAR_SHEEP_FESTIVAL_ACTIVE"),
        (433, "VAR_SHEEP_FESTIVAL_RESULT"),
        (434, "VAR_PUMPKIN_FESTIVAL_MAY_TREAT_VISIT_STATE"),
        (435, "VAR_PUMPKIN_FESTIVAL_STU_TREAT_VISIT_STATE"),
        (436, "VAR_PUMPKIN_FESTIVAL_POPURI_TREAT_VISIT_STATE"),
        (437, "VAR_PUMPKIN_FESTIVAL_FARMHOUSE_SPOUSE_EVENT_STATE"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: festival runtime variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_festival_runtime_fields_follow_verified_dispatch_and_cleanup_paths() {
    let entries = [
        (438, "VAR_SPRING_HORSE_RACE_RESULT"),
        (440, "VAR_COOKING_FESTIVAL_ACTIVE"),
        (441, "VAR_COOKING_FESTIVAL_DISH_CATEGORY"),
        (442, "VAR_COOKING_FESTIVAL_COMPLETED"),
        (443, "VAR_COOKING_FESTIVAL_PLAYER_ENTERED"),
        (444, "VAR_COOKING_FESTIVAL_PLAYER_DISH_RATING"),
        (445, "VAR_COOKING_FESTIVAL_PLAYER_DISH_FOOD_ID"),
        (452, "VAR_CHICKEN_FESTIVAL_ACTIVE"),
        (453, "VAR_CHICKEN_FESTIVAL_RESULT"),
        (456, "VAR_COW_FESTIVAL_ACTIVE"),
        (457, "VAR_COW_FESTIVAL_RESULT"),
        (458, "VAR_FIREWORKS_FESTIVAL_ACTIVE"),
        (462, "VAR_MUSIC_FESTIVAL_ACTIVE"),
        (464, "VAR_HARVEST_FESTIVAL_ACTIVE"),
        (472, "VAR_FALL_HORSE_RACE_FESTIVAL_ACTIVE"),
        (473, "VAR_FALL_HORSE_RACE_RESULT"),
        (476, "VAR_SHEEP_FESTIVAL_ACTIVE"),
        (477, "VAR_SHEEP_FESTIVAL_RESULT"),
        (478, "VAR_PUMPKIN_FESTIVAL_MAY_TREAT_VISIT_STATE"),
        (479, "VAR_PUMPKIN_FESTIVAL_STU_TREAT_VISIT_STATE"),
        (480, "VAR_PUMPKIN_FESTIVAL_POPURI_TREAT_VISIT_STATE"),
        (481, "VAR_PUMPKIN_FESTIVAL_FARMHOUSE_SPOUSE_EVENT_STATE"),
        (483, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_POPURI"),
        (485, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_ANN"),
        (487, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_ELLI"),
        (489, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_KAREN"),
        (491, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_MARY"),
        (
            492,
            "VAR_WINTER_THANKSGIVING_FARMHOUSE_SPOUSE_GIFT_EVENT_STATE",
        ),
        (493, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_KAI"),
        (494, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_CLIFF"),
        (495, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_DOCTOR"),
        (496, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_RICK"),
        (497, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_GRAY"),
        (498, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_WON"),
        (499, "VAR_WINTER_THANKSGIVING_GIFT_GIVEN_TO_GOURMET"),
        (500, "VAR_STARRY_NIGHT_INVITATION_MAIL_DELIVERY_EVENT_STATE"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: festival runtime variable {id}"
            );
        }
    }
}

#[test]
fn animal_festival_invitation_and_entry_variables_use_gender_specific_blocks() {
    let cases = [
        ("MARY_FOMT_US", [402, 403, 406, 407, 410, 411, 430, 431]),
        ("MARY_FOMT_JP", [402, 403, 406, 407, 410, 411, 430, 431]),
        ("MARY_MFOMT_US", [446, 447, 450, 451, 454, 455, 474, 475]),
        ("MARY_MFOMT_JP", [446, 447, 450, 451, 454, 455, 474, 475]),
    ];
    let symbols = [
        "VAR_FRISBEE_TOURNAMENT_INVITATION_EVENT_STATE",
        "VAR_FRISBEE_TOURNAMENT_PLAYER_DOG_ENTRY_SELECTED",
        "VAR_CHICKEN_FESTIVAL_INVITATION_EVENT_STATE",
        "VAR_CHICKEN_FESTIVAL_PLAYER_ENTRY_SELECTED",
        "VAR_COW_FESTIVAL_INVITATION_EVENT_STATE",
        "VAR_COW_FESTIVAL_PLAYER_ENTRY_SELECTED",
        "VAR_SHEEP_FESTIVAL_INVITATION_EVENT_STATE",
        "VAR_SHEEP_FESTIVAL_PLAYER_ENTRY_SELECTED",
    ];
    for (target, ids) in cases {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in ids.into_iter().zip(symbols) {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn festival_invitation_and_entry_state_symbols_compile_to_original_values() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFestival, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestFestival(void) { VarSet(VAR_CHICKEN_FESTIVAL_INVITATION_EVENT_STATE, 2); VarSet(VAR_CHICKEN_FESTIVAL_PLAYER_ENTRY_SELECTED, 1); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let symbolic = parse_named_scripts(
            "void TestFestival(void) { VarSet(VAR_CHICKEN_FESTIVAL_INVITATION_EVENT_STATE, FESTIVAL_INVITATION_COMPLETED); VarSet(VAR_CHICKEN_FESTIVAL_PLAYER_ENTRY_SELECTED, FESTIVAL_PLAYER_ENTRY_SELECTED); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn music_harvest_moon_and_fall_race_variables_use_gender_specific_blocks() {
    let cases = [
        ("MARY_FOMT_US", [416, 417, 419, 421, 424, 425, 426, 427]),
        ("MARY_FOMT_JP", [416, 417, 419, 421, 424, 425, 426, 427]),
        ("MARY_MFOMT_US", [460, 461, 463, 465, 468, 469, 470, 471]),
        ("MARY_MFOMT_JP", [460, 461, 463, 465, 468, 469, 470, 471]),
    ];
    let symbols = [
        "VAR_MUSIC_FESTIVAL_INVITATION_EVENT_STATE",
        "VAR_MUSIC_FESTIVAL_PLAYER_PERFORMANCE_ACCEPTED",
        "VAR_HARVEST_FESTIVAL_INVITATION_EVENT_STATE",
        "VAR_HARVEST_FESTIVAL_CONTRIBUTED_INGREDIENT_ACCEPTED",
        "VAR_MOON_VIEWING_FESTIVAL_EVENT_STATE",
        "VAR_MOON_VIEWING_PARTNER_INDEX",
        "VAR_FALL_HORSE_RACE_INVITATION_EVENT_STATE",
        "VAR_FALL_HORSE_RACE_PLAYER_ENTRY_SELECTED",
    ];
    for (target, ids) in cases {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in ids.into_iter().zip(symbols) {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fireworks_partner_values_and_variable_are_gender_specific() {
    for (target, variable_id, partner_symbol) in [
        ("MARY_FOMT_US", 415, "FIREWORKS_FESTIVAL_PARTNER_ANN"),
        ("MARY_FOMT_JP", 415, "FIREWORKS_FESTIVAL_PARTNER_ANN"),
        ("MARY_MFOMT_US", 459, "FIREWORKS_FESTIVAL_PARTNER_DOCTOR"),
        ("MARY_MFOMT_JP", 459, "FIREWORKS_FESTIVAL_PARTNER_DOCTOR"),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, variable_id),
            Some("VAR_FIREWORKS_FESTIVAL_PARTNER"),
            "{target}"
        );
        let partner_type = constants.user_type("MaryFireworksFestivalPartner").unwrap();
        assert_eq!(
            constants.typed_int_const_name(partner_type, 5),
            Some(partner_symbol),
            "{target}"
        );
    }
}

#[test]
fn thanksgiving_exchange_variables_follow_spouse_candidates_and_gender_layout() {
    let cases: [(&str, &[(i64, &str)]); 4] = [
        (
            "MARY_FOMT_US",
            &[
                (386, "KAREN"),
                (387, "POPURI"),
                (388, "MARY"),
                (389, "ELLI"),
                (390, "ANN"),
            ],
        ),
        (
            "MARY_FOMT_JP",
            &[
                (386, "KAREN"),
                (387, "POPURI"),
                (388, "MARY"),
                (389, "ELLI"),
                (390, "ANN"),
            ],
        ),
        (
            "MARY_MFOMT_US",
            &[
                (421, "RICK"),
                (422, "KAI"),
                (423, "GRAY"),
                (424, "DOCTOR"),
                (425, "CLIFF"),
                (426, "WON"),
                (427, "GOURMET"),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            &[
                (421, "RICK"),
                (422, "KAI"),
                (423, "GRAY"),
                (424, "DOCTOR"),
                (425, "CLIFF"),
                (426, "WON"),
                (427, "GOURMET"),
            ],
        ),
    ];
    for (target, entries) in cases {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for &(id, candidate) in entries {
            let symbol = format!("VAR_THANKSGIVING_GIFT_EXCHANGED_WITH_{candidate}");
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol.as_str()),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_winter_thanksgiving_visit_and_received_gift_states_follow_event_pairs() {
    let entries = [
        (438, "VAR_WINTER_THANKSGIVING_POPURI_VISIT_EVENT_STATE"),
        (439, "VAR_WINTER_THANKSGIVING_GIFT_RECEIVED_FROM_POPURI"),
        (440, "VAR_WINTER_THANKSGIVING_ANN_VISIT_EVENT_STATE"),
        (441, "VAR_WINTER_THANKSGIVING_GIFT_RECEIVED_FROM_ANN"),
        (442, "VAR_WINTER_THANKSGIVING_ELLI_VISIT_EVENT_STATE"),
        (443, "VAR_WINTER_THANKSGIVING_GIFT_RECEIVED_FROM_ELLI"),
        (444, "VAR_WINTER_THANKSGIVING_KAREN_VISIT_EVENT_STATE"),
        (445, "VAR_WINTER_THANKSGIVING_GIFT_RECEIVED_FROM_KAREN"),
        (446, "VAR_WINTER_THANKSGIVING_MARY_VISIT_EVENT_STATE"),
        (447, "VAR_WINTER_THANKSGIVING_GIFT_RECEIVED_FROM_MARY"),
        (
            448,
            "VAR_WINTER_THANKSGIVING_FARMHOUSE_SPOUSE_GIFT_EVENT_STATE",
        ),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, fomt_symbol) in entries {
            assert_ne!(
                constants.typed_int_const_name(variable_type, id),
                Some(fomt_symbol),
                "{target}: FoMT-only variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_thanksgiving_gift_interaction_states_follow_character_slots() {
    let entries = [
        (428, "RICK"),
        (429, "KAI"),
        (430, "GRAY"),
        (431, "DOCTOR"),
        (432, "CLIFF"),
        (433, "WON"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, character) in entries {
            let symbol = format!("VAR_THANKSGIVING_GIFT_INTERACTION_STATE_WITH_{character}");
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol.as_str()),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn gamecube_link_level_and_called_script_argument_follow_gender_layouts() {
    let cases = [
        ("MARY_FOMT_US", 525, 580),
        ("MARY_FOMT_JP", 525, 580),
        ("MARY_MFOMT_US", 617, 672),
        ("MARY_MFOMT_JP", 617, 672),
    ];
    for (target, link_level_id, script_argument_id) in cases {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, link_level_id),
            Some("VAR_GAMECUBE_LINK_LEVEL"),
            "{target}"
        );
        assert_eq!(
            constants.typed_int_const_name(variable_type, script_argument_id),
            Some("VAR_SHARED_CALLED_SCRIPT_ARGUMENT"),
            "{target}"
        );
    }
}

#[test]
fn mfomt_seven_ring_collection_states_follow_verified_event_writers() {
    let entries = [
        (708, "VAR_ALL_SEVEN_RINGS_COMPLETION_STATE"),
        (721, "VAR_ANNIVERSARY_RING_OBTAINED"),
        (722, "VAR_TEN_MILLION_STEPS_RING_OBTAINED"),
        (723, "VAR_BIRTHDAY_RING_OBTAINED"),
        (724, "VAR_WEDDING_RING_OBTAINED"),
        (725, "VAR_MAILBOX_RING_OBTAINED"),
        (726, "VAR_STARRY_NIGHT_RING_OBTAINED"),
        (727, "VAR_WINTER_THANKSGIVING_RING_OBTAINED"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_shop_purchase_counter_uses_its_girl_version_slot() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, 688),
            Some("VAR_SHOP_PURCHASE_COUNT_CYCLIC"),
            "{target}"
        );
    }
}

#[test]
fn fomt_starry_night_host_invitation_and_event_states_follow_candidate_order() {
    let entries = [
        (450, "VAR_STARRY_NIGHT_FESTIVAL_HOST_INDEX"),
        (451, "VAR_STARRY_NIGHT_POPURI_INVITATION_HANDLED"),
        (452, "VAR_STARRY_NIGHT_ANN_INVITATION_HANDLED"),
        (453, "VAR_STARRY_NIGHT_ELLI_INVITATION_HANDLED"),
        (454, "VAR_STARRY_NIGHT_KAREN_INVITATION_HANDLED"),
        (455, "VAR_STARRY_NIGHT_MARY_INVITATION_HANDLED"),
        (456, "VAR_STARRY_NIGHT_POPURI_EVENT_STATE"),
        (457, "VAR_STARRY_NIGHT_ANN_EVENT_STATE"),
        (458, "VAR_STARRY_NIGHT_ELLI_EVENT_STATE"),
        (459, "VAR_STARRY_NIGHT_KAREN_EVENT_STATE"),
        (460, "VAR_STARRY_NIGHT_MARY_EVENT_STATE"),
        (461, "VAR_STARRY_NIGHT_FARMHOUSE_SPOUSE_EVENT_STATE"),
        (462, "VAR_THOMAS_STOCKING_DELIVERY_EVENT_STATE"),
        (463, "VAR_THOMAS_STOCKING_GIFT_SELECTION"),
        (464, "VAR_NEW_YEARS_EVE_NOODLE_FESTIVAL_EVENT_STATE"),
        (465, "VAR_NEW_YEAR_SUNRISE_EVENT_STATE"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_character_recipe_teaching_states_follow_verified_dialogue_writers() {
    let entries = [
        (467, "VAR_RECIPE_LEARNED_FROM_LILLIA"),
        (468, "VAR_RECIPE_LEARNED_FROM_BARLEY"),
        (469, "VAR_RECIPE_LEARNED_FROM_SAIBARA"),
        (470, "VAR_RECIPE_LEARNED_FROM_MANNA"),
        (471, "VAR_RECIPE_LEARNED_FROM_BASIL"),
        (472, "VAR_RECIPE_LEARNED_FROM_HARRIS"),
        (473, "VAR_RECIPE_LEARNED_FROM_ELLEN"),
        (474, "VAR_RECIPE_LEARNED_FROM_SASHA"),
        (475, "VAR_RECIPE_LEARNED_FROM_DOCTOR"),
        (476, "VAR_RECIPE_LEARNED_FROM_CARTER"),
        (477, "VAR_RECIPE_LEARNED_FROM_DOUG"),
        (478, "VAR_RECIPE_LEARNED_FROM_KAI"),
        (479, "VAR_RECIPE_LEARNED_FROM_GOTZ"),
        (480, "VAR_RECIPE_LEARNED_FROM_ZACK"),
        (481, "VAR_RECIPE_LEARNED_FROM_CHEF"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_television_program_cursors_follow_daily_update_and_program_readers() {
    let entries = [
        (482, "VAR_TV_CARD_COLLECTOR_CHISATO_EPISODE_INDEX"),
        (483, "VAR_TV_MY_DEAR_PRINCESS_EPISODE_INDEX"),
        (484, "VAR_TV_DUELING_CHEFS_EPISODE_INDEX"),
        (485, "VAR_TV_FAIRY_AND_ME_HIS_STORY_EPISODE_INDEX"),
        (486, "VAR_TV_FAIRY_AND_ME_HER_STORY_EPISODE_INDEX"),
        (487, "VAR_TV_AARON_CHANGES_EPISODE_INDEX"),
        (488, "VAR_TV_MECHABOT_ULTROR_EPISODE_INDEX"),
        (489, "VAR_TV_MECHABOT_ULTROR_ZERO_REMINDER_INDEX"),
        (490, "VAR_TV_STAR_LILY_BANDIT_GIRL_EPISODE_INDEX"),
        (491, "VAR_TV_ST_EMERALD_ACADEMY_EPISODE_INDEX"),
        (492, "VAR_TV_MINE_RESEARCH_GROUP_EPISODE_INDEX"),
        (493, "VAR_TV_MINERAL_TOWN_FRIENDS_PROFILE_INDEX"),
        (494, "VAR_TV_HARVEST_GODDESS_NUMBER_GAME_PROGRESS"),
        (495, "VAR_TV_FISHING_HOUR_SEGMENT_INDEX"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_gamecube_link_completion_milestone_guards_follow_verified_predicates() {
    let entries = [
        (513, "VAR_GAMECUBE_LINK_VACATION_VILLA_MILESTONE_RECORDED"),
        (
            514,
            "VAR_GAMECUBE_LINK_ALL_VILLAGERS_MAX_FRIENDSHIP_MILESTONE_RECORDED",
        ),
        (
            515,
            "VAR_GAMECUBE_LINK_ALL_CROPS_SHIPPED_MILESTONE_RECORDED",
        ),
        (
            516,
            "VAR_GAMECUBE_LINK_ALL_FARM_ANIMALS_MAX_AFFECTION_MILESTONE_RECORDED",
        ),
        (
            517,
            "VAR_GAMECUBE_LINK_ALL_MINERALS_SHIPPED_MILESTONE_RECORDED",
        ),
        (
            518,
            "VAR_GAMECUBE_LINK_ALL_FISH_SPECIES_CAUGHT_MILESTONE_RECORDED",
        ),
        (
            519,
            "VAR_GAMECUBE_LINK_ONE_MILLION_STEPS_MILESTONE_RECORDED",
        ),
        (520, "VAR_GAMECUBE_LINK_FIFTY_YEARS_MILESTONE_RECORDED"),
        (
            521,
            "VAR_GAMECUBE_LINK_TEN_THOUSAND_FISH_CAUGHT_MILESTONE_RECORDED",
        ),
        (
            522,
            "VAR_GAMECUBE_LINK_GOLDEN_LUMBER_ON_FARM_MILESTONE_RECORDED",
        ),
        (
            523,
            "VAR_GAMECUBE_LINK_MOUNTAIN_AND_SEASIDE_COTTAGES_MILESTONE_RECORDED",
        ),
        (524, "VAR_GAMECUBE_LINK_MYTHIC_TOOL_MILESTONE_RECORDED"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_gamecube_link_dialogue_recipe_and_introduction_fields_keep_proven_gaps() {
    let entries = [
        (496, "VAR_GAMECUBE_LINK_VAN_DIALOGUE_PENDING"),
        (497, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_DIALOGUE_PENDING"),
        (498, "VAR_GAMECUBE_LINK_ELLEN_DIALOGUE_PENDING"),
        (499, "VAR_GAMECUBE_LINK_BARLEY_DIALOGUE_PENDING"),
        (500, "VAR_GAMECUBE_LINK_RICK_DIALOGUE_PENDING"),
        (501, "VAR_GAMECUBE_LINK_THOMAS_DIALOGUE_PENDING"),
        (502, "VAR_GAMECUBE_LINK_JEFF_DIALOGUE_PENDING"),
        (503, "VAR_GAMECUBE_LINK_CARTER_DIALOGUE_PENDING"),
        (504, "VAR_GAMECUBE_LINK_GOTZ_DIALOGUE_PENDING"),
        (505, "VAR_GAMECUBE_LINK_HARVEST_SPRITES_DIALOGUE_PENDING"),
        (506, "VAR_GAMECUBE_LINK_VAN_ALBUM_UNLOCK_DIALOGUE_PENDING"),
        (508, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_RECIPES_LEARNED"),
        (
            509,
            "VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING",
        ),
        (510, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_RECIPE_UNLOCKS_PENDING"),
        (511, "VAR_GAMECUBE_LINK_VAN_INTRODUCTION_AVAILABLE"),
        (512, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_INTRODUCTION_AVAILABLE"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(variable_type, 507),
            Some("VAR_GAMECUBE_LINK_UPDATE_IN_PROGRESS"),
            "{target}: link update guard"
        );
    }
}

#[test]
fn ruby_reward_dialogues_pending_remains_a_counter_for_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestLouOrRubyCounter, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestLouOrRubyCounter(void) { \
             VarSet(VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING, 2); \
             if (VarGet(VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING) >= 1) { \
             VarSet(VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING, \
             VarGet(VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING) - 1); } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestLouOrRubyCounter",
        )
        .unwrap();
        let source = format_named_script("TestLouOrRubyCounter", &raised).unwrap();
        assert!(source.contains(">= 1"), "{target}: {source}");
        assert!(!source.contains(">= TRUE"), "{target}: {source}");
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn fomt_jewel_of_truth_guards_name_script_and_native_sources() {
    let entries = [
        (526, "VAR_JEWEL_OF_TRUTH_FROM_REFRIGERATOR_COLLECTED"),
        (
            527,
            "VAR_JEWEL_OF_TRUTH_FROM_HORSE_RACE_PRIZE_EXCHANGE_COLLECTED",
        ),
        (528, "VAR_JEWEL_OF_TRUTH_PURCHASED_FROM_WON"),
        (529, "VAR_JEWEL_OF_TRUTH_FROM_CALENDAR_COLLECTED"),
        (530, "VAR_JEWEL_OF_TRUTH_FROM_BOOKSHELF_COLLECTED"),
        (
            531,
            "VAR_JEWEL_OF_TRUTH_FROM_HARVEST_GODDESS_MATH_QUIZ_COLLECTED",
        ),
        (532, "VAR_JEWEL_OF_TRUTH_FROM_DOG_HOUSE_COLLECTED"),
        (533, "VAR_JEWEL_OF_TRUTH_FROM_STREET_LAMP_COLLECTED"),
        (534, "VAR_JEWEL_OF_TRUTH_FROM_WATER_TANK_COLLECTED"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_awl_bookshelf_profile_levels_follow_menu_and_reference_page_order() {
    let names = [
        (535, "PLAYER"),
        (537, "TAKAKURA"),
        (538, "ROMANA"),
        (539, "LUMINA"),
        (540, "SEBASTIAN"),
        (541, "WALLY"),
        (542, "CHRIS"),
        (543, "HUGH"),
        (544, "GRANT"),
        (545, "SAMANTHA"),
        (546, "KATE"),
        (547, "GALEN"),
        (548, "NINA"),
        (549, "DARYL"),
        (550, "GUSTAFA"),
        (551, "CODY"),
        (552, "KASSEY"),
        (553, "PATRICK"),
        (554, "MURREY"),
        (555, "TIM"),
        (556, "LOU_OR_RUBY"),
        (557, "NAMI"),
        (558, "ROCK"),
        (559, "GRIFFIN"),
        (560, "MUFFY"),
        (561, "CARTER"),
        (562, "FLORA"),
        (563, "VESTA"),
        (564, "MARLIN"),
        (565, "CELIA"),
        (566, "HARDY"),
        (567, "VAN"),
        (568, "MOOKY"),
        (569, "NAK"),
        (570, "NIC"),
        (571, "FLAK"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, name) in names {
            let symbol = format!("VAR_GAMECUBE_LINK_AWL_{name}_PROFILE_LEVEL");
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol.as_str()),
                "{target}: variable {id}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(variable_type, 536),
            Some("VAR_UNKNOWN_SLOT_536"),
            "{target}: skipped profile slot 536"
        );
    }
}

#[test]
fn awl_character_profile_menu_uses_official_character_semantics_on_all_targets() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let expected = [
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_NoProfilesUnlocked",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_PreviousPage",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterTakakura",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterRomana",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_NextPage",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterLumina",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterSebastian",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterWally",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterChris",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterHugh",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterGrant",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterSamantha",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterKate",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterGalen",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterNina",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterDaryl",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterGustafa",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterCody",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterKassey",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterPatrick",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterMurrey",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterTim",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterLouOrRuby",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterNami",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterRock",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterGriffin",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterMuffy",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterCarter",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterFlora",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterVesta",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterMarlin",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterCelia",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterHardy",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterVan",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterMooky",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterNak",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterNic",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterFlak",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_CharacterPlayer",
        "gText_SystemEvent_GameCubeLinkAWLCharacterProfileMenu_ProfileNotUnlocked",
    ];

    for (target, script_id) in [
        ("MARY_FOMT_US", 316),
        ("MARY_FOMT_JP", 316),
        ("MARY_MFOMT_US", 325),
        ("MARY_MFOMT_JP", 325),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_SystemEvent_GameCubeLinkAWLCharacterProfileMenu"),
            "{target} script slot"
        );
        let actual = symbols
            .names(script_id, symbols.text_count(script_id))
            .into_iter()
            .map(|name| name.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{target} AWL profile menu text order");
    }
}

#[test]
fn awl_profile_progress_levels_print_symbolically_and_round_trip_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestAwlProfile, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestAwlProfile(void) {\n\
             if (VarGet(VAR_GAMECUBE_LINK_AWL_TAKAKURA_PROFILE_LEVEL) == 0) {\n\
             VarSet(VAR_GAMECUBE_LINK_AWL_TAKAKURA_PROFILE_LEVEL, 1); }\n\
             if (VarGet(VAR_GAMECUBE_LINK_AWL_TAKAKURA_PROFILE_LEVEL) <= 2) {\n\
             VarSet(VAR_GAMECUBE_LINK_AWL_TAKAKURA_PROFILE_LEVEL, 3); }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAwlProfile")
                .unwrap();
        let source = format_named_script("TestAwlProfile", &raised).unwrap();
        for symbol in [
            "AWL_PROFILE_LOCKED",
            "AWL_PROFILE_LEVEL_1",
            "AWL_PROFILE_LEVEL_2",
            "AWL_PROFILE_LEVEL_3",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn debug_tool_texts_are_scoped_to_their_actual_editor() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id, script_name, text_prefix) in [
        (
            "MARY_FOMT_US",
            1031,
            "EventScript_SystemEvent_DebugCharacterAffectionEditor",
            "gText_SystemEvent_DebugCharacterAffectionEditor_",
        ),
        (
            "MARY_FOMT_JP",
            1031,
            "EventScript_SystemEvent_DebugCharacterAffectionEditor",
            "gText_SystemEvent_DebugCharacterAffectionEditor_",
        ),
        (
            "MARY_MFOMT_US",
            359,
            "EventScript_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_",
        ),
        (
            "MARY_MFOMT_JP",
            359,
            "EventScript_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_",
        ),
        (
            "MARY_MFOMT_US",
            433,
            "EventScript_SystemEvent_DebugPortraitExpressionViewer",
            "gText_SystemEvent_DebugPortraitExpressionViewer_",
        ),
        (
            "MARY_MFOMT_JP",
            433,
            "EventScript_SystemEvent_DebugPortraitExpressionViewer",
            "gText_SystemEvent_DebugPortraitExpressionViewer_",
        ),
        (
            "MARY_MFOMT_US",
            1101,
            "EventScript_SystemEvent_DebugBachelorFriendshipEditor",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_",
        ),
        (
            "MARY_MFOMT_JP",
            1101,
            "EventScript_SystemEvent_DebugBachelorFriendshipEditor",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.script_name(script_id), Some(script_name));
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(
            !names.is_empty(),
            "{target} {script_name} has no text symbols"
        );
        for name in names.into_iter().flatten() {
            assert!(
                name.starts_with(text_prefix),
                "{target} {script_name} has an unscoped text symbol: {name}"
            );
        }
    }
}

#[test]
fn stu_cold_clinic_event_has_an_event_level_name_and_scoped_dialogue() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 776),
        ("MARY_FOMT_JP", 776),
        ("MARY_MFOMT_US", 785),
        ("MARY_MFOMT_JP", 785),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_NPCEvent_ElliAndStu_StuColdClinicVisit"),
            "{target} script slot"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(!names.is_empty());
        for name in names.iter().flatten() {
            assert!(
                name.starts_with("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_"),
                "{target}: unscoped Stu cold event text: {name}"
            );
        }
        assert_eq!(
            names[0].as_deref(),
            Some("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_ElliSuggestsDoctorVisit")
        );
        assert_eq!(
            names[10].as_deref(),
            Some(
                "gText_NPCEvent_ElliAndStu_StuColdClinicVisit_DoctorDiagnosesColdAndPrescribesMedicine"
            )
        );
        assert_eq!(
            names[17].as_deref(),
            Some("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_DoctorAsksPlayerToEscortStuHome")
        );
        if target.starts_with("MARY_FOMT_") {
            assert_eq!(
                names[8].as_deref(),
                Some("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_ElliAsksPlayerToAccompanyStu")
            );
            assert_eq!(
                names[16].as_deref(),
                Some("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_StuThanksPlayer")
            );
        } else {
            assert_eq!(
                names[8].as_deref(),
                Some(
                    "gText_NPCEvent_ElliAndStu_StuColdClinicVisit_ElliExplainsStuSuddenlyBecameSick"
                )
            );
            assert_eq!(
                names[16].as_deref(),
                Some("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_StuThanksElli")
            );
        }
        if target == "MARY_MFOMT_JP" {
            assert_eq!(
                names[18].as_deref(),
                Some("gText_NPCEvent_ElliAndStu_StuColdClinicVisit_ChoiceEscortStuHome")
            );
        }
    }
}

#[test]
fn functional_menu_texts_are_scoped_to_their_own_dispatcher() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id, script_name, text_prefix) in [
        (
            "MARY_FOMT_US",
            285,
            "EventScript_TV_Shopping_PhoneOrder",
            "gText_TV_Shopping_PhoneOrder_",
        ),
        (
            "MARY_FOMT_JP",
            285,
            "EventScript_TV_Shopping_PhoneOrder",
            "gText_TV_Shopping_PhoneOrder_",
        ),
        (
            "MARY_MFOMT_US",
            294,
            "EventScript_TV_Shopping_PhoneOrder",
            "gText_TV_Shopping_PhoneOrder_",
        ),
        (
            "MARY_MFOMT_JP",
            294,
            "EventScript_TV_Shopping_PhoneOrder",
            "gText_TV_Shopping_PhoneOrder_",
        ),
        (
            "MARY_FOMT_US",
            317,
            "EventScript_SystemMenu_TelephoneDirectory",
            "gText_SystemMenu_TelephoneDirectory_",
        ),
        (
            "MARY_FOMT_JP",
            317,
            "EventScript_SystemMenu_TelephoneDirectory",
            "gText_SystemMenu_TelephoneDirectory_",
        ),
        (
            "MARY_MFOMT_US",
            326,
            "EventScript_SystemMenu_TelephoneDirectory",
            "gText_SystemMenu_TelephoneDirectory_",
        ),
        (
            "MARY_MFOMT_JP",
            326,
            "EventScript_SystemMenu_TelephoneDirectory",
            "gText_SystemMenu_TelephoneDirectory_",
        ),
        (
            "MARY_FOMT_US",
            351,
            "EventScript_SystemEvent_RecordPlayerInteraction",
            "gText_SystemEvent_RecordPlayerInteraction_",
        ),
        (
            "MARY_FOMT_JP",
            351,
            "EventScript_SystemEvent_RecordPlayerInteraction",
            "gText_SystemEvent_RecordPlayerInteraction_",
        ),
        (
            "MARY_MFOMT_US",
            360,
            "EventScript_SystemEvent_RecordPlayerInteraction",
            "gText_SystemEvent_RecordPlayerInteraction_",
        ),
        (
            "MARY_MFOMT_JP",
            360,
            "EventScript_SystemEvent_RecordPlayerInteraction",
            "gText_SystemEvent_RecordPlayerInteraction_",
        ),
        (
            "MARY_MFOMT_US",
            365,
            "EventScript_TV_MainMenuAndProgramDispatcher",
            "gText_TV_MainMenuAndProgramDispatcher_",
        ),
        (
            "MARY_MFOMT_JP",
            365,
            "EventScript_TV_MainMenuAndProgramDispatcher",
            "gText_TV_MainMenuAndProgramDispatcher_",
        ),
        (
            "MARY_MFOMT_US",
            412,
            "EventScript_SystemEvent_FestivalMenu",
            "gText_SystemEvent_FestivalMenu_",
        ),
        (
            "MARY_MFOMT_JP",
            412,
            "EventScript_SystemEvent_FestivalMenu",
            "gText_SystemEvent_FestivalMenu_",
        ),
        (
            "MARY_FOMT_US",
            487,
            "EventScript_ShopEvent_Blacksmith_OrderCounter",
            "gText_ShopEvent_Blacksmith_OrderCounter_",
        ),
        (
            "MARY_FOMT_JP",
            487,
            "EventScript_ShopEvent_Blacksmith_OrderCounter",
            "gText_ShopEvent_Blacksmith_OrderCounter_",
        ),
        (
            "MARY_MFOMT_US",
            496,
            "EventScript_ShopEvent_Blacksmith_OrderCounter",
            "gText_ShopEvent_Blacksmith_OrderCounter_",
        ),
        (
            "MARY_MFOMT_JP",
            496,
            "EventScript_ShopEvent_Blacksmith_OrderCounter",
            "gText_ShopEvent_Blacksmith_OrderCounter_",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.script_name(script_id), Some(script_name));
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(
            !names.is_empty(),
            "{target} {script_name} has no text symbols"
        );
        for name in names.into_iter().flatten() {
            assert!(
                name.starts_with(text_prefix),
                "{target} {script_name} has an unscoped text symbol: {name}"
            );
        }
    }
}

#[test]
fn mfomt_awl_bookshelf_profile_levels_follow_reference_page_ids() {
    let names = [
        (627, "PLAYER"),
        (629, "TAKAKURA"),
        (630, "ROMANA"),
        (631, "LUMINA"),
        (632, "SEBASTIAN"),
        (633, "WALLY"),
        (634, "CHRIS"),
        (635, "HUGH"),
        (636, "GRANT"),
        (637, "SAMANTHA"),
        (638, "KATE"),
        (639, "GALEN"),
        (640, "NINA"),
        (641, "DARYL"),
        (642, "GUSTAFA"),
        (643, "CODY"),
        (644, "KASSEY"),
        (645, "PATRICK"),
        (646, "MURREY"),
        (647, "TIM"),
        (648, "LOU_OR_RUBY"),
        (649, "NAMI"),
        (650, "ROCK"),
        (651, "GRIFFIN"),
        (652, "MUFFY"),
        (653, "CARTER"),
        (654, "FLORA"),
        (655, "VESTA"),
        (656, "MARLIN"),
        (657, "CELIA"),
        (658, "HARDY"),
        (659, "VAN"),
        (660, "MOOKY"),
        (661, "NAK"),
        (662, "NIC"),
        (663, "FLAK"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, name) in names {
            let symbol = format!("VAR_GAMECUBE_LINK_AWL_{name}_PROFILE_LEVEL");
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol.as_str()),
                "{target}: variable {id}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(variable_type, 628),
            Some("VAR_UNKNOWN_SLOT_628"),
            "{target}: skipped Child profile slot 628"
        );
    }
}

#[test]
fn mfomt_jewels_daily_fields_rival_routes_and_rewards_follow_verified_lifecycles() {
    let entries = [
        (618, "VAR_JEWEL_OF_TRUTH_FROM_REFRIGERATOR_COLLECTED"),
        (
            619,
            "VAR_JEWEL_OF_TRUTH_FROM_HORSE_RACE_PRIZE_EXCHANGE_COLLECTED",
        ),
        (620, "VAR_JEWEL_OF_TRUTH_PURCHASED_FROM_WON"),
        (621, "VAR_JEWEL_OF_TRUTH_FROM_CALENDAR_COLLECTED"),
        (622, "VAR_JEWEL_OF_TRUTH_FROM_BOOKSHELF_COLLECTED"),
        (
            623,
            "VAR_JEWEL_OF_TRUTH_FROM_HARVEST_GODDESS_MATH_QUIZ_COLLECTED",
        ),
        (624, "VAR_JEWEL_OF_TRUTH_FROM_DOG_HOUSE_COLLECTED"),
        (625, "VAR_JEWEL_OF_TRUTH_FROM_STREET_LAMP_COLLECTED"),
        (626, "VAR_JEWEL_OF_TRUTH_FROM_WATER_TANK_COLLECTED"),
        (664, "VAR_GOLDEN_LUMBER_WAS_ON_FARM_AT_DAY_START"),
        (665, "VAR_HARVEST_GODDESS_OFFERING_MADE_TODAY"),
        (666, "VAR_HARVEST_GODDESS_OFFERING_REWARD_CYCLE"),
        (667, "VAR_KAPPA_CUCUMBER_OFFERING_MADE_TODAY"),
        (668, "VAR_KAPPA_CUCUMBER_OFFERING_REWARD_CYCLE"),
        (669, "VAR_CHURCH_CONFESSION_USED_TODAY"),
        (670, "VAR_ENTERED_FARMHOUSE_TODAY"),
        (671, "VAR_KAPPA_MARRIAGE_BLESSING_RECEIVED"),
        (672, "VAR_SHARED_CALLED_SCRIPT_ARGUMENT"),
        (673, "VAR_RICK_AND_KAREN_RIVAL_WEDDING_ROUTING_STATE"),
        (674, "VAR_POPURI_AND_KAI_RIVAL_WEDDING_ROUTING_STATE"),
        (675, "VAR_MARY_AND_GRAY_RIVAL_WEDDING_ROUTING_STATE"),
        (676, "VAR_ELLI_AND_DOCTOR_RIVAL_WEDDING_ROUTING_STATE"),
        (677, "VAR_ANN_AND_CLIFF_RIVAL_WEDDING_ROUTING_STATE"),
        (678, "VAR_WEDDING_ANNIVERSARIES_ELAPSED"),
        (679, "VAR_HORSE_RACE_POWER_BERRY_OBTAINED"),
        (680, "VAR_FRISBEE_TOURNAMENT_POWER_BERRY_OBTAINED"),
        (681, "VAR_ZACK_EMPTY_SHIPPING_BIN_ADVICE_CYCLE"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_late_event_television_and_fish_pond_variables_use_verified_ids() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let cooking_request_type = constants.user_type("MaryGirlsCookingRequestState").unwrap();

        for (name, value) in [
            (
                "VAR_QUEEN_OF_THE_NIGHT_OFFERINGS_TOWARD_PRESSED_FLOWER",
                689,
            ),
            ("VAR_COOKED_EVERY_RECIPE_ACHIEVEMENT_RECORDED", 690),
            ("VAR_SAVED_ONE_BILLION_G_ACHIEVEMENT_RECORDED", 691),
            (
                "VAR_HARVEST_GODDESS_ROCK_PAPER_SCISSORS_100_WIN_ACHIEVEMENT_RECORDED",
                692,
            ),
            (
                "VAR_HARVEST_GODDESS_NUMBER_GUESSING_100_WIN_ACHIEVEMENT_RECORDED",
                693,
            ),
            ("VAR_GOURMET_LUNCH_VISIT_OCCURRED_THIS_MONTH", 694),
            ("VAR_GOURMET_DINNER_VISIT_OCCURRED_THIS_MONTH", 695),
            ("VAR_GIRLS_SLEEP_OVER_EVENT_COMPLETED", 696),
            ("VAR_POPURI_COOKING_REQUEST_STATE", 698),
            ("VAR_ANN_COOKING_REQUEST_STATE", 699),
            ("VAR_ELLI_COOKING_REQUEST_STATE", 700),
            ("VAR_KAREN_COOKING_REQUEST_STATE", 701),
            ("VAR_MARY_COOKING_REQUEST_STATE", 702),
            ("VAR_WON_APPLE_CLEANUP_EVENT_COMPLETED", 703),
            ("VAR_WON_APPLE_SHUFFLE_GAME_INTRODUCED", 704),
            ("VAR_WON_ITEM_SELLING_SERVICE_UNLOCKED", 706),
            ("VAR_HUNDRED_QUESTION_QUIZ_SCORE_REWARD_RECEIVED", 707),
            ("VAR_TV_FAIRY_AND_ME_HIS_STORY_COMPLETED", 709),
            ("VAR_TV_CARD_COLLECTOR_CHISATO_COMPLETED", 710),
            ("VAR_TV_MY_DEAR_PRINCESS_COMPLETED", 711),
            ("VAR_TV_AARON_CHANGES_COMPLETED", 712),
            ("VAR_TV_MECHABOT_ULTROR_COMPLETED", 713),
            ("VAR_TV_MECHABOT_GENESIS_COMPLETED", 714),
            ("VAR_TV_ST_EMERALD_ACADEMY_COMPLETED", 715),
            ("VAR_TV_STAR_LILY_BANDIT_GIRL_COMPLETED", 716),
            ("VAR_TV_FAIRY_AND_ME_HER_STORY_COMPLETED", 717),
            ("VAR_FISH_POND_LARGE_FISH_COUNT", 718),
            ("VAR_FISH_POND_MEDIUM_FISH_COUNT", 719),
            ("VAR_FISH_POND_SMALL_FISH_COUNT", 720),
        ] {
            assert_eq!(
                constants.typed_int_const_name(variable_type, value),
                Some(name),
                "{target}: {name}"
            );
        }
        for (name, value) in [
            ("GIRLS_COOKING_REQUEST_STATE_INACTIVE", 0),
            ("GIRLS_COOKING_REQUEST_STATE_RECIPE_SEED_1", 1),
            ("GIRLS_COOKING_REQUEST_STATE_RECIPE_SEED_2", 2),
            ("GIRLS_COOKING_REQUEST_STATE_COMPLETED", 3),
        ] {
            assert_eq!(
                constants.typed_int_const_name(cooking_request_type, value),
                Some(name),
                "{target}: {name}"
            );
        }
    }
}

#[test]
fn mfomt_spring_thanksgiving_girl_gift_fields_follow_character_order() {
    let entries = [
        (416, "VAR_SPRING_THANKSGIVING_GIFT_EXCHANGED_WITH_KAREN"),
        (417, "VAR_SPRING_THANKSGIVING_GIFT_EXCHANGED_WITH_POPURI"),
        (418, "VAR_SPRING_THANKSGIVING_GIFT_EXCHANGED_WITH_MARY"),
        (419, "VAR_SPRING_THANKSGIVING_GIFT_EXCHANGED_WITH_ELLI"),
        (420, "VAR_SPRING_THANKSGIVING_GIFT_EXCHANGED_WITH_ANN"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_festival_session_fields_follow_us_and_jp_script_lifecycles() {
    let entries = [
        (434, "VAR_THANKSGIVING_GIFT_INTERACTION_STATE_WITH_GOURMET"),
        (437, "VAR_HORSE_RACE_FESTIVAL_SESSION_STATE"),
        (448, "VAR_FRISBEE_TOURNAMENT_SESSION_STATE"),
        (449, "VAR_FRISBEE_TOURNAMENT_RESULT"),
        (466, "VAR_HARVEST_FESTIVAL_SESSION_PHASE"),
        (467, "VAR_UNKNOWN_SLOT_467"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_cliff_employment_and_cow_festival_fields_follow_us_and_jp_lifecycles() {
    let entries = [
        (250, "VAR_CLIFF_WINERY_EMPLOYMENT_STATUS"),
        (412, "VAR_COW_FESTIVAL_SESSION_STATE"),
        (413, "VAR_COW_FESTIVAL_PLAYER_RESULT"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_cliff_employment_field_follows_the_shifted_event_family() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, 258),
            Some("VAR_CLIFF_WINERY_EMPLOYMENT_STATUS"),
            "{target}: variable 258"
        );
    }
}

#[test]
fn cliff_collapse_followup_delay_counter_follows_the_gender_shift() {
    for (target, id) in [
        ("MARY_FOMT_US", 293),
        ("MARY_FOMT_JP", 293),
        ("MARY_MFOMT_US", 301),
        ("MARY_MFOMT_JP", 301),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, id),
            Some("VAR_CLIFF_COLLAPSE_FOLLOWUP_DELAY_DAY_COUNTER"),
            "{target}: variable {id}"
        );
    }
}

#[test]
fn ellen_stocking_eligibility_gate_stays_explicitly_unknown() {
    for (target, id, symbol) in [
        ("MARY_FOMT_US", 275, "VAR_UNKNOWN_SLOT_275"),
        ("MARY_FOMT_JP", 275, "VAR_UNKNOWN_SLOT_275"),
        ("MARY_MFOMT_US", 283, "VAR_UNKNOWN_SLOT_283"),
        ("MARY_MFOMT_JP", 283, "VAR_UNKNOWN_SLOT_283"),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, id),
            Some(symbol),
            "{target}: variable {id} must not inherit a guessed stocking meaning"
        );
    }
}

#[test]
fn daily_reset_only_boolean_slots_stay_explicitly_unknown() {
    for (target, id, symbol) in [
        ("MARY_FOMT_US", 423, "VAR_UNKNOWN_SLOT_423"),
        ("MARY_FOMT_JP", 423, "VAR_UNKNOWN_SLOT_423"),
        ("MARY_MFOMT_US", 467, "VAR_UNKNOWN_SLOT_467"),
        ("MARY_MFOMT_JP", 467, "VAR_UNKNOWN_SLOT_467"),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, id),
            Some(symbol),
            "{target}: reset-only slot {id} must not inherit a festival name from adjacency"
        );
        assert_eq!(
            constants.variable_value_type(id),
            constants.constant_value_type("TRUE"),
            "{target}: reset-only slot {id} must retain its proven boolean domain"
        );
    }
}

#[test]
fn unknown_multibit_fields_do_not_inherit_adjacent_boolean_or_lifecycle_types() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        if target.starts_with("MARY_FOMT_") {
            // Physical widths 3/4/4 are known; event semantics are not.
            // A nearby lifecycle enum is not evidence for these fields.
            for id in [314, 327, 333] {
                assert_eq!(constants.variable_value_type(id), None, "{target}: {id}");
            }
            assert_eq!(constants.variable_value_type(320), None);
        } else {
            assert_eq!(constants.variable_value_type(325), None);
            // Same numeric ID327 has a different meaning in the two families.
            assert_eq!(constants.variable_value_type(327), None);
        }
    }
}

#[test]
fn unknown_multibit_slots_do_not_acquire_types_from_adjacency() {
    let expected = [
        (
            "MARY_FOMT_US",
            &[
                (224, "VAR_UNKNOWN_SLOT_224"),
                (225, "VAR_UNKNOWN_SLOT_225"),
                (236, "VAR_UNKNOWN_SLOT_236"),
                (237, "VAR_UNKNOWN_SLOT_237"),
                (242, "VAR_UNKNOWN_SLOT_242"),
                (276, "VAR_UNKNOWN_SLOT_276"),
                (316, "VAR_UNKNOWN_SLOT_316"),
                (317, "VAR_UNKNOWN_SLOT_317"),
                (319, "VAR_UNKNOWN_SLOT_319"),
                (320, "VAR_UNKNOWN_SLOT_320"),
                (334, "VAR_UNKNOWN_SLOT_334"),
                (335, "VAR_UNKNOWN_SLOT_335"),
                (336, "VAR_UNKNOWN_SLOT_336"),
                (344, "VAR_UNKNOWN_SLOT_344"),
                (381, "VAR_UNKNOWN_SLOT_381"),
                (382, "VAR_UNKNOWN_SLOT_382"),
                (383, "VAR_UNKNOWN_SLOT_383"),
                (385, "VAR_UNKNOWN_SLOT_385"),
                (449, "VAR_UNKNOWN_SLOT_449"),
            ][..],
        ),
        (
            "MARY_MFOMT_US",
            &[
                (232, "VAR_UNKNOWN_SLOT_232"),
                (233, "VAR_UNKNOWN_SLOT_233"),
                (244, "VAR_UNKNOWN_SLOT_244"),
                (245, "VAR_UNKNOWN_SLOT_245"),
                (250, "VAR_UNKNOWN_SLOT_250"),
                (284, "VAR_UNKNOWN_SLOT_284"),
                (324, "VAR_UNKNOWN_SLOT_324"),
                (325, "VAR_UNKNOWN_SLOT_325"),
                (327, "VAR_UNKNOWN_SLOT_327"),
                (328, "VAR_UNKNOWN_SLOT_328"),
                (343, "VAR_UNKNOWN_SLOT_343"),
                (344, "VAR_UNKNOWN_SLOT_344"),
                (352, "VAR_UNKNOWN_SLOT_352"),
                (411, "VAR_UNKNOWN_SLOT_411"),
                (412, "VAR_UNKNOWN_SLOT_412"),
                (413, "VAR_UNKNOWN_SLOT_413"),
                (415, "VAR_UNKNOWN_SLOT_415"),
            ][..],
        ),
    ];

    for (family_target, slots) in expected {
        for target in [family_target, &family_target.replace("_US", "_JP")] {
            let options = Options::default().define(target).unwrap();
            let constants = parse_constant_header(
                &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
                &options,
            )
            .unwrap();
            let variable_type = constants.user_type("MaryVarId").unwrap();
            for (id, symbol) in slots {
                assert_eq!(
                    constants.typed_int_const_name(variable_type, *id),
                    Some(*symbol),
                    "{target}: unproven lifecycle slot {id}"
                );
                assert_eq!(
                    constants.variable_value_type(*id),
                    None,
                    "{target}: unproven value domain for slot {id}"
                );
            }
        }
    }
}

#[test]
fn harvest_festival_active_state_follows_the_gender_specific_layout() {
    for (target, id) in [
        ("MARY_FOMT_US", 420),
        ("MARY_FOMT_JP", 420),
        ("MARY_MFOMT_US", 464),
        ("MARY_MFOMT_JP", 464),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, id),
            Some("VAR_HARVEST_FESTIVAL_ACTIVE"),
            "{target}: variable {id}"
        );
    }
}

#[test]
fn new_year_sunrise_write_marker_follows_the_gender_specific_layout() {
    for (target, id) in [
        ("MARY_FOMT_US", 466),
        ("MARY_FOMT_JP", 466),
        ("MARY_MFOMT_US", 527),
        ("MARY_MFOMT_JP", 527),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, id),
            Some("VAR_NEW_YEAR_SUNRISE_SCENE_WRITE_MARKER"),
            "{target}: variable {id}"
        );
    }
}

#[test]
fn gamecube_link_update_marker_follows_the_gender_specific_layout() {
    for (target, id) in [
        ("MARY_FOMT_US", 507),
        ("MARY_FOMT_JP", 507),
        ("MARY_MFOMT_US", 593),
        ("MARY_MFOMT_JP", 593),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(variable_type, id),
            Some("VAR_GAMECUBE_LINK_UPDATE_IN_PROGRESS"),
            "{target}: variable {id}"
        );
    }
}

#[test]
fn fomt_daily_offering_confession_and_farmhouse_fields_follow_event_lifecycle() {
    let entries = [
        (572, "VAR_GOLDEN_LUMBER_WAS_ON_FARM_AT_DAY_START"),
        (573, "VAR_HARVEST_GODDESS_OFFERING_MADE_TODAY"),
        (574, "VAR_HARVEST_GODDESS_OFFERING_REWARD_CYCLE"),
        (575, "VAR_KAPPA_CUCUMBER_OFFERING_MADE_TODAY"),
        (576, "VAR_KAPPA_CUCUMBER_OFFERING_REWARD_CYCLE"),
        (577, "VAR_CHURCH_CONFESSION_USED_TODAY"),
        (578, "VAR_ENTERED_FARMHOUSE_TODAY"),
        (579, "VAR_HARVEST_GODDESS_MARRIAGE_BLESSING_RECEIVED"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn fomt_rival_wedding_routing_anniversary_rewards_and_zack_advice_follow_writers() {
    let entries = [
        (581, "VAR_RICK_AND_KAREN_RIVAL_WEDDING_ROUTING_STATE"),
        (582, "VAR_POPURI_AND_KAI_RIVAL_WEDDING_ROUTING_STATE"),
        (583, "VAR_MARY_AND_GRAY_RIVAL_WEDDING_ROUTING_STATE"),
        (584, "VAR_ELLI_AND_DOCTOR_RIVAL_WEDDING_ROUTING_STATE"),
        (585, "VAR_ANN_AND_CLIFF_RIVAL_WEDDING_ROUTING_STATE"),
        (586, "VAR_WEDDING_ANNIVERSARIES_ELAPSED"),
        (587, "VAR_HORSE_RACE_POWER_BERRY_OBTAINED"),
        (588, "VAR_FRISBEE_TOURNAMENT_POWER_BERRY_OBTAINED"),
        (589, "VAR_ZACK_EMPTY_SHIPPING_BIN_ADVICE_CYCLE"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_starry_night_host_invitation_and_event_states_follow_verified_writers() {
    let entries = [
        (501, "VAR_STARRY_NIGHT_FESTIVAL_HOST_INDEX"),
        (502, "VAR_STARRY_NIGHT_POPURI_INVITATION_HANDLED"),
        (503, "VAR_STARRY_NIGHT_ANN_INVITATION_HANDLED"),
        (504, "VAR_STARRY_NIGHT_ELLI_INVITATION_HANDLED"),
        (505, "VAR_STARRY_NIGHT_KAREN_INVITATION_HANDLED"),
        (506, "VAR_STARRY_NIGHT_MARY_INVITATION_HANDLED"),
        (507, "VAR_STARRY_NIGHT_KAI_INVITATION_HANDLED"),
        (508, "VAR_STARRY_NIGHT_CLIFF_INVITATION_HANDLED"),
        (509, "VAR_STARRY_NIGHT_DOCTOR_INVITATION_HANDLED"),
        (510, "VAR_STARRY_NIGHT_RICK_INVITATION_HANDLED"),
        (511, "VAR_STARRY_NIGHT_GRAY_INVITATION_HANDLED"),
        (512, "VAR_STARRY_NIGHT_POPURI_HOUSEHOLD_EVENT_STATE"),
        (513, "VAR_STARRY_NIGHT_ANN_HOUSEHOLD_EVENT_STATE"),
        (514, "VAR_STARRY_NIGHT_ELLI_HOUSEHOLD_EVENT_STATE"),
        (515, "VAR_STARRY_NIGHT_KAREN_HOUSEHOLD_EVENT_STATE"),
        (516, "VAR_STARRY_NIGHT_MARY_HOUSEHOLD_EVENT_STATE"),
        (517, "VAR_STARRY_NIGHT_FARMHOUSE_SPOUSE_EVENT_STATE"),
        (518, "VAR_STARRY_NIGHT_KAI_EVENT_STATE"),
        (519, "VAR_STARRY_NIGHT_CLIFF_EVENT_STATE"),
        (520, "VAR_STARRY_NIGHT_DOCTOR_EVENT_STATE"),
        (521, "VAR_STARRY_NIGHT_RICK_EVENT_STATE"),
        (522, "VAR_STARRY_NIGHT_GRAY_EVENT_STATE"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
    }
}

#[test]
fn mfomt_post_starry_night_event_recipe_and_tv_fields_follow_both_regions() {
    let entries = [
        (523, "VAR_THOMAS_STOCKING_DELIVERY_EVENT_STATE"),
        (524, "VAR_THOMAS_STOCKING_GIFT_SELECTION"),
        (525, "VAR_NEW_YEARS_EVE_NOODLE_FESTIVAL_EVENT_STATE"),
        (526, "VAR_NEW_YEAR_SUNRISE_EVENT_STATE"),
        (528, "VAR_KAREN_INTRODUCTION_EVENT_STATE"),
        (529, "VAR_POPURI_INTRODUCTION_EVENT_STATE"),
        (530, "VAR_ANN_INTRODUCTION_EVENT_STATE"),
        (531, "VAR_MARY_INTRODUCTION_EVENT_STATE"),
        (532, "VAR_ELLI_INTRODUCTION_EVENT_STATE"),
        (533, "VAR_RECIPE_LEARNED_FROM_LILLIA"),
        (534, "VAR_RECIPE_LEARNED_FROM_BARLEY"),
        (535, "VAR_RECIPE_LEARNED_FROM_SAIBARA"),
        (536, "VAR_RECIPE_LEARNED_FROM_MANNA"),
        (537, "VAR_RECIPE_LEARNED_FROM_BASIL"),
        (538, "VAR_RECIPE_LEARNED_FROM_HARRIS"),
        (539, "VAR_RECIPE_LEARNED_FROM_ELLEN"),
        (540, "VAR_RECIPE_LEARNED_FROM_SASHA"),
        (541, "VAR_RECIPE_LEARNED_FROM_DOCTOR"),
        (542, "VAR_RECIPE_LEARNED_FROM_CARTER"),
        (543, "VAR_RECIPE_LEARNED_FROM_DOUG"),
        (544, "VAR_RECIPE_LEARNED_FROM_KAI"),
        (545, "VAR_RECIPE_LEARNED_FROM_GOTZ"),
        (546, "VAR_RECIPE_LEARNED_FROM_ZACK"),
        (547, "VAR_RECIPE_LEARNED_FROM_CHEF"),
        (548, "VAR_TV_CARD_COLLECTOR_CHISATO_EPISODE_INDEX"),
        (549, "VAR_TV_MY_DEAR_PRINCESS_EPISODE_INDEX"),
        (550, "VAR_TV_DUELING_CHEFS_EPISODE_INDEX"),
        (551, "VAR_TV_FAIRY_AND_ME_HIS_STORY_EPISODE_INDEX"),
        (552, "VAR_TV_FAIRY_AND_ME_HER_STORY_EPISODE_INDEX"),
        (553, "VAR_TV_AARON_CHANGES_EPISODE_INDEX"),
        (554, "VAR_TV_MECHABOT_ULTROR_EPISODE_INDEX"),
        (555, "VAR_TV_MECHABOT_GENESIS_REMINDER_INDEX"),
        (556, "VAR_TV_STAR_LILY_BANDIT_GIRL_EPISODE_INDEX"),
        (557, "VAR_TV_ST_EMERALD_ACADEMY_EPISODE_INDEX"),
        (558, "VAR_TV_MINE_RESEARCH_GROUP_EPISODE_INDEX"),
        (559, "VAR_TV_MINERAL_TOWN_FRIENDS_PROFILE_INDEX"),
        (560, "VAR_TV_HARVEST_GODDESS_NUMBER_GAME_PROGRESS"),
        (561, "VAR_TV_FISHING_HOUR_SEGMENT_INDEX"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(variable_type, 527),
            Some("VAR_NEW_YEAR_SUNRISE_SCENE_WRITE_MARKER"),
            "{target}: sunrise scene write marker 527"
        );
    }
}

#[test]
fn mfomt_first_gamecube_link_dialogue_family_matches_received_milestones() {
    let entries = [
        (562, "VAR_GAMECUBE_LINK_VAN_DIALOGUE_PENDING"),
        (563, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_DIALOGUE_PENDING"),
        (564, "VAR_GAMECUBE_LINK_ELLEN_DIALOGUE_PENDING"),
        (565, "VAR_GAMECUBE_LINK_BARLEY_DIALOGUE_PENDING"),
        (566, "VAR_GAMECUBE_LINK_RICK_DIALOGUE_PENDING"),
        (567, "VAR_GAMECUBE_LINK_THOMAS_DIALOGUE_PENDING"),
        (568, "VAR_GAMECUBE_LINK_JEFF_DIALOGUE_PENDING"),
        (569, "VAR_GAMECUBE_LINK_CARTER_DIALOGUE_PENDING"),
        (570, "VAR_GAMECUBE_LINK_GOTZ_DIALOGUE_PENDING"),
        (571, "VAR_GAMECUBE_LINK_HARVEST_SPRITES_DIALOGUE_PENDING"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
        let topics = [
            (
                572,
                "VAR_GAMECUBE_LINK_THOMAS_COOKING_RECORD_DIALOGUE_PENDING",
            ),
            (573, "VAR_GAMECUBE_LINK_HARRIS_MURREY_DIALOGUE_PENDING"),
            (
                574,
                "VAR_GAMECUBE_LINK_CARTER_STRANGE_FOREST_CREATURE_DIALOGUE_PENDING",
            ),
            (575, "VAR_GAMECUBE_LINK_STU_VALLEY_LIZARD_DIALOGUE_PENDING"),
            (576, "VAR_GAMECUBE_LINK_BASIL_TARTAN_DIALOGUE_PENDING"),
            (
                577,
                "VAR_GAMECUBE_LINK_SASHA_HUGH_TRACK_RECORD_DIALOGUE_PENDING",
            ),
            (578, "VAR_GAMECUBE_LINK_ZACK_TAKAKURA_DIALOGUE_PENDING"),
            (579, "VAR_GAMECUBE_LINK_JEFF_CODY_DIALOGUE_PENDING"),
            (580, "VAR_GAMECUBE_LINK_BARLEY_LAND_TURTLE_DIALOGUE_PENDING"),
            (581, "VAR_GAMECUBE_LINK_LILLIA_CHIHUAHUA_DIALOGUE_PENDING"),
            (582, "VAR_GAMECUBE_LINK_DOUG_GRIFFIN_DIALOGUE_PENDING"),
            (
                583,
                "VAR_GAMECUBE_LINK_MAY_MOVING_TEDDY_BEAR_DIALOGUE_PENDING",
            ),
        ];
        for (id, symbol) in topics {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: topic-specific link dialogue slot {id}"
            );
        }
    }
}

#[test]
fn mfomt_gamecube_progress_guards_and_contest_win_counters_follow_writers() {
    let entries = [
        (592, "VAR_GAMECUBE_LINK_VAN_ALBUM_UNLOCK_DIALOGUE_PENDING"),
        (594, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_RECIPES_LEARNED"),
        (596, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_RECIPE_UNLOCKS_PENDING"),
        (597, "VAR_GAMECUBE_LINK_VAN_INTRODUCTION_AVAILABLE"),
        (598, "VAR_GAMECUBE_LINK_LOU_OR_RUBY_INTRODUCTION_AVAILABLE"),
        (599, "VAR_GAMECUBE_LINK_VACATION_VILLA_MILESTONE_RECORDED"),
        (
            600,
            "VAR_GAMECUBE_LINK_ALL_VILLAGERS_MAX_FRIENDSHIP_MILESTONE_RECORDED",
        ),
        (
            601,
            "VAR_GAMECUBE_LINK_ALL_CROPS_SHIPPED_MILESTONE_RECORDED",
        ),
        (
            602,
            "VAR_GAMECUBE_LINK_ALL_FARM_ANIMALS_MAX_AFFECTION_MILESTONE_RECORDED",
        ),
        (
            603,
            "VAR_GAMECUBE_LINK_ALL_MINERALS_SHIPPED_MILESTONE_RECORDED",
        ),
        (
            604,
            "VAR_GAMECUBE_LINK_ALL_FISH_SPECIES_CAUGHT_MILESTONE_RECORDED",
        ),
        (
            605,
            "VAR_GAMECUBE_LINK_ONE_MILLION_STEPS_MILESTONE_RECORDED",
        ),
        (606, "VAR_GAMECUBE_LINK_FIFTY_YEARS_MILESTONE_RECORDED"),
        (
            607,
            "VAR_GAMECUBE_LINK_TEN_THOUSAND_FISH_CAUGHT_MILESTONE_RECORDED",
        ),
        (
            608,
            "VAR_GAMECUBE_LINK_GOLDEN_LUMBER_ON_FARM_MILESTONE_RECORDED",
        ),
        (
            609,
            "VAR_GAMECUBE_LINK_MOUNTAIN_AND_SEASIDE_COTTAGES_MILESTONE_RECORDED",
        ),
        (610, "VAR_GAMECUBE_LINK_MYTHIC_TOOL_MILESTONE_RECORDED"),
        (611, "VAR_GAMECUBE_LINK_TEN_FRISBEE_WINS_MILESTONE_RECORDED"),
        (
            612,
            "VAR_GAMECUBE_LINK_TEN_HORSE_RACE_WINS_MILESTONE_RECORDED",
        ),
        (
            613,
            "VAR_GAMECUBE_LINK_TEN_CHICKEN_FESTIVAL_WINS_MILESTONE_RECORDED",
        ),
        (
            614,
            "VAR_GAMECUBE_LINK_TEN_COW_FESTIVAL_WINS_MILESTONE_RECORDED",
        ),
        (
            615,
            "VAR_GAMECUBE_LINK_TEN_SHEEP_FESTIVAL_WINS_MILESTONE_RECORDED",
        ),
        (
            616,
            "VAR_GAMECUBE_LINK_THOMAS_COOKING_RECORD_DIALOGUE_STAGE",
        ),
        (
            595,
            "VAR_GAMECUBE_LINK_LOU_OR_RUBY_REWARD_DIALOGUES_PENDING",
        ),
        (682, "VAR_FRISBEE_TOURNAMENT_WINS"),
        (683, "VAR_HORSE_RACE_WINS"),
        (684, "VAR_CHICKEN_FESTIVAL_WINS"),
        (685, "VAR_COW_FESTIVAL_WINS"),
        (686, "VAR_SHEEP_FESTIVAL_WINS"),
        (687, "VAR_COOKING_FESTIVAL_WINS"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        for (id, symbol) in entries {
            assert_eq!(
                constants.typed_int_const_name(variable_type, id),
                Some(symbol),
                "{target}: variable {id}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(variable_type, 593),
            Some("VAR_GAMECUBE_LINK_UPDATE_IN_PROGRESS"),
            "{target}: guarded GameCube link update slot 593"
        );
    }
}

#[test]
fn achievement_variables_follow_the_eight_slot_gender_shift() {
    let symbols = [
        "VAR_ACHIEVEMENT_WALKED_10000_STEPS_STATE",
        "VAR_ACHIEVEMENT_WALKED_100000_STEPS_STATE",
        "VAR_ACHIEVEMENT_WALKED_1000000_STEPS_STATE",
        "VAR_ACHIEVEMENT_WALKED_10000000_STEPS_STATE",
        "VAR_ACHIEVEMENT_WALKED_100000000_STEPS_STATE",
        "VAR_ACHIEVEMENT_WALKED_1000000000_STEPS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_10000_ITEMS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_100000_ITEMS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_1000000_ITEMS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_10000000_ITEMS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_100000000_ITEMS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_1000000000_ITEMS_STATE",
        "VAR_ACHIEVEMENT_SHIPPED_EVERY_ITEM_KIND_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_10000_FISH_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_100000_FISH_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_1000000_FISH_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_10000000_FISH_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_100000000_FISH_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_1000000000_FISH_STATE",
        "VAR_ACHIEVEMENT_CAUGHT_EVERY_FISH_SPECIES_STATE",
        "VAR_ACHIEVEMENT_REACHED_SPRING_MINE_B100_STATE",
        "VAR_ACHIEVEMENT_REACHED_SPRING_MINE_B200_STATE",
        "VAR_ACHIEVEMENT_REACHED_SPRING_MINE_BOTTOM_STATE",
        "VAR_ACHIEVEMENT_COLLECTED_EVERY_SPRING_MINE_ITEM_STATE",
        "VAR_ACHIEVEMENT_COLLECTED_EVERY_LAKE_MINE_ITEM_STATE",
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let base = if target.contains("MFOMT") { 353 } else { 345 };
        for (offset, symbol) in symbols.iter().enumerate() {
            assert_eq!(
                constants.typed_int_const_name(variable_type, base + offset as i64),
                Some(*symbol),
                "{target}: variable {}",
                base + offset as i64
            );
        }
    }
}

#[test]
fn goddess_kappa_and_town_reward_variables_follow_gender_shift() {
    let variables = [
        (321, "VAR_HARVEST_SPRITE_TEA_PARTY_EVENT_STATE"),
        (322, "VAR_HARVEST_GODDESS_FIRST_OFFERING_EVENT_STATE"),
        (
            323,
            "VAR_HARVEST_GODDESS_TEN_OFFERINGS_MATCHMAKING_EVENT_STATE",
        ),
        (324, "VAR_HARVEST_GODDESS_CHOSEN_AS_FAVORITE_VALUE"),
        (325, "VAR_HARVEST_GODDESS_TEN_OFFERINGS_POWER_BERRY_STATE"),
        (326, "VAR_HARVEST_GODDESS_JEWEL_EXCHANGE_STATE"),
        (328, "VAR_KAPPA_FIRST_CUCUMBER_OFFERING_STATE"),
        (329, "VAR_COLLECT_POWER_BERRY_EVENT_STATE"),
        (330, "VAR_KAPPA_MYSTIC_BERRY_REWARD_STATE"),
        (331, "VAR_KAPPA_DAILY_APPEARANCE_EVENT_STATE"),
        (332, "VAR_KAPPA_JEWEL_EXCHANGE_STATE"),
        (337, "VAR_GOLDEN_LUMBER_ANGER_DIALOGUE_VARIANT_STATE"),
        (338, "VAR_GOLDEN_LUMBER_ANGER_EVENT_TODAY_STATE"),
        (339, "VAR_JEWELS_OF_TRUTH_EXCHANGE_RETRY_STATE"),
        (340, "VAR_JEWELS_OF_TRUTH_FOUND_COUNT"),
        (341, "VAR_JEWELS_OF_TRUTH_REWARD_STATE"),
        (342, "VAR_SHOOTING_STAR_WISH_EVENT_STATE"),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let shift = if target.contains("MFOMT") { 8 } else { 0 };
        for &(fomt_id, symbol) in &variables {
            assert_eq!(
                constants.typed_int_const_name(variable_type, fomt_id + shift),
                Some(symbol),
                "{target}: variable {}",
                fomt_id + shift
            );
        }
    }
}

#[test]
fn family_romance_and_pet_variables_round_trip_on_all_targets() {
    let common = [
        (45, "VAR_PLAYER_STEP_COUNT"),
        (46, "VAR_CHILD_AGE_DAYS"),
        (47, "VAR_CHILD_CAN_WALK"),
        (48, "VAR_CHILD_BIRTHDAY_SEASON"),
        (49, "VAR_CHILD_BIRTHDAY_DAY"),
        (50, "VAR_DAYS_SINCE_PREGNANCY_EVENT"),
        (51, "VAR_WEDDING_SEASON"),
        (52, "VAR_WEDDING_DAY"),
    ];
    let fomt = [
        (53, "VAR_SPOUSE_CONTINUES_FAMILY_WORK"),
        (54, "VAR_DAYS_SINCE_KAREN_FINAL_LOVE_EVENT"),
        (55, "VAR_DAYS_SINCE_POPURI_FINAL_LOVE_EVENT"),
        (56, "VAR_DAYS_SINCE_MARY_FINAL_LOVE_EVENT"),
        (57, "VAR_DAYS_SINCE_ELLI_FINAL_LOVE_EVENT"),
        (58, "VAR_DAYS_SINCE_ANN_FINAL_LOVE_EVENT"),
        (59, "VAR_DAYS_SINCE_HARVEST_GODDESS_FINAL_LOVE_EVENT"),
        (60, "VAR_DAYS_SINCE_KAREN_FINAL_RIVAL_EVENT"),
        (61, "VAR_DAYS_SINCE_POPURI_FINAL_RIVAL_EVENT"),
        (62, "VAR_DAYS_SINCE_MARY_FINAL_RIVAL_EVENT"),
        (63, "VAR_DAYS_SINCE_ELLI_FINAL_RIVAL_EVENT"),
        (64, "VAR_DAYS_SINCE_ANN_FINAL_RIVAL_EVENT"),
        (65, "VAR_DOG_GROWTH_STAGE"),
        (66, "VAR_DOG_AFFECTION"),
        (67, "VAR_HAS_HORSE"),
        (68, "VAR_HORSE_GROWTH_STAGE"),
        (69, "VAR_HORSE_AFFECTION"),
    ];
    let mfomt = [
        (54, "VAR_DAYS_SINCE_RICK_FINAL_LOVE_EVENT"),
        (55, "VAR_DAYS_SINCE_KAI_FINAL_LOVE_EVENT"),
        (56, "VAR_DAYS_SINCE_GRAY_FINAL_LOVE_EVENT"),
        (57, "VAR_DAYS_SINCE_DOCTOR_FINAL_LOVE_EVENT"),
        (58, "VAR_DAYS_SINCE_CLIFF_FINAL_LOVE_EVENT"),
        (59, "VAR_DAYS_SINCE_KAPPA_FINAL_LOVE_EVENT"),
        (60, "VAR_DAYS_SINCE_WON_FINAL_LOVE_EVENT"),
        (61, "VAR_DAYS_SINCE_GOURMET_FINAL_LOVE_EVENT"),
        (62, "VAR_DAYS_SINCE_RICK_FINAL_RIVAL_EVENT"),
        (63, "VAR_DAYS_SINCE_KAI_FINAL_RIVAL_EVENT"),
        (64, "VAR_DAYS_SINCE_GRAY_FINAL_RIVAL_EVENT"),
        (65, "VAR_DAYS_SINCE_DOCTOR_FINAL_RIVAL_EVENT"),
        (66, "VAR_DAYS_SINCE_CLIFF_FINAL_RIVAL_EVENT"),
        (67, "VAR_DOG_GROWTH_STAGE"),
        (68, "VAR_DOG_AFFECTION"),
        (69, "VAR_HAS_HORSE"),
        (70, "VAR_HORSE_GROWTH_STAGE"),
        (71, "VAR_HORSE_AFFECTION"),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFamilyVariable, };\n", &options).unwrap();
        let target_specific = if target.contains("MFOMT") {
            &mfomt[..]
        } else {
            &fomt[..]
        };

        for &(variable_id, symbol) in common.iter().chain(target_specific) {
            let numeric = parse_named_scripts(
                &format!(
                    "void TestFamilyVariable(void) {{ if (VarGet({variable_id}) != 0) {{ return; }} }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestFamilyVariable",
            )
            .unwrap();
            let source = format_named_script("TestFamilyVariable", &raised).unwrap();
            assert!(
                source.contains(&format!("VarGet({symbol})")),
                "{target}: {source}"
            );
            let symbolic =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {symbol}"
            );
        }
    }
}

#[test]
fn heart_event_variables_follow_each_games_candidate_layout() {
    let fomt = [
        (70, "VAR_KAREN_BLACK_HEART_EVENT_STATE"),
        (71, "VAR_KAREN_PURPLE_HEART_EVENT_STATE"),
        (72, "VAR_KAREN_PURPLE_HEART_EVENT_CHOICE"),
        (73, "VAR_KAREN_BLUE_HEART_EVENT_STATE"),
        (74, "VAR_KAREN_YELLOW_HEART_EVENT_STATE"),
        (75, "VAR_KAREN_PROPOSAL_EVENT_STATE"),
        (77, "VAR_KAREN_NICKNAME_EVENT_STATE"),
        (78, "VAR_POPURI_BLACK_HEART_EVENT_STATE"),
        (79, "VAR_POPURI_PURPLE_HEART_EVENT_STATE"),
        (80, "VAR_POPURI_BLUE_HEART_EVENT_STATE"),
        (81, "VAR_POPURI_BLUE_HEART_EVENT_CHOICE"),
        (82, "VAR_POPURI_YELLOW_HEART_EVENT_STATE"),
        (83, "VAR_POPURI_YELLOW_HEART_EVENT_CHOICE"),
        (84, "VAR_POPURI_PROPOSAL_EVENT_STATE"),
        (86, "VAR_POPURI_NICKNAME_EVENT_STATE"),
        (87, "VAR_ANN_BLACK_HEART_EVENT_STATE"),
        (88, "VAR_ANN_PURPLE_HEART_EVENT_STATE"),
        (89, "VAR_ANN_PURPLE_HEART_EVENT_CHOICE"),
        (90, "VAR_ANN_BLUE_HEART_EVENT_STATE"),
        (91, "VAR_ANN_BLUE_HEART_EVENT_CHOICE"),
        (92, "VAR_ANN_YELLOW_HEART_EVENT_STATE"),
        (93, "VAR_ANN_YELLOW_HEART_EVENT_CHOICE"),
        (94, "VAR_ANN_PROPOSAL_EVENT_STATE"),
        (96, "VAR_ANN_NICKNAME_EVENT_STATE"),
        (97, "VAR_MARY_BLACK_HEART_EVENT_STATE"),
        (98, "VAR_MARY_PURPLE_HEART_EVENT_STATE"),
        (99, "VAR_MARY_PURPLE_HEART_EVENT_CHOICE"),
        (100, "VAR_MARY_BLUE_HEART_EVENT_STATE"),
        (101, "VAR_MARY_YELLOW_HEART_EVENT_STATE"),
        (102, "VAR_MARY_PROPOSAL_EVENT_STATE"),
        (104, "VAR_MARY_NICKNAME_EVENT_STATE"),
        (105, "VAR_ELLI_BLACK_HEART_EVENT_STATE"),
        (106, "VAR_ELLI_PURPLE_HEART_EVENT_STATE"),
        (107, "VAR_ELLI_BLUE_HEART_EVENT_STATE"),
        (108, "VAR_ELLI_BLUE_HEART_EVENT_CHOICE"),
        (109, "VAR_ELLI_YELLOW_HEART_EVENT_STATE"),
        (110, "VAR_ELLI_PROPOSAL_EVENT_STATE"),
        (112, "VAR_ELLI_NICKNAME_EVENT_STATE"),
    ];
    let mfomt = [
        (72, "VAR_RICK_BLACK_HEART_EVENT_STATE"),
        (73, "VAR_RICK_BLACK_HEART_EVENT_CHOICE"),
        (74, "VAR_RICK_PURPLE_HEART_EVENT_STATE"),
        (75, "VAR_RICK_PURPLE_HEART_EVENT_CHOICE"),
        (76, "VAR_RICK_BLUE_HEART_EVENT_STATE"),
        (77, "VAR_RICK_BLUE_HEART_EVENT_CHOICE"),
        (78, "VAR_RICK_YELLOW_HEART_EVENT_STATE"),
        (79, "VAR_RICK_PROPOSAL_EVENT_STATE"),
        (81, "VAR_RICK_NICKNAME_EVENT_STATE"),
        (82, "VAR_KAI_BLACK_HEART_EVENT_STATE"),
        (83, "VAR_KAI_PURPLE_HEART_EVENT_STATE"),
        (84, "VAR_KAI_PURPLE_HEART_EVENT_CHOICE"),
        (85, "VAR_KAI_BLUE_HEART_EVENT_STATE"),
        (87, "VAR_KAI_YELLOW_HEART_EVENT_STATE"),
        (88, "VAR_KAI_YELLOW_HEART_EVENT_CHOICE"),
        (89, "VAR_KAI_PROPOSAL_EVENT_STATE"),
        (91, "VAR_KAI_NICKNAME_EVENT_STATE"),
        (92, "VAR_CLIFF_BLACK_HEART_EVENT_STATE"),
        (93, "VAR_CLIFF_PURPLE_HEART_EVENT_STATE"),
        (95, "VAR_CLIFF_BLUE_HEART_EVENT_STATE"),
        (97, "VAR_CLIFF_YELLOW_HEART_EVENT_STATE"),
        (99, "VAR_CLIFF_PROPOSAL_EVENT_STATE"),
        (101, "VAR_CLIFF_NICKNAME_EVENT_STATE"),
        (102, "VAR_GRAY_BLACK_HEART_EVENT_STATE"),
        (103, "VAR_GRAY_PURPLE_HEART_EVENT_STATE"),
        (104, "VAR_GRAY_PURPLE_HEART_EVENT_CHOICE"),
        (105, "VAR_GRAY_BLUE_HEART_EVENT_STATE"),
        (106, "VAR_GRAY_YELLOW_HEART_EVENT_STATE"),
        (107, "VAR_GRAY_YELLOW_HEART_EVENT_CHOICE"),
        (108, "VAR_GRAY_PROPOSAL_EVENT_STATE"),
        (110, "VAR_GRAY_NICKNAME_EVENT_STATE"),
        (111, "VAR_DOCTOR_BLACK_HEART_EVENT_STATE"),
        (112, "VAR_DOCTOR_BLACK_HEART_EVENT_CHOICE"),
        (113, "VAR_DOCTOR_PURPLE_HEART_EVENT_STATE"),
        (114, "VAR_DOCTOR_BLUE_HEART_EVENT_STATE"),
        (116, "VAR_DOCTOR_YELLOW_HEART_EVENT_STATE"),
        (117, "VAR_DOCTOR_YELLOW_HEART_EVENT_CHOICE"),
        (118, "VAR_DOCTOR_PROPOSAL_EVENT_STATE"),
        (120, "VAR_DOCTOR_NICKNAME_EVENT_STATE"),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestHeartEvent, };\n", &options).unwrap();
        let variables = if target.contains("MFOMT") {
            &mfomt[..]
        } else {
            &fomt[..]
        };
        for &(variable_id, symbol) in variables {
            let numeric = parse_named_scripts(
                &format!(
                    "void TestHeartEvent(void) {{ if (VarGet({variable_id}) != 0) {{ return; }} }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestHeartEvent")
                    .unwrap();
            let source = format_named_script("TestHeartEvent", &raised).unwrap();
            assert!(
                source.contains(&format!("VarGet({symbol})")),
                "{target}: {source}"
            );
            let symbolic =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {symbol}"
            );
        }
    }
}

#[test]
fn rick_blue_heart_response_uses_its_literal_answer_domain() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestRickBlueHeart, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestRickBlueHeart(void) {\n\
             if (VarGet(77) == 0) { return; }\n\
             if (VarGet(77) == 1) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestRickBlueHeart")
                .unwrap();
        let source = format_named_script("TestRickBlueHeart", &raised).unwrap();
        assert!(
            source.contains("VAR_RICK_BLUE_HEART_EVENT_CHOICE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RICK_BLUE_HEART_RESPONSE_BORED"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RICK_BLUE_HEART_RESPONSE_NOT_BORED"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn gray_purple_heart_response_uses_its_literal_answer_domain() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestGrayPurpleHeart, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestGrayPurpleHeart(void) {\n\
             if (VarGet(104) == 0) { return; }\n\
             if (VarGet(104) == 1) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestGrayPurpleHeart",
        )
        .unwrap();
        let source = format_named_script("TestGrayPurpleHeart", &raised).unwrap();
        assert!(
            source.contains("VAR_GRAY_PURPLE_HEART_EVENT_CHOICE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GRAY_PURPLE_HEART_RESPONSE_FUTURE_MASTER"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GRAY_PURPLE_HEART_RESPONSE_ALREADY_GREAT"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn mary_purple_heart_response_uses_its_literal_answer_domain() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestMaryPurpleHeart, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestMaryPurpleHeart(void) {\n\
             if (VarGet(99) == 0) { return; }\n\
             if (VarGet(99) == 1) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestMaryPurpleHeart",
        )
        .unwrap();
        let source = format_named_script("TestMaryPurpleHeart", &raised).unwrap();
        assert!(
            source.contains("VAR_MARY_PURPLE_HEART_EVENT_CHOICE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("MARY_PURPLE_HEART_RESPONSE_HELP_FIND_BOOK"),
            "{target}: {source}"
        );
        assert!(
            source.contains("MARY_PURPLE_HEART_RESPONSE_REFUSE_TO_HELP"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn all_two_choice_heart_variables_use_literal_answer_domains() {
    let cases = [
        (
            "MARY_FOMT_US",
            vec![
                (
                    91,
                    "ANN_BLUE_HEART_RESPONSE_TAKE_TO_CLINIC",
                    "ANN_BLUE_HEART_RESPONSE_DO_NOT_KNOW",
                ),
                (
                    89,
                    "ANN_PURPLE_HEART_RESPONSE_LIKE_CLEANING",
                    "ANN_PURPLE_HEART_RESPONSE_HATE_CLEANING",
                ),
                (
                    93,
                    "ANN_YELLOW_HEART_RESPONSE_LIKE_ANN",
                    "ANN_YELLOW_HEART_RESPONSE_ONLY_FRIENDS",
                ),
                (
                    108,
                    "ELLI_BLUE_HEART_RESPONSE_REFUSE_TO_SEARCH",
                    "ELLI_BLUE_HEART_RESPONSE_HELP_FIND_STU",
                ),
                (
                    72,
                    "KAREN_PURPLE_HEART_RESPONSE_MOONDROP_SEEDS",
                    "KAREN_PURPLE_HEART_RESPONSE_PINK_CAT_SEEDS",
                ),
                (
                    81,
                    "POPURI_BLUE_HEART_RESPONSE_PLAY_WITH_CHILDREN",
                    "POPURI_BLUE_HEART_RESPONSE_RETURN_TO_WORK",
                ),
            ],
        ),
        (
            "MARY_MFOMT_US",
            vec![
                (
                    112,
                    "DOCTOR_BLACK_HEART_RESPONSE_REFUSE_MEDICINE",
                    "DOCTOR_BLACK_HEART_RESPONSE_TRY_MEDICINE",
                ),
                (
                    117,
                    "DOCTOR_YELLOW_HEART_RESPONSE_OPEN_YOUR_HEART",
                    "DOCTOR_YELLOW_HEART_RESPONSE_SAY_DOCTOR_IS_FINE",
                ),
                (
                    107,
                    "GRAY_YELLOW_HEART_RESPONSE_BELIEVE_HE_WILL_STAY",
                    "GRAY_YELLOW_HEART_RESPONSE_EXPRESS_UNCERTAINTY",
                ),
                (
                    88,
                    "KAI_YELLOW_HEART_RESPONSE_FORGET_ABOUT_PAST",
                    "KAI_YELLOW_HEART_RESPONSE_SUGGEST_CHANGING",
                ),
                (
                    98,
                    "CLIFF_YELLOW_HEART_DOUG_FOLLOWUP_ASK_CARE_FOR_ANN",
                    "CLIFF_YELLOW_HEART_DOUG_FOLLOWUP_OFFER_MEAL",
                ),
                (
                    73,
                    "RICK_BLACK_HEART_RESPONSE_LOVE_SPA_BOILED_EGGS",
                    "RICK_BLACK_HEART_RESPONSE_DISLIKE_SPA_BOILED_EGGS",
                ),
                (
                    75,
                    "RICK_PURPLE_HEART_RESPONSE_THREE_DAYS",
                    "RICK_PURPLE_HEART_RESPONSE_ONE_WEEK",
                ),
            ],
        ),
    ];

    for (target, variables) in cases {
        let jp_target = target.replace("_US", "_JP");
        for region_target in [target, jp_target.as_str()] {
            let options = Options::default().define(region_target).unwrap();
            let constants = parse_constant_header(
                &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
                &options,
            )
            .unwrap();
            let callables = parse_callable_table_with_scope(
                &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
                &options,
                &constants,
            )
            .unwrap();
            let script_table =
                parse_script_table("mary_script_table { TestHeartResponses, };\n", &options)
                    .unwrap();
            let mut numeric_source = String::from("void TestHeartResponses(void) {\n");
            for (variable_id, _, _) in &variables {
                numeric_source.push_str(&format!(
                    "if (VarGet({variable_id}) == 0) {{ return; }}\n\
                     if (VarGet({variable_id}) == 1) {{ return; }}\n"
                ));
            }
            numeric_source.push_str("}\n");
            let numeric =
                parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                    .unwrap();
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestHeartResponses",
            )
            .unwrap();
            let source = format_named_script("TestHeartResponses", &raised).unwrap();
            for (_, zero, one) in &variables {
                assert!(source.contains(zero), "{region_target}: {zero}: {source}");
                assert!(source.contains(one), "{region_target}: {one}: {source}");
            }
            let symbolic =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{region_target}"
            );
        }
    }
}

#[test]
fn rival_event_variables_follow_the_eight_slot_gender_shift() {
    let couples = [
        ("RICK_KAREN", 113),
        ("POPURI_KAI", 122),
        ("ANN_CLIFF", 130),
        ("MARY_GRAY", 138),
        ("ELLI_DOCTOR", 146),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let shift = if target.contains("MFOMT") { 8 } else { 0 };
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestRivalEvent, };\n", &options).unwrap();

        for (couple, fomt_base) in couples {
            let base = fomt_base + shift;
            for event in 1..=4 {
                let variable_id = base + event - 1;
                let symbol = format!("VAR_{couple}_RIVAL_EVENT_{event}_STATE");
                let numeric = parse_named_scripts(
                    &format!(
                        "void TestRivalEvent(void) {{ if (VarGet({variable_id}) != 0) {{ return; }} }}\n"
                    ),
                    &options,
                    &callables.scope,
                    &script_table,
                )
                .unwrap();
                let raised = decompile_script_named(
                    &numeric.scripts[0].2,
                    &callables.scope,
                    "TestRivalEvent",
                )
                .unwrap();
                let source = format_named_script("TestRivalEvent", &raised).unwrap();
                assert!(
                    source.contains(&format!("VarGet({symbol})")),
                    "{target}: {source}"
                );
                let symbolic =
                    parse_named_scripts(&source, &options, &callables.scope, &script_table)
                        .unwrap();
                assert_eq!(
                    encode_script(&numeric.scripts[0].2),
                    encode_script(&symbolic.scripts[0].2),
                    "{target}: {symbol}"
                );
            }

            let wedding_id = base + 4;
            let wedding_symbol = format!("VAR_{couple}_WEDDING_EVENT_STATE");
            let numeric = parse_named_scripts(
                &format!(
                    "void TestRivalEvent(void) {{ if (VarGet({wedding_id}) != 0) {{ return; }} }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestRivalEvent")
                    .unwrap();
            let source = format_named_script("TestRivalEvent", &raised).unwrap();
            assert!(
                source.contains(&format!("VarGet({wedding_symbol})")),
                "{target}: {source}"
            );
        }

        let popuri_intro_id = 118 + shift;
        let numeric = parse_named_scripts(
            &format!(
                "void TestRivalEvent(void) {{ if (VarGet({popuri_intro_id}) != 0) {{ return; }} }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestRivalEvent")
                .unwrap();
        let source = format_named_script("TestRivalEvent", &raised).unwrap();
        assert!(
            source.contains("VarGet(VAR_RICK_KAREN_WEDDING_POPURI_INTRO_EVENT_STATE)"),
            "{target}: {source}"
        );

        for (couple, fomt_marriage_id) in [
            ("RICK_KAREN", 119),
            ("POPURI_KAI", 127),
            ("ANN_CLIFF", 135),
            ("MARY_GRAY", 143),
            ("ELLI_DOCTOR", 151),
        ] {
            for (offset, suffix) in [(1, "WEDDING_ATTENDANCE_DEFERRED"), (2, "WEDDING_MISSED")] {
                let variable_id = fomt_marriage_id + shift + offset;
                let symbol = format!("VAR_{couple}_{suffix}");
                let numeric = parse_named_scripts(
                    &format!(
                        "void TestRivalEvent(void) {{ if (VarGet({variable_id}) != 0) {{ return; }} }}\n"
                    ),
                    &options,
                    &callables.scope,
                    &script_table,
                )
                .unwrap();
                let raised = decompile_script_named(
                    &numeric.scripts[0].2,
                    &callables.scope,
                    "TestRivalEvent",
                )
                .unwrap();
                let source = format_named_script("TestRivalEvent", &raised).unwrap();
                assert!(
                    source.contains(&format!("VarGet({symbol})")),
                    "{target}: {source}"
                );
            }
        }
    }
}

#[test]
fn child_and_pet_state_values_compile_to_their_numeric_bytes() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPetState, };\n", &options).unwrap();
        let dog_variable = if target.contains("MFOMT") { 67 } else { 65 };
        let numeric = parse_named_scripts(
            &format!(
                "void TestPetState(void) {{ if (VarGet(46) >= 60 && VarGet(46) >= 120 && VarGet(46) < 255 && VarGet(47) == 1 && VarGet({dog_variable}) == 1) {{ return; }} }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let symbolic = parse_named_scripts(
            &format!(
                "void TestPetState(void) {{ if (VarGet(VAR_CHILD_AGE_DAYS) >= CHILD_AGE_DAYS_FAMILY_SCENE_AND_INJURY_EVENT_START && VarGet(VAR_CHILD_AGE_DAYS) >= CHILD_AGE_DAYS_FIRST_STEPS_EVENT_START && VarGet(VAR_CHILD_AGE_DAYS) < CHILD_AGE_DAYS_SATURATED_MAXIMUM && VarGet(VAR_CHILD_CAN_WALK) == CHILD_WALKING_CAN_WALK && VarGet({dog_variable}) == PET_GROWTH_STAGE_ADULT) {{ return; }} }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );

        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPetState")
                .unwrap();
        let source = format_named_script("TestPetState", &raised).unwrap();
        for symbol in [
            "CHILD_AGE_DAYS_FAMILY_SCENE_AND_INJURY_EVENT_START",
            "CHILD_AGE_DAYS_FIRST_STEPS_EVENT_START",
            "CHILD_AGE_DAYS_SATURATED_MAXIMUM",
        ] {
            assert!(source.contains(symbol), "{target}: {source}");
        }
    }
}

#[test]
fn pregnancy_day_thresholds_compile_and_raise_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPregnancyDays, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestPregnancyDays(void) { if (VarGet(50) >= 1 && VarGet(50) >= 9 && VarGet(50) >= 20 && VarGet(50) >= 40 && VarGet(50) >= 59 && VarGet(50) >= 60) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let symbolic = parse_named_scripts(
            "void TestPregnancyDays(void) { if (VarGet(VAR_DAYS_SINCE_PREGNANCY_EVENT) >= PREGNANCY_DAYS_CLINIC_CONFIRMED_START && VarGet(VAR_DAYS_SINCE_PREGNANCY_EVENT) >= PREGNANCY_DAYS_SPOUSE_DIALOGUE_SECOND_STAGE_START && VarGet(VAR_DAYS_SINCE_PREGNANCY_EVENT) >= PREGNANCY_DAYS_CLINIC_VISIBLE_CHANGE_START && VarGet(VAR_DAYS_SINCE_PREGNANCY_EVENT) >= PREGNANCY_DAYS_LATE_STAGE_START && VarGet(VAR_DAYS_SINCE_PREGNANCY_EVENT) >= PREGNANCY_DAYS_BIRTH_IMMINENT_START && VarGet(VAR_DAYS_SINCE_PREGNANCY_EVENT) >= PREGNANCY_DAYS_CHILDBIRTH_EVENT_START) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );

        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPregnancyDays")
                .unwrap();
        let source = format_named_script("TestPregnancyDays", &raised).unwrap();
        for symbol in [
            "PREGNANCY_DAYS_CLINIC_CONFIRMED_START",
            "PREGNANCY_DAYS_SPOUSE_DIALOGUE_SECOND_STAGE_START",
            "PREGNANCY_DAYS_CLINIC_VISIBLE_CHANGE_START",
            "PREGNANCY_DAYS_LATE_STAGE_START",
            "PREGNANCY_DAYS_BIRTH_IMMINENT_START",
            "PREGNANCY_DAYS_CHILDBIRTH_EVENT_START",
        ] {
            assert!(source.contains(symbol), "{target}: {source}");
        }
    }
}

#[test]
fn shop_interface_callables_follow_all_four_target_tables() {
    let names = [
        "OpenSupermarketShop",
        "OpenWonShop",
        "OpenCarpenterShop",
        "OpenBlacksmithShop",
        "OpenClinicShop",
        "OpenBeachCafeShop",
        "OpenYodelRanchShop",
        "OpenWineryShop",
        "OpenInnShop",
        "OpenPoultryFarmShop",
        "OpenSpecialMerchantShop",
        "OpenGiftWrappingMenu",
    ];

    for (target, ids) in [
        (
            "MARY_FOMT_US",
            [
                0x08B, 0x08D, 0x08E, 0x08F, 0x090, 0x091, 0x092, 0x093, 0x094, 0x095, 0x096, 0x097,
            ],
        ),
        (
            "MARY_FOMT_JP",
            [
                0x08B, 0x08D, 0x08E, 0x08F, 0x090, 0x091, 0x092, 0x093, 0x094, 0x095, 0x096, 0x097,
            ],
        ),
        (
            "MARY_MFOMT_US",
            [
                0x08E, 0x090, 0x091, 0x092, 0x093, 0x094, 0x095, 0x096, 0x097, 0x098, 0x099, 0x09A,
            ],
        ),
        (
            "MARY_MFOMT_JP",
            [
                0x08E, 0x090, 0x091, 0x092, 0x093, 0x094, 0x095, 0x096, 0x097, 0x098, 0x099, 0x09A,
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        for (name, id) in names.iter().zip(ids) {
            assert_eq!(map[*name].0 .0, id, "{target}: {name}");
        }

        let script_table =
            parse_script_table("mary_script_table { TestShops, };\n", &options).unwrap();
        let body = format!(
            "void TestShops(void) {{ {} }}\n",
            names
                .iter()
                .map(|name| format!("{name}();"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let parsed = parse_named_scripts(&body, &options, &callables.scope, &script_table).unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestShops").unwrap();
        let source = format_named_script("TestShops", &raised).unwrap();
        for name in names {
            assert!(source.contains(&format!("{name}();")), "{target}: {source}");
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn farmhouse_storage_interfaces_follow_all_four_target_tables() {
    let names = ["OpenShelf", "OpenToolChest", "OpenRefrigerator"];

    for (target, ids) in [
        ("MARY_FOMT_US", [0x09C, 0x09D, 0x09E]),
        ("MARY_FOMT_JP", [0x09C, 0x09D, 0x09E]),
        ("MARY_MFOMT_US", [0x09F, 0x0A0, 0x0A1]),
        ("MARY_MFOMT_JP", [0x09F, 0x0A0, 0x0A1]),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        for (name, id) in names.iter().zip(ids) {
            assert_eq!(map[*name].0 .0, id, "{target}: {name}");
        }

        let script_table =
            parse_script_table("mary_script_table { TestStorage, };\n", &options).unwrap();
        let body = format!(
            "void TestStorage(void) {{ {} }}\n",
            names
                .iter()
                .map(|name| format!("{name}();"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let parsed = parse_named_scripts(&body, &options, &callables.scope, &script_table).unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestStorage").unwrap();
        let source = format_named_script("TestStorage", &raised).unwrap();
        for name in names {
            assert!(source.contains(&format!("{name}();")), "{target}: {source}");
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn book_letter_and_cooking_interfaces_follow_all_four_target_tables() {
    for (target, book_id, letter_id, cooking_id, recipes_id) in [
        ("MARY_FOMT_US", 0x099, 0x09A, 0x0A0, 0x0A1),
        ("MARY_FOMT_JP", 0x099, 0x09A, 0x0A0, 0x0A1),
        ("MARY_MFOMT_US", 0x09C, 0x09D, 0x0A3, 0x0A4),
        ("MARY_MFOMT_JP", 0x09C, 0x09D, 0x0A3, 0x0A4),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["OpenBookList"].0 .0, book_id, "{target}");
        assert_eq!(map["OpenLetterList"].0 .0, letter_id, "{target}");
        assert_eq!(map["OpenCookingMenu"].0 .0, cooking_id, "{target}");
        assert_eq!(map["OpenRecipeList"].0 .0, recipes_id, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestMenus, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestMenus(void) { OpenBookList(); OpenLetterList(); \
             OpenCookingMenu(); OpenRecipeList(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestMenus").unwrap();
        let source = format_named_script("TestMenus", &raised).unwrap();
        for name in [
            "OpenBookList",
            "OpenLetterList",
            "OpenCookingMenu",
            "OpenRecipeList",
        ] {
            assert!(source.contains(&format!("{name}();")), "{target}: {source}");
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn calendar_clock_and_gamecube_link_follow_all_four_target_tables() {
    for (target, calendar_id, clock_id, link_id) in [
        ("MARY_FOMT_US", 0x09B, 0x09F, 0x0A2),
        ("MARY_FOMT_JP", 0x09B, 0x09F, 0x0A2),
        ("MARY_MFOMT_US", 0x09E, 0x0A2, 0x0A5),
        ("MARY_MFOMT_JP", 0x09E, 0x0A2, 0x0A5),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["OpenCalendar"].0 .0, calendar_id, "{target}");
        assert_eq!(map["OpenClock"].0 .0, clock_id, "{target}");
        assert_eq!(map["RunGameCubeLink"].0 .0, link_id, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestHouseUi, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestHouseUi(void) { OpenCalendar(); OpenClock(); \
             if (RunGameCubeLink() == GAMECUBE_LINK_RESULT_SUCCESS) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestHouseUi").unwrap();
        let source = format_named_script("TestHouseUi", &raised).unwrap();
        assert!(source.contains("OpenCalendar();"), "{target}: {source}");
        assert!(source.contains("OpenClock();"), "{target}: {source}");
        assert!(
            source.contains("RunGameCubeLink() == GAMECUBE_LINK_RESULT_SUCCESS"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn rucksack_lookup_failure_uses_the_proven_shared_sentinel() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestRucksackLookup, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestRucksackLookup(void) { \
             if (FindFoodInRucksack(0) == -1) { return; } \
             if (FindArticleInRucksack(0) == -1) { return; } \
             if (GetFirstFreeRucksackToolSlot() == -1) { return; } \
             if (GetFirstFreeRucksackItemSlot() == -1) { return; } \
             ClearRucksackItemSlot(7); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestRucksackLookup",
        )
        .unwrap();
        let source = format_named_script("TestRucksackLookup", &raised).unwrap();
        assert_eq!(
            source.matches("== RUCKSACK_SLOT_NOT_FOUND").count(),
            4,
            "{target}: {source}"
        );
        assert!(source.contains("ClearRucksackItemSlot(RUCKSACK_SLOT_8)"));
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        parse_named_scripts(
            "void TestRucksackLookup(void) { \
             if (FindFoodInRucksack(FOOD_TURNIP) == RUCKSACK_SLOT_NOT_FOUND) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn festival_animal_selector_and_farming_tutorial_use_symbolic_kinds() {
    for (target, selector_id, tutorial_id) in [
        ("MARY_FOMT_US", 0x0A7, 0x0A9),
        ("MARY_FOMT_JP", 0x0A7, 0x0A9),
        ("MARY_MFOMT_US", 0x0AA, 0x0AC),
        ("MARY_MFOMT_JP", 0x0AA, 0x0AC),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["SelectFestivalAnimal"].0 .0, selector_id, "{target}");
        assert_eq!(map["OpenFarmingTutorial"].0 .0, tutorial_id, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestGuidance, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestGuidance(void) { switch (SelectFestivalAnimal(0)) { \
             case -1: OpenFarmingTutorial(1); return; \
             case 7: return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestGuidance")
                .unwrap();
        let source = format_named_script("TestGuidance", &raised).unwrap();
        assert!(
            source.contains("SelectFestivalAnimal(FESTIVAL_ANIMAL_CHICKEN)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_NONE:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_8:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("OpenFarmingTutorial(FARMING_TUTORIAL_ANIMALS)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );

        // A source-level `-1` is encoded as push(1) + negate. Named negative
        // constants must preserve that same shape while exposing semantics.
        let negative_table =
            parse_script_table("mary_script_table { TestNegativeSentinel, };\n", &options).unwrap();
        let negative = parse_named_scripts(
            "void TestNegativeSentinel(void) { \
             if (SelectFestivalAnimal(0) == -1) { return; } }\n",
            &options,
            &callables.scope,
            &negative_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &negative.scripts[0].2,
            &callables.scope,
            "TestNegativeSentinel",
        )
        .unwrap();
        let source = format_named_script("TestNegativeSentinel", &raised).unwrap();
        assert!(
            source.contains("== CHICKEN_SLOT_NONE"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &negative_table).unwrap();
        assert_eq!(
            encode_script(&negative.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn horse_race_and_frisbee_interfaces_use_symbolic_modes() {
    for (target, race_id, entries_id, exchange_id, frisbee_id, opponents_id) in [
        ("MARY_FOMT_US", 0x0D7, 0x0D8, 0x0D9, 0x0DA, 0x0DC),
        ("MARY_FOMT_JP", 0x0D7, 0x0D8, 0x0D9, 0x0DA, 0x0DC),
        ("MARY_MFOMT_US", 0x0DA, 0x0DB, 0x0DC, 0x0DD, 0x0DF),
        ("MARY_MFOMT_JP", 0x0DA, 0x0DB, 0x0DC, 0x0DD, 0x0DF),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["RunHorseRace"].0 .0, race_id, "{target}");
        assert_eq!(map["PrepareHorseRaceEntries"].0 .0, entries_id, "{target}");
        assert_eq!(
            map["OpenHorseRaceMedalExchange"].0 .0, exchange_id,
            "{target}"
        );
        assert_eq!(map["RunFrisbeeGame"].0 .0, frisbee_id, "{target}");
        assert_eq!(
            map["PrepareAnimalFestivalOpponents"].0 .0, opponents_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestFestivals, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestFestivals(void) { PrepareHorseRaceEntries(1); \
             if (RunHorseRace(1) == 1) { \
             OpenHorseRaceMedalExchange(); } if (RunFrisbeeGame(0) == 0) { return; } \
             PrepareAnimalFestivalOpponents(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestFestivals")
                .unwrap();
        let source = format_named_script("TestFestivals", &raised).unwrap();
        assert!(
            source.contains("RunHorseRace(HORSE_RACE_MODE_COMPETE)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("PrepareHorseRaceEntries(HORSE_RACE_ENTRIES_INCLUDE_PLAYER_HORSE)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("OpenHorseRaceMedalExchange();"),
            "{target}: {source}"
        );
        assert!(
            source.contains(
                "RunFrisbeeGame(FRISBEE_MODE_PRACTICE) == FESTIVAL_CONTEST_RESULT_NOT_WON"
            ),
            "{target}: {source}"
        );
        assert!(
            source.contains("PrepareAnimalFestivalOpponents();"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn call_script_id_is_printed_as_the_ordered_script_symbol() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
        &options,
        &constants,
    )
    .unwrap();
    let script_table = parse_script_table(
        "mary_script_table { EventScript_Opening, NULL, EventScript_Wedding };\n",
        &options,
    )
    .unwrap();
    let numeric = parse_named_scripts(
        "void EventScript_Wedding(void) { CallScript(2); }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let mut decorated_scope = callables.scope.clone();
    script_table.add_constants(&mut decorated_scope);
    let raised = decompile_script_named(
        &numeric.scripts[0].2,
        &decorated_scope,
        "EventScript_Wedding",
    )
    .unwrap();
    let source = format_named_script("EventScript_Wedding", &raised).unwrap();
    assert!(
        source.contains("CallScript(EventScript_Wedding);"),
        "{source}"
    );
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn typed_id_constants_are_printed_and_round_trip_to_the_same_bytes() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryPortraitId {\nTALK_PORTRAIT_KAREN_HAPPY = 73,\n} MaryPortraitId;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { SetTalkPortrait, };\n\
         void SetTalkPortrait(MaryPortraitId portrait_id);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestPortrait, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestPortrait(void) { SetTalkPortrait(73); }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPortrait").unwrap();
    let source = format_named_script("TestPortrait", &raised).unwrap();
    assert!(source.contains("SetTalkPortrait(TALK_PORTRAIT_KAREN_HAPPY);"));
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn typed_callable_return_constants_are_printed_in_comparisons() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryArticleId {\nARTICLE_GOLDEN_LUMBER = 90,\n} MaryArticleId;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { GetVaseArticleId, };\n\
         MaryArticleId GetVaseArticleId(void);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestArticle, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestArticle(void) { if (GetVaseArticleId() == 90) { return; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestArticle").unwrap();
    let source = format_named_script("TestArticle", &raised).unwrap();
    assert!(
        source.contains("GetVaseArticleId() == ARTICLE_GOLDEN_LUMBER"),
        "{source}"
    );
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn typed_callable_return_type_propagates_through_a_local_variable() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MarySeason {\nSEASON_FALL = 2,\n} MarySeason;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { GetSeason, }\n\
         MarySeason GetSeason(void);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestSeason, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestSeason(void) { int season; season = GetSeason(); if (season == 2) { return; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestSeason").unwrap();
    let source = format_named_script("TestSeason", &raised).unwrap();
    assert!(source.contains("var_0 == SEASON_FALL"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn typed_callable_return_decorates_enum_base_but_not_numeric_offset() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryMapId {\nMAP_SPRING_MINE_FLOOR_0 = 52,\n} MaryMapId;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { GetMap, }\n\
         MaryMapId GetMap(void);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table = parse_script_table("mary_script_table { TestMap, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestMap(void) { int map; map = GetMap(); if (map < 52 + 256) { return; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMap").unwrap();
    let source = format_named_script("TestMap", &raised).unwrap();
    assert!(
        source.contains("var_0 < MAP_SPRING_MINE_FLOOR_0 + 256"),
        "{source}"
    );
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn enum_type_survives_a_branch_when_the_other_path_holds_a_valid_sentinel() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MarySlot {\nSLOT_NONE = -1,\nSLOT_1 = 0,\n} MarySlot;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { GetCondition, SelectSlot, }\n\
         int GetCondition(void);\n\
         MarySlot SelectSlot(void);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table = parse_script_table("mary_script_table { TestSlot, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestSlot(void) { int slot; slot = -1; if (GetCondition()) { slot = SelectSlot(); } if (slot == -1) { return; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestSlot").unwrap();
    let source = format_named_script("TestSlot", &raised).unwrap();
    assert!(source.contains("var_0 == SLOT_NONE"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn local_enum_types_join_conservatively_across_control_flow() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryFirst {\n\
         FIRST_ONE = 1,\n\
         } MaryFirst;\n\
         typedef enum MarySecond {\n\
         SECOND_ONE = 1,\n\
         } MarySecond;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { GetCondition, GetFirst, GetSecond, }\n\
         int GetCondition(void);\n\
         MaryFirst GetFirst(void);\n\
         MarySecond GetSecond(void);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table = parse_script_table("mary_script_table { TestFlow, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestFlow(void) {\n\
         int value;\n\
         value = GetFirst();\n\
         if (GetCondition()) { value = GetSecond(); }\n\
         if (value == 1) { GetCondition(); }\n\
         value = GetFirst();\n\
         if (GetCondition()) { value = GetFirst(); }\n\
         if (value == 1) { GetCondition(); }\n\
         value = GetFirst();\n\
         for (int index = 0; index < 1; index++) { value = GetSecond(); }\n\
         if (value == 1) { GetCondition(); }\n\
         do { value = GetSecond(); } while (GetCondition());\n\
         if (value == 1) { GetCondition(); }\n\
         }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestFlow").unwrap();
    let source = format_named_script("TestFlow", &raised).unwrap();
    assert!(source.contains("var_0 == 1"), "{source}");
    assert!(source.contains("var_0 == FIRST_ONE"), "{source}");
    assert!(source.contains("var_0 == SECOND_ONE"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn calendar_values_are_printed_symbolically_for_all_targets() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestCalendar, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestCalendar(void) {\n\
             if (VarGet(1) == 3) { return; }\n\
             if (VarGet(2) == 30) { return; }\n\
             if (VarGet(5) == 6) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestCalendar")
                .unwrap();
        let source = format_named_script("TestCalendar", &raised).unwrap();

        assert!(
            source.contains("VarGet(VAR_SEASON) == SEASON_WINTER"),
            "{target}: {source}"
        );
        assert!(
            source.contains("VarGet(VAR_DAY) == DAY_OF_MONTH_30"),
            "{target}: {source}"
        );
        assert!(
            source.contains("VarGet(VAR_DAY_OF_WEEK) == DAY_OF_WEEK_SATURDAY"),
            "{target}: {source}"
        );

        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn boolean_callable_results_and_arguments_are_printed_symbolically() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryBool {\nFALSE = 0,\nTRUE = 1,\n} MaryBool;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { IsReady, SetSuspended, };\n\
         MaryBool IsReady(void);\n\
         void SetSuspended(MaryBool suspended);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestBoolean, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestBoolean(void) { if (IsReady() == 1) { SetSuspended(0); } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestBoolean").unwrap();
    let source = format_named_script("TestBoolean", &raised).unwrap();
    assert!(source.contains("IsReady() == TRUE"), "{source}");
    assert!(source.contains("SetSuspended(FALSE);"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn every_declared_boolean_callable_round_trips_true_symbolically() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let bool_type = constants.user_type("MaryBool").unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestBoolean, };\n", &options).unwrap();
        let mut boolean_callables = callables
            .scope
            .callable_map()
            .iter()
            .filter(|(_, (_, shape))| {
                shape.return_type() == mary::ir::ValueType::UserType(bool_type)
            })
            .map(|(name, (_, shape))| (name.clone(), shape.clone()))
            .collect::<Vec<_>>();
        boolean_callables.sort_by(|left, right| left.0.cmp(&right.0));

        assert!(!boolean_callables.is_empty(), "{target}");
        for (name, shape) in boolean_callables {
            let arguments = shape
                .parameter_types()
                .iter()
                .map(|parameter_type| match parameter_type {
                    mary::ir::ValueType::String => "\"\"",
                    mary::ir::ValueType::Integer | mary::ir::ValueType::UserType(_) => "0",
                    mary::ir::ValueType::Undefined => {
                        panic!("{target}: {name} has an undefined parameter type")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let numeric_source = format!(
                "void TestBoolean(void) {{ if ({name}({arguments}) == 1) {{ return; }} }}\n"
            );
            let numeric =
                parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| panic!("{target}: failed to parse {name}: {error}"));
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestBoolean")
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to decompile {name}: {error}")
                    });
            let symbolic_source = format_named_script("TestBoolean", &raised)
                .unwrap_or_else(|error| panic!("{target}: failed to format {name}: {error}"));

            assert!(
                symbolic_source.contains("== TRUE"),
                "{target}: {name} did not recover TRUE: {symbolic_source}"
            );
            let symbolic =
                parse_named_scripts(&symbolic_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| panic!("{target}: failed to reparse {name}: {error}"));
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {name}"
            );
        }
    }
}

#[test]
fn every_fixed_enum_return_callable_recovers_a_symbol_and_round_trips() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestTypedReturn, };\n", &options).unwrap();
        let mut typed_callables = callables
            .scope
            .callable_map()
            .iter()
            .filter_map(|(name, (_, shape))| match shape.return_type() {
                mary::ir::ValueType::UserType(type_id)
                    if !constants_source
                        .contains(&format!("mary_callable_return_type_when({name},")) =>
                {
                    Some((name.clone(), shape.clone(), type_id))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        typed_callables.sort_by(|left, right| left.0.cmp(&right.0));

        let mut representative_by_type = HashMap::new();
        let mut tested = 0usize;
        for (name, shape, type_id) in typed_callables {
            let representative = representative_by_type.entry(type_id).or_insert_with(|| {
                (-32768i64..=65535).find_map(|value| {
                    constants
                        .typed_int_const_name(type_id, value)
                        .map(|symbol| (value, symbol.to_owned()))
                })
            });
            let Some((value, symbol)) = representative else {
                continue;
            };
            let arguments = shape
                .parameter_types()
                .iter()
                .map(|parameter_type| match parameter_type {
                    mary::ir::ValueType::String => "\"\"",
                    mary::ir::ValueType::Integer | mary::ir::ValueType::UserType(_) => "0",
                    mary::ir::ValueType::Undefined => {
                        panic!("{target}: {name} has an undefined parameter type")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let numeric_source = format!(
                "void TestTypedReturn(void) {{ if ({name}({arguments}) == {value}) {{ return; }} }}\n"
            );
            let numeric =
                parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| panic!("{target}: failed to parse {name}: {error}"));
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestTypedReturn")
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to decompile {name}: {error}")
                    });
            let symbolic_source = format_named_script("TestTypedReturn", &raised)
                .unwrap_or_else(|error| panic!("{target}: failed to format {name}: {error}"));

            assert!(
                symbolic_source.contains(&format!("== {symbol}")),
                "{target}: {name} did not recover {symbol}: {symbolic_source}"
            );
            let symbolic =
                parse_named_scripts(&symbolic_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| panic!("{target}: failed to reparse {name}: {error}"));
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {name}"
            );
            tested += 1;
        }
        assert!(
            tested > 0,
            "{target}: no fixed enum return callable was tested"
        );
    }
}

#[test]
fn plain_integer_returns_are_explicitly_numeric_or_dynamically_typed() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let shared_expected = [
        "GetEventContextValue",
        "RandomIntInclusive",
        "RandomU15",
        "VarGet",
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let mut expected = shared_expected.to_vec();
        expected.sort_unstable();
        let mut actual = callables
            .scope
            .callable_map()
            .iter()
            .filter_map(|(name, (_, shape))| {
                if shape.return_type() != mary::ir::ValueType::Integer
                    || constants_source.contains(&format!("mary_callable_return_type_when({name},"))
                    || constants_source
                        .contains(&format!("mary_callable_return_type_when_callable({name},"))
                {
                    None
                } else {
                    Some(name.as_str())
                }
            })
            .collect::<Vec<_>>();
        actual.sort_unstable();
        assert_eq!(
            actual, expected,
            "{target}: a plain integer return was added, removed, or left without semantic typing"
        );
    }
}

#[test]
fn semantic_scalar_returns_keep_distinct_types_without_output_wrappers() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let shared = [
        ("GetNpcFriendship", "MaryNpcFriendshipValue"),
        (
            "GetDaysSinceLastSpokenToNpc",
            "MaryDaysSinceNpcConversation",
        ),
        ("GetCharacterLove", "MaryCharacterLoveValue"),
        ("GetIncubatorCapacity", "MaryIncubatorCapacity"),
        ("GetPregnancyStallCapacity", "MaryPregnancyStallCapacity"),
        ("GetHarvestSpriteWorkDaysLeft", "MaryHarvestSpriteWorkDays"),
        (
            "GetHarvestSpriteTaskExperience",
            "MaryHarvestSpriteTaskExperience",
        ),
        ("GetTotalFishCaught", "MaryFishCount"),
        ("GetMoney", "MaryMoneyBalance"),
        ("GetWaitingLetterCount", "MaryLetterCount"),
        ("GetSavedLetterCount", "MaryLetterCount"),
        (
            "GetAnimalHealthyPregnancyDays",
            "MaryAnimalHealthyPregnancyDays",
        ),
        ("GetAnimalAge", "MaryAnimalAgeDays"),
        ("GetAnimalAffection", "MaryAnimalAffectionValue"),
        ("CountAnimalsByLifeState", "MaryAnimalCount"),
        ("GetCowCount", "MaryAnimalCount"),
        ("GetSheepCount", "MaryAnimalCount"),
        ("GetChickenCount", "MaryAnimalCount"),
        ("GetCaughtFishSize", "MaryFishSize"),
        ("GetAmountShipped", "MaryProductShippedCount"),
        ("GetKnownRecipeCount", "MaryKnownRecipeCount"),
    ];
    let mfomt_only = [
        ("GetFishCatchCount", "MaryFishCount"),
        ("GetLargestCaughtFishSize", "MaryFishSize"),
        ("GetToolExperience", "MaryToolExperienceValue"),
        (
            "CountFestivalWinningAnimals",
            "MaryFestivalWinningAnimalCount",
        ),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        for &(name, type_name) in &shared {
            assert_eq!(
                callables.scope.callable_map()[name].1.return_type(),
                mary::ir::ValueType::UserType(constants.user_type(type_name).unwrap()),
                "{target}: {name}"
            );
        }
        if target.starts_with("MARY_MFOMT_") {
            for &(name, type_name) in &mfomt_only {
                assert_eq!(
                    callables.scope.callable_map()[name].1.return_type(),
                    mary::ir::ValueType::UserType(constants.user_type(type_name).unwrap()),
                    "{target}: {name}"
                );
            }
        }
        assert!(!constants_source.contains("mary_typed_identity(MaryMoneyBalance"));
        assert!(!constants_source.contains("mary_typed_identity(MaryAnimalAffectionValue"));
        assert!(!constants_source.contains("mary_typed_identity(MaryFishCount"));
    }
}

#[test]
fn plain_integer_parameters_are_an_explicit_audited_allowlist() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let expected = [
        "NoOp014",
        "NoOpAnimalEventEntityInitialization",
        "NoOpTutorialEggDefinition",
        "NoOpTutorialEggSelection",
        "NoOpTutorialFieldObject",
        "NoOpTutorialFieldTile",
        "OpenRucksackMenu",
        "RandomIntInclusive",
        "RemoveFarmHorse",
        "SetTextVariableNumber",
        "SetTextVariableNumberFieldWidth",
        "VarSet",
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let mut actual = callables
            .scope
            .callable_map()
            .iter()
            .filter_map(|(name, (_, shape))| {
                shape
                    .parameter_types()
                    .contains(&mary::ir::ValueType::Integer)
                    .then_some(name.as_str())
            })
            .collect::<Vec<_>>();
        actual.sort_unstable();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        assert_eq!(
            actual, expected,
            "{target}: a plain integer parameter was added, removed, or left without semantic typing"
        );
    }
}

#[test]
fn semantic_scalar_parameters_keep_distinct_types_without_output_wrappers() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let expected = [
        ("WaitFrames", &["MaryFrameCount"][..]),
        (
            "ChangePlayerStaminaAndFatigue",
            &["MaryStaminaDelta", "MaryFatigueDelta"][..],
        ),
        (
            "AddNpcFriendship",
            &["MaryCharacterId", "MaryNpcFriendshipDelta"][..],
        ),
        (
            "SetNpcFriendship",
            &["MaryCharacterId", "MaryNpcFriendshipValue"][..],
        ),
        (
            "AddCharacterLove",
            &["MaryCharacterId", "MaryCharacterLoveDelta"][..],
        ),
        (
            "SetCharacterLove",
            &["MaryCharacterId", "MaryCharacterLoveValue"][..],
        ),
        (
            "StartHarvestSpriteTask",
            &[
                "MaryCharacterId",
                "MaryHarvestSpriteTask",
                "MaryHarvestSpriteWorkDays",
            ][..],
        ),
        ("AddMoney", &["MaryMoneyAmount"][..]),
        ("SubtractMoney", &["MaryMoneyAmount"][..]),
        ("SetGameTime", &["MaryClockHour", "MaryClockMinute"][..]),
        (
            "AddAnimalAffection",
            &[
                "MaryAnimalKind",
                "MaryAnimalSlotIndex",
                "MaryAnimalAffectionDelta",
            ][..],
        ),
        (
            "AddAffectionToAllFarmAnimals",
            &["MaryAnimalAffectionDelta"][..],
        ),
        (
            "GenerateMineFloorLayout",
            &["MaryMineKind", "MaryMineFloorIndex"][..],
        ),
        (
            "SetTextVariableNumberFieldWidth",
            &["MaryTextVariableSlot", "int", "MaryTextNumberFieldWidth"][..],
        ),
        (
            "SetPlayerHeldTool",
            &["MaryToolId", "MaryRequestedToolStackCount"][..],
        ),
        (
            "AddArticleToRucksack",
            &["MaryArticleId", "MaryRequestedInventoryCount"][..],
        ),
        (
            "AddFoodToRucksack",
            &["MaryFoodId", "MaryRequestedInventoryCount"][..],
        ),
        (
            "AddToolToRucksack",
            &["MaryToolId", "MaryRequestedInventoryCount"][..],
        ),
        (
            "OpenNameEntry",
            &["MaryNameEntryKind", "MaryNameEntryTargetIndex"][..],
        ),
        (
            "FlashScreenColor",
            &["MaryRgb5Channel", "MaryRgb5Channel", "MaryRgb5Channel"][..],
        ),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();

        for &(name, type_names) in &expected {
            let actual = callables.scope.callable_map()[name]
                .1
                .parameter_types()
                .to_vec();
            let wanted = type_names
                .iter()
                .map(|type_name| {
                    if *type_name == "int" {
                        mary::ir::ValueType::Integer
                    } else {
                        mary::ir::ValueType::UserType(constants.user_type(type_name).unwrap())
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, wanted, "{target}: {name}");
        }

        assert_ne!(
            constants.user_type("MaryRequestedToolStackCount").unwrap(),
            constants.user_type("MaryHeldToolStackCount").unwrap(),
            "{target}: setter input and getter result domains must remain distinct"
        );

        assert!(!constants_source.contains("mary_typed_identity(MaryFrameCount"));
        assert!(!constants_source.contains("mary_typed_identity(MaryClockHour"));
        assert!(!constants_source.contains("mary_typed_identity(MaryAnimalAffectionDelta"));
    }
}

#[test]
fn every_parameter_dependent_return_rule_recovers_its_declared_domain() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let rules = constants_source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("mary_callable_return_type_when(")
                .and_then(|line| line.strip_suffix(");"))
                .map(|arguments| arguments.split(',').map(str::trim).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    assert!(!rules.is_empty());

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestDependentReturn, };\n", &options).unwrap();
        let mut tested = 0usize;

        for rule in &rules {
            assert_eq!(rule.len(), 4, "malformed dependent return rule: {rule:?}");
            let callable_name = rule[0];
            let discriminator_index = rule[1].parse::<usize>().unwrap();
            let discriminator_symbol = rule[2];
            let return_type_name = rule[3];
            let Some(discriminator_value) = constants.const_int_value(discriminator_symbol) else {
                continue;
            };
            let Some(return_type_id) = constants.user_type(return_type_name) else {
                continue;
            };
            let (_, shape) = callables
                .scope
                .callable_map()
                .get(callable_name)
                .unwrap_or_else(|| panic!("{target}: missing callable {callable_name}"));
            assert!(
                discriminator_index < shape.num_parameters(),
                "{target}: {callable_name} discriminator index {discriminator_index} is out of range"
            );
            let (return_value, return_symbol) = (-32768i64..=65535)
                .find_map(|value| {
                    constants
                        .typed_int_const_name(return_type_id, value)
                        .map(|symbol| (value, symbol.to_owned()))
                })
                .unwrap_or_else(|| {
                    panic!("{target}: dependent return type {return_type_name} has no constants")
                });
            let arguments = shape
                .parameter_types()
                .iter()
                .enumerate()
                .map(|(index, parameter_type)| {
                    if index == discriminator_index {
                        discriminator_value.to_string()
                    } else {
                        match parameter_type {
                            mary::ir::ValueType::String => "\"\"".to_owned(),
                            mary::ir::ValueType::Integer | mary::ir::ValueType::UserType(_) => {
                                "0".to_owned()
                            }
                            mary::ir::ValueType::Undefined => {
                                panic!("{target}: {callable_name} has an undefined parameter type")
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let numeric_source = format!(
                "void TestDependentReturn(void) {{ if ({callable_name}({arguments}) == {return_value}) {{ return; }} }}\n"
            );
            let numeric =
                parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to parse {callable_name}: {error}")
                    });
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestDependentReturn",
            )
            .unwrap_or_else(|error| {
                panic!("{target}: failed to decompile {callable_name}: {error}")
            });
            let symbolic_source = format_named_script("TestDependentReturn", &raised)
                .unwrap_or_else(|error| {
                    panic!("{target}: failed to format {callable_name}: {error}")
                });

            assert!(
                symbolic_source.contains(&format!("== {return_symbol}")),
                "{target}: {callable_name} with {discriminator_symbol} did not recover {return_symbol}: {symbolic_source}"
            );
            let symbolic =
                parse_named_scripts(&symbolic_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to reparse {callable_name}: {error}")
                    });
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {callable_name} with {discriminator_symbol}"
            );
            tested += 1;
        }
        assert_eq!(
            tested,
            rules.len(),
            "{target}: not every rule was applicable"
        );
    }
}

#[test]
fn every_parameter_dependent_parameter_rule_recovers_its_declared_domain() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let rules = constants_source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("mary_callable_parameter_type_when(")
                .and_then(|line| line.strip_suffix(");"))
                .map(|arguments| arguments.split(',').map(str::trim).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    assert!(!rules.is_empty());

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestDependentParameter, };\n", &options)
                .unwrap();
        let mut tested = 0usize;

        for rule in &rules {
            assert_eq!(
                rule.len(),
                5,
                "malformed dependent parameter rule: {rule:?}"
            );
            let callable_name = rule[0];
            let discriminator_index = rule[1].parse::<usize>().unwrap();
            let discriminator_symbol = rule[2];
            let target_index = rule[3].parse::<usize>().unwrap();
            let target_type_name = rule[4];
            let Some(discriminator_value) = constants.const_int_value(discriminator_symbol) else {
                continue;
            };
            let Some(target_type_id) = constants.user_type(target_type_name) else {
                continue;
            };
            let (_, shape) = callables
                .scope
                .callable_map()
                .get(callable_name)
                .unwrap_or_else(|| panic!("{target}: missing callable {callable_name}"));
            assert!(
                discriminator_index < shape.num_parameters(),
                "{target}: {callable_name} discriminator index {discriminator_index} is out of range"
            );
            assert!(
                target_index < shape.num_parameters(),
                "{target}: {callable_name} target index {target_index} is out of range"
            );
            assert_ne!(
                discriminator_index, target_index,
                "{target}: {callable_name} cannot refine its discriminator parameter"
            );
            let (target_value, target_symbol) = (-32768i64..=65535)
                .find_map(|value| {
                    constants
                        .typed_int_const_name(target_type_id, value)
                        .map(|symbol| (value, symbol.to_owned()))
                })
                .unwrap_or_else(|| {
                    panic!("{target}: dependent parameter type {target_type_name} has no constants")
                });
            let arguments = shape
                .parameter_types()
                .iter()
                .enumerate()
                .map(|(index, parameter_type)| {
                    if index == discriminator_index {
                        discriminator_value.to_string()
                    } else if index == target_index {
                        target_value.to_string()
                    } else {
                        match parameter_type {
                            mary::ir::ValueType::String => "\"\"".to_owned(),
                            mary::ir::ValueType::Integer | mary::ir::ValueType::UserType(_) => {
                                "0".to_owned()
                            }
                            mary::ir::ValueType::Undefined => {
                                panic!("{target}: {callable_name} has an undefined parameter type")
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let statement = if shape.is_func() {
                format!("int result = {callable_name}({arguments});")
            } else {
                format!("{callable_name}({arguments});")
            };
            let numeric_source = format!("void TestDependentParameter(void) {{ {statement} }}\n");
            let numeric =
                parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to parse {callable_name}: {error}")
                    });
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestDependentParameter",
            )
            .unwrap_or_else(|error| {
                panic!("{target}: failed to decompile {callable_name}: {error}")
            });
            let symbolic_source = format_named_script("TestDependentParameter", &raised)
                .unwrap_or_else(|error| {
                    panic!("{target}: failed to format {callable_name}: {error}")
                });

            assert!(
                symbolic_source.contains(target_symbol.as_str()),
                "{target}: {callable_name} with {discriminator_symbol} did not recover {target_symbol}: {symbolic_source}"
            );
            let symbolic =
                parse_named_scripts(&symbolic_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to reparse {callable_name}: {error}")
                    });
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {callable_name} with {discriminator_symbol}"
            );
            tested += 1;
        }
        assert_eq!(
            tested,
            rules.len(),
            "{target}: not every rule was applicable"
        );
    }
}

#[test]
fn every_callable_dependent_return_rule_recovers_its_declared_domain() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");
    let rules = constants_source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("mary_callable_return_type_when_callable(")
                .and_then(|line| line.strip_suffix(");"))
                .map(|arguments| arguments.split(',').map(str::trim).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    assert!(!rules.is_empty());

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table = parse_script_table(
            "mary_script_table { TestCallableDependentReturn, };\n",
            &options,
        )
        .unwrap();
        let mut tested = 0usize;

        for rule in &rules {
            assert_eq!(rule.len(), 4, "malformed callable-dependent rule: {rule:?}");
            let value_callable = rule[0];
            let discriminator_callable = rule[1];
            let discriminator_symbol = rule[2];
            let return_type_name = rule[3];
            let Some(discriminator_value) = constants.const_int_value(discriminator_symbol) else {
                continue;
            };
            let Some(return_type_id) = constants.user_type(return_type_name) else {
                continue;
            };
            let (_, value_shape) = callables
                .scope
                .callable_map()
                .get(value_callable)
                .unwrap_or_else(|| panic!("{target}: missing callable {value_callable}"));
            let (_, discriminator_shape) = callables
                .scope
                .callable_map()
                .get(discriminator_callable)
                .unwrap_or_else(|| panic!("{target}: missing callable {discriminator_callable}"));
            assert!(
                value_shape.is_func(),
                "{target}: {value_callable} is not a function"
            );
            assert!(
                discriminator_shape.is_func(),
                "{target}: {discriminator_callable} is not a function"
            );
            let (return_value, return_symbol) = (-32768i64..=65535)
                .find_map(|value| {
                    constants
                        .typed_int_const_name(return_type_id, value)
                        .map(|symbol| (value, symbol.to_owned()))
                })
                .unwrap_or_else(|| {
                    panic!("{target}: dependent return type {return_type_name} has no constants")
                });
            let default_arguments = |shape: &mary::ir::CallableShape| {
                shape
                    .parameter_types()
                    .iter()
                    .map(|parameter_type| match parameter_type {
                        mary::ir::ValueType::String => "\"\"",
                        mary::ir::ValueType::Integer | mary::ir::ValueType::UserType(_) => "0",
                        mary::ir::ValueType::Undefined => panic!(
                            "{target}: callable-dependent metadata references an undefined parameter"
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let value_arguments = default_arguments(value_shape);
            let discriminator_arguments = default_arguments(discriminator_shape);
            let numeric_source = format!(
                "void TestCallableDependentReturn(void) {{ if ({discriminator_callable}({discriminator_arguments}) == {discriminator_value}) {{ if ({value_callable}({value_arguments}) == {return_value}) {{ return; }} }} }}\n"
            );
            let numeric =
                parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to parse {value_callable}: {error}")
                    });
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestCallableDependentReturn",
            )
            .unwrap_or_else(|error| {
                panic!("{target}: failed to decompile {value_callable}: {error}")
            });
            let symbolic_source = format_named_script("TestCallableDependentReturn", &raised)
                .unwrap_or_else(|error| {
                    panic!("{target}: failed to format {value_callable}: {error}")
                });

            assert!(
                symbolic_source.contains(discriminator_symbol),
                "{target}: {discriminator_callable} did not recover {discriminator_symbol}: {symbolic_source}"
            );
            assert!(
                symbolic_source.contains(&format!("== {return_symbol}")),
                "{target}: {value_callable} did not recover {return_symbol}: {symbolic_source}"
            );
            let symbolic =
                parse_named_scripts(&symbolic_source, &options, &callables.scope, &script_table)
                    .unwrap_or_else(|error| {
                        panic!("{target}: failed to reparse {value_callable}: {error}")
                    });
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}: {value_callable} with {discriminator_symbol}"
            );
            tested += 1;
        }
        assert_eq!(
            tested,
            rules.len(),
            "{target}: not every rule was applicable"
        );
    }
}

#[test]
fn every_fixed_enum_parameter_recovers_a_symbol_and_round_trips() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestTypedParameter, };\n", &options).unwrap();
        let mut callable_shapes = callables
            .scope
            .callable_map()
            .iter()
            .filter(|(name, _)| {
                !constants_source.contains(&format!("mary_callable_parameter_type_when({name},"))
            })
            .map(|(name, (_, shape))| (name.clone(), shape.clone()))
            .collect::<Vec<_>>();
        callable_shapes.sort_by(|left, right| left.0.cmp(&right.0));

        let mut representative_by_type = HashMap::new();
        let mut tested = 0usize;
        for (name, shape) in callable_shapes {
            for (parameter_index, parameter_type) in shape.parameter_types().iter().enumerate() {
                let mary::ir::ValueType::UserType(type_id) = parameter_type else {
                    continue;
                };
                let representative = representative_by_type.entry(*type_id).or_insert_with(|| {
                    (-32768i64..=65535).find_map(|value| {
                        constants
                            .typed_int_const_name(*type_id, value)
                            .map(|symbol| (value, symbol.to_owned()))
                    })
                });
                let Some((value, symbol)) = representative else {
                    continue;
                };
                let arguments = shape
                    .parameter_types()
                    .iter()
                    .enumerate()
                    .map(|(index, argument_type)| {
                        if index == parameter_index {
                            value.to_string()
                        } else {
                            match argument_type {
                                mary::ir::ValueType::String => "\"\"".to_owned(),
                                mary::ir::ValueType::Integer | mary::ir::ValueType::UserType(_) => {
                                    "0".to_owned()
                                }
                                mary::ir::ValueType::Undefined => {
                                    panic!("{target}: {name} has an undefined parameter type")
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let statement = if shape.is_func() {
                    format!("int result = {name}({arguments});")
                } else {
                    format!("{name}({arguments});")
                };
                let numeric_source = format!("void TestTypedParameter(void) {{ {statement} }}\n");
                let numeric = parse_named_scripts(
                    &numeric_source,
                    &options,
                    &callables.scope,
                    &script_table,
                )
                .unwrap_or_else(|error| {
                    panic!("{target}: failed to parse {name} parameter {parameter_index}: {error}")
                });
                let raised = decompile_script_named(
                    &numeric.scripts[0].2,
                    &callables.scope,
                    "TestTypedParameter",
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "{target}: failed to decompile {name} parameter {parameter_index}: {error}"
                    )
                });
                let symbolic_source = format_named_script("TestTypedParameter", &raised)
                    .unwrap_or_else(|error| {
                        panic!(
                            "{target}: failed to format {name} parameter {parameter_index}: {error}"
                        )
                    });

                assert!(
                    symbolic_source.contains(symbol.as_str()),
                    "{target}: {name} parameter {parameter_index} did not recover {symbol}: {symbolic_source}"
                );
                let symbolic = parse_named_scripts(
                    &symbolic_source,
                    &options,
                    &callables.scope,
                    &script_table,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "{target}: failed to reparse {name} parameter {parameter_index}: {error}"
                    )
                });
                assert_eq!(
                    encode_script(&numeric.scripts[0].2),
                    encode_script(&symbolic.scripts[0].2),
                    "{target}: {name} parameter {parameter_index}"
                );
                tested += 1;
            }
        }
        assert!(tested > 0, "{target}: no fixed enum parameter was tested");
    }
}

#[test]
fn inventory_predicates_and_rucksack_level_use_their_verified_domains() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestInventoryDomains, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestInventoryDomains(void) {\n\
             if (PlayerOwnsTool(0) == 0) { return; }\n\
             if (PlayerOwnsFood(0) == 1) { return; }\n\
             if (PlayerOwnsArticle(0) == 0) { return; }\n\
             if (PlayerHasBasket() == 1) { return; }\n\
             if (RecordPlayerHasAlbum() == 0) { return; }\n\
             if (UnlockNextVanAlbum() == 1) { return; }\n\
             if (AdvanceCursedToolLiftProgress(5) == 0) { return; }\n\
             if (GetRucksackUpgradeLevel() == 2) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestInventoryDomains",
        )
        .unwrap();
        let source = format_named_script("TestInventoryDomains", &raised).unwrap();

        assert!(
            source.contains("PlayerOwnsTool(TOOL_SICKLE_IRON) == FALSE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("PlayerOwnsFood(FOOD_TURNIP) == TRUE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("PlayerOwnsArticle(ARTICLE_FLOWER_MOON_DROP) == FALSE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("PlayerHasBasket() == TRUE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RecordPlayerHasAlbum() == FALSE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("UnlockNextVanAlbum() == TRUE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AdvanceCursedToolLiftProgress(TOOL_SICKLE_CURSED) == FALSE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GetRucksackUpgradeLevel() == RUCKSACK_UPGRADE_LARGE"),
            "{target}: {source}"
        );

        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn minigame_and_horse_race_results_use_their_verified_domains() {
    let constants_source = include_str!("../goodies/mary_constants.mary.h");
    let callables_source = include_str!("../goodies/mary_callables.mary.h");

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(constants_source, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(callables_source, &options, &constants).unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestMinigameDomains, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestMinigameDomains(void) {\n\
             int result;\n\
             if (RunHarvestSpriteWateringMinigame(39) == 1) { return; }\n\
             if (RunChickenFestivalContest() == 0) { return; }\n\
             result = RunHorseRace(1);\n\
             switch (result) {\n\
             case -1: return;\n\
             case 0: return;\n\
             case 1: return;\n\
             case 2: return;\n\
             }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestMinigameDomains",
        )
        .unwrap();
        let source = format_named_script("TestMinigameDomains", &raised).unwrap();

        assert!(
            source.contains("RunHarvestSpriteWateringMinigame(CHARACTER_CHEF) == TRUE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RunChickenFestivalContest() == FESTIVAL_CONTEST_RESULT_NOT_WON"),
            "{target}: {source}"
        );
        for result in [
            "HORSE_RACE_INTERFACE_CANCELLED",
            "HORSE_RACE_INTERFACE_CLOSED_WITHOUT_RESULT",
            "HORSE_RACE_INTERFACE_PLAYER_WON",
            "HORSE_RACE_INTERFACE_PLAYER_LOST",
        ] {
            assert!(
                source.contains(&format!("case {result}:")),
                "{target}: {source}"
            );
        }

        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn typed_callable_return_constants_are_printed_in_switch_cases() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryMapId {\nMAP_NONE = 564,\n} MaryMapId;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { GetPreservedPlayerMapId, };\n\
         MaryMapId GetPreservedPlayerMapId(void);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table = parse_script_table("mary_script_table { TestMap, };\n", &options).unwrap();
    let numeric = parse_named_scripts(
        "void TestMap(void) { switch (GetPreservedPlayerMapId()) { case 564: return; default: break; } }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMap").unwrap();
    let source = format_named_script("TestMap", &raised).unwrap();
    assert!(source.contains("case MAP_NONE:"), "{source}");
    let symbolic = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&symbolic.scripts[0].2)
    );
}

#[test]
fn project_dynamic_presented_item_kind_prints_and_round_trips() {
    for (target, kind_id, item_id, wrapped_id) in [
        ("MARY_FOMT_US", 0x06C, 0x06D, 0x06E),
        ("MARY_FOMT_JP", 0x06C, 0x06D, 0x06E),
        ("MARY_MFOMT_US", 0x06D, 0x06E, 0x06F),
        ("MARY_MFOMT_JP", 0x06D, 0x06E, 0x06F),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["GetPresentedItemKind"].0 .0, kind_id, "{target}");
        assert_eq!(map["GetPresentedItemId"].0 .0, item_id, "{target}");
        assert_eq!(
            map["IsPresentedItemGiftWrapped"].0 .0, wrapped_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestPresent, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestPresent(void) {\n\
             switch (GetPresentedItemKind()) { case 0: return; case 1: return; default: break; }\n\
             if (IsPresentedItemGiftWrapped()) { GetPresentedItemId(); }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPresent").unwrap();
        let source = format_named_script("TestPresent", &raised).unwrap();
        assert!(
            source.contains("case HELD_ITEM_KIND_FOOD:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case HELD_ITEM_KIND_ARTICLE:"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn presented_item_id_uses_the_kind_branch_domain_and_round_trips() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPresentedId, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestPresentedId(void) { int kind = GetPresentedItemKind(); int id = GetPresentedItemId(); switch (kind) { case 0: switch (id) { case 0: return; default: break; } break; case 1: switch (id) { case 0: return; default: break; } break; default: break; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPresentedId")
                .unwrap();
        let source = format_named_script("TestPresentedId", &raised).unwrap();
        for symbol in [
            "case HELD_ITEM_KIND_FOOD:",
            "case FOOD_TURNIP:",
            "case HELD_ITEM_KIND_ARTICLE:",
            "case ARTICLE_FLOWER_MOON_DROP:",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: related callable return typing changed emitted bytecode"
        );
    }
}

#[test]
fn presented_item_id_refinement_works_in_if_and_else_branches() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPresentedIf, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestPresentedIf(void) { int kind = GetPresentedItemKind(); int id = GetPresentedItemId(); if (kind == 0) { switch (id) { case 0: return; default: break; } } else { if (kind == 1) { switch (id) { case 0: return; default: break; } } } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPresentedIf")
                .unwrap();
        let source = format_named_script("TestPresentedIf", &raised).unwrap();
        for symbol in [
            "HELD_ITEM_KIND_FOOD",
            "case FOOD_TURNIP:",
            "HELD_ITEM_KIND_ARTICLE",
            "case ARTICLE_FLOWER_MOON_DROP:",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: if-branch callable refinement changed emitted bytecode"
        );
    }
}

#[test]
fn presented_item_id_refinement_works_with_direct_discriminator_calls() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPresentedDirect, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestPresentedDirect(void) { int kind = GetPresentedItemKind(); int id = GetPresentedItemId(); if (kind == 0 && id == 0) { TalkClose(); } if (kind != 0 || id == 0) { TalkClose(); } if (GetPresentedItemKind() == 0) { switch (GetPresentedItemId()) { case 0: break; default: break; } } else { if (GetPresentedItemKind() == 1) { switch (GetPresentedItemId()) { case 0: break; default: break; } } } if (GetPresentedItemKind() == 0 && GetPresentedItemId() == 0) { TalkClose(); } if (GetPresentedItemKind() != 0 || GetPresentedItemId() == 0) { TalkClose(); } switch (GetPresentedItemKind()) { case 0: switch (GetPresentedItemId()) { case 0: break; default: break; } break; case 1: switch (GetPresentedItemId()) { case 0: break; default: break; } break; default: break; } switch (GetPresentedItemId()) { case 0: return; default: break; } if (id == 0) { TalkClose(); } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestPresentedDirect",
        )
        .unwrap();
        let source = format_named_script("TestPresentedDirect", &raised).unwrap();
        for symbol in [
            "HELD_ITEM_KIND_FOOD",
            "case FOOD_TURNIP:",
            "HELD_ITEM_KIND_ARTICLE",
            "case ARTICLE_FLOWER_MOON_DROP:",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol} for direct dependent callable use: {source}"
            );
        }
        assert_eq!(
            source.matches("case 0:").count(),
            1,
            "{target}: branch-only dependent type leaked beyond its discriminator scope: {source}"
        );
        assert_eq!(
            source.matches("FOOD_TURNIP").count(),
            6,
            "{target}: {source}"
        );
        assert!(
            source.contains("var_1 == 0"),
            "{target}: short-circuit local refinement leaked after the condition: {source}"
        );
        assert_eq!(
            source.matches("case ARTICLE_FLOWER_MOON_DROP:").count(),
            2,
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: direct dependent callable refinement changed emitted bytecode"
        );
    }
}

#[test]
fn related_callable_origins_survive_local_copies_without_leaking() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestCopiedOrigins, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestCopiedOrigins(void) {\n\
             int kind = GetPresentedItemKind();\n\
             int copied_kind = kind;\n\
             int id = GetPresentedItemId();\n\
             int copied_id = id;\n\
             if (copied_kind == 0) { switch (copied_id) { case 0: break; default: break; } }\n\
             else { if (copied_kind == 1) { switch (copied_id) { case 0: break; default: break; } } }\n\
             switch (copied_id) { case 0: return; default: break; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestCopiedOrigins")
                .unwrap();
        let source = format_named_script("TestCopiedOrigins", &raised).unwrap();
        for expected in [
            "var_1 == HELD_ITEM_KIND_FOOD",
            "case FOOD_TURNIP:",
            "var_1 == HELD_ITEM_KIND_ARTICLE",
            "case ARTICLE_FLOWER_MOON_DROP:",
        ] {
            assert!(
                source.contains(expected),
                "{target}: missing {expected}: {source}"
            );
        }
        assert_eq!(
            source.matches("case 0:").count(),
            1,
            "{target}: copied callable origin leaked beyond the refined branch: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: copied related-callable origins changed emitted bytecode"
        );
    }
}

#[test]
fn mfomt_outfit_color_symbols_print_and_round_trip() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestOutfit, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestOutfit(void) {\n\
             SetPlayerOutfitColor(5);\n\
             switch (GetPlayerOutfitColor()) { case 0: return; default: break; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestOutfit").unwrap();
        let source = format_named_script("TestOutfit", &raised).unwrap();
        assert!(
            source.contains("SetPlayerOutfitColor(OUTFIT_COLOR_RED)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case OUTFIT_COLOR_BLUE:"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn project_tv_shopping_ids_are_printed_for_reads_and_round_trip() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestTVShopping, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestTVShopping(void) {\n\
             if (GetPendingTVShoppingItem() == 16) { return; }\n\
             switch (GetTVShoppingSelection()) { case 15: return; default: break; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestTVShopping")
                .unwrap();
        let source = format_named_script("TestTVShopping", &raised).unwrap();
        assert!(
            source.contains("GetPendingTVShoppingItem() == TV_SHOPPING_ITEM_NONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case TV_SHOPPING_ITEM_POWER_BERRY:"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn project_blacksmith_order_symbols_print_and_round_trip_for_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestBlacksmith, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestBlacksmith(void) {\n\
             if (GetBlacksmithOrderId() == 37) { return; }\n\
             switch (CollectBlacksmithOrder()) { case 3: return; default: break; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestBlacksmith")
                .unwrap();
        let source = format_named_script("TestBlacksmith", &raised).unwrap();
        assert!(
            source.contains("GetBlacksmithOrderId() == BLACKSMITH_ORDER_YARN_MAKER"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case BLACKSMITH_COLLECTION_NO_SPACE:"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn project_item_id_constants_print_and_round_trip_for_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestItems, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestItems(void) {\n\
             SetPlayerHeldFood(150);\n\
             SetPlayerHeldArticle(39);\n\
             SetPlayerHeldTool(5, 1);\n\
             ShowPlayerHoldingTool(5);\n\
             if (GetPlayerHeldToolStackCount() == -1) { return; }\n\
             if (AddArticleToRucksack(39, 2) != 0) { return; }\n\
             if (AddFoodToRucksack(150, 2) != 0) { return; }\n\
             if (AddToolToRucksack(5, 2) != 0) { return; }\n\
             if (GetPlayerHeldItemKind() == 2) { return; }\n\
             if (GetPlayerHeldItemKind() == 3) { return; }\n\
             if (GetPlayerHeldItemKind() == 4) { return; }\n\
             if (GetPlayerHeldItemKind() == 5) { return; }\n\
             switch (GetPlayerHeldChickenId()) {\n\
             case -1: return;\n\
             case 0: return;\n\
             case 7: return;\n\
             }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestItems").unwrap();
        let source = format_named_script("TestItems", &raised).unwrap();
        assert!(source.contains("FOOD_MOON_DUMPLINGS"), "{target}: {source}");
        assert!(
            source.contains("ARTICLE_JEWEL_OF_TRUTH"),
            "{target}: {source}"
        );
        assert!(source.contains("TOOL_SICKLE_CURSED"), "{target}: {source}");
        assert!(
            source.contains("GetPlayerHeldToolStackCount() == HELD_TOOL_STACK_NOT_PRESENT"),
            "{target}: {source}"
        );
        assert!(
            source.contains(
                "AddArticleToRucksack(ARTICLE_JEWEL_OF_TRUTH, 2) != UNADDED_ITEM_COUNT_NONE"
            ),
            "{target}: {source}"
        );
        assert!(
            source.contains("AddFoodToRucksack(FOOD_MOON_DUMPLINGS, 2) != UNADDED_ITEM_COUNT_NONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AddToolToRucksack(TOOL_SICKLE_CURSED, 2) != UNADDED_ITEM_COUNT_NONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ShowPlayerHoldingTool(TOOL_SICKLE_CURSED)"),
            "{target}: {source}"
        );
        assert!(source.contains("HELD_ITEM_KIND_DOG"), "{target}: {source}");
        assert!(
            source.contains("HELD_ITEM_KIND_CHICKEN"),
            "{target}: {source}"
        );
        assert!(
            source.contains("HELD_ITEM_KIND_BASKET"),
            "{target}: {source}"
        );
        assert!(
            source.contains("HELD_ITEM_KIND_ACTOR_GRAPHIC"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_NONE:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_1:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_8:"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn static_entity_ids_cover_player_and_every_character_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let entity_type = constants.user_type("MaryEntityId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(entity_type, 0),
            Some("ENTITY_PLAYER"),
            "{target}"
        );
        for id in 1..=42 {
            assert!(
                constants.typed_int_const_name(entity_type, id).is_some(),
                "{target}: missing static entity ID {id}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(entity_type, 42),
            Some("ENTITY_TIMID"),
            "{target}"
        );
        for (id, expected) in [
            (43, "ENTITY_FARM_DOG"),
            (44, "ENTITY_FARM_HORSE"),
            (46, "ENTITY_CHICKEN_SLOT_1"),
            (53, "ENTITY_CHICKEN_SLOT_8"),
            (54, "ENTITY_BARN_ANIMAL_SLOT_1"),
            (69, "ENTITY_BARN_ANIMAL_SLOT_16"),
            (74, "ENTITY_BASKET"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(entity_type, id),
                Some(expected),
                "{target}"
            );
        }
        for (id, expected) in [
            (45, "ENTITY_FRISBEE"),
            (70, "ENTITY_SCRIPT_VISUAL_EFFECT_SLOT_0"),
            (71, "ENTITY_SCRIPT_VISUAL_EFFECT_SLOT_1"),
            (72, "ENTITY_SCRIPT_VISUAL_EFFECT_SLOT_2"),
            (73, "ENTITY_SCRIPT_VISUAL_EFFECT_SLOT_3"),
            (75, "ENTITY_BALL"),
            (94, "ENTITY_TUTORIAL_PLAYER"),
            (95, "ENTITY_TUTORIAL_ADULT_ANIMAL_SLOT_1"),
            (96, "ENTITY_TUTORIAL_ADULT_ANIMAL_SLOT_2"),
            (97, "ENTITY_TUTORIAL_YOUNG_ANIMAL"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(entity_type, id),
                Some(expected),
                "{target}: dynamic entity ID {id} has the wrong stable symbol"
            );
        }

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestEntities, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestEntities(void) { int animal_slot = 3; SetEntityPosition(0, 10, 20, 0); SetEntityFacing(25, 1); SetEntityAnim(42, 0); HideEntity(43); HideEntity(44); HideEntity(46); HideEntity(53); HideEntity(54); HideEntity(69); HideEntity(54 + animal_slot); HideEntity(74); HideEntity(75); HideEntity(70); HideEntity(94); HideEntity(95); HideEntity(96); HideEntity(97); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestEntities")
                .unwrap();
        let source = format_named_script("TestEntities", &raised).unwrap();
        for expected in [
            "SetEntityPosition(ENTITY_PLAYER, X(10), Y(20), FACING_DOWN)",
            "SetEntityFacing(ENTITY_ANN, FACING_UP)",
            "SetEntityAnim(ENTITY_TIMID, ANIMATION_ID_0000)",
            "HideEntity(ENTITY_FARM_DOG)",
            "HideEntity(ENTITY_FARM_HORSE)",
            "HideEntity(ENTITY_CHICKEN_SLOT_1)",
            "HideEntity(ENTITY_CHICKEN_SLOT_8)",
            "HideEntity(ENTITY_BARN_ANIMAL_SLOT_1)",
            "HideEntity(ENTITY_BARN_ANIMAL_SLOT_16)",
            "HideEntity(ENTITY_BARN_ANIMAL_SLOT_1 + var_0)",
            "HideEntity(ENTITY_BASKET)",
            "HideEntity(ENTITY_BALL)",
            "HideEntity(ENTITY_SCRIPT_VISUAL_EFFECT_SLOT_0)",
            "HideEntity(ENTITY_TUTORIAL_PLAYER)",
            "HideEntity(ENTITY_TUTORIAL_ADULT_ANIMAL_SLOT_1)",
            "HideEntity(ENTITY_TUTORIAL_ADULT_ANIMAL_SLOT_2)",
            "HideEntity(ENTITY_TUTORIAL_YOUNG_ANIMAL)",
        ] {
            assert!(
                source.contains(expected),
                "{target}: missing {expected}: {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: entity symbols changed emitted bytecode"
        );
    }
}

#[test]
fn only_open_coordinate_domains_use_typed_identity_wrappers() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let declarations = header
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("mary_typed_identity("))
        .collect::<Vec<_>>();
    assert_eq!(
        declarations,
        [
            "mary_typed_identity(MaryMapSpaceX, X);",
            "mary_typed_identity(MaryMapSpaceY, Y);",
        ],
        "finite and observed ID domains must use ordinary constants; only open X/Y coordinates use identity wrappers"
    );
    for forbidden in [
        "EVENT_ICON_SLOT(",
        "EVENT_ICON_LAYER(",
        "ENTITY_ID(",
        "CAMERA_SPEED(",
    ] {
        assert!(
            !header.contains(forbidden),
            "non-coordinate function-style constant survived: {forbidden}"
        );
    }
}

#[test]
fn harvest_sprite_work_and_minigame_callables_round_trip_for_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestSprite, };\n", &options).unwrap();
        let source = "void TestSprite(void) {\n\
             SetTalkNameplateCharacter(CHARACTER_CHILD);\n\
             SetTalkNameplateCharacter(CHARACTER_STAID);\n\
             SetTalkNameplateCharacter(CHARACTER_TIMID);\n\
             if (IsHarvestSpriteDailyWorkComplete(CHARACTER_NAPPY)) { return; }\n\
             ScheduleHarvestSpriteTaskToEndAfterToday(CHARACTER_NAPPY);\n\
             RunHarvestSpriteAnimalCareMinigame(CHARACTER_NAPPY);\n\
             RunHarvestSpriteHarvestingMinigame(CHARACTER_NAPPY);\n\
             RunHarvestSpriteWateringMinigame(CHARACTER_NAPPY);\n\
             if (RunChickenFestivalContest()) { return; }\n\
             }\n";
        let symbolic =
            parse_named_scripts(source, &options, &callables.scope, &script_table).unwrap();
        let numeric_source = source
            .replace("CHARACTER_CHILD", "35")
            .replace("CHARACTER_STAID", "36")
            .replace("CHARACTER_NAPPY", "37")
            .replace("CHARACTER_TIMID", "42");
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        assert_eq!(
            encode_script(&symbolic.scripts[0].2),
            encode_script(&numeric.scripts[0].2),
            "{target}"
        );
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestSprite").unwrap();
        let raised_source = format_named_script("TestSprite", &raised).unwrap();
        for expected in [
            "SetTalkNameplateCharacter(CHARACTER_CHILD);",
            "SetTalkNameplateCharacter(CHARACTER_STAID);",
            "SetTalkNameplateCharacter(CHARACTER_TIMID);",
        ] {
            assert!(
                raised_source.contains(expected),
                "{target}: {raised_source}"
            );
        }
    }
}

#[test]
fn unknown_id_constant_reports_a_compile_error() {
    let options = Options::default().define("MARY_FOMT_US").unwrap();
    let constants = parse_constant_header(
        "typedef enum MaryPortraitId {\nTALK_PORTRAIT_KAREN_HAPPY = 73,\n} MaryPortraitId;\n",
        &options,
    )
    .unwrap();
    let callables = parse_callable_table_with_scope(
        "mary_callable_table { SetTalkPortrait, };\n\
         void SetTalkPortrait(MaryPortraitId portrait_id);\n",
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestPortrait, };\n", &options).unwrap();
    let error = parse_named_scripts(
        "void TestPortrait(void) { SetTalkPortrait(TALK_PORTRAIT_UNKNOWN); }\n",
        &options,
        &callables.scope,
        &script_table,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("TALK_PORTRAIT_UNKNOWN"), "{error}");
}

#[test]
fn event_context_and_product_name_callables_keep_their_target_ids() {
    for (target, product_name_id, context_id) in [
        ("MARY_FOMT_US", 0x144, 0x145),
        ("MARY_FOMT_JP", 0x144, 0x145),
        ("MARY_MFOMT_US", 0x148, 0x149),
        ("MARY_MFOMT_JP", 0x148, 0x149),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["SetTextVariableToProductName"].0 .0, product_name_id,
            "{target}"
        );
        assert_eq!(
            callable_map["GetEventContextValue"].0 .0, context_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestContext, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestContext(void) {\n\
             int product_id = GetEventContextValue();\n\
             SetTextVariableToProductName(0, product_id);\n\
             switch (product_id) { case 0: return; case 1: return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestContext").unwrap();
        let source = format_named_script("TestContext", &raised).unwrap();
        assert!(
            source.contains("case PRODUCT_TURNIP:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case PRODUCT_POTATO:"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn fomt_love_event_symbols_use_gba_heart_stages() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (
                16,
                "EventScript_LoveEvent_Karen_01_BlackHeart_ReceiveWelcomeSeedGift",
            ),
            (
                20,
                "EventScript_LoveEvent_Karen_02_PurpleHeart_ChooseFlowerSeeds",
            ),
            (
                21,
                "EventScript_LoveEvent_Karen_02_PurpleHeart_FollowupJeffDialogue",
            ),
            (
                22,
                "EventScript_LoveEvent_Karen_02_PurpleHeart_FollowupKarenDialogue",
            ),
            (
                23,
                "EventScript_LoveEvent_Karen_02_PurpleHeart_FollowupSashaDialogue",
            ),
            (
                24,
                "EventScript_LoveEvent_Karen_03_BlueHeart_RetrieveFamilyWine",
            ),
            (
                29,
                "EventScript_LoveEvent_Karen_04_YellowHeart_TasteHomeCooking",
            ),
            (33, "EventScript_LoveEvent_Karen_ProposalAccepted"),
            (
                36,
                "EventScript_LoveEvent_Popuri_01_BlackHeart_CatchRunawayChicken",
            ),
            (
                37,
                "EventScript_LoveEvent_Popuri_02_PurpleHeart_VisitPlayerFarm",
            ),
            (
                38,
                "EventScript_LoveEvent_Popuri_03_BlueHeart_PlayHouseWithChildren",
            ),
            (
                39,
                "EventScript_LoveEvent_Popuri_04_YellowHeart_ResolveFamilyArgument",
            ),
            (
                40,
                "EventScript_LoveEvent_Popuri_04_YellowHeart_FollowupRickDialogue",
            ),
            (
                41,
                "EventScript_LoveEvent_Popuri_04_YellowHeart_FollowupPopuriDialogue",
            ),
            (
                42,
                "EventScript_LoveEvent_Popuri_04_YellowHeart_FollowupLilliaDialogue",
            ),
            (43, "EventScript_LoveEvent_Popuri_ProposalAccepted"),
            (
                46,
                "EventScript_LoveEvent_Ann_01_BlackHeart_ReceiveWelcomeMeal",
            ),
            (
                49,
                "EventScript_LoveEvent_Ann_02_PurpleHeart_DiscussCleaning",
            ),
            (
                50,
                "EventScript_LoveEvent_Ann_02_PurpleHeart_FollowupAnnDialogue",
            ),
            (
                51,
                "EventScript_LoveEvent_Ann_03_BlueHeart_VisitClinicAfterOvereating",
            ),
            (
                52,
                "EventScript_LoveEvent_Ann_04_YellowHeart_DougAsksAboutAnn",
            ),
            (
                53,
                "EventScript_LoveEvent_Ann_04_YellowHeart_FollowupDougDialogue",
            ),
            (54, "EventScript_LoveEvent_Ann_ProposalAccepted"),
            (
                57,
                "EventScript_LoveEvent_Mary_01_BlackHeart_LibraryIntroduction",
            ),
            (
                59,
                "EventScript_LoveEvent_Mary_02_PurpleHeart_FindMissingBook",
            ),
            (
                60,
                "EventScript_LoveEvent_Mary_02_PurpleHeart_FollowupMaryDialogue",
            ),
            (
                61,
                "EventScript_LoveEvent_Mary_03_BlueHeart_DiscussEmotionalNovel",
            ),
            (
                62,
                "EventScript_LoveEvent_Mary_04_YellowHeart_SuggestNovelSubject",
            ),
            (63, "EventScript_LoveEvent_Mary_ProposalAccepted"),
            (
                66,
                "EventScript_LoveEvent_Elli_01_BlackHeart_MeetElliAndStu",
            ),
            (
                69,
                "EventScript_LoveEvent_Elli_02_PurpleHeart_MedicineMixup",
            ),
            (
                72,
                "EventScript_LoveEvent_Elli_03_BlueHeart_FindStuAndReceiveDriedFlowers",
            ),
            (
                73,
                "EventScript_LoveEvent_Elli_03_BlueHeart_FollowupEllenDialogue",
            ),
            (
                74,
                "EventScript_LoveEvent_Elli_03_BlueHeart_FollowupElliDialogue",
            ),
            (
                75,
                "EventScript_LoveEvent_Elli_03_BlueHeart_FollowupStuDialogue",
            ),
            (
                76,
                "EventScript_LoveEvent_Elli_04_YellowHeart_DiscussRaisingStu",
            ),
            (77, "EventScript_LoveEvent_Elli_ProposalAccepted"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn fomt_family_event_symbols_keep_spouse_and_child_stage_context() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (
                589,
                "EventScript_FamilyEvent_Popuri_AnniversaryDateDialogueChoice",
            ),
            (594, "EventScript_FamilyEvent_Ann_AnniversaryBabyGooGoo"),
            (
                597,
                "EventScript_FamilyEvent_Elli_AnniversaryDateDialogueChoice",
            ),
            (602, "EventScript_FamilyEvent_Karen_AnniversaryBabyGooGoo"),
            (
                605,
                "EventScript_FamilyEvent_Mary_AnniversaryDateDialogueChoice",
            ),
            (
                610,
                "EventScript_FamilyEvent_Popuri_BabyBirthdayDateDialogueChoice",
            ),
            (619, "EventScript_FamilyEvent_Karen_BabyBirthdayBabbles"),
            (632, "EventScript_FamilyEvent_Elli_FamilyDateDialogueChoice"),
            (
                650,
                "EventScript_FamilyEvent_Karen_FamilyDateDialogueChoiceAlternate",
            ),
            (654, "EventScript_FamilyEvent_Popuri_PregnancyDiscovery"),
            (659, "EventScript_FamilyEvent_Popuri_Childbirth"),
            (663, "EventScript_FamilyEvent_Mary_Childbirth"),
            (664, "EventScript_FamilyEvent_Popuri_ChildFirstSteps"),
            (668, "EventScript_FamilyEvent_Ann_ChildFirstStepsDaDa"),
            (670, "EventScript_FamilyEvent_Elli_ChildFirstSteps"),
            (674, "EventScript_FamilyEvent_Karen_ChildFirstStepsDaDa"),
            (676, "EventScript_FamilyEvent_Mary_ChildFirstSteps"),
            (684, "EventScript_FamilyEvent_Popuri_ChildInjury"),
            (685, "EventScript_FamilyEvent_Ann_ChildInjury"),
            (686, "EventScript_FamilyEvent_Elli_ChildInjury"),
            (687, "EventScript_FamilyEvent_Karen_ChildInjury"),
            (688, "EventScript_FamilyEvent_Mary_ChildInjury"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn mfomt_love_event_symbols_use_gba_heart_stages() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (
                16,
                "EventScript_LoveEvent_Rick_01_BlackHeart_ReceiveSpaBoiledEgg",
            ),
            (
                20,
                "EventScript_LoveEvent_Rick_02_PurpleHeart_AnswerEggHatchingQuiz",
            ),
            (
                24,
                "EventScript_LoveEvent_Rick_03_BlueHeart_DiscussFatherAndReceiveWatch",
            ),
            (
                28,
                "EventScript_LoveEvent_Rick_04_YellowHeart_ShareDrinkAndEscortHome",
            ),
            (29, "EventScript_LoveEvent_Rick_ProposalAccepted"),
            (
                32,
                "EventScript_LoveEvent_Kai_01_BlackHeart_DiscussLeavingHometown",
            ),
            (
                33,
                "EventScript_LoveEvent_Kai_02_PurpleHeart_DiscussCropsAndPineapples",
            ),
            (
                35,
                "EventScript_LoveEvent_Kai_03_BlueHeart_TreatHeatExhaustionAndReceiveLuckyCharm",
            ),
            (
                36,
                "EventScript_LoveEvent_Kai_04_YellowHeart_DiscussChangingHimself",
            ),
            (38, "EventScript_LoveEvent_Kai_ProposalAccepted"),
            (
                41,
                "EventScript_LoveEvent_Cliff_01_BlackHeart_CarterIntroducesCliff",
            ),
            (
                42,
                "EventScript_LoveEvent_Cliff_02_PurpleHeart_HelpPrepareChurchDinner",
            ),
            (
                43,
                "EventScript_LoveEvent_Cliff_03_BlueHeart_DiscussWineryWorkAndReceiveFlowerDecoration",
            ),
            (
                44,
                "EventScript_LoveEvent_Cliff_04_YellowHeart_DiscussCliffsFaultsAtWinery",
            ),
            (46, "EventScript_LoveEvent_Cliff_ProposalAccepted"),
            (
                49,
                "EventScript_LoveEvent_Gray_01_BlackHeart_EncourageBlacksmithTraining",
            ),
            (
                52,
                "EventScript_LoveEvent_Gray_02_PurpleHeart_DiscussBecomingMasterBlacksmith",
            ),
            (
                54,
                "EventScript_LoveEvent_Gray_03_BlueHeart_DiscussJewelryAndReceiveBrooch",
            ),
            (
                55,
                "EventScript_LoveEvent_Gray_04_YellowHeart_PromiseToStayInTown",
            ),
            (57, "EventScript_LoveEvent_Gray_ProposalAccepted"),
            (
                60,
                "EventScript_LoveEvent_Doctor_01_BlackHeart_TestBitterMedicine",
            ),
            (
                63,
                "EventScript_LoveEvent_Doctor_02_PurpleHeart_TestStrongMedicine",
            ),
            (
                66,
                "EventScript_LoveEvent_Doctor_03_BlueHeart_ReceiveNegativeIonAccessory",
            ),
            (
                69,
                "EventScript_LoveEvent_Doctor_04_YellowHeart_DiscussPatientCare",
            ),
            (72, "EventScript_LoveEvent_Doctor_ProposalAccepted"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn mfomt_family_event_symbols_keep_spouse_and_child_stage_context() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (
                598,
                "EventScript_FamilyEvent_Kai_AnniversaryDateDialogueChoice",
            ),
            (599, "EventScript_FamilyEvent_Kai_AnniversaryBabyMama"),
            (
                602,
                "EventScript_FamilyEvent_Cliff_AnniversaryDateDialogueChoice",
            ),
            (
                606,
                "EventScript_FamilyEvent_Doctor_AnniversaryDateDialogueChoice",
            ),
            (
                610,
                "EventScript_FamilyEvent_Rick_AnniversaryDateDialogueChoice",
            ),
            (
                614,
                "EventScript_FamilyEvent_Gray_AnniversaryDateDialogueChoice",
            ),
            (619, "EventScript_FamilyEvent_Kai_BabyBirthdayBabbles"),
            (
                623,
                "EventScript_FamilyEvent_Cliff_BabyBirthdayDateDialogueChoice",
            ),
            (
                641,
                "EventScript_FamilyEvent_Doctor_FamilyDateDialogueChoice",
            ),
            (
                658,
                "EventScript_FamilyEvent_Rick_FamilyDateDialogueChoiceAlternate",
            ),
            (663, "EventScript_FamilyEvent_Kai_PregnancyDiscovery"),
            (667, "EventScript_FamilyEvent_Gray_PregnancyDiscovery"),
            (668, "EventScript_FamilyEvent_Kai_Childbirth"),
            (672, "EventScript_FamilyEvent_Gray_Childbirth"),
            (673, "EventScript_FamilyEvent_Kai_ChildFirstSteps"),
            (676, "EventScript_FamilyEvent_Cliff_ChildFirstSteps"),
            (679, "EventScript_FamilyEvent_Doctor_ChildFirstSteps"),
            (682, "EventScript_FamilyEvent_Rick_ChildFirstSteps"),
            (685, "EventScript_FamilyEvent_Gray_ChildFirstSteps"),
            (901, "EventScript_FamilyEvent_Gourmet_PregnancyDiscovery"),
            (902, "EventScript_FamilyEvent_Kappa_PregnancyDiscovery"),
            (903, "EventScript_FamilyEvent_Won_PregnancyDiagnosis"),
            (904, "EventScript_FamilyEvent_Gourmet_Childbirth"),
            (905, "EventScript_FamilyEvent_Kappa_Childbirth"),
            (906, "EventScript_FamilyEvent_Won_Childbirth"),
            (907, "EventScript_FamilyEvent_Gourmet_ChildFirstSteps"),
            (910, "EventScript_FamilyEvent_Kappa_ChildFirstSteps"),
            (912, "EventScript_FamilyEvent_Won_ChildFirstSteps"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn spouse_festival_dispatchers_use_event_level_names() {
    for (target, id, text_count, expected) in [
        (
            "MARY_FOMT_US",
            1306,
            29,
            "EventScript_FestivalEvent_StarryNight_WithSpouseAndChild",
        ),
        (
            "MARY_FOMT_JP",
            1306,
            32,
            "EventScript_FestivalEvent_StarryNight_WithSpouseAndChild",
        ),
        (
            "MARY_MFOMT_US",
            1323,
            21,
            "EventScript_FestivalEvent_MoonViewing_WithPartnerAndDumplingGift",
        ),
        (
            "MARY_MFOMT_JP",
            1323,
            21,
            "EventScript_FestivalEvent_MoonViewing_WithPartnerAndDumplingGift",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(id),
            Some(expected),
            "{target} slot {id}"
        );
        for text in symbols.names(id, text_count).iter().flatten() {
            assert!(
                text.starts_with(&format!(
                    "gText_{}",
                    expected.trim_start_matches("EventScript_")
                )),
                "{target} slot {id} has an out-of-scope text {text}"
            );
        }
    }
}

#[test]
fn sentence_derived_script_names_are_replaced_with_event_level_names() {
    for (target, cases) in
        [
            (
                "MARY_FOMT_US",
                vec![
                (
                    150,
                    "EventScript_LocationInteraction_InspectZackHouseLilliaPhoto",
                ),
                (
                    202,
                    "EventScript_LocationInteraction_InspectSupermarketCounterDuringEvent",
                ),
                (
                    229,
                    "EventScript_LocationInteraction_InspectClinicBedDuringCliffRecovery",
                ),
                (
                    509,
                    "EventScript_LocationInteraction_InspectPlayerCottageSign",
                ),
                (
                    970,
                    "EventScript_LocationInteraction_InspectLargeStoneRequiresHigherHammerLevel",
                ),
                (
                    1277,
                    "EventScript_FestivalEvent_SheepFestival_ClosingAnnouncement",
                ),
                (1304, "EventScript_FestivalEvent_StarryNight_WithKarenFamily"),
            ],
            ),
            (
                "MARY_MFOMT_US",
                vec![
                (
                    211,
                    "EventScript_LocationInteraction_InspectSupermarketCounterDuringEvent",
                ),
                (
                    518,
                    "EventScript_LocationInteraction_InspectPlayerCottageSign",
                ),
                (921, "EventScript_NPCEvent_Gourmet_MonthlyLunchVisit"),
                (922, "EventScript_NPCEvent_Gourmet_MonthlyDinnerVisit"),
                (
                    1038,
                    "EventScript_LocationInteraction_InspectLargeStoneRequiresHigherHammerLevel",
                ),
                (1052, "EventScript_FarmEvent_Livestock_NewBirthAndNaming"),
                (
                    1357,
                    "EventScript_FestivalEvent_SheepFestival_ClosingAnnouncement",
                ),
                (1389, "EventScript_FestivalEvent_StarryNight_WithKarenFamily"),
            ],
            ),
        ]
    {
        for region in ["US", "JP"] {
            let target = target.replace("US", region);
            let options = Options::default().define(&target).unwrap();
            let symbols = parse_text_name_table(
                &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
                &options,
            )
            .unwrap();
            for (id, expected) in &cases {
                assert_eq!(
                    symbols.script_name(*id),
                    Some(*expected),
                    "{target} slot {id}"
                );
            }
        }
    }
}

#[test]
fn regional_script_names_match_at_every_physical_slot() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let mut differences = Vec::new();
    for (family, slot_count) in [("FOMT", 1329), ("MFOMT", 1416)] {
        let us_options = Options::default()
            .define(&format!("MARY_{family}_US"))
            .unwrap();
        let jp_options = Options::default()
            .define(&format!("MARY_{family}_JP"))
            .unwrap();
        let us = parse_text_name_table(&source, &us_options).unwrap();
        let jp = parse_text_name_table(&source, &jp_options).unwrap();
        for id in 0..slot_count {
            if us.script_name(id) != jp.script_name(id) {
                differences.push(format!(
                    "{family} {id:04}: US={:?}, JP={:?}",
                    us.script_name(id),
                    jp.script_name(id)
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn numeric_script_names_are_reserved_for_textless_placeholders() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let mut audited = 0usize;

    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();

        for script_id in 0..slot_count {
            let Some(script_name) = symbols.script_name(script_id) else {
                continue;
            };
            let Some(suffix) = script_name.strip_prefix("EventScript_") else {
                continue;
            };
            if suffix.len() != 4 || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
                continue;
            }

            audited += 1;
            assert_eq!(
                symbols.text_count(script_id),
                0,
                "{target} slot {script_id:04} retains numeric name {script_name} despite owning text"
            );
        }
    }

    assert!(
        audited >= 70,
        "numeric placeholder audit unexpectedly covered only {audited} target scripts"
    );
}

#[test]
fn script_symbols_are_unique_within_each_target() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let mut owners = HashMap::<String, usize>::new();
        let mut duplicates = Vec::new();
        for script_id in 0..slot_count {
            let Some(name) = symbols.script_name(script_id) else {
                continue;
            };
            if let Some(first_id) = owners.insert(name.to_owned(), script_id) {
                duplicates.push(format!("{name}: {first_id:04} and {script_id:04}"));
            }
        }
        assert!(
            duplicates.is_empty(),
            "{target} has ambiguous script symbols:\n{}",
            duplicates.join("\n")
        );
    }
}

#[test]
fn duplicate_text_variants_use_branch_semantics_instead_of_numeric_suffixes() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    for (target, water_tank_id, rucksack_id) in [
        ("MARY_FOMT_US", 98usize, 479usize),
        ("MARY_FOMT_JP", 98, 479),
        ("MARY_MFOMT_US", 106, 488),
        ("MARY_MFOMT_JP", 106, 488),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let water_tank_names = symbols.names(water_tank_id, symbols.text_count(water_tank_id));
        let rucksack_names = symbols.names(rucksack_id, symbols.text_count(rucksack_id));

        for expected in [
            "gText_LocationInteraction_InspectWaterTank_JewelOfTruthFound",
            "gText_LocationInteraction_InspectWaterTank_JewelOfTruthFoundButHandsFull",
        ] {
            assert!(
                water_tank_names
                    .iter()
                    .flatten()
                    .any(|name| name == expected),
                "{target} slot {water_tank_id:04} lacks {expected}"
            );
        }
        for expected in [
            "gText_ShopEvent_Rucksack_PurchaseChoice_InsufficientGoldForSmallRucksack",
            "gText_ShopEvent_Rucksack_PurchaseChoice_InsufficientGoldForLargeRucksack",
        ] {
            assert!(
                rucksack_names.iter().flatten().any(|name| name == expected),
                "{target} slot {rucksack_id:04} lacks {expected}"
            );
        }
    }
}

#[test]
fn clinic_diagnosis_texts_encode_the_full_stamina_and_fatigue_matrix() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let stamina_bands = [
        "StaminaAtLeast70Percent",
        "Stamina50To69Percent",
        "Stamina20To49Percent",
        "Stamina5To19Percent",
        "StaminaBelow5Percent",
    ];
    let fatigue_bands = [
        "FatigueBelow50",
        "Fatigue50To69",
        "Fatigue70To89",
        "FatigueAtLeast90",
    ];

    for (target, script_id) in [
        ("MARY_FOMT_US", 483usize),
        ("MARY_FOMT_JP", 483),
        ("MARY_MFOMT_US", 492),
        ("MARY_MFOMT_JP", 492),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));

        for stamina in stamina_bands {
            for fatigue in fatigue_bands {
                let expected = format!(
                    "gText_ShopEvent_Clinic_ExaminationChoice_Diagnosis_{stamina}_{fatigue}"
                );
                assert!(
                    names.iter().flatten().any(|name| name == &expected),
                    "{target} slot {script_id:04} lacks {expected}"
                );
            }
        }
        for expected in [
            "gText_ShopEvent_Clinic_ExaminationChoice_Recommendation_TurbojoltOrRest",
            "gText_ShopEvent_Clinic_ExaminationChoice_Recommendation_TurbojoltOrFullDayRest",
            "gText_ShopEvent_Clinic_ExaminationChoice_Recommendation_BodigizerOrRest",
            "gText_ShopEvent_Clinic_ExaminationChoice_Recommendation_BodigizerAndTurbojoltOrFullDayRest",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot {script_id:04} lacks {expected}"
            );
        }
    }
}

#[test]
fn spouse_bedtime_regional_text_splits_follow_their_actual_callers() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let cases = [
        (
            "MARY_FOMT_US",
            343usize,
            47usize,
            vec![
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_Karen30000To39999_Ann40000To49999_PopuriAtLeast60000_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_KarenAndElli40000To49999_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Mary30000To49999_FirstConversation",
            ],
        ),
        (
            "MARY_FOMT_JP",
            343,
            49,
            vec![
                "gText_NPCEvent_Spouse_BedtimeDialogue_Karen30000To39999_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_PopuriAtLeast60000_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Ann40000To49999_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_MaryAndElli40000To49999_FirstConversation",
            ],
        ),
        (
            "MARY_MFOMT_US",
            352,
            64,
            vec![
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_WonBelow30000_Cliff30000To39999_FirstConversation_Gray40000To49999_RepeatConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_RickKaiGrayDoctor50000To59999_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_KaiAndCliffAtLeast60000_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_GrayAtLeast60000_FirstConversation",
            ],
        ),
        (
            "MARY_MFOMT_JP",
            352,
            68,
            vec![
                "gText_NPCEvent_Spouse_BedtimeDialogue_WonBelow30000_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Cliff30000To39999_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_Shared_RickAndDoctor50000To59999_GrayAtLeast60000_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_KaiAtLeast60000_FirstConversation",
                "gText_NPCEvent_Spouse_BedtimeDialogue_CliffAtLeast60000_FirstConversation",
            ],
        ),
    ];

    for (target, script_id, expected_count, expected_names) in cases {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(script_id), expected_count, "{target}");
        let names = symbols.names(script_id, expected_count);
        for expected in expected_names {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot {script_id:04} lacks {expected}"
            );
        }
    }
}

#[test]
fn won_lottery_drawing_uses_minigame_and_reel_state_symbols() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(933),
            Some("EventScript_Minigame_Won_LotteryDrawing"),
            "{target}"
        );
        assert_eq!(symbols.text_count(933), 29, "{target}");
        let names = symbols.names(933, 29);
        for expected in [
            "gText_Minigame_Won_LotteryDrawing_AllDigitsSpinning_FrameA",
            "gText_Minigame_Won_LotteryDrawing_FirstDigitLocked_FrameD",
            "gText_Minigame_Won_LotteryDrawing_ThirdDigitSpinningPotentialWin_FrameC",
            "gText_Minigame_Won_LotteryDrawing_ResultThreeMatchingOrSequentialDigits",
            "gText_Minigame_Won_LotteryDrawing_RewardSequentialDigits",
            "gText_Minigame_Won_LotteryDrawing_RecordPlayerRequiredAfterMarriageNickname",
            "gText_Minigame_Won_LotteryDrawing_FailureAllDigitsDifferent",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 0933 lacks {expected}"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("Var1Var2Var3")),
            "{target} slot 0933 retains placeholder-derived text symbols"
        );
    }
}

#[test]
fn mfomt_record_player_navigation_texts_are_named_as_pages_and_arrows() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(360), 53, "{target}");
        let names = symbols.names(360, 53);
        for expected in [
            "gText_SystemEvent_RecordPlayerInteraction_PreviousPage",
            "gText_SystemEvent_RecordPlayerInteraction_NextPage",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 0360 lacks {expected}"
            );
        }
        for page in 1..=10 {
            let expected =
                format!("gText_SystemEvent_RecordPlayerInteraction_PageIndicator{page:02}Of10");
            assert!(
                names.iter().flatten().any(|name| name == &expected),
                "{target} slot 0360 lacks {expected}"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("NonverbalReaction")),
            "{target} slot 0360 still mislabels navigation glyphs as reactions"
        );
    }
}

#[test]
fn fomt_debug_affection_editor_texts_describe_navigation_and_adjustments() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_FOMT_US", 115usize), ("MARY_FOMT_JP", 113usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1031), count, "{target}");
        let names = symbols.names(1031, count);
        for expected in [
            "gText_SystemEvent_DebugCharacterAffectionEditor_MainMenuPrompt",
            "gText_SystemEvent_DebugCharacterAffectionEditor_EditMarriageCandidateLove",
            "gText_SystemEvent_DebugCharacterAffectionEditor_EditNpcFriendship",
            "gText_SystemEvent_DebugCharacterAffectionEditor_IncreaseLoveBy1000WithThousandsSeparator",
            "gText_SystemEvent_DebugCharacterAffectionEditor_DecreaseLoveBy1000WithThousandsSeparator",
            "gText_SystemEvent_DebugCharacterAffectionEditor_ReturnToCharacterList",
            "gText_SystemEvent_DebugCharacterAffectionEditor_ReturnToMainMenu",
            "gText_SystemEvent_DebugCharacterAffectionEditor_IncreaseFriendshipBy10",
            "gText_SystemEvent_DebugCharacterAffectionEditor_DecreaseFriendshipBy10",
            "gText_SystemEvent_DebugCharacterAffectionEditor_PreviousPage",
            "gText_SystemEvent_DebugCharacterAffectionEditor_NextPage",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1031 lacks {expected}"
            );
        }
        for page in 1..=11 {
            let expected = format!(
                "gText_SystemEvent_DebugCharacterAffectionEditor_PageIndicator{page:02}Of11"
            );
            assert!(
                names.iter().flatten().any(|name| name == &expected),
                "{target} slot 1031 lacks {expected}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.contains("NonverbalReaction")
                    && !name.contains("RA1000")
                    && !name.ends_with("Affection10")
                    && !name.ends_with("Affection10_02")
            }),
            "{target} slot 1031 retains content-derived placeholder symbols"
        );
    }
}

#[test]
fn mfomt_debug_bachelor_editor_texts_describe_navigation_and_adjustments() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 118usize), ("MARY_MFOMT_JP", 116usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1101), count, "{target}");
        let names = symbols.names(1101, count);
        for expected in [
            "gText_SystemEvent_DebugBachelorFriendshipEditor_MainMenuPrompt",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_EditBachelorLove",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_EditNpcFriendship",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_IncreaseLoveBy1000",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_DecreaseLoveBy1000",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_ReturnToCharacterList",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_ReturnToMainMenu",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_IncreaseFriendshipBy10",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_DecreaseFriendshipBy10",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_PreviousPage",
            "gText_SystemEvent_DebugBachelorFriendshipEditor_NextPage",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1101 lacks {expected}"
            );
        }
        for page in 1..=11 {
            let expected = format!(
                "gText_SystemEvent_DebugBachelorFriendshipEditor_PageIndicator{page:02}Of11"
            );
            assert!(
                names.iter().flatten().any(|name| name == &expected),
                "{target} slot 1101 lacks {expected}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.contains("NonverbalReaction")
                    && !name.ends_with("Love1000")
                    && !name.ends_with("Love1000_02")
                    && !name.ends_with("Friendship10")
                    && !name.ends_with("Friendship10_02")
            }),
            "{target} slot 1101 retains content-derived placeholder symbols"
        );
    }
}

#[test]
fn mfomt_slot_0359_order_and_mirror_messages_use_branch_semantics() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 100usize), ("MARY_MFOMT_JP", 101usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(359), count, "{target}");
        let names = symbols.names(359, count);
        for expected in [
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_OrderReadyForPickup",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_OrderCompletionNotice",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_OrderDeliveredByGray",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_OrderedItemReadyButRucksackFullGrayWillDeliverLater",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_RucksackFullGrayWillDeliverOrderLater",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_OrderReadyButRucksackFull",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_PaleFromFatigueGoToBed",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_PaleFromFatigueEatRecoveryFood",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_EveningMirrorGreeting",
            "gText_LocationInteraction_InspectFarmhouseMirrorOrUseShoppingMaster_DuskStopWorking",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 0359 lacks {expected}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("TheVar1YouOrderedIsReady_02")
                    && !name.ends_with("TheVar1IsReadyButYou_02")
                    && !name.ends_with("ILookSoPaleIShould_02")
                    && !name.ends_with("GoodEvening_02")
                    && !name.ends_with("ItsGettingDarkIShouldCall_02")
            }),
            "{target} slot 0359 retains ambiguous duplicate suffixes"
        );
    }
}

#[test]
fn mfomt_tv_program_editor_names_region_specific_japanese_residue() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 56usize), ("MARY_MFOMT_JP", 52usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(549), count, "{target}");
        let names = symbols.names(549, count);
        for expected in [
            "gText_SystemEvent_CardCollectorChisatoScoreMessages_LateNightNoProgramSeparator",
            "gText_SystemEvent_CardCollectorChisatoScoreMessages_FairyAndMeEpisodeTitle",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 0549 lacks {expected}"
            );
        }
        if target == "MARY_MFOMT_US" {
            for expected in [
                "gText_SystemEvent_CardCollectorChisatoScoreMessages_JapaneseFairyAndMeFinalEpisodeTitle",
                "gText_SystemEvent_CardCollectorChisatoScoreMessages_JapaneseIncrementEpisode",
                "gText_SystemEvent_CardCollectorChisatoScoreMessages_JapaneseDecrementEpisode",
                "gText_SystemEvent_CardCollectorChisatoScoreMessages_JapaneseWatchTelevision",
                "gText_SystemEvent_CardCollectorChisatoScoreMessages_JapaneseTurnOffTelevision",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot 0549 lacks {expected}"
                );
            }
        } else {
            assert!(
                names.iter().flatten().any(|name| {
                    name == "gText_SystemEvent_CardCollectorChisatoScoreMessages_FairyAndMeFinalEpisodeTitle"
                }),
                "{target} slot 0549 lacks the JP final-episode title"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("NonverbalReaction")),
            "{target} slot 0549 retains generic reaction placeholders"
        );
    }
}

#[test]
fn mfomt_rick_dialogue_names_location_state_and_regional_residue() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 164usize), ("MARY_MFOMT_JP", 163usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1061), count, "{target}");
        let names = symbols.names(1061, count);
        for expected in [
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_LikedGiftResponseSingle",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_GameCubeLinkedFarmRumorMarried",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_GameCubeLinkedFarmRumorSingle",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_SouthSideTownPregnancyDialogueFirstConversation",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_SouthSideTownAfterChildbirthDialogueRepeatConversation",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_SouthSideTownLowLoveDialogueRepeatConversation",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHouseLowLoveDialogueRepeatConversation",
            "gText_NPCEvent_Rick_DialogueAndGiftResponses_PopuriAndLilliaHappyAfterSummerReturn",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1061 lacks {expected}"
            );
        }
        if target == "MARY_MFOMT_US" {
            for expected in [
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHousePregnancyDialogueFirstConversation",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHouseAfterChildbirthDialogueRepeatConversation",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_JapaneseRivalProgressRealizesKarenIsImportant",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_JapaneseRivalProgressSaysOnlyKarenUnderstandsHim",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot 1061 lacks {expected}"
                );
            }
        } else {
            for expected in [
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_LikedGiftResponseMarried",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHouseLove40000FirstConversation",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHouseLove50000FirstConversation",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHouseLove60000RepeatConversation",
                "gText_NPCEvent_Rick_DialogueAndGiftResponses_PoultryFarmHouseEngagedRepeatConversation",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot 1061 lacks {expected}"
                );
            }
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("NonverbalReaction")),
            "{target} slot 1061 retains generic reaction placeholders"
        );
    }
}

#[test]
fn pumpkin_festival_family_texts_follow_spouse_child_stage_and_region() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1293), 17, "{target}");
        let names = symbols.names(1293, 17);
        for expected in [
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_KarenIntroductionWithWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_MaryIntroductionWithYoungChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_ElliIntroductionWithoutWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_PlayerOvereatsPumpkinSweets",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1293 lacks {expected}"
            );
        }
        let regional = if target == "MARY_FOMT_US" {
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_PopuriAndAnnInviteFamilyToEatSweets"
        } else {
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_AnnInvitesFamilyToEatSweets"
        };
        assert!(
            names.iter().flatten().any(|name| name == regional),
            "{target} slot 1293 lacks {regional}"
        );
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1373), 23, "{target}");
        let names = symbols.names(1373, 23);
        for expected in [
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_RickIntroductionWithWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_KaiIntroductionWithoutWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_CliffIntroductionWithWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_GrayIntroductionWithoutWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_DoctorInvitesFamilyToEat",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_WonIntroductionWithWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_GourmetIntroductionWithoutWalkingChild",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_PlayerWorriesAboutCavities",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1373 lacks {expected}"
            );
        }
        let regional = if target == "MARY_MFOMT_US" {
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_CliffAndGrayInviteFamilyToEat"
        } else {
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_GrayInvitesFamilyToEat"
        };
        assert!(
            names.iter().flatten().any(|name| name == regional),
            "{target} slot 1373 lacks {regional}"
        );
    }
}

#[test]
fn child_dialogue_variants_follow_age_friendship_and_selector_branches() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot, count) in [
        ("MARY_FOMT_US", 995usize, 37usize),
        ("MARY_FOMT_JP", 995usize, 38usize),
        ("MARY_MFOMT_US", 1064usize, 37usize),
        ("MARY_MFOMT_JP", 1064usize, 38usize),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(slot), count, "{target}");
        let names = symbols.names(slot, count);
        assert!(
            names.iter().flatten().any(|name| {
                name == "gText_NPCEvent_Baby_DialogueAndInteractions_YoungChildQuestioningBabbleVariants9And10"
            }),
            "{target} slot {slot:04} lacks the selector 9/10 babble"
        );
        if target.ends_with("_US") {
            for expected in [
                "gText_NPCEvent_Baby_DialogueAndInteractions_YoungChildCryingBabbleVariants5And8",
                "gText_NPCEvent_Baby_DialogueAndInteractions_OlderChildFriendship101To200FirstConversation",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot {slot:04} lacks {expected}"
                );
            }
        } else {
            for expected in [
                "gText_NPCEvent_Baby_DialogueAndInteractions_YoungChildNearbyHappyGreeting",
                "gText_NPCEvent_Baby_DialogueAndInteractions_YoungChildHappyBabbleVariant7",
                "gText_NPCEvent_Baby_DialogueAndInteractions_YoungChildCryingBabbleVariant8",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot {slot:04} lacks {expected}"
                );
            }
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("GaaaGaa_02")
                    && !name.ends_with("GaaaGaa_03")
                    && !name.ends_with("Noooo_02")
                    && !name.ends_with("NonverbalReaction_02")
                    && !name.ends_with("Da_02")
            }),
            "{target} slot {slot:04} retains mechanical duplicate suffixes"
        );
    }
}

#[test]
fn fishing_reward_texts_name_units_and_exact_milestones() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot) in [
        ("MARY_FOMT_US", 976usize),
        ("MARY_FOMT_JP", 976usize),
        ("MARY_MFOMT_US", 1044usize),
        ("MARY_MFOMT_JP", 1044usize),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(slot), 13, "{target}");
        let names = symbols.names(slot, 13);
        for expected in [
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtFishWithoutSize",
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtFishSizeInCentimeters",
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtFishSizeInMeters",
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtFishSizeInMetersAndCentimeters",
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtKingFish",
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtMaximumSizeFish",
            "gText_AchievementEvent_Fishing_MilestoneReward_Caught10000FishMilestone",
            "gText_AchievementEvent_Fishing_MilestoneReward_Caught100000FishMilestone",
            "gText_AchievementEvent_Fishing_MilestoneReward_Caught1000000FishMilestone",
            "gText_AchievementEvent_Fishing_MilestoneReward_Caught10000000FishMilestone",
            "gText_AchievementEvent_Fishing_MilestoneReward_Caught100000000FishMilestone",
            "gText_AchievementEvent_Fishing_MilestoneReward_Caught1000000000FishMilestone",
            "gText_AchievementEvent_Fishing_MilestoneReward_CaughtEveryFishSpeciesMilestone",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot {slot:04} lacks {expected}"
            );
        }
    }
}

#[test]
fn cliff_dialogue_names_starry_night_departure_church_and_married_work_states() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot, count) in [
        ("MARY_FOMT_US", 996usize, 56usize),
        ("MARY_FOMT_JP", 996usize, 55usize),
        ("MARY_MFOMT_US", 1065usize, 213usize),
        ("MARY_MFOMT_JP", 1065usize, 213usize),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(slot), count, "{target}");
        let names = symbols.names(slot, count);
        if target != "MARY_FOMT_JP" {
            assert!(
                names.iter().flatten().any(|name| {
                    name
                        == "gText_NPCEvent_Cliff_DialogueAndGiftResponses_FirstAutumnBeforeDepartureFirstConversationSilentPause"
                }),
                "{target} slot {slot:04} lacks Cliff's pre-departure silence"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("GoodIllBeWaitingForYou_02")
                    && !name.ends_with("SilentPause_02")
                    && !name.ends_with("SilentPause_03")
                    && !name.ends_with("NickName_02")
                    && !name.ends_with("ImGoingHomeNow_02")
                    && !name.ends_with("HeyNickNameImGoingHomeNow_02")
            }),
            "{target} slot {slot:04} retains mechanical duplicate suffixes"
        );

        if target.starts_with("MARY_MFOMT") {
            for expected in [
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_StarryNightInvitationAcceptedMeetAtWineryCellar",
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_StarryNightInvitationAcceptedMeetAtInn",
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_WineryLowLoveFirstConversationLongSilentPause",
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_WineryMarriedLove30000FirstConversationHesitantGreeting",
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_WineryMarriedHighestLoveFirstConversationLeavingForHome",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot {slot:04} lacks {expected}"
                );
            }
        }

        if target == "MARY_MFOMT_JP" {
            for expected in [
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_ChurchMarriedHighLoveGratefulForTownAndNickName",
                "gText_NPCEvent_Cliff_DialogueAndGiftResponses_WineryMarriedLove50000FirstConversationLeavingForHome",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot {slot:04} lacks {expected}"
                );
            }
        }
    }
}

#[test]
fn mfomt_zack_shipment_dialogue_names_shared_and_japanese_farm_variants() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 41usize), ("MARY_MFOMT_JP", 42usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1066), count, "{target}");
        let names = symbols.names(1066, count);
        assert!(
            names.iter().flatten().any(|name| {
                name == "gText_NPCEvent_Zack_DialogueAndGiftResponses_DaytimeShipmentCheck"
            }),
            "{target} slot 1066 lacks the shared daytime shipment check"
        );
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.ends_with("HowIsTodaysShipmentGood_02")),
            "{target} slot 1066 retains a mechanical duplicate suffix"
        );
        if target == "MARY_MFOMT_JP" {
            assert!(
                names.iter().flatten().any(|name| {
                    name
                        == "gText_NPCEvent_Zack_DialogueAndGiftResponses_FarmDaytimeLowFriendshipShipmentCheck"
                }),
                "{target} slot 1066 lacks the JP-only farm greeting variant"
            );
        }
    }
}

#[test]
fn mfomt_gourmet_dialogue_names_gift_sharing_child_age_and_heart_stages() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 45usize), ("MARY_MFOMT_JP", 46usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1075), count, "{target}");
        let names = symbols.names(1075, count);
        for expected in [
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_ChildAge60To109MorningGreeting",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_BelowGreenHeartAffectionStatusFirstConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_BelowGreenHeartAffectionStatusRepeatConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_GreenHeartAffectionStatusFirstConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_GreenHeartAffectionStatusRepeatConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_YellowHeartAffectionStatusFirstConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_YellowHeartAffectionStatusRepeatConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_OrangeHeartAffectionStatusFirstConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_OrangeHeartAffectionStatusRepeatConversation",
            "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_RedHeartAffectionStatusFirstConversation",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1075 lacks {expected}"
            );
        }
        let regional = if target == "MARY_MFOMT_US" {
            [
                "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_ElliLeavesOrCookedDishGiftResponse",
                "",
            ]
        } else {
            [
                "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_ElliLeavesFavoriteGiftResponse",
                "gText_NPCEvent_Gourmet_DialogueAndGiftResponses_CookedDishGiftResponse",
            ]
        };
        for expected in regional.into_iter().filter(|name| !name.is_empty()) {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1075 lacks {expected}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("LooksDeliciousThankYouVeryMuch_02")
                    && !name.ends_with("GoodMorning_02")
                    && !name.contains("MyAffectionRateTowardYouIs_0")
                    && !name.contains("ISaidMyAffectionRateToward_0")
            }),
            "{target} slot 1075 retains mechanical duplicate suffixes"
        );
    }
}

#[test]
fn mfomt_ruby_gamecube_recipe_dialogue_names_follow_unlock_order() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let count = symbols.text_count(1082);
        let names = symbols.names(1082, count);
        for expected in [
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe01WildGrapeJuice",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe02CornFlakes",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe03BuckwheatChips",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe04MountainStew",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe05ToastedRiceCake",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe06BakedCorn",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe07TurbojoltXL",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe08VegetableJuice",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe09CurryBread",
            "gText_NPCEvent_LouOrRuby_DialogueAndGiftResponses_GameCubeRecipe10AppleSouffle",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1082 lacks {expected}"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("PlayerDoYouLikeToCook")),
            "{target} slot 1082 retains sentence-derived recipe names"
        );
    }
}

#[test]
fn fomt_lou_gifts_and_gamecube_recipes_use_item_categories_and_unlock_order() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let count = symbols.text_count(1012);
        let names = symbols.names(1012, count);
        for expected in [
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_JewelryGiftResponse",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_DressOrCosmeticGiftResponse",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipeIntroductionVariant01",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipeIntroductionVariant02",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipeIntroductionVariant03",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe01WildGrapeJuice",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe02CornFlakes",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe03BuckwheatChips",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe04MountainStew",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe05ToastedRiceCake",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe06BakedCorn",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe07TurbojoltXL",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe08VegetableJuice",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe09CurryBread",
            "gText_NPCEvent_LouOrRuby_DailyDialogueAndCookingRecipes_GameCubeRecipe10AppleSouffle",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1012 lacks {expected}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.contains("PlayerDoYouLikeToCook")
                    && !name.ends_with("YouMeanThisIsForMe_02")
                    && !name.ends_with("YouMeanThisIsForMe_03")
                    && !name.contains("DoYouKnowHowToMake_0")
            }),
            "{target} slot 1012 retains sentence-derived or mechanical recipe names"
        );
    }
}

#[test]
fn mfomt_won_dialogue_names_gift_marriage_location_and_heart_state() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let count = symbols.text_count(1086);
        let names = symbols.names(1086, count);
        for expected in [
            "gText_NPCEvent_Won_DialogueAndGiftResponses_MarriedDressOrCosmeticGiftResponse",
            "gText_NPCEvent_Won_DialogueAndGiftResponses_ZackHouseMarriedBlueHeartRepeatNoDiscount",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1086 lacks {expected}"
            );
        }
        if target == "MARY_MFOMT_JP" {
            for expected in [
                "gText_NPCEvent_Won_DialogueAndGiftResponses_SingleDressOrCosmeticGiftResponse",
                "gText_NPCEvent_Won_DialogueAndGiftResponses_MineralBeachMarriedGreenHeartRepeatTooTiredToTalk",
                "gText_NPCEvent_Won_DialogueAndGiftResponses_SouthSideTownMarriedGreenHeartFirstConversationGreeting",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot 1086 lacks {expected}"
                );
            }
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("OhMyNickNameIsItFor_02")
                    && !name.ends_with("OhYoureMyFavoriteCustomerYoure_02")
                    && !name.ends_with("ICantGiveYouADiscount_02")
                    && !name.ends_with("ImSoTiredImGoingHome_02")
                    && !name.ends_with("NickNameWhatsTheMatter_02")
            }),
            "{target} slot 1086 retains mechanical duplicate suffixes"
        );
    }
}

#[test]
fn fomt_popuri_dialogue_names_gifts_starry_night_locations_and_heart_states() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_FOMT_US", 144usize), ("MARY_FOMT_JP", 142usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(993), count, "{target}");
        let names = symbols.names(993, count);
        for expected in [
            "gText_NPCEvent_Popuri_DialogueAndGiftResponses_SingleDressOrCosmeticBirthdayGiftResponse",
            "gText_NPCEvent_Popuri_DialogueAndGiftResponses_FarmhouseStarryNightSpousePartyInvitation",
            "gText_NPCEvent_Popuri_DialogueAndGiftResponses_FarmhouseBelowGreenHeartFirstConversationAngrySilence",
            "gText_NPCEvent_Popuri_DialogueAndGiftResponses_FarmhouseOrangeHeartMorningFirstConversationGreeting",
            "gText_NPCEvent_Popuri_DialogueAndGiftResponses_PoultryFarmHouseYellowHeartRepeatEncouragesPlayerToWork",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 0993 lacks {expected}"
            );
        }
        if target == "MARY_FOMT_US" {
            assert!(
                names.iter().flatten().any(|name| {
                    name
                        == "gText_NPCEvent_Popuri_DialogueAndGiftResponses_PoultryFarmHouseBelowGreenHeartFirstConversationAngrySilence"
                }),
                "{target} slot 0993 lacks the US-only poultry-house silence"
            );
        } else {
            assert!(
                names.iter().flatten().any(|name| {
                    name
                        == "gText_NPCEvent_Popuri_DialogueAndGiftResponses_RivalMarriedDressOrCosmeticGiftResponse"
                }),
                "{target} slot 0993 lacks the JP-only rival-married gift response"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("ReallyABirthdayPresentForMe_02")
                    && !name.ends_with("YoureGivingThisToMeHow_02")
                    && !name.ends_with("TomorrowIsTheStarryNightFestival_02")
                    && !name.ends_with("SilentPause_02")
                    && !name.ends_with("SilentPause_03")
                    && !name.ends_with("GoodMorningNickName_02")
                    && !name.ends_with("ImTryingToGetSomeWork_02")
            }),
            "{target} slot 0993 retains mechanical duplicate suffixes"
        );
    }
}

#[test]
fn fomt_ann_dialogue_names_gifts_inn_heart_stages_and_regional_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_FOMT_US", 152usize), ("MARY_FOMT_JP", 151usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1011), count, "{target}");
        let names = symbols.names(1011, count);
        for expected in [
            "gText_NPCEvent_Ann_DialogueAndGiftResponses_SingleNeutralGiftResponseReservedThanks",
            "gText_NPCEvent_Ann_DialogueAndGiftResponses_YoungAnimalGiftResponse",
            "gText_NPCEvent_Ann_DialogueAndGiftResponses_InnOrangeHeartFirstConversationNervousGreeting",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1011 lacks {expected}"
            );
        }
        if target == "MARY_FOMT_US" {
            assert!(
                names.iter().flatten().any(|name| {
                    name
                        == "gText_NPCEvent_Ann_DialogueAndGiftResponses_SharedFarmhouseBelowGreenAndInnRedHeartFirstConversationSilence"
                }),
                "{target} slot 1011 lacks the physically shared US silence"
            );
        } else {
            for expected in [
                "gText_NPCEvent_Ann_DialogueAndGiftResponses_FarmhouseBelowGreenHeartFirstConversationSilence",
                "gText_NPCEvent_Ann_DialogueAndGiftResponses_InnPurpleHeartFirstConversationGreeting",
                "gText_NPCEvent_Ann_DialogueAndGiftResponses_InnRedHeartFirstConversationSilence",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot 1011 lacks {expected}"
                );
            }
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("Thanks_02")
                    && !name.ends_with("AwwItsSoCute_02")
                    && !name.ends_with("SilentPause_02")
                    && !name.ends_with("SilentPause_03")
                    && !name.ends_with("Welcome_02")
                    && !name.ends_with("Welcome_03")
            }),
            "{target} slot 1011 retains mechanical duplicate suffixes"
        );
    }
}

#[test]
fn fomt_chicken_festival_invitation_names_animal_eligibility_branches() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1146), 9, "{target}");
        let names = symbols.names(1146, 9);
        for expected in [
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_NoChickensAvailable",
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_OnlyBabyChicksAvailable",
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_HealthyAdultChickenAvailableEntryPrompt",
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_AdultChickensExistButNoneAreHealthy",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1146 lacks {expected}"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("TheChickenFestivalIsTomorrowIn")),
            "{target} slot 1146 retains sentence-derived duplicate invitation names"
        );
    }
}

#[test]
fn mfomt_chicken_festival_invitation_names_marriage_and_regional_slot_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 10usize), ("MARY_MFOMT_JP", 13usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1226), count, "{target}");
        let names = symbols.names(1226, count);
        for expected in [
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_NoChickensUnmarriedPlayer",
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_NoChickensMarriedToRick",
        ] {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1226 lacks {expected}"
            );
        }
        let regional: &[&str] = if target == "MARY_MFOMT_US" {
            &[
                "gText_FestivalEvent_ChickenFestival_EntryInvitation_OnlyBabyChicksAnyMarriageState",
                "gText_FestivalEvent_ChickenFestival_EntryInvitation_HealthyAdultChickenEntryPromptAnyMarriageState",
            ]
        } else {
            &[
                "gText_FestivalEvent_ChickenFestival_EntryInvitation_OnlyBabyChicksUnmarriedPlayer",
                "gText_FestivalEvent_ChickenFestival_EntryInvitation_OnlyBabyChicksMarriedToRick",
                "gText_FestivalEvent_ChickenFestival_EntryInvitation_HealthyAdultChickenEntryPromptUnmarriedPlayer",
                "gText_FestivalEvent_ChickenFestival_EntryInvitation_HealthyAdultChickenEntryPromptMarriedToRick",
            ]
        };
        for expected in regional {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1226 lacks {expected}"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.contains("TheChickenFestivalIsTomorrowIn")),
            "{target} slot 1226 retains sentence-derived duplicate invitation names"
        );
    }
}

#[test]
fn cooking_festival_theme_announcements_use_dish_categories_on_all_targets() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot) in [
        ("MARY_FOMT_US", 1105usize),
        ("MARY_FOMT_JP", 1105usize),
        ("MARY_MFOMT_US", 1185usize),
        ("MARY_MFOMT_JP", 1185usize),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(slot), 5, "{target}");
        let names = symbols.names(slot, 5);
        let expected = [
            "gText_FestivalEvent_CookingFestival_ThemeAnnouncement_JuiceTheme",
            "gText_FestivalEvent_CookingFestival_ThemeAnnouncement_DessertTheme",
            "gText_FestivalEvent_CookingFestival_ThemeAnnouncement_BreadTheme",
            "gText_FestivalEvent_CookingFestival_ThemeAnnouncement_NoodlesTheme",
            "gText_FestivalEvent_CookingFestival_ThemeAnnouncement_RiceTheme",
        ];
        assert_eq!(
            names.iter().map(|name| name.as_deref()).collect::<Vec<_>>(),
            expected.iter().copied().map(Some).collect::<Vec<_>>(),
            "{target} slot {slot:04} theme order"
        );
    }
}

#[test]
fn mfomt_kai_dialogue_names_gift_sharing_rival_stage_and_inn_red_heart() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, count) in [("MARY_MFOMT_US", 170usize), ("MARY_MFOMT_JP", 173usize)] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1095), count, "{target}");
        let names = symbols.names(1095, count);
        assert!(
            names.iter().flatten().any(|name| {
                name
                    == "gText_NPCEvent_Kai_DialogueAndGiftResponses_PopuriKaiRivalEvent03CompletedFirstConversationSigh"
            }),
            "{target} slot 1095 lacks the rival-stage reaction"
        );
        let regional: &[&str] = if target == "MARY_MFOMT_US" {
            &[
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_NeutralGiftResponseAnyMarriageState",
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_YoungAnimalGiftResponseAnyMarriageState",
            ]
        } else {
            &[
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_MarriedNeutralGiftResponse",
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_SingleNeutralGiftResponse",
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_MarriedYoungAnimalGiftResponse",
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_SingleYoungAnimalGiftResponse",
                "gText_NPCEvent_Kai_DialogueAndGiftResponses_Inn2FRedHeartFirstConversationGreeting",
            ]
        };
        for expected in regional {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1095 lacks {expected}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("HeyThanksIAppreciateIt_02")
                    && !name.ends_with("ArentYouTooOldToDo_02")
                    && !name.ends_with("QuestionsInSurprise_02")
                    && !name.ends_with("YoPlayerWhatsUp_02")
            }),
            "{target} slot 1095 retains mechanical duplicate suffixes"
        );
    }
}

#[test]
fn mfomt_gray_japanese_insertions_name_marriage_black_heart_and_library_context() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let count = symbols.text_count(1096);
        let names = symbols.names(1096, count);
        if target == "MARY_MFOMT_JP" {
            for expected in [
                "gText_NPCEvent_Gray_DialogueAndGiftResponses_MarriedNeutralGiftResponse",
                "gText_NPCEvent_Gray_DialogueAndGiftResponses_BlacksmithSingleBlackHeartFirstConversationGreeting",
                "gText_NPCEvent_Gray_DialogueAndGiftResponses_LibraryAfterMaryMarriageRepeatConversationRemainsCommittedToBlacksmithing",
            ] {
                assert!(
                    names.iter().flatten().any(|name| name == expected),
                    "{target} slot 1096 lacks {expected}"
                );
            }
        }
        assert!(
            names.iter().flatten().all(|name| {
                !name.ends_with("ThanksNickNameIAppreciateIt_02")
                    && !name.ends_with("AhCanIHelpYou_02")
                    && !name.ends_with("ILikeBooksTooButIm_02")
            }),
            "{target} slot 1096 retains mechanical JP insertion suffixes"
        );
    }
}

#[test]
fn mfomt_starry_night_spouse_texts_follow_child_stage_and_regional_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let cases = [
        (
            "MARY_MFOMT_US",
            40usize,
            vec![
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_SharedCliffAndGrayDinnerBeforeChildCanWalk",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_SharedCliffGrayDoctorDinnerWithWalkingChild",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_GourmetDinnerBeforeAndAfterChildCanWalk",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_SharedCliffAndGourmetSummitInvitation",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_SharedWonGourmetAndStandardSpouseRingGift",
            ],
        ),
        (
            "MARY_MFOMT_JP",
            47,
            vec![
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_CliffDinnerBeforeChildCanWalk",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_GrayDinnerWithWalkingChild",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_DoctorDinnerWithWalkingChild",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_GourmetSummitInvitation",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_WonRingGift",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_GourmetRingGift",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_StandardSpouseRingGift",
            ],
        ),
    ];

    for (target, expected_count, expected_names) in cases {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(symbols.text_count(1392), expected_count, "{target}");
        let names = symbols.names(1392, expected_count);
        for expected in expected_names {
            assert!(
                names.iter().flatten().any(|name| name == expected),
                "{target} slot 1392 lacks {expected}"
            );
        }
    }
}

#[test]
fn supermarket_script_symbols_follow_each_games_shifted_slots() {
    for (target, basket_id, feather_id, bread_id, wrapping_id, rucksack_id) in [
        ("MARY_FOMT_US", 469, 470, 471, 477, 479),
        ("MARY_FOMT_JP", 469, 470, 471, 477, 479),
        ("MARY_MFOMT_US", 478, 479, 480, 486, 488),
        ("MARY_MFOMT_JP", 478, 479, 480, 486, 488),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (
                basket_id,
                "EventScript_ShopEvent_Supermarket_BasketPurchase",
            ),
            (
                feather_id,
                "EventScript_ShopEvent_Supermarket_BlueFeatherPurchase",
            ),
            (bread_id, "EventScript_ShopEvent_Supermarket_BreadPurchase"),
            (
                wrapping_id,
                "EventScript_ShopEvent_Supermarket_GiftWrappingChoice",
            ),
            (rucksack_id, "EventScript_ShopEvent_Rucksack_PurchaseChoice"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn town_shop_counter_symbols_follow_each_games_shifted_slots() {
    for (
        target,
        clinic_id,
        exam_id,
        inn_id,
        winery_id,
        cafe_id,
        smith_id,
        carpenter_id,
        ranch_id,
    ) in [
        ("MARY_FOMT_US", 482, 483, 484, 485, 486, 487, 488, 489),
        ("MARY_FOMT_JP", 482, 483, 484, 485, 486, 487, 488, 489),
        ("MARY_MFOMT_US", 491, 492, 493, 494, 495, 496, 497, 498),
        ("MARY_MFOMT_JP", 491, 492, 493, 494, 495, 496, 497, 498),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (clinic_id, "EventScript_ShopEvent_Clinic_Counter"),
            (exam_id, "EventScript_ShopEvent_Clinic_ExaminationChoice"),
            (inn_id, "EventScript_ShopEvent_Inn_Counter"),
            (winery_id, "EventScript_ShopEvent_Winery_Counter"),
            (cafe_id, "EventScript_ShopEvent_BeachCafe_Counter"),
            (smith_id, "EventScript_ShopEvent_Blacksmith_OrderCounter"),
            (carpenter_id, "EventScript_ShopEvent_Carpenter_Counter"),
            (ranch_id, "EventScript_ShopEvent_YodelRanch_Counter"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn farmhouse_system_script_symbols_follow_each_games_shifted_slots() {
    for (target, bath_id, sleep_id, bookshelf_id, kitchen_id, record_id, stocking_id, vase_id) in [
        ("MARY_FOMT_US", 341, 342, 344, 349, 351, 354, 359),
        ("MARY_FOMT_JP", 341, 342, 344, 349, 351, 354, 359),
        ("MARY_MFOMT_US", 350, 351, 353, 358, 360, 363, 368),
        ("MARY_MFOMT_JP", 350, 351, 353, 358, 360, 363, 368),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (
                bath_id,
                "EventScript_SystemEvent_BathroomUseAndFamilyExitChoice",
            ),
            (sleep_id, "EventScript_SystemEvent_SleepForDayChoice"),
            (bookshelf_id, "EventScript_SystemEvent_BookshelfMenuChoice"),
            (kitchen_id, "EventScript_SystemEvent_KitchenMenuChoice"),
            (record_id, "EventScript_SystemEvent_RecordPlayerInteraction"),
            (
                stocking_id,
                "EventScript_SystemEvent_StockingGiftCollection",
            ),
            (vase_id, "EventScript_SystemEvent_VaseFlowerDiscardChoice"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn mountain_cottage_system_scripts_follow_each_games_shifted_slots() {
    for (target, fireplace_id, bed_id) in [
        ("MARY_FOMT_US", 531, 533),
        ("MARY_FOMT_JP", 531, 533),
        ("MARY_MFOMT_US", 540, 542),
        ("MARY_MFOMT_JP", 540, 542),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(fireplace_id),
            Some("EventScript_SystemEvent_FireplaceInteraction"),
            "{target} slot {fireplace_id}"
        );
        assert_eq!(
            symbols.script_name(bed_id),
            Some("EventScript_SystemEvent_MountainCottageBedSleepChoice"),
            "{target} slot {bed_id}"
        );
    }
}

#[test]
fn poultry_farm_family_events_have_event_level_symbols() {
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 9),
        ("MARY_MFOMT_JP", 9),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, expected) in [
            (696, "EventScript_NPCEvent_Rick_WorriesAboutLilliasHealth"),
            (699, "EventScript_NPCEvent_Lillia_ReadsRodsLetter"),
            (703, "EventScript_NPCEvent_Rick_ConfrontsKaiAboutPopuri"),
            (704, "EventScript_NPCEvent_Karen_ComfortsLonelyRick"),
            (
                705,
                "EventScript_NPCEvent_RickAndKaren_MarriedDialogueInteractWithRick",
            ),
            (
                706,
                "EventScript_NPCEvent_RickAndKaren_MarriedDialogueInteractWithKaren",
            ),
            (707, "EventScript_NPCEvent_Popuri_AsksKaiForNecklace"),
            (708, "EventScript_NPCEvent_Kai_ReflectsOnPopuri"),
            (709, "EventScript_NPCEvent_Popuri_PlansLilliaBirthdayGift"),
        ] {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(expected),
                "{target} slot {}",
                id + offset
            );
        }
    }
}

#[test]
fn mfomt_fish_pond_interaction_has_a_farm_event_symbol() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(165),
            Some("EventScript_FarmEvent_FishPond_Interaction"),
            "{target} slot 165"
        );
    }
}

#[test]
fn system_and_tutorial_collections_have_event_level_symbols() {
    for (target, expected) in [
        (
            "MARY_FOMT_US",
            &[
                (285, "EventScript_TV_Shopping_PhoneOrder"),
                (716, "EventScript_NPCEvent_Duke_GrapeHarvestInvitation"),
                (
                    737,
                    "EventScript_FarmEvent_FarmIntroduction_ShippingTutorial",
                ),
                (
                    738,
                    "EventScript_SystemEvent_HarvestGoddessItemRequestChoice",
                ),
                (1040, "EventScript_TV_HarvestGoddessRockPaperScissors"),
                (1050, "EventScript_TV_HarvestGoddessMathQuiz"),
                (1052, "EventScript_TV_NewYearSpecialPrograms"),
                (1055, "EventScript_TV_StEmeraldAcademy"),
            ][..],
        ),
        (
            "MARY_FOMT_JP",
            &[
                (285, "EventScript_TV_Shopping_PhoneOrder"),
                (716, "EventScript_NPCEvent_Duke_GrapeHarvestInvitation"),
                (
                    737,
                    "EventScript_FarmEvent_FarmIntroduction_ShippingTutorial",
                ),
                (
                    738,
                    "EventScript_SystemEvent_HarvestGoddessItemRequestChoice",
                ),
                (1040, "EventScript_TV_HarvestGoddessRockPaperScissors"),
                (1050, "EventScript_TV_HarvestGoddessMathQuiz"),
                (1052, "EventScript_TV_NewYearSpecialPrograms"),
                (1055, "EventScript_TV_StEmeraldAcademy"),
            ][..],
        ),
        (
            "MARY_MFOMT_US",
            &[
                (294, "EventScript_TV_Shopping_PhoneOrder"),
                (725, "EventScript_NPCEvent_Duke_GrapeHarvestInvitation"),
                (
                    746,
                    "EventScript_FarmEvent_FarmIntroduction_ShippingTutorial",
                ),
                (
                    747,
                    "EventScript_SystemEvent_HarvestGoddessItemRequestChoice",
                ),
                (1111, "EventScript_TV_HarvestGoddessRockPaperScissors"),
                (1121, "EventScript_TV_HarvestGoddessMathQuiz"),
                (1123, "EventScript_TV_NewYearSpecialPrograms"),
            ][..],
        ),
        (
            "MARY_MFOMT_JP",
            &[
                (294, "EventScript_TV_Shopping_PhoneOrder"),
                (725, "EventScript_NPCEvent_Duke_GrapeHarvestInvitation"),
                (
                    746,
                    "EventScript_FarmEvent_FarmIntroduction_ShippingTutorial",
                ),
                (
                    747,
                    "EventScript_SystemEvent_HarvestGoddessItemRequestChoice",
                ),
                (1111, "EventScript_TV_HarvestGoddessRockPaperScissors"),
                (1121, "EventScript_TV_HarvestGoddessMathQuiz"),
                (1123, "EventScript_TV_NewYearSpecialPrograms"),
            ][..],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in expected {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }
}

#[test]
fn horse_race_spectator_dialogue_symbols_match_across_versions() {
    let fomt_events = [
        (
            1083,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Rick",
        ),
        (
            1084,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Popuri",
        ),
        (
            1085,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Zack",
        ),
        (
            1086,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Jeff",
        ),
        (
            1087,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Karen",
        ),
        (
            1088,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Sasha",
        ),
        (
            1089,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Doug",
        ),
        (
            1090,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Ann",
        ),
        (
            1091,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Thomas",
        ),
        (
            1092,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Duke",
        ),
        (
            1093,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Manna",
        ),
        (
            1094,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Harris",
        ),
        (
            1095,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Saibara",
        ),
        (
            1096,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_May",
        ),
        (
            1097,
            "EventScript_FestivalEvent_SpringHorseRace_SpectatorDialogue_Barley",
        ),
        (
            1105,
            "EventScript_FestivalEvent_CookingFestival_ThemeAnnouncement",
        ),
    ];
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 80),
        ("MARY_MFOMT_JP", 80),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in &fomt_events {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(name),
                "{target} slot {}",
                id + offset
            );
        }
    }
}

#[test]
fn frisbee_tournament_dialogue_symbols_match_across_versions() {
    let fomt_events = [
        (
            1132,
            "EventScript_FestivalEvent_FrisbeeTournament_PreTournamentDialogue_Popuri",
        ),
        (
            1133,
            "EventScript_FestivalEvent_FrisbeeTournament_PreTournamentDialogue_Jeff",
        ),
        (
            1134,
            "EventScript_FestivalEvent_FrisbeeTournament_PreTournamentDialogue_Sasha",
        ),
        (
            1135,
            "EventScript_FestivalEvent_FrisbeeTournament_PreTournamentDialogue_Thomas",
        ),
        (
            1136,
            "EventScript_FestivalEvent_FrisbeeTournament_PreTournamentDialogue_Kai",
        ),
        (
            1137,
            "EventScript_FestivalEvent_FrisbeeTournament_PreTournamentDialogue_Zack",
        ),
        (
            1139,
            "EventScript_FestivalEvent_FrisbeeTournament_PostTournamentDialogue_Popuri",
        ),
        (
            1140,
            "EventScript_FestivalEvent_FrisbeeTournament_PostTournamentDialogue_Jeff",
        ),
        (
            1141,
            "EventScript_FestivalEvent_FrisbeeTournament_PostTournamentDialogue_Sasha",
        ),
        (
            1142,
            "EventScript_FestivalEvent_FrisbeeTournament_PostTournamentDialogue_Thomas",
        ),
        (
            1143,
            "EventScript_FestivalEvent_FrisbeeTournament_PostTournamentDialogue_Kai",
        ),
        (
            1144,
            "EventScript_FestivalEvent_FrisbeeTournament_PostTournamentDialogue_Zack",
        ),
    ];
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 80),
        ("MARY_MFOMT_JP", 80),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in &fomt_events {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(name),
                "{target} slot {}",
                id + offset
            );
        }
    }
}

#[test]
fn chicken_and_cow_festival_dialogue_symbols_match_across_versions() {
    let fomt_events = [
        (
            1149,
            "EventScript_FestivalEvent_ChickenFestival_PreTournamentDialogue_Doug",
        ),
        (
            1150,
            "EventScript_FestivalEvent_ChickenFestival_PreTournamentDialogue_Thomas",
        ),
        (
            1151,
            "EventScript_FestivalEvent_ChickenFestival_PreTournamentDialogue_Duke",
        ),
        (
            1154,
            "EventScript_FestivalEvent_ChickenFestival_PostTournamentDialogue_Rick",
        ),
        (
            1157,
            "EventScript_FestivalEvent_ChickenFestival_PostTournamentDialogue_Thomas",
        ),
        (
            1158,
            "EventScript_FestivalEvent_ChickenFestival_PostTournamentDialogue_Duke",
        ),
        (
            1167,
            "EventScript_FestivalEvent_CowFestival_PreJudgingDialogue_Popuri",
        ),
        (
            1168,
            "EventScript_FestivalEvent_CowFestival_PreJudgingDialogue_Doctor",
        ),
        (
            1175,
            "EventScript_FestivalEvent_CowFestival_PreJudgingDialogue_May",
        ),
        (
            1176,
            "EventScript_FestivalEvent_CowFestival_StartJudgingChoice_Barley",
        ),
    ];
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 80),
        ("MARY_MFOMT_JP", 80),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in &fomt_events {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(name),
                "{target} slot {}",
                id + offset
            );
        }
    }
}

#[test]
fn mfomt_regional_text_insertions_keep_their_actual_semantic_slots() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    for (target, expected) in [
        (
            "MARY_MFOMT_US",
            &[
                "gText_FestivalEvent_CowFestival_PreJudgingDialogue_Doctor_DoctorRecommendsDailyMilkUsingPlayerName",
                "gText_FestivalEvent_CowFestival_PreJudgingDialogue_Doctor_DoctorAsksCowContestConfidence",
            ][..],
        ),
        (
            "MARY_MFOMT_JP",
            &[
                "gText_FestivalEvent_CowFestival_PreJudgingDialogue_Doctor_DoctorRecommendsDailyMilkUsingNickname",
                "gText_FestivalEvent_CowFestival_PreJudgingDialogue_Doctor_DoctorRecommendsDailyMilkUsingPlayerName",
                "gText_FestivalEvent_CowFestival_PreJudgingDialogue_Doctor_DoctorAsksCowContestConfidence",
            ][..],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let actual = symbols
            .names(1248, symbols.text_count(1248))
            .into_iter()
            .map(|name| name.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{target} Cow Festival Doctor text order");
    }

    for (target, expected_slot_1, expected_slot_11) in [
        (
            "MARY_MFOMT_US",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_ChildCallsMamaForRickKaiAndCliff",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_ChildCallsMamaForGrayDoctorWonAndGourmet",
        ),
        (
            "MARY_MFOMT_JP",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_ChildCallsMamaForAllSpouses",
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_GrayInvitesFamilyToEat",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1373, symbols.text_count(1373));
        assert_eq!(
            names[1].as_deref(),
            Some(expected_slot_1),
            "{target} shared child line"
        );
        assert_eq!(
            names[11].as_deref(),
            Some(expected_slot_11),
            "{target} region-dependent slot 11"
        );
    }
}

#[test]
fn regional_branch_text_symbols_remain_scoped_to_their_owning_event() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for forbidden in [
        "gText_BabysBirthday,",
        "gText_DukeWelcomesCliffToWinery,",
        "gText_MyAnniversary,",
        "gText_MyBirthday_02,",
        "gText_FarmSuitsPlayerNamePrefix,",
        "gText_DoctorRecommendsDailyMilk,",
        "gText_DoctorAsksCowContestConfidence,",
        "gText_Mama_02,",
        "gText_DigIn_03,",
    ] {
        assert!(
            !source.lines().any(|line| line.trim() == forbidden),
            "unscoped regional text symbol remains: {forbidden}"
        );
    }
}

#[test]
fn every_region_conditional_text_symbol_has_an_event_scope() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let mut conditions = Vec::<bool>::new();

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#if ") {
            conditions.push(trimmed == "#if defined(MARY_US)" || trimmed == "#if defined(MARY_JP)");
            continue;
        }
        if trimmed == "#endif" {
            conditions.pop();
            continue;
        }
        if !conditions.iter().any(|is_region| *is_region) {
            continue;
        }

        for token in trimmed.split(|character: char| {
            character.is_whitespace() || matches!(character, ',' | '{' | '}' | '(' | ')')
        }) {
            let Some(name) = token.strip_prefix("gText_") else {
                continue;
            };
            assert!(
                name.contains('_'),
                "region-conditional text symbol lacks an owning-event scope at line {}: {}",
                line_index + 1,
                token
            );
        }
    }
}

#[test]
fn mfomt_nickname_region_slots_distinguish_values_from_confirmation_dialogue() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    let us_options = Options::default().define("MARY_MFOMT_US").unwrap();
    let us = parse_text_name_table(&source, &us_options).unwrap();
    let us_rick = us.names(31, us.text_count(31));
    let us_kai = us.names(40, us.text_count(40));
    let us_cliff = us.names(48, us.text_count(48));
    assert_eq!(
        us_rick[7].as_deref(),
        Some("gText_LoveEvent_Rick_Nickname_HiddenJapaneseHoneyNicknameValue")
    );
    assert_eq!(
        us_cliff[7].as_deref(),
        Some("gText_LoveEvent_Cliff_Nickname_HiddenJapaneseHoneyNicknameValue")
    );
    assert_eq!(
        us_kai[6].as_deref(),
        Some("gText_LoveEvent_Kai_Nickname_KaiConfirmsSelectedNickname")
    );

    let jp_options = Options::default().define("MARY_MFOMT_JP").unwrap();
    let jp = parse_text_name_table(&source, &jp_options).unwrap();
    let jp_rick = jp.names(31, jp.text_count(31));
    let jp_kai = jp.names(40, jp.text_count(40));
    let jp_cliff = jp.names(48, jp.text_count(48));
    assert_eq!(
        jp_rick[7].as_deref(),
        Some("gText_LoveEvent_Rick_Nickname_RickConfirmsChickyNickname")
    );
    assert_eq!(
        jp_cliff[7].as_deref(),
        Some("gText_LoveEvent_Cliff_Nickname_CliffConfirmsFruityNickname")
    );
    assert_eq!(
        jp_cliff[8].as_deref(),
        Some("gText_LoveEvent_Cliff_Nickname_CliffConfirmsHoneyNickname")
    );
    assert_eq!(
        jp_kai[6].as_deref(),
        Some("gText_LoveEvent_Kai_Nickname_KaiConfirmsPlayerNameNickname")
    );
}

#[test]
fn cow_fireworks_and_music_festival_symbols_match_their_target_scripts() {
    let common = [
        (
            1179,
            "EventScript_FestivalEvent_CowFestival_PostJudgingDialogue_Popuri",
        ),
        (
            1183,
            "EventScript_FestivalEvent_CowFestival_PostJudgingDialogue_Thomas",
        ),
        (
            1185,
            "EventScript_FestivalEvent_CowFestival_PostJudgingDialogue_Stu",
        ),
        (
            1186,
            "EventScript_FestivalEvent_CowFestival_PostJudgingDialogue_Carter",
        ),
        (
            1204,
            "EventScript_FestivalEvent_MusicFestival_EntryCheck_Carter",
        ),
        (
            1213,
            "EventScript_FestivalEvent_MusicFestival_StartPerformanceChoice_Carter",
        ),
    ];
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 80),
        ("MARY_MFOMT_JP", 80),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in &common {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(name),
                "{target} slot {}",
                id + offset
            );
        }
    }

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, character) in [
            (1194, "Popuri"),
            (1197, "Karen"),
            (1198, "Ann"),
            (1199, "Mary"),
            (1200, "Elli"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_FireworksFestival_Invitation_{character}"
                )),
                "{target} slot {id}"
            );
        }
    }
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, character) in [
            (1274, "Popuri"),
            (1277, "Karen"),
            (1278, "Ann"),
            (1279, "Mary"),
            (1280, "Elli"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_FireworksFestival_Dialogue_{character}"
                )),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn harvest_fall_horse_and_sheep_festival_symbols_match_across_versions() {
    let harvest_characters = [
        "Karen", "Sasha", "Doug", "Ann", "Mary", "Anna", "Thomas", "Elli", "Manna", "Carter",
        "Gotz",
    ];
    let fall_horse_events = [
        (1249, "Rick"),
        (1251, "Zack"),
        (1252, "Doctor"),
        (1253, "Doug"),
        (1254, "Mary"),
        (1255, "Thomas"),
        (1256, "Elli"),
        (1257, "Duke"),
        (1258, "Harris"),
        (1259, "Carter"),
        (1260, "Gray"),
        (1261, "Saibara"),
        (1262, "Gotz"),
        (1263, "Barley"),
    ];
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 80),
        ("MARY_MFOMT_JP", 80),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (index, character) in harvest_characters.iter().enumerate() {
            let id = 1219 + index + offset;
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_HarvestFestival_Dialogue_{character}"
                )),
                "{target} slot {id}"
            );
        }
        assert_eq!(
            symbols.script_name(1236 + offset),
            Some("EventScript_FestivalEvent_HarvestFestival_IngredientContributionChoice_Thomas")
        );
        for &(base_id, character) in &fall_horse_events {
            let id = base_id + offset;
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_FallHorseRace_SpectatorDialogue_{character}"
                )),
                "{target} slot {id}"
            );
        }
        for (id, name) in [
            (
                1267,
                "EventScript_FestivalEvent_SheepFestival_PreJudgingDialogue_Rick",
            ),
            (
                1276,
                "EventScript_FestivalEvent_SheepFestival_StartJudgingChoice_Barley",
            ),
            (
                1284,
                "EventScript_FestivalEvent_SheepFestival_PostJudgingDialogue_Gray",
            ),
        ] {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(name),
                "{target} slot {}",
                id + offset
            );
        }
    }
}

#[test]
fn pumpkin_and_winter_thanksgiving_visits_have_character_symbols() {
    for (target, offset) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 80),
        ("MARY_MFOMT_JP", 80),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, character) in [(1290, "May"), (1291, "Stu"), (1292, "Popuri")] {
            assert_eq!(
                symbols.script_name(id + offset),
                Some(&*format!(
                    "EventScript_FestivalEvent_PumpkinFestival_TreatVisit_{character}"
                )),
                "{target} slot {}",
                id + offset
            );
        }
    }
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, character) in [
            (1294, "Popuri"),
            (1295, "Ann"),
            (1296, "Elli"),
            (1297, "Karen"),
            (1298, "Mary"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_WinterThanksgiving_GiftVisit_{character}"
                )),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn remaining_major_festival_and_character_events_have_stable_symbols() {
    for (target, ids) in [
        ("MARY_FOMT_US", [736, 1066, 1124, 1309]),
        ("MARY_FOMT_JP", [736, 1066, 1124, 1309]),
        ("MARY_MFOMT_US", [745, 1140, 1204, 1395]),
        ("MARY_MFOMT_JP", [745, 1140, 1204, 1395]),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, name) in ids.into_iter().zip([
            "EventScript_NPCEvent_Mary_MarriedLifeAndWriting",
            "EventScript_FestivalEvent_NewYearRiceCakeFestival_Meal",
            "EventScript_FestivalEvent_CookingFestival_DishEntryAndStartJudgingChoice",
            "EventScript_FestivalEvent_NewYearsEve_NoodleFestivalMeal",
        ]) {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, character) in [
            (1153, "Rick"),
            (1154, "Cliff"),
            (1155, "Doctor"),
            (1157, "Gray"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_WinterThanksgiving_GiftVisit_{character}"
                )),
                "{target} slot {id}"
            );
        }
        for (id, character) in [
            (1273, "Rick"),
            (1275, "Cliff"),
            (1276, "Doctor"),
            (1281, "Kai"),
            (1282, "Gray"),
        ] {
            assert_eq!(
                symbols.script_name(id),
                Some(&*format!(
                    "EventScript_FestivalEvent_FireworksFestival_Invitation_{character}"
                )),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn mfomt_new_year_dreams_and_winter_thanksgiving_spouse_gift_have_event_symbols() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();

        assert_eq!(
            symbols.script_name(1130),
            Some("EventScript_FestivalEvent_NewYearsEve_BadDreams"),
            "{target}"
        );
        let dream_names = symbols.names(1130, symbols.text_count(1130));
        assert!(
            dream_names
                .iter()
                .flatten()
                .all(|name| name.starts_with("gText_FestivalEvent_NewYearsEve_BadDreams_")),
            "{target}: {dream_names:?}"
        );
        for branch in ["BlondeMenace_", "MarryMayor_", "SpritesFusion_"] {
            assert!(
                dream_names
                    .iter()
                    .flatten()
                    .any(|name| name.contains(branch)),
                "{target}: missing dream branch {branch}: {dream_names:?}"
            );
        }

        assert_eq!(
            symbols.script_name(1158),
            Some("EventScript_FestivalEvent_WinterThanksgiving_SpouseNighttimeGift"),
            "{target}"
        );
        let gift_names = symbols.names(1158, symbols.text_count(1158));
        assert!(
            gift_names.iter().flatten().all(|name| name
                .starts_with("gText_FestivalEvent_WinterThanksgiving_SpouseNighttimeGift_")),
            "{target}: {gift_names:?}"
        );
        for spouse in [
            "Rick", "Gray", "Cliff", "Kai", "Doctor", "Kappa", "Won", "Gourmet",
        ] {
            assert!(
                gift_names
                    .iter()
                    .flatten()
                    .any(|name| name.contains(spouse)),
                "{target}: missing spouse branch {spouse}: {gift_names:?}"
            );
        }
    }
}

#[test]
fn cooking_festival_judging_texts_are_scoped_to_the_judging_event() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1128),
        ("MARY_FOMT_JP", 1128),
        ("MARY_MFOMT_US", 1208),
        ("MARY_MFOMT_JP", 1208),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_FestivalEvent_CookingFestival_Judging"),
            "{target}"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| name.starts_with("gText_FestivalEvent_CookingFestival_Judging_")),
            "{target}: {names:?}"
        );
    }
}

#[test]
fn major_npc_event_dialogue_is_scoped_to_its_own_event() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id, script_name, text_prefix) in [
        (
            "MARY_FOMT_US",
            753,
            "EventScript_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit",
            "gText_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit_",
        ),
        (
            "MARY_FOMT_JP",
            753,
            "EventScript_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit",
            "gText_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit_",
        ),
        (
            "MARY_MFOMT_US",
            762,
            "EventScript_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit",
            "gText_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit_",
        ),
        (
            "MARY_MFOMT_JP",
            762,
            "EventScript_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit",
            "gText_NPCEvent_Sasha_TeachesJeffToRefuseStoreCredit_",
        ),
        (
            "MARY_FOMT_US",
            717,
            "EventScript_NPCEvent_Cliff_PermanentWineryJob",
            "gText_NPCEvent_Cliff_PermanentWineryJob_",
        ),
        (
            "MARY_FOMT_JP",
            717,
            "EventScript_NPCEvent_Cliff_PermanentWineryJob",
            "gText_NPCEvent_Cliff_PermanentWineryJob_",
        ),
        (
            "MARY_FOMT_US",
            764,
            "EventScript_NPCEvent_Won_MeetsKaren",
            "gText_NPCEvent_Won_MeetsKaren_",
        ),
        (
            "MARY_FOMT_JP",
            764,
            "EventScript_NPCEvent_Won_MeetsKaren",
            "gText_NPCEvent_Won_MeetsKaren_",
        ),
        (
            "MARY_MFOMT_US",
            773,
            "EventScript_NPCEvent_Won_MeetsKaren",
            "gText_NPCEvent_Won_MeetsKaren_",
        ),
        (
            "MARY_MFOMT_JP",
            773,
            "EventScript_NPCEvent_Won_MeetsKaren",
            "gText_NPCEvent_Won_MeetsKaren_",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some(script_name),
            "{target}"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| name.starts_with(text_prefix)),
            "{target}: {names:?}"
        );
    }
}

#[test]
fn harvest_goddess_offering_and_matchmaking_texts_keep_event_scope() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 571),
        ("MARY_FOMT_JP", 571),
        ("MARY_MFOMT_US", 580),
        ("MARY_MFOMT_JP", 580),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_NPCEvent_HarvestGoddess_OfferingsAndMatchmakingChoices"),
            "{target}"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(
            names.iter().flatten().all(|name| name
                .starts_with("gText_NPCEvent_HarvestGoddess_OfferingsAndMatchmakingChoices_")),
            "{target}: {names:?}"
        );
    }
}

#[test]
fn church_confessional_and_elli_doctor_rival_dialogue_keep_event_scope() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id, script_name, text_prefix) in [
        (
            "MARY_FOMT_US",
            139,
            "EventScript_LocationInteraction_ChurchConfessional",
            "gText_LocationInteraction_ChurchConfessional_",
        ),
        (
            "MARY_FOMT_JP",
            139,
            "EventScript_LocationInteraction_ChurchConfessional",
            "gText_LocationInteraction_ChurchConfessional_",
        ),
        (
            "MARY_MFOMT_US",
            147,
            "EventScript_LocationInteraction_ChurchConfessional",
            "gText_LocationInteraction_ChurchConfessional_",
        ),
        (
            "MARY_MFOMT_JP",
            147,
            "EventScript_LocationInteraction_ChurchConfessional",
            "gText_LocationInteraction_ChurchConfessional_",
        ),
        (
            "MARY_FOMT_US",
            944,
            "EventScript_RivalEvent_ElliAndDoctor_01_BlackHeart",
            "gText_RivalEvent_ElliAndDoctor_01_BlackHeart_",
        ),
        (
            "MARY_FOMT_JP",
            944,
            "EventScript_RivalEvent_ElliAndDoctor_01_BlackHeart",
            "gText_RivalEvent_ElliAndDoctor_01_BlackHeart_",
        ),
        (
            "MARY_MFOMT_US",
            1012,
            "EventScript_RivalEvent_ElliAndDoctor_01_BlackHeart",
            "gText_RivalEvent_ElliAndDoctor_01_BlackHeart_",
        ),
        (
            "MARY_MFOMT_JP",
            1012,
            "EventScript_RivalEvent_ElliAndDoctor_01_BlackHeart",
            "gText_RivalEvent_ElliAndDoctor_01_BlackHeart_",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some(script_name),
            "{target}"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| name.starts_with(text_prefix)),
            "{target}: {names:?}"
        );
    }
}

#[test]
fn won_apple_challenge_and_jeff_painting_dialogue_keep_event_scope() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, apple_id, painting_id) in [
        ("MARY_FOMT_US", 811, 751),
        ("MARY_FOMT_JP", 811, 751),
        ("MARY_MFOMT_US", 820, 760),
        ("MARY_MFOMT_JP", 820, 760),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (script_id, script_name, text_prefix) in [
            (
                apple_id,
                "EventScript_NPCEvent_Won_AppleChallenge",
                "gText_NPCEvent_Won_AppleChallenge_",
            ),
            (
                painting_id,
                "EventScript_NPCEvent_Won_DiscoversJeffPaintingTalent",
                "gText_NPCEvent_Won_DiscoversJeffPaintingTalent_",
            ),
        ] {
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name),
                "{target}"
            );
            let names = symbols.names(script_id, symbols.text_count(script_id));
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with(text_prefix)),
                "{target}: {names:?}"
            );
        }
    }
}

#[test]
fn newly_scoped_story_and_system_dialogue_keeps_owning_event_prefix() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, cases) in [
        (
            "MARY_FOMT_US",
            vec![
                (
                    740,
                    "EventScript_NPCEvent_Harris_AjaLetterAdvice",
                    "gText_NPCEvent_Harris_AjaLetterAdvice_",
                ),
                (
                    926,
                    "EventScript_RivalEvent_MaryAndGray_01_BlackHeart",
                    "gText_RivalEvent_MaryAndGray_01_BlackHeart_",
                ),
                (
                    709,
                    "EventScript_NPCEvent_Popuri_PlansLilliaBirthdayGift",
                    "gText_NPCEvent_Popuri_PlansLilliaBirthdayGift_",
                ),
                (
                    1059,
                    "EventScript_SystemMenu_SelectAnimalCategory",
                    "gText_SystemMenu_SelectAnimalCategory_",
                ),
            ],
        ),
        (
            "MARY_FOMT_JP",
            vec![
                (
                    740,
                    "EventScript_NPCEvent_Harris_AjaLetterAdvice",
                    "gText_NPCEvent_Harris_AjaLetterAdvice_",
                ),
                (
                    926,
                    "EventScript_RivalEvent_MaryAndGray_01_BlackHeart",
                    "gText_RivalEvent_MaryAndGray_01_BlackHeart_",
                ),
                (
                    709,
                    "EventScript_NPCEvent_Popuri_PlansLilliaBirthdayGift",
                    "gText_NPCEvent_Popuri_PlansLilliaBirthdayGift_",
                ),
                (
                    1059,
                    "EventScript_SystemMenu_SelectAnimalCategory",
                    "gText_SystemMenu_SelectAnimalCategory_",
                ),
            ],
        ),
        (
            "MARY_MFOMT_US",
            vec![
                (
                    920,
                    "EventScript_SystemEvent_FarmhousePowerOutageSpouseDialogue",
                    "gText_SystemEvent_FarmhousePowerOutageSpouseDialogue_",
                ),
                (
                    749,
                    "EventScript_NPCEvent_Harris_AjaLetterAdvice",
                    "gText_NPCEvent_Harris_AjaLetterAdvice_",
                ),
                (
                    994,
                    "EventScript_RivalEvent_MaryAndGray_01_BlackHeart",
                    "gText_RivalEvent_MaryAndGray_01_BlackHeart_",
                ),
                (
                    718,
                    "EventScript_NPCEvent_Popuri_PlansLilliaBirthdayGift",
                    "gText_NPCEvent_Popuri_PlansLilliaBirthdayGift_",
                ),
                (
                    1132,
                    "EventScript_SystemMenu_SelectAnimalCategory",
                    "gText_SystemMenu_SelectAnimalCategory_",
                ),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            vec![
                (
                    920,
                    "EventScript_SystemEvent_FarmhousePowerOutageSpouseDialogue",
                    "gText_SystemEvent_FarmhousePowerOutageSpouseDialogue_",
                ),
                (
                    749,
                    "EventScript_NPCEvent_Harris_AjaLetterAdvice",
                    "gText_NPCEvent_Harris_AjaLetterAdvice_",
                ),
                (
                    994,
                    "EventScript_RivalEvent_MaryAndGray_01_BlackHeart",
                    "gText_RivalEvent_MaryAndGray_01_BlackHeart_",
                ),
                (
                    718,
                    "EventScript_NPCEvent_Popuri_PlansLilliaBirthdayGift",
                    "gText_NPCEvent_Popuri_PlansLilliaBirthdayGift_",
                ),
                (
                    1132,
                    "EventScript_SystemMenu_SelectAnimalCategory",
                    "gText_SystemMenu_SelectAnimalCategory_",
                ),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (script_id, script_name, text_prefix) in cases {
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name),
                "{target}"
            );
            let names = symbols.names(script_id, symbols.text_count(script_id));
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with(text_prefix)),
                "{target}: {names:?}"
            );
        }
    }
}

#[test]
fn moon_viewing_won_shop_argument_and_starry_night_texts_keep_event_scope() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(786),
            Some("EventScript_NPCEvent_DougAndDuke_ArgumentChoice"),
            "{target}"
        );
        let names = symbols.names(786, symbols.text_count(786));
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| name.starts_with("gText_NPCEvent_DougAndDuke_ArgumentChoice_")),
            "{target}: {names:?}"
        );
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (script_id, script_name, text_prefix) in [
            (
                1324,
                "EventScript_FestivalEvent_MoonViewing_WithSelectedPartner",
                "gText_FestivalEvent_MoonViewing_WithSelectedPartner_",
            ),
            (
                476,
                "EventScript_ShopEvent_Won_LotteryAndShopChoice",
                "gText_ShopEvent_Won_LotteryAndShopChoice_",
            ),
            (
                795,
                "EventScript_NPCEvent_DougAndDuke_ArgumentChoice",
                "gText_NPCEvent_DougAndDuke_ArgumentChoice_",
            ),
            (
                1392,
                "EventScript_FestivalEvent_StarryNight_SpouseAndChildCelebration",
                "gText_FestivalEvent_StarryNight_SpouseAndChildCelebration_",
            ),
        ] {
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name),
                "{target}"
            );
            let names = symbols.names(script_id, symbols.text_count(script_id));
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with(text_prefix)),
                "{target}: {names:?}"
            );
        }
    }
}

#[test]
fn system_farm_tutorial_and_new_year_texts_keep_event_scope() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, cases) in [
        (
            "MARY_FOMT_US",
            vec![
                (
                    318,
                    "EventScript_SystemEvent_BrowseRecipeBook",
                    "gText_SystemEvent_BrowseRecipeBook_",
                ),
                (
                    350,
                    "EventScript_SystemEvent_PlayerConditionTimeAndWeatherMessages",
                    "gText_SystemEvent_PlayerConditionTimeAndWeatherMessages_",
                ),
                (
                    354,
                    "EventScript_SystemEvent_StockingGiftCollection",
                    "gText_SystemEvent_StockingGiftCollection_",
                ),
                (
                    406,
                    "EventScript_FestivalEvent_NewYearCelebration_StartChoice",
                    "gText_FestivalEvent_NewYearCelebration_StartChoice_",
                ),
                (
                    1060,
                    "EventScript_TutorialEvent_CowCareInstructions",
                    "gText_TutorialEvent_CowCareInstructions_",
                ),
                (
                    1062,
                    "EventScript_TutorialEvent_SheepCareInstructions",
                    "gText_TutorialEvent_SheepCareInstructions_",
                ),
            ],
        ),
        (
            "MARY_FOMT_JP",
            vec![
                (
                    318,
                    "EventScript_SystemEvent_BrowseRecipeBook",
                    "gText_SystemEvent_BrowseRecipeBook_",
                ),
                (
                    350,
                    "EventScript_SystemEvent_PlayerConditionTimeAndWeatherMessages",
                    "gText_SystemEvent_PlayerConditionTimeAndWeatherMessages_",
                ),
                (
                    354,
                    "EventScript_SystemEvent_StockingGiftCollection",
                    "gText_SystemEvent_StockingGiftCollection_",
                ),
                (
                    406,
                    "EventScript_FestivalEvent_NewYearCelebration_StartChoice",
                    "gText_FestivalEvent_NewYearCelebration_StartChoice_",
                ),
                (
                    1060,
                    "EventScript_TutorialEvent_CowCareInstructions",
                    "gText_TutorialEvent_CowCareInstructions_",
                ),
                (
                    1062,
                    "EventScript_TutorialEvent_SheepCareInstructions",
                    "gText_TutorialEvent_SheepCareInstructions_",
                ),
            ],
        ),
        (
            "MARY_MFOMT_US",
            vec![
                (
                    165,
                    "EventScript_FarmEvent_FishPond_Interaction",
                    "gText_FarmEvent_FishPond_Interaction_",
                ),
                (
                    327,
                    "EventScript_SystemEvent_BrowseRecipeBook",
                    "gText_SystemEvent_BrowseRecipeBook_",
                ),
                (
                    363,
                    "EventScript_SystemEvent_StockingGiftCollection",
                    "gText_SystemEvent_StockingGiftCollection_",
                ),
                (
                    415,
                    "EventScript_FestivalEvent_NewYearCelebration_StartChoice",
                    "gText_FestivalEvent_NewYearCelebration_StartChoice_",
                ),
                (
                    1133,
                    "EventScript_TutorialEvent_CowCareInstructions",
                    "gText_TutorialEvent_CowCareInstructions_",
                ),
                (
                    1135,
                    "EventScript_TutorialEvent_SheepCareInstructions",
                    "gText_TutorialEvent_SheepCareInstructions_",
                ),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            vec![
                (
                    165,
                    "EventScript_FarmEvent_FishPond_Interaction",
                    "gText_FarmEvent_FishPond_Interaction_",
                ),
                (
                    327,
                    "EventScript_SystemEvent_BrowseRecipeBook",
                    "gText_SystemEvent_BrowseRecipeBook_",
                ),
                (
                    363,
                    "EventScript_SystemEvent_StockingGiftCollection",
                    "gText_SystemEvent_StockingGiftCollection_",
                ),
                (
                    415,
                    "EventScript_FestivalEvent_NewYearCelebration_StartChoice",
                    "gText_FestivalEvent_NewYearCelebration_StartChoice_",
                ),
                (
                    1133,
                    "EventScript_TutorialEvent_CowCareInstructions",
                    "gText_TutorialEvent_CowCareInstructions_",
                ),
                (
                    1135,
                    "EventScript_TutorialEvent_SheepCareInstructions",
                    "gText_TutorialEvent_SheepCareInstructions_",
                ),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (script_id, script_name, text_prefix) in cases {
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name),
                "{target}"
            );
            let names = symbols.names(script_id, symbols.text_count(script_id));
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with(text_prefix)),
                "{target}: {names:?}"
            );
        }
    }
}

#[test]
fn mfomt_family_and_girls_cooking_events_have_stable_symbols() {
    let expected = [
        (876, "EventScript_NPCEvent_Gourmet_FamilyCelebrationChoice"),
        (882, "EventScript_NPCEvent_Won_FamilyCelebrationChoice"),
        (
            885,
            "EventScript_FamilyEvent_Gourmet_FamilyCelebrationAndAnniversaryChoice",
        ),
        (
            894,
            "EventScript_FamilyEvent_Gourmet_FamilyCelebrationWithBirthdayGiftChoice",
        ),
        (
            897,
            "EventScript_FamilyEvent_Kappa_FamilyCelebrationWithBirthdayGiftChoice",
        ),
        (
            900,
            "EventScript_FamilyEvent_Won_FamilyCelebrationWithBirthdayGiftChoice",
        ),
        (
            920,
            "EventScript_SystemEvent_FarmhousePowerOutageSpouseDialogue",
        ),
        (
            925,
            "EventScript_NPCEvent_VillageGirlsCookingRequest_InvitationChoice",
        ),
        (
            926,
            "EventScript_NPCEvent_VillageGirlsCookingRequest_PopuriOrderChoice",
        ),
        (
            927,
            "EventScript_NPCEvent_VillageGirlsCookingRequest_KarenOrderChoice",
        ),
        (
            928,
            "EventScript_NPCEvent_VillageGirlsCookingRequest_AnnOrderChoice",
        ),
        (
            929,
            "EventScript_NPCEvent_VillageGirlsCookingRequest_MaryOrderChoice",
        ),
        (
            930,
            "EventScript_NPCEvent_VillageGirlsCookingRequest_ElliOrderChoice",
        ),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in &expected {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }
}

#[test]
fn mfomt_system_dispatcher_debug_viewer_and_quiz_have_stable_symbols() {
    let expected = [
        (
            413,
            "EventScript_SystemEvent_FestivalHostInteractionDispatcher",
        ),
        (433, "EventScript_SystemEvent_DebugPortraitExpressionViewer"),
        (1131, "EventScript_SystemEvent_OneHundredQuestionQuiz"),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for &(id, name) in &expected {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }
}

#[test]
fn entrance_event_dispatchers_are_named_by_verified_destination_location() {
    let fomt_ids = [
        105, 108, 136, 179, 205, 362, 370, 371, 372, 373, 374, 375, 376, 377, 378, 523, 528, 530,
        560,
    ];
    let mfomt_ids = [
        113, 116, 144, 188, 214, 371, 379, 380, 381, 382, 383, 384, 385, 386, 387, 532, 537, 539,
        569,
    ];
    let locations = [
        "ZackHouse",
        "KaiRestaurant",
        "BehindChurch",
        "GotzHouse",
        "SupermarketBackRoom",
        "HarvestSpritesHut",
        "Church",
        "EllenHouse",
        "Supermarket",
        "MineralClinic1F",
        "Inn1F",
        "BasilHouse1F",
        "Library1F",
        "MayorHouse",
        "AjaWinery1F",
        "PoultryFarmHouse1F",
        "Blacksmith",
        "YodelRanchHouse1F",
        "SouthSideTown",
    ];
    for (target, ids) in [
        ("MARY_FOMT_US", &fomt_ids[..]),
        ("MARY_FOMT_JP", &fomt_ids[..]),
        ("MARY_MFOMT_US", &mfomt_ids[..]),
        ("MARY_MFOMT_JP", &mfomt_ids[..]),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (&id, &location) in ids.iter().zip(&locations) {
            let expected =
                format!("EventScript_LocationTransition_Enter{location}WithEventDispatch");
            assert_eq!(
                symbols.script_name(id),
                Some(expected.as_str()),
                "{target} slot {id}"
            );
            let names = symbols.names(id, symbols.text_count(id));
            assert!(
                names.iter().flatten().all(|name| name
                    == &format!("gText_LocationTransition_Enter{location}WithEventDispatch")),
                "{target} slot {id}: {names:?}"
            );
        }
    }
}

#[test]
fn semantic_symbol_table_has_no_generated_generic_script_names() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for forbidden in [
        "    EventScript_NPCEvent_Cutscene",
        "    EventScript_NPCEvent_Dialogue_",
        "    EventScript_CharacterDialogueCollection",
        "    EventScript_Handle",
        "    EventScript_SystemEvent_Numbered",
    ] {
        assert!(
            !source.contains(forbidden),
            "generic script name remains: {forbidden}"
        );
    }
}

#[test]
fn event_symbol_hierarchy_uses_npc_names_and_ordered_relationship_stages() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    assert!(
        !source.contains("CharacterEvent_"),
        "NPC event symbols must use the NPCEvent category"
    );
    for stage in ["_Stage1_", "_Stage2_", "_Stage3_", "_Stage4_"] {
        assert!(
            !source.contains(stage),
            "relationship symbol retains a mechanical stage label: {stage}"
        );
    }

    for (category, stage) in [
        ("LoveEvent", "BlackHeart"),
        ("LoveEvent", "PurpleHeart"),
        ("LoveEvent", "BlueHeart"),
        ("LoveEvent", "YellowHeart"),
        ("RivalEvent", "BlackHeart"),
        ("RivalEvent", "BlueHeart"),
        ("RivalEvent", "GreenHeart"),
        ("RivalEvent", "OrangeHeart"),
    ] {
        for line in source
            .lines()
            .filter(|line| line.contains(category) && line.contains(stage))
        {
            assert!(
                line.contains("_01_BlackHeart")
                    || line.contains("_02_PurpleHeart")
                    || line.contains("_02_BlueHeart")
                    || line.contains("_03_BlueHeart")
                    || line.contains("_03_GreenHeart")
                    || line.contains("_04_YellowHeart")
                    || line.contains("_04_OrangeHeart"),
                "relationship stage lacks its ordered hierarchy: {line}"
            );
        }
    }
}

#[test]
fn text_symbols_are_unique_to_their_owning_script_on_every_target() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let mut owners = std::collections::HashMap::<String, String>::new();

        for script_id in 0..slot_count {
            let Some(script_name) = symbols.script_name(script_id) else {
                continue;
            };
            for text_name in symbols
                .names(script_id, symbols.text_count(script_id))
                .into_iter()
                .flatten()
            {
                if let Some(previous) = owners.insert(text_name.clone(), script_name.to_owned()) {
                    assert_eq!(
                        previous, script_name,
                        "{target}: {text_name} is shared by {previous} and {script_name}"
                    );
                }
            }
        }
    }
}

#[test]
fn npc_dialogue_collection_texts_are_scoped_to_their_owning_script() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let mut audited_scripts = 0usize;

    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for script_id in 0..slot_count {
            let Some(script_name) = symbols.script_name(script_id) else {
                continue;
            };
            let is_dialogue_collection = script_name.starts_with("EventScript_NPCEvent_")
                && (script_name.ends_with("_DialogueAndGiftResponses")
                    || script_name.ends_with("_DialogueAndInteractions")
                    || script_name == "EventScript_NPCEvent_Spouse_BedtimeDialogue");
            if !is_dialogue_collection {
                continue;
            }
            audited_scripts += 1;
            let expected_prefix = format!(
                "gText_{}_",
                script_name.strip_prefix("EventScript_").unwrap()
            );
            for text_name in symbols
                .names(script_id, symbols.text_count(script_id))
                .into_iter()
                .flatten()
            {
                assert!(
                    text_name.starts_with(&expected_prefix),
                    "{target} slot {script_id} {script_name} has an unscoped text: {text_name}"
                );
            }
        }
    }

    assert!(
        audited_scripts >= 60,
        "dialogue collection audit unexpectedly covered only {audited_scripts} target scripts"
    );
}

#[test]
fn paired_npc_family_and_ann_cliff_rival_events_keep_event_scoped_texts() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let cases = [
        (
            "MARY_FOMT_US",
            695usize,
            "gText_NPCEvent_RickAndPopuri_RushToSickLillia_",
        ),
        (
            "MARY_MFOMT_US",
            704,
            "gText_NPCEvent_RickAndPopuri_RushToSickLillia_",
        ),
        (
            "MARY_FOMT_US",
            711,
            "gText_NPCEvent_May_PhoneCallWithJoanna_",
        ),
        (
            "MARY_MFOMT_US",
            720,
            "gText_NPCEvent_May_PhoneCallWithJoanna_",
        ),
        ("MARY_FOMT_US", 726, "gText_NPCEvent_Basil_PublishingAward_"),
        (
            "MARY_MFOMT_US",
            735,
            "gText_NPCEvent_Basil_PublishingAward_",
        ),
        (
            "MARY_FOMT_US",
            792,
            "gText_FamilyEvent_Doug_BirthdayGiftFromAnn_",
        ),
        (
            "MARY_MFOMT_US",
            801,
            "gText_FamilyEvent_Doug_BirthdayGiftFromAnn_",
        ),
        (
            "MARY_FOMT_US",
            795,
            "gText_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe_",
        ),
        (
            "MARY_MFOMT_US",
            804,
            "gText_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe_",
        ),
        ("MARY_FOMT_US", 804, "gText_NPCEvent_Zack_VisitsSickLillia_"),
        (
            "MARY_MFOMT_US",
            813,
            "gText_NPCEvent_Zack_VisitsSickLillia_",
        ),
        (
            "MARY_FOMT_US",
            908,
            "gText_RivalEvent_AnnAndCliff_02_BlueHeart_",
        ),
        (
            "MARY_MFOMT_US",
            976,
            "gText_RivalEvent_AnnAndCliff_02_BlueHeart_",
        ),
        (
            "MARY_FOMT_US",
            889,
            "gText_RivalEvent_PopuriAndKai_02_BlueHeart_",
        ),
        (
            "MARY_MFOMT_US",
            957,
            "gText_RivalEvent_PopuriAndKai_02_BlueHeart_",
        ),
        ("MARY_MFOMT_US", 80, "gText_NPCEvent_Ann_Introduction_"),
        (
            "MARY_MFOMT_US",
            726,
            "gText_NPCEvent_DukeAndManna_GrapeHarvestJobOffer_",
        ),
        (
            "MARY_MFOMT_US",
            927,
            "gText_NPCEvent_VillageGirlsCookingRequest_KarenOrderChoice_",
        ),
        (
            "MARY_MFOMT_US",
            932,
            "gText_NPCEvent_Won_AppleShuffleIntroductionChoice_",
        ),
        (
            "MARY_FOMT_US",
            886,
            "gText_RivalEvent_PopuriAndKai_01_BlackHeart_",
        ),
        (
            "MARY_MFOMT_US",
            954,
            "gText_RivalEvent_PopuriAndKai_01_BlackHeart_",
        ),
        (
            "MARY_FOMT_US",
            876,
            "gText_RivalEvent_RickAndKaren_04_OrangeHeart_",
        ),
        (
            "MARY_MFOMT_US",
            944,
            "gText_RivalEvent_RickAndKaren_04_OrangeHeart_",
        ),
        (
            "MARY_FOMT_US",
            737,
            "gText_FarmEvent_FarmIntroduction_ShippingTutorial_",
        ),
        (
            "MARY_MFOMT_US",
            746,
            "gText_FarmEvent_FarmIntroduction_ShippingTutorial_",
        ),
        (
            "MARY_FOMT_US",
            1146,
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_",
        ),
        (
            "MARY_MFOMT_US",
            1226,
            "gText_FestivalEvent_ChickenFestival_EntryInvitation_",
        ),
        (
            "MARY_MFOMT_US",
            85,
            "gText_NPCEvent_Elli_IntroductionTreatStusInjury_",
        ),
        (
            "MARY_MFOMT_US",
            925,
            "gText_NPCEvent_VillageGirlsCookingRequest_InvitationChoice_",
        ),
        (
            "MARY_MFOMT_US",
            929,
            "gText_NPCEvent_VillageGirlsCookingRequest_MaryOrderChoice_",
        ),
        (
            "MARY_FOMT_US",
            344,
            "gText_SystemEvent_BookshelfMenuChoice_",
        ),
        (
            "MARY_MFOMT_US",
            353,
            "gText_SystemEvent_BookshelfMenuChoice_",
        ),
        (
            "MARY_FOMT_US",
            359,
            "gText_SystemEvent_VaseFlowerDiscardChoice_",
        ),
        (
            "MARY_MFOMT_US",
            368,
            "gText_SystemEvent_VaseFlowerDiscardChoice_",
        ),
        (
            "MARY_FOMT_US",
            731,
            "gText_NPCEvent_Anna_CookingLessonsInvitationChoice_",
        ),
        (
            "MARY_FOMT_US",
            1001,
            "gText_NPCEvent_Nappy_DialogueAndWorkChoices_",
        ),
        (
            "MARY_MFOMT_US",
            1070,
            "gText_NPCEvent_Nappy_DialogueAndWorkChoices_",
        ),
        ("MARY_FOMT_US", 349, "gText_SystemEvent_KitchenMenuChoice_"),
        ("MARY_MFOMT_US", 358, "gText_SystemEvent_KitchenMenuChoice_"),
        (
            "MARY_FOMT_US",
            479,
            "gText_ShopEvent_Rucksack_PurchaseChoice_",
        ),
        (
            "MARY_MFOMT_US",
            488,
            "gText_ShopEvent_Rucksack_PurchaseChoice_",
        ),
        (
            "MARY_FOMT_US",
            483,
            "gText_ShopEvent_Clinic_ExaminationChoice_",
        ),
        (
            "MARY_MFOMT_US",
            492,
            "gText_ShopEvent_Clinic_ExaminationChoice_",
        ),
        (
            "MARY_FOMT_US",
            873,
            "gText_RivalEvent_RickAndKaren_01_BlackHeart_",
        ),
        (
            "MARY_MFOMT_US",
            941,
            "gText_RivalEvent_RickAndKaren_01_BlackHeart_",
        ),
        (
            "MARY_FOMT_US",
            894,
            "gText_RivalEvent_PopuriAndKai_04_OrangeHeart_",
        ),
        (
            "MARY_MFOMT_US",
            962,
            "gText_RivalEvent_PopuriAndKai_04_OrangeHeart_",
        ),
        (
            "MARY_FOMT_US",
            932,
            "gText_RivalEvent_MaryAndGray_03_GreenHeart_",
        ),
        (
            "MARY_MFOMT_US",
            1000,
            "gText_RivalEvent_MaryAndGray_03_GreenHeart_",
        ),
        (
            "MARY_FOMT_US",
            950,
            "gText_RivalEvent_ElliAndDoctor_03_GreenHeart_",
        ),
        (
            "MARY_MFOMT_US",
            1018,
            "gText_RivalEvent_ElliAndDoctor_03_GreenHeart_",
        ),
        (
            "MARY_FOMT_US",
            703,
            "gText_NPCEvent_Rick_ConfrontsKaiAboutPopuri_",
        ),
        (
            "MARY_MFOMT_US",
            712,
            "gText_NPCEvent_Rick_ConfrontsKaiAboutPopuri_",
        ),
        (
            "MARY_FOMT_US",
            713,
            "gText_NPCEvent_Saibara_VisitsEllenSecondVisit_",
        ),
        (
            "MARY_MFOMT_US",
            722,
            "gText_NPCEvent_Saibara_VisitsEllenSecondVisit_",
        ),
        ("MARY_FOMT_US", 721, "gText_NPCEvent_Manna_FlattersJeff_"),
        ("MARY_MFOMT_US", 730, "gText_NPCEvent_Manna_FlattersJeff_"),
        (
            "MARY_FOMT_US",
            733,
            "gText_NPCEvent_MaryAndGray_BookAndHealth_",
        ),
        (
            "MARY_MFOMT_US",
            742,
            "gText_NPCEvent_MaryAndGray_BookAndHealth_",
        ),
        (
            "MARY_FOMT_US",
            756,
            "gText_NPCEvent_Manna_DiscussesAjasDeparture_MainScene_",
        ),
        (
            "MARY_MFOMT_US",
            765,
            "gText_NPCEvent_Manna_DiscussesAjasDeparture_MainScene_",
        ),
        (
            "MARY_MFOMT_US",
            75,
            "gText_NPCEvent_Popuri_IntroductionReturnChickenChoice_",
        ),
        ("MARY_MFOMT_US", 723, "gText_NPCEvent_Kai_CooksForGray_"),
        (
            "MARY_FOMT_US",
            509,
            "gText_LocationInteraction_InspectPlayerCottageSign_",
        ),
        (
            "MARY_MFOMT_US",
            518,
            "gText_LocationInteraction_InspectPlayerCottageSign_",
        ),
        (
            "MARY_FOMT_US",
            783,
            "gText_NPCEvent_Cliff_LeavesMineralTown_",
        ),
        (
            "MARY_MFOMT_US",
            792,
            "gText_NPCEvent_Cliff_LeavesMineralTown_",
        ),
        (
            "MARY_FOMT_US",
            821,
            "gText_FestivalEvent_ShootingStar_WishChoice_",
        ),
        (
            "MARY_MFOMT_US",
            830,
            "gText_FestivalEvent_ShootingStar_WishChoice_",
        ),
        (
            "MARY_FOMT_US",
            899,
            "gText_RivalMarriageEvent_PopuriAndKai_05_WeddingAttendanceChoice_",
        ),
        (
            "MARY_MFOMT_US",
            967,
            "gText_RivalMarriageEvent_PopuriAndKai_05_WeddingAttendanceChoice_",
        ),
        (
            "MARY_FOMT_US",
            1292,
            "gText_FestivalEvent_PumpkinFestival_TreatVisit_Popuri_",
        ),
        (
            "MARY_MFOMT_US",
            1372,
            "gText_FestivalEvent_PumpkinFestival_TreatVisit_Popuri_",
        ),
        (
            "MARY_MFOMT_US",
            928,
            "gText_NPCEvent_VillageGirlsCookingRequest_AnnOrderChoice_",
        ),
        (
            "MARY_MFOMT_US",
            930,
            "gText_NPCEvent_VillageGirlsCookingRequest_ElliOrderChoice_",
        ),
        (
            "MARY_FOMT_US",
            104,
            "gText_FarmEvent_Dog_StartFrisbeePracticeChoice_",
        ),
        (
            "MARY_MFOMT_US",
            112,
            "gText_FarmEvent_Dog_StartFrisbeePracticeChoice_",
        ),
        (
            "MARY_FOMT_US",
            123,
            "gText_NPCEvent_PoultryFarmFamily_BlocksUpstairsAccess_",
        ),
        (
            "MARY_MFOMT_US",
            131,
            "gText_NPCEvent_PoultryFarmFamily_BlocksUpstairsAccess_",
        ),
        (
            "MARY_FOMT_US",
            788,
            "gText_NPCEvent_DougAndDuke_ArgumentFollowupAnnDialogue_",
        ),
        (
            "MARY_MFOMT_US",
            797,
            "gText_NPCEvent_DougAndDuke_ArgumentFollowupAnnDialogue_",
        ),
        (
            "MARY_FOMT_US",
            1000,
            "gText_NPCEvent_Timid_DialogueAndWorkChoices_",
        ),
        (
            "MARY_MFOMT_US",
            1069,
            "gText_NPCEvent_Timid_DialogueAndWorkChoices_",
        ),
        (
            "MARY_FOMT_US",
            1003,
            "gText_NPCEvent_Chef_DialogueAndWorkChoices_",
        ),
        (
            "MARY_MFOMT_US",
            1072,
            "gText_NPCEvent_Chef_DialogueAndWorkChoices_",
        ),
        (
            "MARY_FOMT_US",
            1029,
            "gText_NPCEvent_May_PhoneCallWithJoanna_FollowupMayDialogue_",
        ),
        (
            "MARY_MFOMT_US",
            1099,
            "gText_NPCEvent_May_PhoneCallWithJoanna_FollowupMayDialogue_",
        ),
        (
            "MARY_FOMT_US",
            1293,
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_",
        ),
        (
            "MARY_MFOMT_US",
            1373,
            "gText_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration_",
        ),
        (
            "MARY_FOMT_US",
            1305,
            "gText_FestivalEvent_StarryNight_WithMaryFamily_",
        ),
        (
            "MARY_MFOMT_US",
            1390,
            "gText_FestivalEvent_StarryNight_WithMaryFamily_",
        ),
        ("MARY_FOMT_US", 724, "gText_NPCEvent_Basil_LetterAdvice_"),
        ("MARY_MFOMT_US", 733, "gText_NPCEvent_Basil_LetterAdvice_"),
        (
            "MARY_FOMT_US",
            732,
            "gText_NPCEvent_Anna_CookingLesson_Cheese_",
        ),
        (
            "MARY_FOMT_US",
            779,
            "gText_NPCEvent_Carter_OpensChurchBackDoorChoice_",
        ),
        (
            "MARY_MFOMT_US",
            788,
            "gText_NPCEvent_Carter_OpensChurchBackDoorChoice_",
        ),
        (
            "MARY_MFOMT_US",
            924,
            "gText_NPCEvent_MineralTownGirls_BigBedSleepover_",
        ),
        (
            "MARY_MFOMT_US",
            549,
            "gText_SystemEvent_CardCollectorChisatoScoreMessages_",
        ),
        ("MARY_MFOMT_US", 76, "gText_NPCEvent_Karen_Introduction_"),
        (
            "MARY_FOMT_US",
            1244,
            "gText_FestivalEvent_MoonViewing_Choice_",
        ),
        (
            "MARY_FOMT_US",
            745,
            "gText_NPCEvent_Ellen_GrandfathersLetter_",
        ),
        (
            "MARY_MFOMT_US",
            754,
            "gText_NPCEvent_Ellen_GrandfathersLetter_",
        ),
        (
            "MARY_MFOMT_US",
            1388,
            "gText_FestivalEvent_StarryNight_WithRickFamily_",
        ),
        (
            "MARY_FOMT_US",
            710,
            "gText_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall_",
        ),
        (
            "MARY_MFOMT_US",
            719,
            "gText_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall_",
        ),
        (
            "MARY_FOMT_US",
            718,
            "gText_NPCEvent_DukeAndManna_MissingJuiceArgument_",
        ),
        (
            "MARY_MFOMT_US",
            727,
            "gText_NPCEvent_DukeAndManna_MissingJuiceArgument_",
        ),
        (
            "MARY_FOMT_US",
            742,
            "gText_NPCEvent_Ellen_WhiteFlowerLegend_",
        ),
        (
            "MARY_MFOMT_US",
            751,
            "gText_NPCEvent_Ellen_WhiteFlowerLegend_",
        ),
        (
            "MARY_FOMT_US",
            744,
            "gText_NPCEvent_Ellen_WhiteFlowerDiscovery_",
        ),
        (
            "MARY_MFOMT_US",
            753,
            "gText_NPCEvent_Ellen_WhiteFlowerDiscovery_",
        ),
        (
            "MARY_FOMT_US",
            747,
            "gText_NPCEvent_Ellen_GrandfathersLetterFollowupElliDialogue_",
        ),
        (
            "MARY_MFOMT_US",
            756,
            "gText_NPCEvent_Ellen_GrandfathersLetterFollowupElliDialogue_",
        ),
        (
            "MARY_FOMT_US",
            777,
            "gText_NPCEvent_Carter_MysteriousVoice_",
        ),
        (
            "MARY_MFOMT_US",
            786,
            "gText_NPCEvent_Carter_MysteriousVoice_",
        ),
        (
            "MARY_FOMT_US",
            787,
            "gText_NPCEvent_DougAndDuke_ArgumentFollowupDougDialogue_",
        ),
        (
            "MARY_MFOMT_US",
            796,
            "gText_NPCEvent_DougAndDuke_ArgumentFollowupDougDialogue_",
        ),
        (
            "MARY_FOMT_US",
            1002,
            "gText_NPCEvent_Bold_DialogueAndWorkChoices_",
        ),
        (
            "MARY_MFOMT_US",
            1071,
            "gText_NPCEvent_Bold_DialogueAndWorkChoices_",
        ),
        (
            "MARY_FOMT_US",
            1004,
            "gText_NPCEvent_Aqua_DialogueAndWorkChoices_",
        ),
        (
            "MARY_MFOMT_US",
            1073,
            "gText_NPCEvent_Aqua_DialogueAndWorkChoices_",
        ),
        (
            "MARY_MFOMT_US",
            740,
            "gText_NPCEvent_Anna_CookingLessonsInvitation_",
        ),
        ("MARY_MFOMT_US", 741, "gText_NPCEvent_Anna_CookingClass_"),
        ("MARY_MFOMT_US", 759, "gText_NPCEvent_ElliAndStu_Storytime_"),
        (
            "MARY_FOMT_US",
            915,
            "gText_RivalEvent_AnnAndCliff_04_OrangeHeart_",
        ),
        (
            "MARY_MFOMT_US",
            983,
            "gText_RivalEvent_AnnAndCliff_04_OrangeHeart_",
        ),
    ];

    for (target, script_id, expected_prefix) in cases {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for text_name in symbols
            .names(script_id, symbols.text_count(script_id))
            .into_iter()
            .flatten()
        {
            assert!(
                text_name.starts_with(expected_prefix),
                "{target} slot {script_id} has an unscoped text: {text_name}"
            );
        }
    }
}

#[test]
fn numbered_script_symbols_are_textless_placeholder_slots() {
    let fomt_numbered = [
        4, 14, 15, 80, 81, 82, 83, 84, 570, 835, 836, 837, 838, 839, 840, 841, 868, 869, 870, 871,
        872, 964, 965, 966, 967, 968, 1064, 1327, 1328,
    ];
    let mfomt_numbered = [
        4, 14, 15, 88, 89, 90, 91, 92, 579, 844, 845, 846, 847, 848, 849, 850, 916, 935, 936, 937,
        938, 939, 940, 1032, 1033, 1034, 1035, 1036, 1060, 1102, 1137, 1138, 1156, 1374, 1375,
        1376, 1377, 1378, 1379, 1380, 1383, 1413, 1414, 1415,
    ];
    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();

        let mut actual_numbered = Vec::new();
        for id in 0..slot_count {
            let Some(name) = symbols.script_name(id) else {
                continue;
            };
            let numbered_name = format!("EventScript_{id:04}");
            if name == numbered_name {
                actual_numbered.push(id);
                assert_eq!(
                    symbols.text_count(id),
                    0,
                    "{target} slot {id} still has text and therefore needs an event-level name"
                );
            }
        }
        let expected: &[usize] = if target.starts_with("MARY_FOMT_") {
            &fomt_numbered
        } else {
            &mfomt_numbered
        };
        assert_eq!(
            actual_numbered, expected,
            "{target}: numbered script inventory changed without an evidence-backed audit update"
        );
    }
}

#[test]
fn text_symbols_are_owned_by_only_one_script_per_target() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let mut owners = HashMap::new();
        for script_id in 0..slot_count {
            for text_name in symbols
                .names(script_id, symbols.text_count(script_id))
                .into_iter()
                .flatten()
            {
                if let Some(previous_id) = owners.insert(text_name.clone(), script_id) {
                    panic!(
                        "{target}: text symbol {text_name} is shared by scripts {previous_id} and {script_id}"
                    );
                }
            }
        }
    }
}

#[test]
fn semantic_symbol_table_has_no_unscoped_sentence_derived_text_names() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let sentence_starts = [
        "gText_I",
        "gText_You",
        "gText_We",
        "gText_He",
        "gText_She",
        "gText_They",
        "gText_This",
        "gText_That",
        "gText_What",
        "gText_Why",
        "gText_How",
        "gText_Where",
        "gText_When",
        "gText_Hi",
        "gText_Hello",
        "gText_Oh",
        "gText_Ah",
        "gText_Please",
        "gText_Dont",
        "gText_Cant",
        "gText_Isnt",
        "gText_Lets",
        "gText_Today",
        "gText_Tomorrow",
    ];
    let structured_prefixes = [
        "gText_NPCEvent_",
        "gText_FestivalEvent_",
        "gText_FamilyEvent_",
        "gText_LoveEvent_",
        "gText_RivalEvent_",
        "gText_RivalMarriageEvent_",
        "gText_SystemEvent_",
        "gText_ShopEvent_",
        "gText_LocationInteraction_",
        "gText_LocationTransition_",
        "gText_FarmEvent_",
        "gText_TV_Shopping_",
        "gText_AchievementEvent_",
        "gText_WeddingEvent_",
        "gText_SystemMenu_",
        "gText_TutorialEvent_",
        "gText_CollectibleEvent_",
        "gText_MineEvent_",
    ];

    for line in source.lines() {
        let name = line.trim().trim_end_matches(',');
        if !name.starts_with("gText_") || structured_prefixes.iter().any(|p| name.starts_with(p)) {
            continue;
        }
        assert!(
            !sentence_starts
                .iter()
                .any(|prefix| name.starts_with(prefix)),
            "unscoped sentence-derived text symbol remains: {name}"
        );
    }
}

#[test]
fn remaining_text_bearing_helpers_use_event_level_script_names() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for forbidden in [
        "EventScript_ShowComeAgainMessage",
        "EventScript_UseChurchConfessional",
        "EventScript_ChooseSeason",
        "EventScript_ShowOuchMessage",
        "EventScript_WaitOneSecondYouveComeToo",
        "EventScript_ShowTooBadMessage",
        "EventScript_ShowSilentPauseMessage",
        "EventScript_ShowThatsImpossibleMessage",
        "EventScript_EveryonesGoneHomeNowIfYou",
        "EventScript_EveryonesGoingHomeNowIfYou",
        "EventScript_ListenEveryoneThisYearsChampionIs",
        "EventScript_CongratulationsOnWinningIveNeverSeen",
        "EventScript_AndTheWinnerIsVar1From",
        "EventScript_HiPlayerTomorrowAt10AMIs",
        "EventScript_HiPlayerTomorrowAt10AM",
    ] {
        assert!(
            !source.contains(forbidden),
            "legacy sentence-derived script name remains: {forbidden}"
        );
    }

    for (target, expected) in [
        (
            "MARY_FOMT_US",
            &[
                (
                    308,
                    "EventScript_SystemMenu_ReferenceGuide_OpenHarvestSpriteMinigamesPage",
                ),
                (317, "EventScript_SystemMenu_TelephoneDirectory"),
                (
                    381,
                    "EventScript_LocationTransition_ExitAjaWineryStorageToSouthTown",
                ),
                (
                    415,
                    "EventScript_RivalEvent_AnnAndCliff_04_OrangeHeart_ExitWineryStorageToNorthTown",
                ),
                (424, "EventScript_Debug_HarvestGoddessAndSpritesProcession"),
                (468, "EventScript_ShopEvent_Supermarket_OpenPrimary"),
                (
                    480,
                    "EventScript_ShopEvent_Supermarket_OpenDuplicateEntry01",
                ),
                (
                    481,
                    "EventScript_ShopEvent_Supermarket_OpenDuplicateEntry02",
                ),
                (
                    975,
                    "EventScript_SystemEffect_PlayerAnimationAndScreenFlash_Primary",
                ),
                (
                    989,
                    "EventScript_SystemEffect_PlayerAnimationAndScreenFlash_Duplicate",
                ),
                (
                    1067,
                    "EventScript_FestivalEvent_NewYearsDayFestivalDialogue_Zack",
                ),
                (
                    1068,
                    "EventScript_FestivalEvent_NewYearsDayFestivalDialogue_Doug",
                ),
                (
                    1082,
                    "EventScript_FestivalEvent_SpringHorseRace_MedalExchangeAfterClosing",
                ),
                (
                    1145,
                    "EventScript_FestivalEvent_DogFrisbeeTournament_PlayerVictory",
                ),
                (
                    1163,
                    "EventScript_FestivalEvent_ChickenFestival_PlayerVictory",
                ),
                (1190, "EventScript_FestivalEvent_CowFestival_PlayerVictory"),
                (1216, "EventScript_FestivalEvent_HarvestFestival_Invitation"),
                (
                    1248,
                    "EventScript_FestivalEvent_FallHorseRace_MedalExchangeAfterClosing",
                ),
            ][..],
        ),
        (
            "MARY_MFOMT_US",
            &[
                (326, "EventScript_SystemMenu_TelephoneDirectory"),
                (
                    390,
                    "EventScript_LocationTransition_ExitAjaWineryStorageToSouthTown",
                ),
                (
                    424,
                    "EventScript_RivalEvent_AnnAndCliff_04_OrangeHeart_ExitWineryStorageToNorthTown",
                ),
                (477, "EventScript_ShopEvent_Supermarket_OpenPrimary"),
                (
                    489,
                    "EventScript_ShopEvent_Supermarket_OpenDuplicateEntry01",
                ),
                (
                    490,
                    "EventScript_ShopEvent_Supermarket_OpenDuplicateEntry02",
                ),
                (
                    1043,
                    "EventScript_SystemEffect_PlayerAnimationAndScreenFlash_Primary",
                ),
                (
                    1057,
                    "EventScript_SystemEffect_PlayerAnimationAndScreenFlash_Duplicate",
                ),
                (
                    1162,
                    "EventScript_FestivalEvent_SpringHorseRace_MedalExchangeAfterClosing",
                ),
                (
                    1225,
                    "EventScript_FestivalEvent_DogFrisbeeTournament_PlayerVictory",
                ),
                (
                    1243,
                    "EventScript_FestivalEvent_ChickenFestival_PlayerVictory",
                ),
                (1270, "EventScript_FestivalEvent_CowFestival_PlayerVictory"),
                (1296, "EventScript_FestivalEvent_HarvestFestival_Invitation"),
            ][..],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for &(id, name) in expected {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }
}

#[test]
fn mineral_town_friends_profiles_share_structured_symbols_across_all_targets() {
    for (target, script_id) in [
        ("MARY_FOMT_US", 1048),
        ("MARY_FOMT_JP", 1048),
        ("MARY_MFOMT_US", 1119),
        ("MARY_MFOMT_JP", 1119),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_MineralTownFriendsCharacterProfiles"),
            "{target} profile-program script"
        );
        let count = symbols.text_count(script_id);
        assert!(count >= 170, "{target}: only {count} profile text slots");
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_MineralTownFriends_")),
            "{target}: unstructured profile text symbol remains"
        );
    }
}

#[test]
fn calendar_program_uses_date_and_event_symbols_across_all_targets() {
    for (target, script_id, minimum_texts) in [
        ("MARY_FOMT_US", 1049, 120),
        ("MARY_FOMT_JP", 1049, 120),
        ("MARY_MFOMT_US", 1120, 55),
        ("MARY_MFOMT_JP", 1120, 55),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_CalendarProgram"),
            "{target} calendar script"
        );
        let count = symbols.text_count(script_id);
        assert!(
            count >= minimum_texts,
            "{target}: only {count} calendar texts"
        );
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_Calendar_")),
            "{target}: unstructured calendar symbol remains"
        );
    }
}

#[test]
fn life_on_the_farm_programs_use_season_and_day_symbols_across_all_targets() {
    for (target, advanced_id, beginner_id) in [
        ("MARY_FOMT_US", 1043, 1044),
        ("MARY_FOMT_JP", 1043, 1044),
        ("MARY_MFOMT_US", 1114, 1115),
        ("MARY_MFOMT_JP", 1114, 1115),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (script_id, level) in [(advanced_id, "Advanced"), (beginner_id, "Beginner")] {
            let script_name = format!("EventScript_TV_LifeOnTheFarm{level}");
            let text_prefix = format!("gText_TV_LifeOnTheFarm_{level}_");
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name.as_str()),
                "{target} {level} program"
            );
            let count = symbols.text_count(script_id);
            assert!(count >= 120, "{target}: only {count} {level} texts");
            assert!(
                symbols
                    .names(script_id, count)
                    .into_iter()
                    .flatten()
                    .all(|name| name.starts_with(&text_prefix)),
                "{target}: unstructured {level} program symbol remains"
            );
        }
    }
}

#[test]
fn weather_forecasts_use_functional_symbols_across_all_targets() {
    for (target, primary_id, alternate_id) in [
        ("MARY_FOMT_US", 1057, 1058),
        ("MARY_FOMT_JP", 1057, 1058),
        ("MARY_MFOMT_US", 1128, 1129),
        ("MARY_MFOMT_JP", 1128, 1129),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (script_id, presenter, minimum_texts) in
            [(primary_id, "Primary", 14), (alternate_id, "Alternate", 15)]
        {
            let script_name = format!("EventScript_TV_WeatherForecast{presenter}");
            let text_prefix = format!("gText_TV_WeatherForecast_{presenter}_");
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name.as_str()),
                "{target} {presenter} weather forecast"
            );
            let count = symbols.text_count(script_id);
            assert!(
                count >= minimum_texts,
                "{target}: only {count} {presenter} forecast texts"
            );
            assert!(
                symbols
                    .names(script_id, count)
                    .into_iter()
                    .flatten()
                    .all(|name| name.starts_with(&text_prefix)),
                "{target}: sentence-derived {presenter} forecast symbol remains"
            );
        }
    }
}

#[test]
fn f314m_grand_prix_uses_race_phase_symbols_across_all_targets() {
    for (target, script_id, minimum_texts) in [
        ("MARY_FOMT_US", 1039, 105),
        ("MARY_FOMT_JP", 1039, 100),
        ("MARY_MFOMT_US", 1110, 110),
        ("MARY_MFOMT_JP", 1110, 104),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_F314MGrandPrix"),
            "{target} F314-M program"
        );
        let count = symbols.text_count(script_id);
        assert!(
            count >= minimum_texts,
            "{target}: only {count} F314-M texts"
        );
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_F314MGrandPrix_")),
            "{target}: unstructured F314-M text symbol remains"
        );
    }
}

#[test]
fn harvest_goddess_tv_games_use_game_state_symbols_across_all_targets() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 71),
        ("MARY_MFOMT_JP", 71),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, game, minimum_texts) in [
            (1040, "RockPaperScissors", 25),
            (1041, "NumberGuessing", 18),
            (1050, "MathQuiz", 29),
        ] {
            let script_id = base_id + shift;
            let script_name = format!("EventScript_TV_HarvestGoddess{game}");
            let text_prefix = format!("gText_TV_HarvestGoddess{game}_");
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name.as_str()),
                "{target} {game}"
            );
            let count = symbols.text_count(script_id);
            assert!(
                count >= minimum_texts,
                "{target}: only {count} {game} texts"
            );
            assert!(
                symbols
                    .names(script_id, count)
                    .into_iter()
                    .flatten()
                    .all(|name| name.starts_with(&text_prefix)),
                "{target}: sentence-derived {game} text symbol remains"
            );
        }
    }
}

#[test]
fn harvest_theater_uses_episode_symbols_across_all_targets() {
    for (target, script_id, expected_texts) in [
        ("MARY_FOMT_US", 1042, 60),
        ("MARY_FOMT_JP", 1042, 60),
        ("MARY_MFOMT_US", 1113, 64),
        ("MARY_MFOMT_JP", 1113, 64),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_HarvestTheater"),
            "{target} Harvest Theater"
        );
        let count = symbols.text_count(script_id);
        assert_eq!(
            count, expected_texts,
            "{target}: Harvest Theater text count"
        );
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_HarvestTheater_")),
            "{target}: unstructured Harvest Theater text symbol remains"
        );
    }
}

#[test]
fn aaron_changes_uses_episode_symbols_across_all_targets() {
    for (target, script_id, expected_texts) in [
        ("MARY_FOMT_US", 1045, 39),
        ("MARY_FOMT_JP", 1045, 39),
        ("MARY_MFOMT_US", 1116, 43),
        ("MARY_MFOMT_JP", 1116, 43),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_AaronChanges"),
            "{target} Aaron Changes"
        );
        let count = symbols.text_count(script_id);
        assert_eq!(count, expected_texts, "{target}: Aaron Changes text count");
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_AaronChanges_")),
            "{target}: unstructured Aaron Changes text symbol remains"
        );
    }
}

#[test]
fn mechabot_programs_use_episode_and_reminder_symbols() {
    for (target, ultror_id, expected_texts) in [
        ("MARY_FOMT_US", 1046, 48),
        ("MARY_FOMT_JP", 1046, 48),
        ("MARY_MFOMT_US", 1117, 52),
        ("MARY_MFOMT_JP", 1117, 52),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(ultror_id),
            Some("EventScript_TV_MechabotUltror"),
            "{target} Mechabot Ultror"
        );
        let count = symbols.text_count(ultror_id);
        assert_eq!(count, expected_texts, "{target}: Ultror text count");
        assert!(
            symbols
                .names(ultror_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_MechabotUltror_")),
            "{target}: unstructured Ultror symbol remains"
        );
    }

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(1047),
            Some("EventScript_TV_MechabotUltrorZeroEpisodeReminders")
        );
        assert_eq!(symbols.text_count(1047), 20);
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(1118),
            Some("EventScript_TV_MechabotGenesis")
        );
        let count = symbols.text_count(1118);
        assert_eq!(count, 24);
        assert!(symbols
            .names(1118, count)
            .into_iter()
            .flatten()
            .all(|name| name.starts_with("gText_TV_MechabotGenesis_")));
    }
}

#[test]
fn remaining_tv_serials_use_program_episode_symbols() {
    for (target, shift, header_count) in [
        ("MARY_FOMT_US", 0, 0),
        ("MARY_FOMT_JP", 0, 0),
        ("MARY_MFOMT_US", 71, 4),
        ("MARY_MFOMT_JP", 71, 4),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, program, episodes) in [
            (1054, "StarLilyBanditGirl", 25),
            (1055, "StEmeraldAcademy", 10),
            (1056, "MineResearchGroup", 13),
        ] {
            let script_id = base_id + shift;
            let script_name = format!("EventScript_TV_{program}");
            let text_prefix = format!("gText_TV_{program}_");
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name.as_str()),
                "{target} {program}"
            );
            let count = symbols.text_count(script_id);
            assert_eq!(
                count,
                episodes + header_count,
                "{target}: {program} text count"
            );
            assert!(
                symbols
                    .names(script_id, count)
                    .into_iter()
                    .flatten()
                    .all(|name| name.starts_with(&text_prefix)),
                "{target}: unstructured {program} text symbol remains"
            );
        }
    }
}

#[test]
fn early_tv_serials_use_program_episode_symbols() {
    let programs = [
        ("CardCollectorChisato", 12),
        ("MyDearPrincess", 34),
        ("DuelingChefs", 12),
        ("FairyAndMeHisStory", 38),
        ("FairyAndMeHerStory", 38),
        ("FishingHour", 16),
    ];
    for (target, base_id, header_count) in [
        ("MARY_FOMT_US", 1033, 0),
        ("MARY_FOMT_JP", 1033, 0),
        ("MARY_MFOMT_US", 1104, 4),
        ("MARY_MFOMT_JP", 1104, 4),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (offset, (program, episodes)) in programs.iter().enumerate() {
            let script_id = base_id + offset;
            let script_name = format!("EventScript_TV_{program}");
            let text_prefix = format!("gText_TV_{program}_");
            assert_eq!(
                symbols.script_name(script_id),
                Some(script_name.as_str()),
                "{target} {program}"
            );
            let count = symbols.text_count(script_id);
            assert_eq!(
                count,
                episodes + header_count,
                "{target}: {program} text count"
            );
            assert!(
                symbols
                    .names(script_id, count)
                    .into_iter()
                    .flatten()
                    .all(|name| name.starts_with(&text_prefix)),
                "{target}: unstructured {program} symbol remains"
            );
        }
    }
}

#[test]
fn four_am_hidden_broadcast_uses_schedule_symbols() {
    for (target, script_id, expected_texts) in [
        ("MARY_FOMT_US", 1032, 12),
        ("MARY_FOMT_JP", 1032, 12),
        ("MARY_MFOMT_US", 1103, 16),
        ("MARY_MFOMT_JP", 1103, 16),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_4AMHiddenBroadcast"),
            "{target} hidden broadcast"
        );
        let count = symbols.text_count(script_id);
        assert_eq!(
            count, expected_texts,
            "{target}: hidden-broadcast text count"
        );
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_4AMHiddenBroadcast_")),
            "{target}: generic hidden-broadcast symbol remains"
        );
    }
}

#[test]
fn new_year_tv_specials_use_calendar_function_symbols() {
    for (target, script_id) in [
        ("MARY_FOMT_US", 1052),
        ("MARY_FOMT_JP", 1052),
        ("MARY_MFOMT_US", 1123),
        ("MARY_MFOMT_JP", 1123),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_NewYearSpecialPrograms"),
            "{target} New Year TV special"
        );
        let count = symbols.text_count(script_id);
        assert_eq!(count, 8, "{target}: New Year TV special text count");
        assert!(
            symbols
                .names(script_id, count)
                .into_iter()
                .flatten()
                .all(|name| name.starts_with("gText_TV_NewYearSpecial_")),
            "{target}: sentence-derived New Year TV symbol remains"
        );
    }
}

#[test]
fn tv_shopping_broadcasts_use_product_symbols_across_all_targets() {
    for (target, script_id, expected_texts) in [
        ("MARY_FOMT_US", 1051, 16),
        ("MARY_FOMT_JP", 1051, 16),
        ("MARY_MFOMT_US", 1122, 31),
        ("MARY_MFOMT_JP", 1122, 31),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_TV_Shopping_ProductBroadcasts"),
            "{target} TV shopping"
        );
        let count = symbols.text_count(script_id);
        assert_eq!(count, expected_texts, "{target}: TV-shopping text count");
        let names = symbols.names(script_id, count);
        for product in ["Clock", "LargeBed", "SeasoningSet", "PowerBerry"] {
            let expected = format!("gText_TV_Shopping_Product{product}Broadcast");
            assert!(
                names.iter().flatten().any(|name| *name == expected),
                "{target}: missing {expected}"
            );
        }
    }
}

#[test]
fn television_and_kappa_events_have_event_level_symbols() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(540),
            Some("EventScript_TV_EntertainmentChannel"),
            "{target} slot 540"
        );
        assert_eq!(
            symbols.script_name(572),
            Some("EventScript_NPCEvent_Kappa_CucumberOffering"),
            "{target} slot 572"
        );
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(581),
            Some("EventScript_NPCEvent_Kappa_CucumberOfferingAndStarryNightFestival"),
            "{target} slot 581"
        );
    }
}

#[test]
fn basil_harris_and_ellen_events_follow_each_games_shifted_slots() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 9),
        ("MARY_MFOMT_JP", 9),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, expected) in [
            (724, "EventScript_NPCEvent_Basil_LetterAdvice"),
            (
                725,
                "EventScript_NPCEvent_Basil_LetterAdviceFollowupDialogue",
            ),
            (726, "EventScript_NPCEvent_Basil_PublishingAward"),
            (
                730,
                "EventScript_NPCEvent_Basil_PublishingAwardFollowupGrayDialogue",
            ),
            (740, "EventScript_NPCEvent_Harris_AjaLetterAdvice"),
            (
                741,
                "EventScript_NPCEvent_Harris_AjaLetterAdviceFollowupDialogue",
            ),
            (742, "EventScript_NPCEvent_Ellen_WhiteFlowerLegend"),
            (
                743,
                "EventScript_NPCEvent_Ellen_WhiteFlowerLegend_FollowupEllenDialogue",
            ),
            (744, "EventScript_NPCEvent_Ellen_WhiteFlowerDiscovery"),
            (745, "EventScript_NPCEvent_Ellen_GrandfathersLetter"),
            (746, "EventScript_NPCEvent_Ellen_GrandfatherLetter"),
            (
                747,
                "EventScript_NPCEvent_Ellen_GrandfathersLetterFollowupElliDialogue",
            ),
            (749, "EventScript_NPCEvent_Ellen_KnitsStocking"),
        ] {
            let id = base_id + shift;
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn aja_and_jeff_family_events_follow_each_games_shifted_slots() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 9),
        ("MARY_MFOMT_JP", 9),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, expected) in [
            (
                757,
                "EventScript_NPCEvent_Manna_DiscussesAjasDeparture_MainSceneFollowupSashaDialogue",
            ),
            (
                758,
                "EventScript_NPCEvent_Manna_DiscussesAjasDeparture_MainSceneFollowupAnnaDialogue",
            ),
            (
                759,
                "EventScript_NPCEvent_Manna_DiscussesAjasDeparture_MainSceneFollowupMannaDialogue",
            ),
            (
                760,
                "EventScript_NPCEvent_LilliaAndSasha_ReminisceAboutJeffsMarriage",
            ),
            (
                761,
                "EventScript_NPCEvent_LilliaAndSasha_ReminisceAboutJeffsMarriageFollowupLilliaDialogue",
            ),
            (
                762,
                "EventScript_NPCEvent_LilliaAndSasha_ReminisceAboutJeffsMarriageFollowupSashaDialogue",
            ),
        ] {
            let id = base_id + shift;
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn inn_beach_and_gotz_events_follow_each_games_shifted_slots() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 9),
        ("MARY_MFOMT_JP", 9),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, expected) in [
            (786, "EventScript_NPCEvent_DougAndDuke_ArgumentChoice"),
            (
                787,
                "EventScript_NPCEvent_DougAndDuke_ArgumentFollowupDougDialogue",
            ),
            (
                788,
                "EventScript_NPCEvent_DougAndDuke_ArgumentFollowupAnnDialogue",
            ),
            (
                789,
                "EventScript_NPCEvent_DougAndDuke_ArgumentFollowupThomasDialogue",
            ),
            (
                790,
                "EventScript_NPCEvent_DougAndDuke_ArgumentFollowupDukeDialogue",
            ),
            (
                791,
                "EventScript_NPCEvent_DougAndDuke_ArgumentFollowupHarrisDialogue",
            ),
            (792, "EventScript_FamilyEvent_Doug_BirthdayGiftFromAnn"),
            (
                794,
                "EventScript_NPCEvent_AnnAndCliff_SiblingComparisonArgument",
            ),
            (
                795,
                "EventScript_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe",
            ),
            (796, "EventScript_NPCEvent_Kai_ReturnsForSummer"),
            (798, "EventScript_NPCEvent_Kai_LeavesAfterSummer"),
            (799, "EventScript_NPCEvent_Gotz_LosesMotivation"),
            (800, "EventScript_NPCEvent_Gotz_RegainsMotivation"),
            (801, "EventScript_NPCEvent_GotzAndHarris_PatrolDiscussion"),
            (
                802,
                "EventScript_NPCEvent_GotzAndHarris_PatrolDiscussionFollowupHarrisDialogue",
            ),
            (
                803,
                "EventScript_NPCEvent_GotzAndHarris_PatrolDiscussionFollowupGotzDialogue",
            ),
            (804, "EventScript_NPCEvent_Zack_VisitsSickLillia"),
            (
                805,
                "EventScript_NPCEvent_Zack_VisitsSickLilliaFollowupLilliaDialogue",
            ),
        ] {
            let id = base_id + shift;
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn fishing_merchant_and_jewel_events_follow_each_games_shifted_slots() {
    for (target, shift, guest) in [
        (
            "MARY_FOMT_US",
            0,
            "EventScript_NPCEvent_LouOrRuby_Introduction",
        ),
        (
            "MARY_FOMT_JP",
            0,
            "EventScript_NPCEvent_LouOrRuby_Introduction",
        ),
        (
            "MARY_MFOMT_US",
            9,
            "EventScript_NPCEvent_LouOrRuby_Introduction",
        ),
        (
            "MARY_MFOMT_JP",
            9,
            "EventScript_NPCEvent_LouOrRuby_Introduction",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, expected) in [
            (806, "EventScript_NPCEvent_Zack_GivesFishingRod"),
            (
                807,
                "EventScript_NPCEvent_Zack_GivesFishingRod_FollowupZackDialogue",
            ),
            (808, "EventScript_NPCEvent_Zack_GivesFishingRod_WonDialogue"),
            (809, "EventScript_NPCEvent_Zack_FishingRodFollowup"),
            (810, "EventScript_NPCEvent_Won_Introduction"),
            (812, "EventScript_NPCEvent_Won_VasePurchase"),
            (813, "EventScript_NPCEvent_Van_Introduction"),
            (815, "EventScript_FestivalEvent_HarvestSpriteTeaParty"),
            (816, "EventScript_SystemEvent_HarvestGoddessJewelsExchange"),
            (818, "EventScript_SystemEvent_KappaJewelsExchange"),
            (819, "EventScript_SystemEvent_GoldenLumberTownAngerDialogue"),
            (820, "EventScript_SystemEvent_JewelsOfTruthExchange"),
        ] {
            let id = base_id + shift;
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
        assert_eq!(symbols.script_name(814 + shift), Some(guest), "{target}");
    }
}

#[test]
fn rival_wedding_event_families_match_fomt_and_mfomt_slots() {
    let expected = [
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingCeremony",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingCeremonyWithPopuriIntroduction",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingAttendanceChoice",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingFollowupRickDialogue",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingFollowupLilliaDialogue",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingFollowupJeffDialogue",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingFollowupKarenDialogue",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingFollowupSashaDialogue",
        "EventScript_RivalMarriageEvent_RickAndKaren_05_WeddingAttendanceAfterWorkChoice",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingCeremony",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingAttendanceChoice",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingFollowupRickDialogue",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingFollowupPopuriDialogue",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingFollowupLilliaDialogue",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingFollowupKarenDialogue",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingAttendanceAfterWorkChoice",
        "EventScript_RivalMarriageEvent_PopuriAndKai_05_WeddingFollowupKaiDialogue",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingCeremony",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingAttendanceChoice",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingFollowupCliffDialogue",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingFollowupDougDialogue",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingFollowupAnnDialogue",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingFollowupDukeDialogue",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingFollowupMannaDialogue",
        "EventScript_RivalMarriageEvent_AnnAndCliff_05_WeddingAttendanceAfterWorkChoice",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingCeremony",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingAttendanceChoice",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingFollowupBasilDialogue",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingFollowupMaryDialogue",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingFollowupAnnaDialogue",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingAttendanceAfterWorkChoice",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingFollowupGrayDialogue",
        "EventScript_RivalMarriageEvent_MaryAndGray_05_WeddingFollowupSaibaraDialogue",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingCeremony",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingAttendanceChoice",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingFollowupDoctorDialogue",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingFollowupEllenDialogue",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingFollowupElliDialogue",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingFollowupStuDialogue",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingFollowupHarrisDialogue",
        "EventScript_RivalMarriageEvent_ElliAndDoctor_05_WeddingAttendanceAfterWorkChoice",
    ];
    let fomt_ids = [
        877, 878, 879, 880, 881, 882, 883, 884, 885, 898, 899, 900, 901, 902, 903, 904, 905, 918,
        919, 920, 921, 922, 923, 924, 925, 936, 937, 938, 939, 940, 941, 942, 943, 956, 957, 958,
        959, 960, 961, 962, 963,
    ];
    let mfomt_ids = [
        945, 946, 947, 948, 949, 950, 951, 952, 953, 966, 967, 968, 969, 970, 971, 972, 973, 986,
        987, 988, 989, 990, 991, 992, 993, 1004, 1005, 1006, 1007, 1008, 1009, 1010, 1011, 1024,
        1025, 1026, 1027, 1028, 1029, 1030, 1031,
    ];
    for (target, ids) in [
        ("MARY_FOMT_US", &fomt_ids[..]),
        ("MARY_FOMT_JP", &fomt_ids[..]),
        ("MARY_MFOMT_US", &mfomt_ids[..]),
        ("MARY_MFOMT_JP", &mfomt_ids[..]),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (&id, expected) in ids.iter().zip(expected) {
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn supermarket_shelf_redraw_callables_keep_target_ids() {
    for (target, rucksack_id, feather_id) in [
        ("MARY_FOMT_US", 0x0C9, 0x0CA),
        ("MARY_FOMT_JP", 0x0C9, 0x0CA),
        ("MARY_MFOMT_US", 0x0CC, 0x0CD),
        ("MARY_MFOMT_JP", 0x0CC, 0x0CD),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(
            map["RedrawRucksackShelfAfterPurchase"].0 .0, rucksack_id,
            "{target}"
        );
        assert_eq!(
            map["RedrawBlueFeatherShelfAfterPurchase"].0 .0, feather_id,
            "{target}"
        );
        let table = parse_script_table("mary_script_table { TestShop, };\n", &options).unwrap();
        let scripts = parse_named_scripts(
            "void TestShop(void) { RedrawRucksackShelfAfterPurchase(); RedrawBlueFeatherShelfAfterPurchase(); }\n",
            &options,
            &callables.scope,
            &table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&scripts.scripts[0].2, &callables.scope, "TestShop").unwrap();
        let source = format_named_script("TestShop", &raised).unwrap();
        assert!(
            source.contains("RedrawRucksackShelfAfterPurchase()"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RedrawBlueFeatherShelfAfterPurchase()"),
            "{target}: {source}"
        );
    }
}

#[test]
fn fomt_harvest_goddess_family_events_have_structured_names() {
    let expected = [
        (847, "EventScript_LoveEvent_HarvestGoddess_ProposalResponse"),
        (
            848,
            "EventScript_FamilyEvent_HarvestGoddess_WeddingAndNicknameChoice",
        ),
        (
            849,
            "EventScript_FamilyEvent_HarvestGoddess_AnniversaryAndBirthdayChoice",
        ),
        (
            850,
            "EventScript_FamilyEvent_HarvestGoddess_ChildGooDialogue",
        ),
        (
            851,
            "EventScript_FamilyEvent_HarvestGoddess_AnniversaryGiftDelivery",
        ),
        (
            852,
            "EventScript_FamilyEvent_HarvestGoddess_FiftiethAnniversaryMountainCottageGift",
        ),
        (
            854,
            "EventScript_FamilyEvent_HarvestGoddess_ChildBirthdayChildDialogue",
        ),
        (
            855,
            "EventScript_FamilyEvent_HarvestGoddess_ChildBirthdayDateChoice",
        ),
        (
            857,
            "EventScript_FamilyEvent_HarvestGoddess_EveningChildDialogue",
        ),
        (
            858,
            "EventScript_FamilyEvent_HarvestGoddess_EveningDateChoice",
        ),
        (
            860,
            "EventScript_FamilyEvent_HarvestGoddess_PlayerBirthdayChildDialogue",
        ),
        (
            861,
            "EventScript_FamilyEvent_HarvestGoddess_PlayerBirthdayDateChoice",
        ),
        (
            862,
            "EventScript_FamilyEvent_HarvestGoddess_PregnancyAnnouncement",
        ),
        (863, "EventScript_FamilyEvent_HarvestGoddess_Childbirth"),
        (
            864,
            "EventScript_FamilyEvent_HarvestGoddess_ChildFirstSteps",
        ),
        (
            865,
            "EventScript_FamilyEvent_HarvestGoddess_ChildGooDialogueAfterFirstSteps",
        ),
        (866, "EventScript_FamilyEvent_HarvestGoddess_ChildInjury"),
        (867, "EventScript_SystemEvent_FarmInheritanceIntroduction"),
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, name) in expected {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }
}

#[test]
fn mine_and_tool_system_messages_follow_each_games_slots() {
    for (target, mine_id, purified_id, unavailable_id) in [
        ("MARY_FOMT_US", 979, 990, 991),
        ("MARY_FOMT_JP", 979, 990, 991),
        ("MARY_MFOMT_US", 1047, 1058, 1059),
        ("MARY_MFOMT_JP", 1047, 1058, 1059),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            symbols.script_name(mine_id),
            Some("EventScript_SystemEvent_MineReturnToSurfaceChoice"),
            "{target} slot {mine_id}"
        );
        assert_eq!(
            symbols.script_name(purified_id),
            Some("EventScript_SystemEvent_CursedToolPurificationMessage"),
            "{target} slot {purified_id}"
        );
        assert_eq!(
            symbols.script_name(unavailable_id),
            Some("EventScript_SystemEvent_ToolUseUnavailableMessage"),
            "{target} slot {unavailable_id}"
        );
    }
}

#[test]
fn cliff_and_ann_events_follow_each_games_shifted_slots() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 9),
        ("MARY_MFOMT_JP", 9),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, expected) in [
            (781, "EventScript_NPCEvent_Cliff_CollapsesInSnow"),
            (
                782,
                "EventScript_NPCEvent_Cliff_CollapsesInSnowFollowupCliffDialogue",
            ),
            (783, "EventScript_NPCEvent_Cliff_LeavesMineralTown"),
            (784, "EventScript_FamilyEvent_Ann_MothersDeathAnniversary"),
            (
                785,
                "EventScript_FamilyEvent_Ann_MothersDeathAnniversary_FollowupDougDialogue",
            ),
        ] {
            let id = base_id + shift;
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn town_family_and_request_events_follow_each_games_shifted_slots() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 9),
        ("MARY_MFOMT_JP", 9),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (base_id, expected) in [
            (
                710,
                "EventScript_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall",
            ),
            (
                718,
                "EventScript_NPCEvent_DukeAndManna_MissingJuiceArgument",
            ),
            (733, "EventScript_NPCEvent_MaryAndGray_BookAndHealth"),
            (739, "EventScript_NPCEvent_Thomas_RequestItemDelivery"),
        ] {
            let id = base_id + shift;
            assert_eq!(
                symbols.script_name(id),
                Some(expected),
                "{target} slot {id}"
            );
        }
    }
}

#[test]
fn mfomt_bachelor_love_event_followups_have_character_names() {
    let expected = [
        (
            17,
            "EventScript_LoveEvent_Rick_01_BlackHeart_FollowupRickDialogue",
        ),
        (
            18,
            "EventScript_LoveEvent_Rick_01_BlackHeart_FollowupPopuriDialogue",
        ),
        (
            19,
            "EventScript_LoveEvent_Rick_01_BlackHeart_FollowupLilliaDialogue",
        ),
        (
            21,
            "EventScript_LoveEvent_Rick_02_PurpleHeart_FollowupRickDialogue",
        ),
        (
            22,
            "EventScript_LoveEvent_Rick_02_PurpleHeart_FollowupPopuriDialogue",
        ),
        (
            23,
            "EventScript_LoveEvent_Rick_02_PurpleHeart_FollowupLilliaDialogue",
        ),
        (
            25,
            "EventScript_LoveEvent_Rick_03_BlueHeart_FollowupRickDialogue",
        ),
        (
            26,
            "EventScript_LoveEvent_Rick_03_BlueHeart_FollowupPopuriDialogue",
        ),
        (
            27,
            "EventScript_LoveEvent_Rick_03_BlueHeart_FollowupLilliaDialogue",
        ),
        (
            34,
            "EventScript_LoveEvent_Kai_02_PurpleHeart_FollowupKaiDialogue",
        ),
        (
            37,
            "EventScript_LoveEvent_Kai_04_YellowHeart_FollowupKaiDialogue",
        ),
        (
            45,
            "EventScript_LoveEvent_Cliff_04_YellowHeart_FollowupDougDialogue",
        ),
        (
            53,
            "EventScript_LoveEvent_Gray_02_PurpleHeart_FollowupGrayDialogue",
        ),
        (
            56,
            "EventScript_LoveEvent_Gray_04_YellowHeart_FollowupGrayDialogue",
        ),
        (
            61,
            "EventScript_LoveEvent_Doctor_01_BlackHeart_FollowupDoctorDialogue",
        ),
        (
            62,
            "EventScript_LoveEvent_Doctor_01_BlackHeart_FollowupElliDialogue",
        ),
        (
            70,
            "EventScript_LoveEvent_Doctor_04_YellowHeart_FollowupDoctorDialogue",
        ),
        (
            71,
            "EventScript_LoveEvent_Doctor_04_YellowHeart_FollowupElliDialogue",
        ),
    ];
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(
            &fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap(),
            &options,
        )
        .unwrap();
        for (id, name) in expected {
            assert_eq!(symbols.script_name(id), Some(name), "{target} slot {id}");
        }
    }
}

#[test]
fn karen_profile_dialogue_symbols_encode_location_state_instead_of_numeric_suffixes() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id, expected) in [
        (
            "MARY_FOMT_US",
            1008,
            vec![
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_StarryNightInvitationChoiceDecline",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_PoultryFarmHouseEveningGreeting",
            ],
        ),
        (
            "MARY_FOMT_JP",
            1008,
            vec![
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_StarryNightInvitationChoiceDecline",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_SupermarketBackRoomMediumFriendshipGreeting",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_NorthSideTownMorningGreetingBeforeMarriage",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_PoultryFarmHouseEveningGreeting",
            ],
        ),
        (
            "MARY_MFOMT_US",
            1078,
            vec![
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_PoultryFarmHouseEveningGreeting",
            ],
        ),
        (
            "MARY_MFOMT_JP",
            1078,
            vec![
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_SupermarketBackRoomMediumFriendshipGreeting",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_NorthSideTownAsksAboutPlayersHusband",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_NorthSideTownMorningGreetingBeforeMarriage",
                "gText_NPCEvent_Karen_DialogueAndGiftResponses_PoultryFarmHouseEveningGreeting",
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for name in expected {
            assert!(
                names.iter().any(|actual| actual.as_deref() == Some(name)),
                "{target} slot {script_id}: missing {name}"
            );
        }
        assert!(
            names
                .iter()
                .flatten()
                .all(|name| !name.ends_with("_02")),
            "{target} slot {script_id}: mechanical numeric suffix remains: {names:?}"
        );
    }
}

#[test]
fn mfomt_doctor_profile_dialogue_symbols_encode_choice_location_and_heart_state() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1067, symbols.text_count(1067));
        for expected in [
            "gText_NPCEvent_Doctor_DialogueAndGiftResponses_StarryNightInvitationChoiceAccept",
            "gText_NPCEvent_Doctor_DialogueAndGiftResponses_StarryNightInvitationChoiceDecline",
            "gText_NPCEvent_Doctor_DialogueAndGiftResponses_LibraryYellowHeartFirstGreeting",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 1067: missing {expected}"
            );
        }
        for regional in [
            "gText_NPCEvent_Doctor_DialogueAndGiftResponses_LibraryBelowGreenHeartFirstGreeting",
            "gText_NPCEvent_Doctor_DialogueAndGiftResponses_ChurchGreenHeartRepeatDialogue",
        ] {
            assert_eq!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(regional)),
                target == "MARY_MFOMT_JP",
                "{target} slot 1067: incorrect JP-only symbol visibility for {regional}"
            );
        }
        assert!(
            names.iter().flatten().all(|name| !name.ends_with("_02")),
            "{target} slot 1067: mechanical numeric suffix remains: {names:?}"
        );
    }
}

#[test]
fn mfomt_moon_viewing_symbols_identify_the_owning_bachelor() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (script_id, expected) in [
            (
                1323,
                "gText_FestivalEvent_MoonViewing_WithPartnerAndDumplingGift_DoctorReceivesMoonDumplings",
            ),
            (
                1324,
                "gText_FestivalEvent_MoonViewing_WithSelectedPartner_GrayInvitesPlayer",
            ),
        ] {
            let names = symbols.names(script_id, symbols.text_count(script_id));
            assert!(
                names.iter().any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn fomt_zack_profile_symbols_encode_gift_and_friendship_context() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(997, symbols.text_count(997));
        assert!(
            names.iter().any(|actual| {
                actual.as_deref()
                    == Some(
                        "gText_NPCEvent_Zack_DialogueAndGiftResponses_PlansToRegiftBeautyProduct",
                    )
            }),
            "{target} slot 997: missing beauty-product gift response"
        );
        for regional in [
            "gText_NPCEvent_Zack_DialogueAndGiftResponses_FarmLowFriendshipShipmentGreeting",
            "gText_NPCEvent_Zack_DialogueAndGiftResponses_FarmHighFriendshipShipmentGreeting",
        ] {
            assert_eq!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(regional)),
                target == "MARY_FOMT_JP",
                "{target} slot 997: incorrect JP-only symbol visibility for {regional}"
            );
        }
    }
}

#[test]
fn mary_profile_symbols_distinguish_romance_and_friendship_domains() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id, specific) in [
        (
            "MARY_FOMT_US",
            1014,
            "gText_NPCEvent_Mary_DialogueAndGiftResponses_LibraryYellowHeartFirstGreeting",
        ),
        (
            "MARY_FOMT_JP",
            1014,
            "gText_NPCEvent_Mary_DialogueAndGiftResponses_LibraryYellowHeartFirstGreeting",
        ),
        (
            "MARY_MFOMT_US",
            1084,
            "gText_NPCEvent_Mary_DialogueAndGiftResponses_LibraryFriendship190To219FirstGreeting",
        ),
        (
            "MARY_MFOMT_JP",
            1084,
            "gText_NPCEvent_Mary_DialogueAndGiftResponses_LibraryFriendship190To219FirstGreeting",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            specific,
            "gText_NPCEvent_Mary_DialogueAndGiftResponses_SouthSideTownClosedDayMorningGreeting",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
        assert_eq!(
            names.iter().any(|actual| {
                actual.as_deref()
                    == Some(
                        "gText_NPCEvent_Mary_DialogueAndGiftResponses_LibraryGreetingAfterChildbirth",
                    )
            }),
            target == "MARY_FOMT_US",
            "{target} slot {script_id}: incorrect FoMT-US-only spouse-dialogue visibility"
        );
    }
}

#[test]
fn mfomt_television_dispatcher_symbols_encode_static_and_episode_browser_roles() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(365, symbols.text_count(365));
        assert!(
            names.iter().any(|actual| {
                actual.as_deref()
                    == Some("gText_TV_MainMenuAndProgramDispatcher_LateNightRareSignalInterference")
            }),
            "{target} slot 365: missing late-night rare-signal interference"
        );
        let common_interference = if target == "MARY_MFOMT_US" {
            [
                Some(
                    "gText_TV_MainMenuAndProgramDispatcher_Shared_LateNightCommonSignalInterferenceAAndB",
                ),
                None,
            ]
        } else {
            [
                Some("gText_TV_MainMenuAndProgramDispatcher_LateNightCommonSignalInterferenceA"),
                Some("gText_TV_MainMenuAndProgramDispatcher_LateNightCommonSignalInterferenceB"),
            ]
        };
        for expected in common_interference.into_iter().flatten() {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 365: missing {expected}"
            );
        }
        for us_choice in [
            "gText_TV_MainMenuAndProgramDispatcher_EpisodeBrowserChoiceWatchTV",
            "gText_TV_MainMenuAndProgramDispatcher_EpisodeBrowserChoiceTurnOffTV",
        ] {
            assert_eq!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(us_choice)),
                target == "MARY_MFOMT_US",
                "{target} slot 365: incorrect US-only episode-browser choice visibility"
            );
        }
    }
}

#[test]
fn cow_festival_invitation_symbols_encode_eligibility_branches() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1164),
        ("MARY_FOMT_JP", 1164),
        ("MARY_MFOMT_US", 1244),
        ("MARY_MFOMT_JP", 1244),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_FestivalEvent_CowFestival_Invitation_NoCows",
            "gText_FestivalEvent_CowFestival_Invitation_OnlyCalves",
            "gText_FestivalEvent_CowFestival_Invitation_EligibleCowAvailable",
            "gText_FestivalEvent_CowFestival_Invitation_NoEligibleAdultCow",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn fall_horse_race_invitation_symbols_encode_horse_growth_branches() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1245),
        ("MARY_FOMT_JP", 1245),
        ("MARY_MFOMT_US", 1325),
        ("MARY_MFOMT_JP", 1325),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_FestivalEvent_FallHorseRace_Invitation_NoHorse",
            "gText_FestivalEvent_FallHorseRace_Invitation_YoungHorse",
            "gText_FestivalEvent_FallHorseRace_Invitation_EligibleAdultHorse",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn spring_horse_race_invitation_preserves_fomt_us_physical_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1079),
        ("MARY_FOMT_JP", 1079),
        ("MARY_MFOMT_US", 1159),
        ("MARY_MFOMT_JP", 1159),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        assert!(
            has("gText_FestivalEvent_HorseRace_EntryInvitation_ThomasAsksPlayerToEnterAdultHorse"),
            "{target} slot {script_id}: missing eligible-horse invitation"
        );
        let shared = "gText_FestivalEvent_HorseRace_EntryInvitation_SharedNoHorseOrYoungHorse";
        assert_eq!(
            has(shared),
            target == "MARY_FOMT_US",
            "{target} slot {script_id}: incorrect FoMT-US shared-text visibility"
        );
        for split in [
            "gText_FestivalEvent_HorseRace_EntryInvitation_NoHorse",
            "gText_FestivalEvent_HorseRace_EntryInvitation_YoungHorse",
        ] {
            assert_eq!(
                has(split),
                target != "MARY_FOMT_US",
                "{target} slot {script_id}: incorrect split branch visibility for {split}"
            );
        }
    }
}

#[test]
fn sheep_festival_invitation_symbols_encode_eligibility_branches() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1265),
        ("MARY_FOMT_JP", 1265),
        ("MARY_MFOMT_US", 1345),
        ("MARY_MFOMT_JP", 1345),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_FestivalEvent_SheepFestival_Invitation_NoSheep",
            "gText_FestivalEvent_SheepFestival_Invitation_OnlyLambs",
            "gText_FestivalEvent_SheepFestival_Invitation_EligibleSheepAvailable",
            "gText_FestivalEvent_SheepFestival_Invitation_NoEligibleAdultSheep",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn frisbee_invitation_symbols_align_games_and_preserve_mfomt_us_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1130),
        ("MARY_FOMT_JP", 1130),
        ("MARY_MFOMT_US", 1210),
        ("MARY_MFOMT_JP", 1210),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_FestivalEvent_FrisbeeTournament_DogEntryInvitation_Zack"),
            "{target} slot {script_id}"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        assert!(
            has("gText_FestivalEvent_FrisbeeTournament_DogEntryInvitation_Zack_PuppyIneligibleExplanation"),
            "{target} slot {script_id}: missing puppy eligibility explanation"
        );
        let shared = "gText_FestivalEvent_FrisbeeTournament_DogEntryInvitation_Zack_SharedPuppyAnnouncementOrAdultDogEntryInvitation";
        assert_eq!(
            has(shared),
            target == "MARY_MFOMT_US",
            "{target} slot {script_id}: incorrect MFoMT-US shared-text visibility"
        );
        for split in [
            "gText_FestivalEvent_FrisbeeTournament_DogEntryInvitation_Zack_PuppyFestivalAnnouncement",
            "gText_FestivalEvent_FrisbeeTournament_DogEntryInvitation_Zack_AdultDogEntryInvitation",
        ] {
            assert_eq!(
                has(split),
                target != "MARY_MFOMT_US",
                "{target} slot {script_id}: incorrect split branch visibility for {split}"
            );
        }
    }
}

#[test]
fn fomt_starry_night_spouse_symbols_preserve_us_child_state_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1306, symbols.text_count(1306));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for spouse in ["Karen", "Popuri", "Ann"] {
            for state in ["WithoutChild", "WithChild"] {
                let expected = format!(
                    "gText_FestivalEvent_StarryNight_WithSpouseAndChild_{spouse}Dinner{state}"
                );
                assert!(has(&expected), "{target} slot 1306: missing {expected}");
            }
        }
        for spouse in ["Mary", "Elli", "HarvestGoddess"] {
            let shared = format!(
                "gText_FestivalEvent_StarryNight_WithSpouseAndChild_Shared{spouse}DinnerWithOrWithoutChild"
            );
            assert_eq!(
                has(&shared),
                target == "MARY_FOMT_US",
                "{target} slot 1306: incorrect US shared-text visibility for {shared}"
            );
            for state in ["WithoutChild", "WithChild"] {
                let split = format!(
                    "gText_FestivalEvent_StarryNight_WithSpouseAndChild_{spouse}Dinner{state}"
                );
                assert_eq!(
                    has(&split),
                    target == "MARY_FOMT_JP",
                    "{target} slot 1306: incorrect JP split-text visibility for {split}"
                );
            }
        }
    }
}

#[test]
fn fomt_moon_viewing_partner_symbols_follow_character_switch_cases() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1243, symbols.text_count(1243));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for spouse in ["Karen", "Popuri", "Ann", "Mary", "Elli"] {
            for role in ["OpeningRemark", "MoonReflection", "ClosingReflection"] {
                let expected = format!(
                    "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_{spouse}{role}"
                );
                assert!(has(&expected), "{target} slot 1243: missing {expected}");
            }
            let dumplings = format!(
                "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_{spouse}ReceivesMoonDumplings"
            );
            assert!(has(&dumplings), "{target} slot 1243: missing {dumplings}");
        }
        for us_only in [
            "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_KarenAdditionalReflection",
            "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_PopuriAdditionalReflection",
            "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_MaryAdditionalReflection",
            "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_ElliReceivesMoonDumplingsAlternative",
            "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_KarenQuestionAboutMoon",
            "gText_FestivalEvent_MoonViewing_PartnerDialogueAndClosing_KarenLateNightReflection",
        ] {
            assert_eq!(
                has(us_only),
                target == "MARY_FOMT_US",
                "{target} slot 1243: incorrect US-only visibility for {us_only}"
            );
        }
    }
}

#[test]
fn mfomt_won_apple_shuffle_symbols_preserve_japanese_right_position_forms() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(932, symbols.text_count(932));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for apple in ["Super", "Hyper", "Angel"] {
            let shared = format!(
                "gText_NPCEvent_Won_AppleShuffleIntroductionChoice_Shared{apple}AppleNameAnyPosition"
            );
            assert_eq!(
                has(&shared),
                target == "MARY_MFOMT_US",
                "{target} slot 932: incorrect US shared-name visibility for {shared}"
            );
            for suffix in ["LeftOrCenter", "Right"] {
                let split = format!(
                    "gText_NPCEvent_Won_AppleShuffleIntroductionChoice_{apple}AppleName{suffix}"
                );
                assert_eq!(
                    has(&split),
                    target == "MARY_MFOMT_JP",
                    "{target} slot 932: incorrect JP position-form visibility for {split}"
                );
            }
        }
    }
}

#[test]
fn church_confessional_symbols_follow_outcomes_instead_of_localized_wording() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 139),
        ("MARY_FOMT_JP", 139),
        ("MARY_MFOMT_US", 147),
        ("MARY_MFOMT_JP", 147),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for expected in [
            "gText_LocationInteraction_ChurchConfessional_CursedToolRemovalPrayer",
            "gText_LocationInteraction_ChurchConfessional_CursedHoePurified",
            "gText_LocationInteraction_ChurchConfessional_CursedWateringCanPurified",
        ] {
            assert!(
                has(expected),
                "{target} slot {script_id}: missing {expected}"
            );
        }
        for mfomt_only in [
            "gText_LocationInteraction_ChurchConfessional_InsufficientMoneyForCursedToolRemoval",
            "gText_LocationInteraction_ChurchConfessional_KappaMarriageConfessionForgiven",
        ] {
            assert_eq!(
                has(mfomt_only),
                target.starts_with("MARY_MFOMT_"),
                "{target} slot {script_id}: incorrect MFoMT-only visibility for {mfomt_only}"
            );
        }
    }
}

#[test]
fn fomt_winter_thanksgiving_preserves_us_shared_rucksack_search_text() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1299, symbols.text_count(1299));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        let shared = "gText_FestivalEvent_WinterThanksgiving_RucksackFull_Shared_AnnPopuriAndElliSearchSleepingPlayersRucksack";
        assert_eq!(
            has(shared),
            target == "MARY_FOMT_US",
            "{target} slot 1299: incorrect US shared-text visibility"
        );
        for spouse in ["Ann", "Popuri", "Elli"] {
            let split = format!(
                "gText_FestivalEvent_WinterThanksgiving_RucksackFull_{spouse}SearchesSleepingPlayersRucksack"
            );
            assert_eq!(
                has(&split),
                target == "MARY_FOMT_JP",
                "{target} slot 1299: incorrect JP spouse-text visibility for {split}"
            );
        }
    }
}

#[test]
fn mfomt_starry_night_mail_delivery_symbols_identify_invitation_counts() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(1381),
            Some("EventScript_FestivalEvent_StarryNight_InvitationMailDelivery"),
            "{target} slot 1381"
        );
        let names = symbols.names(1381, symbols.text_count(1381));
        for expected in [
            "gText_FestivalEvent_StarryNight_InvitationMailDelivery_OneAvailableInvitationCount",
            "gText_FestivalEvent_StarryNight_InvitationMailDelivery_TwoAvailableInvitationsCount",
            "gText_FestivalEvent_StarryNight_InvitationMailDelivery_ThreeAvailableInvitationsCount",
            "gText_FestivalEvent_StarryNight_InvitationMailDelivery_FourAvailableInvitationsCount",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 1381: missing {expected}"
            );
        }
    }
}

#[test]
fn duke_special_article_gift_texts_follow_the_article_categories() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1020),
        ("MARY_FOMT_JP", 1020),
        ("MARY_MFOMT_US", 1090),
        ("MARY_MFOMT_JP", 1090),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_NPCEvent_Duke_DialogueAndGiftResponses_JewelryGiftConsidersAjaOrManna",
            "gText_NPCEvent_Duke_DialogueAndGiftResponses_BeautyProductGiftConsidersAjaNotManna",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_festival_menu_year_end_announcements_follow_calendar_windows() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(412, symbols.text_count(412));
        for expected in [
            "gText_SystemEvent_FestivalMenu_YearEndEventsFallAnnouncement",
            "gText_SystemEvent_FestivalMenu_YearEndEventsEarlyWinterAnnouncement",
            "gText_SystemEvent_FestivalMenu_YearEndAndNewYearEventsLateWinterAnnouncement",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 412: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_ann_inn_greetings_preserve_us_friendship_range_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1081, symbols.text_count(1081));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        assert!(
            has("gText_NPCEvent_Ann_DialogueAndGiftResponses_InnFriendship220To249DistractedGreeting"),
            "{target} slot 1081: missing high-friendship greeting"
        );
        assert_eq!(
            has("gText_NPCEvent_Ann_DialogueAndGiftResponses_SharedInnFriendship0To129Greeting"),
            target == "MARY_MFOMT_US",
            "{target} slot 1081: incorrect US shared greeting visibility"
        );
        for split in [
            "gText_NPCEvent_Ann_DialogueAndGiftResponses_InnFriendship0To99Greeting",
            "gText_NPCEvent_Ann_DialogueAndGiftResponses_InnFriendship100To129Greeting",
        ] {
            assert_eq!(
                has(split),
                target == "MARY_MFOMT_JP",
                "{target} slot 1081: incorrect JP split greeting visibility for {split}"
            );
        }
    }
}

#[test]
fn mfomt_karen_duke_drinking_contest_preserves_us_choice_branch_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(772, symbols.text_count(772));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for shared in [
            "gText_NPCEvent_KarenAndDuke_DrinkingContest_SharedKarenContestStartAfterAcceptOrRefuse",
            "gText_NPCEvent_KarenAndDuke_DrinkingContest_SharedDukeContestStartResponseAfterAcceptOrRefuse",
        ] {
            assert_eq!(
                has(shared),
                target == "MARY_MFOMT_US",
                "{target} slot 772: incorrect US shared branch visibility for {shared}"
            );
        }
        for branch in ["Refusal", "Acceptance"] {
            for speaker in ["KarenContestStart", "DukeContestStartResponse"] {
                let split =
                    format!("gText_NPCEvent_KarenAndDuke_DrinkingContest_{speaker}After{branch}");
                assert_eq!(
                    has(&split),
                    target == "MARY_MFOMT_JP",
                    "{target} slot 772: incorrect JP branch visibility for {split}"
                );
            }
        }
    }
}

#[test]
fn basil_weather_analogy_preserves_us_shared_text_and_jp_weather_variants() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1013),
        ("MARY_FOMT_JP", 1013),
        ("MARY_MFOMT_US", 1083),
        ("MARY_MFOMT_JP", 1083),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        assert_eq!(
            has("gText_NPCEvent_Basil_DialogueAndGiftResponses_SharedSunnyOrRainyWeatherHumansNeedSunlightAnalogy"),
            target.ends_with("_US"),
            "{target} slot {script_id}: incorrect US shared-text visibility"
        );
        for split in [
            "gText_NPCEvent_Basil_DialogueAndGiftResponses_SunnyWeatherHumansNeedSunlightAnalogy",
            "gText_NPCEvent_Basil_DialogueAndGiftResponses_RainyWeatherHumansAndPlantsNeedMoreThanSunlight",
        ] {
            assert_eq!(
                has(split),
                target.ends_with("_JP"),
                "{target} slot {script_id}: incorrect JP weather variant visibility for {split}"
            );
        }
    }
}

#[test]
fn jeff_special_article_gift_texts_follow_the_article_categories() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1007),
        ("MARY_FOMT_JP", 1007),
        ("MARY_MFOMT_US", 1077),
        ("MARY_MFOMT_JP", 1077),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_NPCEvent_Jeff_DialogueAndGiftResponses_JewelryGiftThanks",
            "gText_NPCEvent_Jeff_DialogueAndGiftResponses_BeautyProductGiftForSasha",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn thomas_aepfe_apple_rejections_identify_the_wrong_item_category() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 739),
        ("MARY_FOMT_JP", 739),
        ("MARY_MFOMT_US", 748),
        ("MARY_MFOMT_JP", 748),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_NPCEvent_Thomas_RequestItemDelivery_RejectsRegularAppleForAEPFEApple",
            "gText_NPCEvent_Thomas_RequestItemDelivery_RejectsSUGDWOrHMSGBAppleForAEPFEApple",
            "gText_NPCEvent_Thomas_RequestItemDelivery_RejectsAppleDishForAEPFEApple",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn cursed_tool_purification_messages_follow_the_blessed_tool_switch() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 990),
        ("MARY_FOMT_JP", 990),
        ("MARY_MFOMT_US", 1058),
        ("MARY_MFOMT_JP", 1058),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for expected in [
            "gText_SystemEvent_CursedToolPurificationMessage_CursedAxePurified",
            "gText_SystemEvent_CursedToolPurificationMessage_CursedFishingRodPurified",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {script_id}: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_won_shop_symbols_preserve_us_decline_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(476, symbols.text_count(476));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for expected in [
            "gText_ShopEvent_Won_LotteryAndShopChoice_ChoicePlayAppleShuffle",
            "gText_ShopEvent_Won_LotteryAndShopChoice_ChoiceDeclineAppleShuffle",
        ] {
            assert!(has(expected), "{target} slot 476: missing {expected}");
        }
        assert_eq!(
            has("gText_ShopEvent_Won_LotteryAndShopChoice_SharedKeepsRaffleTicketOrDeclinesAppleChallenge"),
            target == "MARY_MFOMT_US",
            "{target} slot 476: incorrect US shared decline visibility"
        );
        for jp_only in [
            "gText_ShopEvent_Won_LotteryAndShopChoice_KeepsRaffleTicketForLater",
            "gText_ShopEvent_Won_LotteryAndShopChoice_DeclinesAppleChallengeForNow",
        ] {
            assert_eq!(
                has(jp_only),
                target == "MARY_MFOMT_JP",
                "{target} slot 476: incorrect JP split decline visibility for {jp_only}"
            );
        }
    }
}

#[test]
fn town_square_exit_guards_use_their_actual_role_and_preserve_mfomt_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, base_id) in [
        ("MARY_FOMT_US", 409),
        ("MARY_FOMT_JP", 409),
        ("MARY_MFOMT_US", 418),
        ("MARY_MFOMT_JP", 418),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (offset, entrance) in [(0, "Beach"), (1, "NorthTown"), (2, "SouthTown")] {
            let script_id = base_id + offset;
            assert_eq!(
                symbols.script_name(script_id),
                Some(match entrance {
                    "Beach" => "EventScript_FestivalEvent_TownSquareExitGuard_FromBeachEntrance",
                    "NorthTown" =>
                        "EventScript_FestivalEvent_TownSquareExitGuard_FromNorthTownEntrance",
                    _ => "EventScript_FestivalEvent_TownSquareExitGuard_FromSouthTownEntrance",
                }),
                "{target} slot {script_id}"
            );
            let names = symbols.names(script_id, symbols.text_count(script_id));
            if !target.starts_with("MARY_MFOMT_") {
                continue;
            }
            let has_suffix = |suffix: &str| {
                let expected = format!(
                    "gText_FestivalEvent_TownSquareExitGuard_From{entrance}Entrance_{suffix}"
                );
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected.as_str()))
            };
            assert_eq!(
                has_suffix("SharedRiceCakeNoodlesOrChickenFestivalNotStarted"),
                target == "MARY_MFOMT_US",
                "{target} slot {script_id}: incorrect US shared-text visibility"
            );
            for suffix in [
                "RiceCakeFestivalNotStarted",
                "YearEndNoodlesFestivalNotStarted",
                "ChickenFestivalNotStarted",
            ] {
                assert_eq!(
                    has_suffix(suffix),
                    target == "MARY_MFOMT_JP",
                    "{target} slot {script_id}: incorrect JP split-text visibility for {suffix}"
                );
            }
        }
    }
}

#[test]
fn mfomt_grape_harvest_job_offer_preserves_us_speaker_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(726, symbols.text_count(726));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        assert_eq!(
            has("gText_NPCEvent_DukeAndManna_GrapeHarvestJobOffer_SharedMannaOrDukeWorkEncouragement"),
            target == "MARY_MFOMT_US",
            "{target} slot 726: incorrect US shared encouragement visibility"
        );
        for split in [
            "gText_NPCEvent_DukeAndManna_GrapeHarvestJobOffer_MannaWorkEncouragement",
            "gText_NPCEvent_DukeAndManna_GrapeHarvestJobOffer_DukeWorkEncouragement",
        ] {
            assert_eq!(
                has(split),
                target == "MARY_MFOMT_JP",
                "{target} slot 726: incorrect JP speaker-specific visibility for {split}"
            );
        }
    }
}

#[test]
fn mfomt_anna_cooking_class_names_recipe_steps_despite_us_text_duplication() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(741, symbols.text_count(741));
        for expected in [
            "gText_NPCEvent_Anna_CookingClass_CheesecakeLesson",
            "gText_NPCEvent_Anna_CookingClass_ApplePieLesson",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 741: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_carter_back_door_choice_preserves_jp_outcome_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(788, symbols.text_count(788));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        for outcome in ["KeepsSecret", "RevealsSecret"] {
            for suffix in ["CarterReturnsToMushrooms", "BackDoorReminder"] {
                let expected =
                    format!("gText_NPCEvent_Carter_OpensChurchBackDoorChoice_{outcome}{suffix}");
                assert_eq!(
                    has(&expected),
                    target == "MARY_MFOMT_US",
                    "{target} slot 788: incorrect US outcome visibility for {expected}"
                );
            }
        }
        for shared in [
            "gText_NPCEvent_Carter_OpensChurchBackDoorChoice_SharedChoiceOutcomeCarterDeparts",
            "gText_NPCEvent_Carter_OpensChurchBackDoorChoice_SharedChoiceOutcomeBackDoorReminder",
        ] {
            assert_eq!(
                has(shared),
                target == "MARY_MFOMT_JP",
                "{target} slot 788: incorrect JP shared outcome visibility for {shared}"
            );
        }
    }
}

#[test]
fn mfomt_cliff_collapse_doctor_thanks_follow_the_marriage_branch() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(790, symbols.text_count(790));
        for expected in [
            "gText_NPCEvent_Cliff_CollapsesInSnow_DoctorSpouseThanksPlayerByNicknameForFindingCliff",
            "gText_NPCEvent_Cliff_CollapsesInSnow_DoctorThanksPlayerForFindingCliff",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 790: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_kai_beach_cafe_help_dialogue_follows_the_marriage_branch() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(804, symbols.text_count(804));
        for expected in [
            "gText_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe_KaiSpouseRequestsHelpByNickname",
            "gText_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe_KaiRequestsHelpByPlayerName",
            "gText_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe_KaiSpouseThanksPlayerByNickname",
            "gText_NPCEvent_Popuri_BringsCustomersToKaiBeachCafe_KaiThanksPlayerByName",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 804: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_animal_death_rick_dialogue_preserves_us_marriage_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(1053),
            Some("EventScript_FarmEvent_Livestock_DeathConsequences"),
            "{target} slot 1053"
        );
        let names = symbols.names(1053, symbols.text_count(1053));
        let has = |expected: &str| {
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected))
        };
        assert_eq!(
            has("gText_FarmEvent_Livestock_DeathConsequences_Shared_RickChastisesPlayerByNameOrNickname"),
            target == "MARY_MFOMT_US",
            "{target} slot 1053: incorrect US shared Rick dialogue visibility"
        );
        for split in [
            "gText_FarmEvent_Livestock_DeathConsequences_RickChastisesPlayerByName",
            "gText_FarmEvent_Livestock_DeathConsequences_RickSpouseChastisesPlayerByNickname",
        ] {
            assert_eq!(
                has(split),
                target == "MARY_MFOMT_JP",
                "{target} slot 1053: incorrect JP marriage branch visibility for {split}"
            );
        }
    }
}

#[test]
fn mfomt_dog_sickness_opening_follows_doctor_marriage_state() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(923, symbols.text_count(923));
        for expected in [
            "gText_FarmEvent_Dog_SicknessFromNeglect_DoctorSpousePresentDogIllnessDiscovery",
            "gText_FarmEvent_Dog_SicknessFromNeglect_DoctorSpouseRequestsClinicTrip",
            "gText_FarmEvent_Dog_SicknessFromNeglect_PlayerAloneDiscoversIllnessAndDecidesClinicTrip",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 923: missing {expected}"
            );
        }
    }
}

#[test]
fn mfomt_big_bed_sleepover_greetings_follow_the_arriving_girl_switch() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(924, symbols.text_count(924));
        for expected in [
            "gText_NPCEvent_MineralTownGirls_BigBedSleepover_PopuriGreetsPlayer",
            "gText_NPCEvent_MineralTownGirls_BigBedSleepover_SharedMaryKarenElliGreeting",
            "gText_NPCEvent_MineralTownGirls_BigBedSleepover_AnnGreeting",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 924: missing {expected}"
            );
        }
    }
}

#[test]
fn sleep_recovery_and_cursed_tool_blessing_is_aligned_across_game_families() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot) in [
        ("MARY_FOMT_US", 986),
        ("MARY_FOMT_JP", 986),
        ("MARY_MFOMT_US", 1054),
        ("MARY_MFOMT_JP", 1054),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(slot),
            Some("EventScript_SystemEvent_SleepRecoveryAndCursedHammerSickleBlessing"),
            "{target} slot {slot}"
        );
        let names = symbols.names(slot, symbols.text_count(slot));
        for expected in [
            "gText_SystemEvent_SleepRecoveryAndCursedHammerSickleBlessing_Overslept",
            "gText_SystemEvent_SleepRecoveryAndCursedHammerSickleBlessing_HammerCurseLifted",
            "gText_SystemEvent_SleepRecoveryAndCursedHammerSickleBlessing_SickleCurseLifted",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {slot}: missing {expected}"
            );
        }
    }
    assert!(!source.contains("EventScript_SystemEvent_WakeUpAndPurifyCursedTool"));
}

#[test]
fn fomt_mine_descent_floor_messages_follow_the_map_ranges() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(978, symbols.text_count(978));
        for expected in [
            "gText_MineEvent_DescendFloorChoiceAndProgress_SpringMineFloorDisplay",
            "gText_MineEvent_DescendFloorChoiceAndProgress_LakeMineFloorDisplay",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 978: missing {expected}"
            );
        }
    }
}

#[test]
fn fomt_spouse_bedtime_silent_reactions_follow_character_love_and_repeat_state() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(343, symbols.text_count(343));
        for expected in [
            "gText_NPCEvent_Spouse_BedtimeDialogue_PopuriBelow30000FirstConversationSilentPause",
            "gText_NPCEvent_Spouse_BedtimeDialogue_AnnAtLeast60000FirstConversationSilentPause",
            "gText_NPCEvent_Spouse_BedtimeDialogue_ElliBelow30000RepeatConversationSilentPause",
        ] {
            assert!(
                names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot 343: missing {expected}"
            );
        }
    }
}

#[test]
fn fomt_won_special_article_gifts_follow_the_article_categories() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1016, symbols.text_count(1016));
        for expected in [
            "gText_NPCEvent_Won_DialogueAndGiftResponses_JewelryGiftPlansHighValueResale",
            "gText_NPCEvent_Won_DialogueAndGiftResponses_BeautyProductGiftPlansResaleToWomen",
        ] {
            assert!(
                names.iter().any(|name| name.as_deref() == Some(expected)),
                "{target} slot 1016: missing {expected}"
            );
        }
    }
}

#[test]
fn fomt_elli_unused_gift_category_8_reactions_preserve_marriage_branches() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1018, symbols.text_count(1018));
        for expected in [
            "gText_NPCEvent_Elli_DialogueAndGiftResponses_UnusedGiftCategory8ElliSpouseAngryReaction",
            "gText_NPCEvent_Elli_DialogueAndGiftResponses_UnusedGiftCategory8ElliSingleAngryReaction",
        ] {
            assert!(names.iter().any(|name| name.as_deref() == Some(expected)), "{target} slot 1018: missing {expected}");
        }
    }
}

#[test]
fn fomt_stu_gift_thanks_preserve_us_liked_neutral_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1019, symbols.text_count(1019));
        let has = |expected: &str| names.iter().any(|name| name.as_deref() == Some(expected));
        assert_eq!(
            has("gText_NPCEvent_Stu_DialogueAndGiftResponses_SharedLikedOrNeutralGiftThanks"),
            target == "MARY_FOMT_US",
            "{target} slot 1019"
        );
        for split in [
            "gText_NPCEvent_Stu_DialogueAndGiftResponses_LikedGiftThanks",
            "gText_NPCEvent_Stu_DialogueAndGiftResponses_NeutralGiftThanks",
        ] {
            assert_eq!(
                has(split),
                target == "MARY_FOMT_JP",
                "{target} slot 1019: {split}"
            );
        }
    }
}

#[test]
fn fomt_gray_married_library_dialogue_preserves_us_location_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1026, symbols.text_count(1026));
        let has = |expected: &str| names.iter().any(|name| name.as_deref() == Some(expected));
        assert_eq!(has("gText_NPCEvent_Gray_DialogueAndGiftResponses_SharedMarriedToMaryTownOrLibraryRepeatBlacksmithCommitment"), target == "MARY_FOMT_US", "{target} slot 1026");
        for split in ["gText_NPCEvent_Gray_DialogueAndGiftResponses_MarriedToMaryTownRepeatBlacksmithCommitment", "gText_NPCEvent_Gray_DialogueAndGiftResponses_MarriedToMaryLibraryRepeatBlacksmithCommitment"] {
            assert_eq!(has(split), target == "MARY_FOMT_JP", "{target} slot 1026: {split}");
        }
    }
}

#[test]
fn fomt_carter_story_transition_texts_preserve_region_specific_slot_reuse() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(1024, symbols.text_count(1024));
        let has = |expected: &str| names.iter().any(|name| name.as_deref() == Some(expected));
        for us_only in [
            "gText_NPCEvent_Carter_DialogueAndGiftResponses_SharedFatherSonStoryConclusionOrScaryStoryIntroduction",
            "gText_NPCEvent_Carter_DialogueAndGiftResponses_SnowWhiteStoryTransition",
        ] {
            assert_eq!(has(us_only), target == "MARY_FOMT_US", "{target} slot 1024: {us_only}");
        }
        assert_eq!(
            has("gText_NPCEvent_Carter_DialogueAndGiftResponses_FatherSonStoryConclusion"),
            target == "MARY_FOMT_JP",
            "{target} slot 1024"
        );
    }
}

#[test]
fn carter_story_texts_use_event_roles_instead_of_sentence_fragments() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let expected = [
        "StorytellingPrompt",
        "StorytellingChoiceListen",
        "StorytellingChoiceDecline",
        "StorytellingDeclineResponse",
        "FatherSonStoryIntroduction",
        "FatherSonStoryBody",
        "FatherSonStoryInterpretationPrompt",
        "FatherSonStoryInterpretation",
        "SnowWhiteStoryPremise",
        "SnowWhiteStoryMirrorQuestion",
        "SnowWhiteStoryHypotheticalTruth",
        "SnowWhiteStoryMirrorLie",
        "SnowWhiteStoryMoralQuestion",
    ];
    for (target, slot) in [
        ("MARY_FOMT_US", 1024),
        ("MARY_FOMT_JP", 1024),
        ("MARY_MFOMT_US", 1094),
        ("MARY_MFOMT_JP", 1094),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols.names(slot, symbols.text_count(slot));
        for suffix in expected {
            let expected = format!("gText_NPCEvent_Carter_DialogueAndGiftResponses_{suffix}");
            assert!(
                names
                    .iter()
                    .any(|name| name.as_deref() == Some(expected.as_str())),
                "{target} slot {slot}: missing {expected}"
            );
        }
    }
}

#[test]
fn talk_choice_results_are_one_based_and_raise_symbolically_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestChoice, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "mary_text_table { const char gText_1[] = \"1\"; const char gText_2[] = \"2\"; const char gText_3[] = \"3\"; const char gText_4[] = \"4\"; const char gText_5[] = \"5\"; const char gText_6[] = \"6\"; }; void TestChoice(void) { int result = TalkChoice6(gText_1, gText_2, gText_3, gText_4, gText_5, gText_6); switch (result) { case 1: return; case 2: return; case 3: return; case 4: return; case 5: return; case 6: return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestChoice").unwrap();
        let source = format_named_script("TestChoice", &raised).unwrap();
        for option in 1..=6 {
            assert!(
                source.contains(&format!("case CHOICE_OPTION_{option}:")),
                "{target}: option {option} was not raised from the one-based result domain:\n{source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: symbolic one-based choice cases changed bytecode"
        );
    }
}

#[test]
fn screen_fade_callables_keep_their_target_ids() {
    for (target, fade_out_id, fade_in_id) in [
        ("MARY_FOMT_US", 0x033, 0x034),
        ("MARY_FOMT_JP", 0x033, 0x034),
        ("MARY_MFOMT_US", 0x034, 0x035),
        ("MARY_MFOMT_JP", 0x034, 0x035),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["FadeOutScreen"].0 .0, fade_out_id, "{target}");
        assert_eq!(callable_map["FadeInScreen"].0 .0, fade_in_id, "{target}");
    }
}

#[test]
fn entity_seat_aux_render_profile_and_fade_alias_callables_follow_all_targets() {
    for (target, fade_alias_id) in [
        ("MARY_FOMT_US", 0x035),
        ("MARY_FOMT_JP", 0x035),
        ("MARY_MFOMT_US", 0x036),
        ("MARY_MFOMT_JP", 0x036),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["SetEntitySpritePriority"].0 .0, 0x007,
            "{target}"
        );
        assert_eq!(
            callable_map["SetEntityAuxRenderProfile"].0 .0, 0x010,
            "{target}"
        );
        assert_eq!(
            callable_map["FadeInScreenAlias"].0 .0, fade_alias_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestEntityState, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestEntityState(void) {\n\
             SetEntitySpritePriority(41, ENTITY_SPRITE_PRIORITY_0);\n\
             SetEntitySpritePriority(41, ENTITY_SPRITE_PRIORITY_1);\n\
             SetEntitySpritePriority(41, ENTITY_SPRITE_PRIORITY_2);\n\
             SetEntitySpritePriority(41, ENTITY_SPRITE_PRIORITY_3);\n\
             SetEntityAuxRenderProfile(70, 0);\n\
             SetEntityAuxRenderProfile(70, 1);\n\
             SetEntityAuxRenderProfile(70, 2);\n\
             SetEntityAuxRenderProfile(70, 3);\n\
             FadeInScreenAlias(3, 1);\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestEntityState")
                .unwrap();
        let source = format_named_script("TestEntityState", &raised).unwrap();
        for symbol in [
            "ENTITY_SPRITE_PRIORITY_0",
            "ENTITY_SPRITE_PRIORITY_1",
            "ENTITY_SPRITE_PRIORITY_2",
            "ENTITY_SPRITE_PRIORITY_3",
            "ENTITY_AUX_RENDER_SMALL_ANIMAL",
            "ENTITY_AUX_RENDER_DEFAULT",
            "ENTITY_AUX_RENDER_LIVESTOCK",
            "ENTITY_AUX_RENDER_DISABLED",
        ] {
            assert!(source.contains(symbol), "{target}: {source}");
        }
    }
}

#[test]
fn inheritance_flashback_and_wedding_finisher_follow_target_tables() {
    for (target, flashback_id, wedding_id) in [
        ("MARY_FOMT_US", 0x0A4, 0x0A8),
        ("MARY_FOMT_JP", 0x0A4, 0x0A8),
        ("MARY_MFOMT_US", 0x0A7, 0x0AB),
        ("MARY_MFOMT_JP", 0x0A7, 0x0AB),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["FinishWeddingSequence"].0 .0, wedding_id, "{target}");
        assert_eq!(
            map["StartFarmInheritanceFlashback"].0 .0, flashback_id,
            "{target}"
        );

        let mut body = String::from("void TestTransition(void) {\n");
        body.push_str("StartFarmInheritanceFlashback();\n");
        body.push_str("FinishWeddingSequence();\n}\n");
        let script_table =
            parse_script_table("mary_script_table { TestTransition, };\n", &options).unwrap();
        parse_named_scripts(&body, &options, &callables.scope, &script_table).unwrap();
    }
}

#[test]
fn farmhouse_modal_menu_lifecycle_hooks_follow_target_tables() {
    for (target, first_id) in [
        ("MARY_FOMT_US", 0x0AA),
        ("MARY_FOMT_JP", 0x0AA),
        ("MARY_MFOMT_US", 0x0AD),
        ("MARY_MFOMT_JP", 0x0AD),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        for (offset, name) in [
            (0, "PrepareClockMenuTransition"),
            (1, "RestoreAfterClockMenu"),
            (2, "PrepareCookingMenuTransition"),
            (3, "RestoreAfterCookingMenu"),
            (4, "PrepareRecipeMenuTransition"),
            (5, "RestoreAfterRecipeMenu"),
        ] {
            assert_eq!(map[name].0 .0, first_id + offset, "{target}: {name}");
        }

        let script_table =
            parse_script_table("mary_script_table { TestMenuHooks, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestMenuHooks(void) {\n\
             PrepareClockMenuTransition(); RestoreAfterClockMenu();\n\
             PrepareCookingMenuTransition(); RestoreAfterCookingMenu();\n\
             PrepareRecipeMenuTransition(); RestoreAfterRecipeMenu();\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn talk_heart_indicator_callables_keep_their_behavioral_slots_on_all_targets() {
    for (target, show_id, hide_id, character) in [
        ("MARY_FOMT_US", 0x031, 0x032, "CHARACTER_KAREN"),
        ("MARY_FOMT_JP", 0x031, 0x032, "CHARACTER_KAREN"),
        ("MARY_MFOMT_US", 0x032, 0x033, "CHARACTER_KAI"),
        ("MARY_MFOMT_JP", 0x032, 0x033, "CHARACTER_KAI"),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["ShowTalkHeartIndicator"]
                .0
                 .0,
            show_id,
            "{target}"
        );
        assert_eq!(
            callables.scope.callable_map()["HideTalkHeartIndicator"]
                .0
                 .0,
            hide_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestHeartUi, };\n", &options).unwrap();
        parse_named_scripts(
            &format!(
                "void TestHeartUi(void) {{\n\
                 ShowTalkHeartIndicator({character});\n\
                 HideTalkHeartIndicator();\n\
                 }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn recovered_unreferenced_callable_handlers_have_stable_target_ids() {
    for (target, shift) in [
        ("MARY_FOMT_US", 0),
        ("MARY_FOMT_JP", 0),
        ("MARY_MFOMT_US", 1),
        ("MARY_MFOMT_JP", 1),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        for (name, expected) in [
            ("NoOp014", 0x00E),
            ("ResetTalkUi", 0x01E),
            ("TalkPromptChoice3", 0x025 + shift),
            ("RandomU15", 0x03B + shift),
            ("AddArticleToRucksack", 0x057 + shift),
            ("PlayerOwnsFood", 0x05E + shift),
            ("CanReceiveTool", 0x06F + shift),
            ("CanReceiveArticle", 0x071 + shift),
            ("GetRucksackUpgradeLevel", 0x075 + shift),
            ("SetCharacterLove", 0x088 + shift * 3),
            ("SetEntityEventScript", 0x089 + shift * 3),
            ("ClearEntityEventScript", 0x08A + shift * 3),
            ("HasHarvestSpriteMinigameExperience", 0x0CF + shift * 3),
            ("HasShippedOneOfEachProduct", 0x0EB + shift * 3),
            ("GetAnimalAge", 0x10B + shift * 3),
            ("GetArticleIconId", 0x13C + shift * 4),
            ("GetToolIconId", 0x13D + shift * 4),
            ("NoOpTutorialFieldTile", 0x13E + shift * 4),
            ("NoOpTutorialFieldObject", 0x13F + shift * 4),
            ("NoOpTutorialEggDefinition", 0x140 + shift * 4),
            ("NoOpTutorialEggSelection", 0x141 + shift * 4),
            ("NoOpAnimalEventEntityInitialization", 0x143 + shift * 4),
        ] {
            assert_eq!(map[name].0 .0, expected, "{target}: {name}");
        }
    }
}

#[test]
fn audited_entity_and_tutorial_helpers_round_trip_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestHelpers, };\n", &options).unwrap();
        let input = "void TestHelpers(void) {\n\
                     NoOp014(0);\n\
                     NoOpTutorialFieldTile(7, 0, 0, 21, 8);\n\
                     NoOpTutorialFieldObject(7, 2, 3);\n\
                     NoOpTutorialEggDefinition(190, 72, 0);\n\
                     NoOpTutorialEggSelection(0);\n\
                     NoOpAnimalEventEntityInitialization(95, 2, 12);\n\
                     SetEntityEventScript(ENTITY_70, 0);\n\
                     SetEntityEventScript(ENTITY_71, 65535);\n\
                     SetEntityEventScript(ENTITY_72, 65536);\n\
                     SetEntityEventScript(ENTITY_73, -1);\n\
                     ClearEntityEventScript(ENTITY_70);\n\
                     SetGameTime(6, 0);\n\
                     SetGameTime(23, 59);\n\
                     SetGameTime(24, 60);\n\
                     SetGameTime(31, 63);\n\
                     SetGameTime(32, 64);\n\
                     SetGameTime(-1, -1);\n\
                     }\n";
        let parsed = parse_named_scripts(input, &options, &callables.scope, &script_table).unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestHelpers").unwrap();
        let source = format_named_script("TestHelpers", &raised).unwrap();
        // Codec-only coverage: unreachable clock targets must never be run
        // in the game. Preserve raw operands rather than normalizing them.
        for call in [
            "SetGameTime(6, 0)",
            "SetGameTime(23, 59)",
            "SetGameTime(24, 60)",
            "SetGameTime(31, 63)",
            "SetGameTime(32, 64)",
            "SetGameTime(-1, -1)",
        ] {
            assert!(source.contains(call), "{target}: missing {call}: {source}");
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: {source}"
        );
    }
}

#[test]
fn name_keyboard_and_rucksack_menu_follow_all_target_tables() {
    for (target, keyboard_id, rucksack_id) in [
        ("MARY_FOMT_US", 0x0A5, 0x0A6),
        ("MARY_FOMT_JP", 0x0A5, 0x0A6),
        ("MARY_MFOMT_US", 0x0A8, 0x0A9),
        ("MARY_MFOMT_JP", 0x0A8, 0x0A9),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["OpenNameEntryKeyboard"].0 .0, keyboard_id, "{target}");
        assert_eq!(map["OpenRucksackMenu"].0 .0, rucksack_id, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestSystemMenus, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestSystemMenus(void) {\n\
             OpenNameEntryKeyboard();\n\
             OpenRucksackMenu(1);\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn frisbee_tournament_round_follows_all_target_tables() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x0DB),
        ("MARY_FOMT_JP", 0x0DB),
        ("MARY_MFOMT_US", 0x0DE),
        ("MARY_MFOMT_JP", 0x0DE),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["RunFrisbeeTournamentRound"]
                .0
                 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestTournament, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestTournament(void) { RunFrisbeeTournamentRound(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn entity_movement_wait_callable_keeps_its_target_id() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["MoveEntityXTo"].0 .0, 0x008, "{target}");
        assert_eq!(callable_map["MoveEntityXToRaw"].0 .0, 0x009, "{target}");
        assert_eq!(callable_map["MoveEntityYTo"].0 .0, 0x00A, "{target}");
        assert_eq!(callable_map["MoveEntityYToRaw"].0 .0, 0x00B, "{target}");
        assert_eq!(
            callable_map["WaitForEntityMovement"].0 .0, 0x00C,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestMove, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestMove(void) {\n\
              MoveEntityXTo(3, 120, 2);\n\
              MoveEntityXToRaw(3, 120, 32768);\n\
              MoveEntityYTo(3, 80, 5);\n\
              MoveEntityYToRaw(3, 80, 65536);\n\
              WaitForEntityMovement(3);\n\
              }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMove").unwrap();
        let source = format_named_script("TestMove", &raised).unwrap();
        for expected in [
            "MoveEntityXTo(ENTITY_POPURI, X(120), ENTITY_MOVE_SPEED_2_PIXELS_PER_FRAME)",
            "MoveEntityXToRaw(ENTITY_POPURI, X(120), ENTITY_MOVE_SPEED_Q16_HALF_PIXEL_PER_FRAME)",
            "MoveEntityYTo(ENTITY_POPURI, Y(80), ENTITY_MOVE_SPEED_5_PIXELS_PER_FRAME)",
            "MoveEntityYToRaw(ENTITY_POPURI, Y(80), ENTITY_MOVE_SPEED_Q16_1_PIXEL_PER_FRAME)",
        ] {
            assert!(
                source.contains(expected),
                "{target}: missing {expected} in {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: movement speed symbols changed emitted bytecode"
        );
    }
}

#[test]
fn hide_entity_callable_keeps_its_target_id() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["HideEntity"].0 .0, 0x00F, "{target}");
    }
}

#[test]
fn camera_movement_callables_keep_their_target_ids() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["ChangeMap"].0 .0, 0x016, "{target}");
        assert_eq!(callable_map["PanCameraTo"].0 .0, 0x017, "{target}");
        assert_eq!(
            callable_map["WaitForCameraMovement"].0 .0, 0x018,
            "{target}"
        );
        let camera_speed = constants.user_type("MaryCameraMoveSpeed").unwrap();
        for (value, name) in [
            (1, "CAMERA_MOVE_SPEED_NOMINAL_1_PIXEL_PER_UPDATE"),
            (2, "CAMERA_MOVE_SPEED_NOMINAL_2_PIXELS_PER_UPDATE"),
            (5, "CAMERA_MOVE_SPEED_NOMINAL_5_PIXELS_PER_UPDATE"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(camera_speed, value),
                Some(name),
                "{target}: camera speed {value}"
            );
        }
        assert!(
            constants.typed_int_const_name(camera_speed, 3).is_none(),
            "{target}: unobserved camera speed 3 must remain numeric"
        );

        let script_table =
            parse_script_table("mary_script_table { TestMapChange, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestMapChange(void) {\n\
             ChangeMap(MAP_SPRING_MINE_FLOOR_58, 120, 192);\n\
             PanCameraTo(40, 120, 1);\n\
             PanCameraTo(120, 192, 2);\n\
             PanCameraTo(184, 164, 5);\n\
             PanCameraTo(184, 164, 3);\n\
             WaitForCameraMovement();\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMapChange")
                .unwrap();
        let source = format_named_script("TestMapChange", &raised).unwrap();
        assert!(
            source.contains("ChangeMap(MAP_SPRING_MINE_FLOOR_58, X(120), Y(192))"),
            "{target}: {source}"
        );
        assert!(
            source.contains(
                "PanCameraTo(X(40), Y(120), CAMERA_MOVE_SPEED_NOMINAL_1_PIXEL_PER_UPDATE)"
            ),
            "{target}: {source}"
        );
        assert!(
            source.contains(
                "PanCameraTo(X(120), Y(192), CAMERA_MOVE_SPEED_NOMINAL_2_PIXELS_PER_UPDATE)"
            ),
            "{target}: {source}"
        );
        assert!(
            source.contains(
                "PanCameraTo(X(184), Y(164), CAMERA_MOVE_SPEED_NOMINAL_5_PIXELS_PER_UPDATE)"
            ),
            "{target}: {source}"
        );
        assert!(
            source.contains("PanCameraTo(X(184), Y(164), 3)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn entity_effect_callables_keep_their_target_ids() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["StartEntityEffect"].0 .0, 0x011, "{target}");
        assert_eq!(callable_map["StopEntityEffect"].0 .0, 0x012, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestEmotes, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestEmotes(void) { StartEntityEffect(0, 0, FALSE); StartEntityEffect(1, 8, TRUE); StopEntityEffect(1); StartEntityEffect(2, 10, FALSE); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestEmotes").unwrap();
        let source = format_named_script("TestEmotes", &raised).unwrap();
        assert!(
            source.contains("StartEntityEffect(ENTITY_PLAYER, ENTITY_EMOTE_ANGRY, FALSE)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("StartEntityEffect(ENTITY_LILLIA, ENTITY_EMOTE_SLEEP, TRUE)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("StopEntityEffect(ENTITY_LILLIA)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("StartEntityEffect(ENTITY_RICK, ENTITY_EMOTE_BAD, FALSE)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn composed_dialogue_and_nameplate_callables_keep_their_target_ids() {
    for (
        target,
        append_id,
        choice_2_id,
        choice_4_id,
        prompt_id,
        clear_prompt_id,
        clear_portrait_id,
    ) in [
        ("MARY_FOMT_US", 0x023, 0x024, 0x026, 0x02D, 0x02E, 0x030),
        ("MARY_FOMT_JP", 0x023, 0x024, 0x026, 0x02D, 0x02E, 0x030),
        ("MARY_MFOMT_US", 0x024, 0x025, 0x027, 0x02E, 0x02F, 0x031),
        ("MARY_MFOMT_JP", 0x024, 0x025, 0x027, 0x02E, 0x02F, 0x031),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        let prompt_choice_type = constants.user_type("MaryPromptChoiceResult").unwrap();
        let choice_type = constants.user_type("MaryChoiceResult").unwrap();
        assert_eq!(
            callable_map["TalkAppendMessage"].0 .0, append_id,
            "{target}"
        );
        assert_eq!(
            callable_map["TalkPromptChoice2"].0 .0, choice_2_id,
            "{target}"
        );
        assert_eq!(
            callable_map["TalkPromptChoice4"].0 .0, choice_4_id,
            "{target}"
        );
        assert_eq!(
            callable_map["TalkPromptChoice2"].1.return_type(),
            mary::ir::ValueType::UserType(prompt_choice_type),
            "{target}"
        );
        assert_eq!(
            callable_map["TalkChoice6"].1.return_type(),
            mary::ir::ValueType::UserType(choice_type),
            "{target}"
        );
        assert_eq!(
            callable_map["ClearTalkPortrait"].0 .0, clear_portrait_id,
            "{target}"
        );
        assert_eq!(
            callable_map["SetTalkNameplateText"].0 .0, prompt_id,
            "{target}"
        );
        assert_eq!(
            callable_map["ClearTalkNameplate"].0 .0, clear_prompt_id,
            "{target}"
        );
        if target.starts_with("MARY_MFOMT") {
            assert_eq!(callable_map["TalkOpenNoPortrait"].0 .0, 0x020, "{target}");
        } else {
            assert!(!callable_map.contains_key("TalkOpenNoPortrait"), "{target}");
        }

        let script_table =
            parse_script_table("mary_script_table { TestNameplate, };\n", &options).unwrap();
        parse_named_scripts(
            "mary_text_table { const char gText_PickOne[] = \"Pick one{Press}\"; };\n\
             void TestNameplate(void) {\n\
             SetTalkNameplateCharacter(CHARACTER_KAREN);\n\
             SetTalkNameplateText(gText_PickOne);\n\
             ClearTalkNameplate();\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn held_item_action_callables_keep_their_target_ids() {
    for (target, use_id, clear_id, throw_id) in [
        ("MARY_FOMT_US", 0x045, 0x046, 0x04D),
        ("MARY_FOMT_JP", 0x045, 0x046, 0x04D),
        ("MARY_MFOMT_US", 0x046, 0x047, 0x04E),
        ("MARY_MFOMT_JP", 0x046, 0x047, 0x04E),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["UsePlayerHeldItem"].0 .0, use_id, "{target}");
        assert_eq!(
            callable_map["ClearPlayerHeldItem"].0 .0, clear_id,
            "{target}"
        );
        assert_eq!(
            callable_map["ThrowPlayerHeldItem"].0 .0, throw_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestHeldItem, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestHeldItem(void) {\n\
             UsePlayerHeldItem();\n\
             ClearPlayerHeldItem();\n\
             ThrowPlayerHeldItem();\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn television_callables_keep_their_target_ids() {
    for (target, show_id, program_id, end_id) in [
        ("MARY_FOMT_US", 0x0FB, 0x0FC, 0x0FD),
        ("MARY_FOMT_JP", 0x0FB, 0x0FC, 0x0FD),
        ("MARY_MFOMT_US", 0x0FE, 0x0FF, 0x100),
        ("MARY_MFOMT_JP", 0x0FE, 0x0FF, 0x100),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["ShowTelevisionMessage"].0 .0, show_id,
            "{target}"
        );
        assert_eq!(
            callable_map["SetTelevisionProgram"].0 .0, program_id,
            "{target}"
        );
        assert_eq!(
            callable_map["EndTelevisionProgram"].0 .0, end_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestTelevision, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "mary_text_table { const char gText_Television[] = \"News{Press}\"; };\n\
             void TestTelevision(void) {\n\
             SetTelevisionProgram(3);\n\
             SetTelevisionProgram(20);\n\
             SetTelevisionProgram(24);\n\
             int action = ShowTelevisionMessage(gText_Television);\n\
             EndTelevisionProgram();\n\
             switch (action) {\n\
             case 0: return;\n\
             case 1: return;\n\
             case 2: return;\n\
             case 3: return;\n\
             case 4: return;\n\
             case 5: return;\n\
             }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestTelevision")
                .unwrap();
        let source = format_named_script("TestTelevision", &raised).unwrap();
        assert!(
            source.contains("SetTelevisionProgram(TELEVISION_PROGRAM_DUELING_CHEFS)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("SetTelevisionProgram(TELEVISION_PROGRAM_TV_SHOPPING)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("SetTelevisionProgram(TELEVISION_PROGRAM_WEATHER_TYPHOON)"),
            "{target}: {source}"
        );
        for input in [
            "TELEVISION_INPUT_UP_WEATHER",
            "TELEVISION_INPUT_DOWN_FARM_PROGRAM",
            "TELEVISION_INPUT_RIGHT_NEWS",
            "TELEVISION_INPUT_LEFT_VARIETY",
            "TELEVISION_INPUT_ADVANCE_TEXT",
            "TELEVISION_INPUT_TURN_OFF",
        ] {
            assert!(
                source.contains(&format!("case {input}:")),
                "{target}: missing {input}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn television_return_type_survives_consistent_switch_branch_definitions() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestTelevisionBranches, };\n", &options)
                .unwrap();
        let parsed = parse_named_scripts(
            "mary_text_table {\n\
                 const char gText_Episode1[] = \"One{Press}\";\n\
                 const char gText_Episode2[] = \"Two{Press}\";\n\
             };\n\
             void TestTelevisionBranches(void) {\n\
                 int selector = RandomU15();\n\
                 int action;\n\
                 switch (selector) {\n\
                     case 0: action = ShowTelevisionMessage(gText_Episode1); break;\n\
                     case 1: action = ShowTelevisionMessage(gText_Episode2); break;\n\
                 }\n\
                 switch (action) {\n\
                     case 0: return;\n\
                     case 1: return;\n\
                     case 2: return;\n\
                     case 3: return;\n\
                     case 4: return;\n\
                     case 5: return;\n\
                 }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &parsed.scripts[0].2,
            &callables.scope,
            "TestTelevisionBranches",
        )
        .unwrap();
        let source = format_named_script("TestTelevisionBranches", &raised).unwrap();
        for input in [
            "TELEVISION_INPUT_UP_WEATHER",
            "TELEVISION_INPUT_DOWN_FARM_PROGRAM",
            "TELEVISION_INPUT_RIGHT_NEWS",
            "TELEVISION_INPUT_LEFT_VARIETY",
            "TELEVISION_INPUT_ADVANCE_TEXT",
            "TELEVISION_INPUT_TURN_OFF",
        ] {
            assert!(
                source.contains(&format!("case {input}:")),
                "{target}: missing {input}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn scripted_npc_control_callables_keep_their_target_ids() {
    for (target, enable_id, disable_id) in [
        ("MARY_FOMT_US", 0x0EF, 0x0F0),
        ("MARY_FOMT_JP", 0x0EF, 0x0F0),
        ("MARY_MFOMT_US", 0x0F2, 0x0F3),
        ("MARY_MFOMT_JP", 0x0F2, 0x0F3),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["EnableScriptedNpcControl"].0 .0, enable_id,
            "{target}"
        );
        assert_eq!(
            callable_map["DisableScriptedNpcControl"].0 .0, disable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestNpcControl, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestNpcControl(void) {\n\
             EnableScriptedNpcControl();\n\
             DisableScriptedNpcControl();\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn refresh_all_npc_schedules_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x0FE),
        ("MARY_FOMT_JP", 0x0FE),
        ("MARY_MFOMT_US", 0x101),
        ("MARY_MFOMT_JP", 0x101),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["RefreshAllNpcSchedules"].0 .0, callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestNpcRefresh, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestNpcRefresh(void) { RefreshAllNpcSchedules(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn event_icon_callables_keep_their_target_ids() {
    for (target, create_id, remove_id) in [
        ("MARY_FOMT_US", 0x139, 0x13A),
        ("MARY_FOMT_JP", 0x139, 0x13A),
        ("MARY_MFOMT_US", 0x13D, 0x13E),
        ("MARY_MFOMT_JP", 0x13D, 0x13E),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["CreateEventIcon"].0 .0, create_id, "{target}");
        assert_eq!(callable_map["RemoveEventIcon"].0 .0, remove_id, "{target}");
        let icon_slot_type = constants.user_type("MaryEventIconSlot").unwrap();
        let map_x_type = constants.user_type("MaryMapSpaceX").unwrap();
        let map_y_type = constants.user_type("MaryMapSpaceY").unwrap();
        let icon_layer_type = constants.user_type("MaryEventIconLayer").unwrap();
        let icon_id_type = constants.user_type("MaryEventIconId").unwrap();
        assert_eq!(
            callable_map["CreateEventIcon"].1.parameter_types(),
            &[
                mary::ir::ValueType::UserType(icon_slot_type),
                mary::ir::ValueType::UserType(map_x_type),
                mary::ir::ValueType::UserType(map_y_type),
                mary::ir::ValueType::UserType(icon_layer_type),
                mary::ir::ValueType::UserType(icon_id_type),
            ],
            "{target}"
        );
        assert_eq!(
            callable_map["RemoveEventIcon"].1.parameter_types(),
            &[mary::ir::ValueType::UserType(icon_slot_type)],
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestEventIcon, };\n", &options).unwrap();
        for layer in 0..=4 {
            let numeric = parse_named_scripts(
                &format!("void TestEventIcon(void) {{ CreateEventIcon(0, 10, 20, {layer}, 0); }}"),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised =
                decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestEventIcon")
                    .unwrap();
            let source = format_named_script("TestEventIcon", &raised).unwrap();
            if layer < 4 {
                assert!(
                    source.contains(&format!("EVENT_ICON_LAYER_{layer}")),
                    "{target}: {source}"
                );
            } else {
                assert!(
                    source.contains("Y(20), 4,"),
                    "must not truncate bytecode to layer zero: {target}: {source}"
                );
            }
            let rebuilt =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&rebuilt.scripts[0].2),
                "{target}: layer {layer}"
            );
        }
        let numeric = parse_named_scripts(
            "void TestEventIcon(void) { CreateEventIcon(0, 288, 123, 2, GetFoodIconId(FOOD_CAKE)); RemoveEventIcon(1); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestEventIcon")
                .unwrap();
        let source = format_named_script("TestEventIcon", &raised).unwrap();
        assert!(
            source.contains("CreateEventIcon(EVENT_ICON_SLOT_0, X(288), Y(123), EVENT_ICON_LAYER_2, GetFoodIconId(FOOD_CAKE))"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RemoveEventIcon(EVENT_ICON_SLOT_1)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
        for callable in ["GetFoodIconId", "GetArticleIconId", "GetToolIconId"] {
            assert_eq!(
                callable_map[callable].1.return_type(),
                mary::ir::ValueType::UserType(icon_id_type),
                "{target}: {callable}"
            );
        }

        let script_table =
            parse_script_table("mary_script_table { TestEventIcon, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestEventIcon(void) {\n\
             CreateEventIcon(2, 176, 152, 1, GetFoodIconId(5));\n\
             RemoveEventIcon(2);\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn map_local_door_indices_are_not_misrepresented_as_global_door_constants() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let door_type = constants.user_type("MaryDoorIndex").unwrap();
        for callable in ["OpenDoor", "CloseDoor"] {
            assert_eq!(
                callables.scope.callable_map()[callable].1.parameter_types(),
                &[mary::ir::ValueType::UserType(door_type)],
                "{target}: {callable}"
            );
        }

        let script_table =
            parse_script_table("mary_script_table { TestDoor, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestDoor(void) { OpenDoor(4); CloseDoor(4); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestDoor").unwrap();
        let source = format_named_script("TestDoor", &raised).unwrap();
        assert!(source.contains("OpenDoor(4)"), "{target}: {source}");
        assert!(source.contains("CloseDoor(4)"), "{target}: {source}");
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn cursed_tool_callables_keep_their_target_ids_and_accept_tool_symbols() {
    for (target, is_cursed_id, advance_id, church_id) in [
        ("MARY_FOMT_US", 0x136, 0x137, 0x138),
        ("MARY_FOMT_JP", 0x136, 0x137, 0x138),
        ("MARY_MFOMT_US", 0x13A, 0x13B, 0x13C),
        ("MARY_MFOMT_JP", 0x13A, 0x13B, 0x13C),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(callable_map["IsToolCursed"].0 .0, is_cursed_id, "{target}");
        assert_eq!(
            callable_map["AdvanceCursedToolLiftProgress"].0 .0, advance_id,
            "{target}"
        );
        assert_eq!(
            callable_map["AttemptChurchCursedToolRemoval"].0 .0, church_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestCursedTool, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestCursedTool(void) {\n\
             if (IsToolCursed(TOOL_SICKLE_CURSED)) {\n\
                 AdvanceCursedToolLiftProgress(TOOL_SICKLE_CURSED);\n\
                 AttemptChurchCursedToolRemoval(TOOL_SICKLE_CURSED);\n\
             }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn reference_page_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x098),
        ("MARY_FOMT_JP", 0x098),
        ("MARY_MFOMT_US", 0x09B),
        ("MARY_MFOMT_JP", 0x09B),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["ShowReferencePage"].0 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestReferencePage, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestReferencePage(void) { ShowReferencePage(61); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let symbolic = parse_named_scripts(
            "void TestReferencePage(void) { ShowReferencePage(REFERENCE_PAGE_PHONE_SAIBARA_BLACKSMITH); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn reference_pages_and_mailbox_letters_follow_their_distinct_domains() {
    for (target, letter_id, letter_name, credits_id) in [
        (
            "MARY_FOMT_US",
            134,
            "LETTER_HARVEST_SPRITE_TEA_PARTY_INVITATION",
            135,
        ),
        (
            "MARY_FOMT_JP",
            134,
            "LETTER_HARVEST_SPRITE_TEA_PARTY_INVITATION",
            135,
        ),
        (
            "MARY_MFOMT_US",
            187,
            "LETTER_HARVEST_SPRITE_TEA_PARTY_INVITATION",
            188,
        ),
        (
            "MARY_MFOMT_JP",
            187,
            "LETTER_HARVEST_SPRITE_TEA_PARTY_INVITATION",
            188,
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPages, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestPages(void) {{ DeliverLetter({letter_id}); ShowReferencePage({credits_id}); }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPages").unwrap();
        let source = format_named_script("TestPages", &raised).unwrap();
        assert!(
            source.contains(&format!("DeliverLetter({letter_name})")),
            "{target}: {source}"
        );
        assert!(
            source.contains("ShowReferencePage(REFERENCE_PAGE_STAFF_CREDITS)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn mailbox_callables_use_the_bounded_letter_id_domain() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let letter_type =
            mary::ir::ValueType::UserType(constants.user_type("MaryLetterId").unwrap());
        for name in [
            "IsLetterWaiting",
            "HasReceivedLetter",
            "DeliverLetter",
            "MarkLetterRead",
        ] {
            assert_eq!(
                callables.scope.callable_map()[name].1.parameter_types(),
                &[letter_type],
                "{target} {name}"
            );
        }
    }
}

#[test]
fn mailbox_letter_symbols_cover_each_native_boundary_without_crossing_it() {
    for (target, last_id, last_name, out_of_range) in [
        ("MARY_FOMT_US", 136, "LETTER_USEFUL_CONTROLS", Some(137)),
        ("MARY_FOMT_JP", 136, "LETTER_USEFUL_CONTROLS", Some(137)),
        ("MARY_MFOMT_US", 189, "LETTER_USEFUL_CONTROLS", None),
        ("MARY_MFOMT_JP", 189, "LETTER_USEFUL_CONTROLS", None),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestLetterBoundary, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            &format!("void TestLetterBoundary(void) {{ DeliverLetter({last_id}); }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestLetterBoundary",
        )
        .unwrap();
        let source = format_named_script("TestLetterBoundary", &raised).unwrap();
        assert!(
            source.contains(&format!("DeliverLetter({last_name})")),
            "{target}: {source}"
        );

        if let Some(value) = out_of_range {
            let numeric = parse_named_scripts(
                &format!("void TestLetterBoundary(void) {{ DeliverLetter({value}); }}\n"),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestLetterBoundary",
            )
            .unwrap();
            let source = format_named_script("TestLetterBoundary", &raised).unwrap();
            assert!(source.contains("DeliverLetter(137)"), "{target}: {source}");
        }
    }
}

#[test]
fn link_milestone_callables_keep_their_target_ids() {
    for (target, local_id, received_id, set_id, clear_id) in [
        ("MARY_FOMT_US", 0x077, 0x078, 0x079, 0x07A),
        ("MARY_FOMT_JP", 0x077, 0x078, 0x079, 0x07A),
        ("MARY_MFOMT_US", 0x07A, 0x07B, 0x07C, 0x07D),
        ("MARY_MFOMT_JP", 0x07A, 0x07B, 0x07C, 0x07D),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["HasLocalLinkMilestone"].0 .0, local_id,
            "{target}"
        );
        assert_eq!(
            callable_map["HasReceivedLinkMilestone"].0 .0, received_id,
            "{target}"
        );
        assert_eq!(
            callable_map["SetLocalLinkMilestone"].0 .0, set_id,
            "{target}"
        );
        assert_eq!(
            callable_map["ClearLocalLinkMilestone"].0 .0, clear_id,
            "{target}"
        );
    }
}

#[test]
fn farmhouse_bed_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x126),
        ("MARY_FOMT_JP", 0x126),
        ("MARY_MFOMT_US", 0x12A),
        ("MARY_MFOMT_JP", 0x12A),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["PlacePlayerAtFarmhouseBed"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn supermarket_item_symbols_print_and_round_trip_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x08C),
        ("MARY_FOMT_JP", 0x08C),
        ("MARY_MFOMT_US", 0x08F),
        ("MARY_MFOMT_JP", 0x08F),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["PurchaseSupermarketItem"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestSupermarket, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestSupermarket(void) { PurchaseSupermarketItem(6); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestSupermarket")
                .unwrap();
        let source = format_named_script("TestSupermarket", &raised).unwrap();
        assert!(
            source.contains("PurchaseSupermarketItem(SUPERMARKET_ITEM_CHOCOLATE)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn name_entry_symbols_respect_target_evidence_and_round_trip() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x0A3),
        ("MARY_FOMT_JP", 0x0A3),
        ("MARY_MFOMT_US", 0x0A6),
        ("MARY_MFOMT_JP", 0x0A6),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["OpenNameEntry"].0 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestNameEntry, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestNameEntry(void) { OpenNameEntry(0, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestNameEntry")
                .unwrap();
        let source = format_named_script("TestNameEntry", &raised).unwrap();
        assert!(
            source.contains("OpenNameEntry(NAME_ENTRY_HORSE, NAME_ENTRY_SINGLETON_SLOT)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );

        let child = parse_named_scripts(
            "void TestNameEntry(void) { OpenNameEntry(4, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&child.scripts[0].2, &callables.scope, "TestNameEntry").unwrap();
        let source = format_named_script("TestNameEntry", &raised).unwrap();
        assert!(
            source.contains("OpenNameEntry(NAME_ENTRY_CHILD, NAME_ENTRY_SINGLETON_SLOT)"),
            "{target}: {source}"
        );

        let dependent_slots = parse_named_scripts(
            "void TestNameEntry(void) {\n\
                 int barn_slot = 2;\n\
                 OpenNameEntry(1, barn_slot);\n\
                 OpenNameEntry(3, 7);\n\
                 OpenNameEntry(4, 0);\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &dependent_slots.scripts[0].2,
            &callables.scope,
            "TestNameEntry",
        )
        .unwrap();
        let source = format_named_script("TestNameEntry", &raised).unwrap();
        assert!(source.contains("ANIMAL_SLOT_3"), "{target}: {source}");
        assert!(
            source.contains("OpenNameEntry(NAME_ENTRY_CHICKEN, CHICKEN_SLOT_8)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("OpenNameEntry(NAME_ENTRY_CHILD, NAME_ENTRY_SINGLETON_SLOT)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&dependent_slots.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );

        let spouse = parse_named_scripts(
            "void TestNameEntry(void) { OpenNameEntry(5, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&spouse.scripts[0].2, &callables.scope, "TestNameEntry")
                .unwrap();
        let source = format_named_script("TestNameEntry", &raised).unwrap();
        assert!(
            source.contains("OpenNameEntry(NAME_ENTRY_SPOUSE_NICKNAME, NAME_ENTRY_SINGLETON_SLOT)"),
            "{target}: {source}"
        );
    }
}

#[test]
fn contest_animal_callables_keep_target_ids_and_symbolic_kind() {
    for (target, set_id, clear_id, get_id) in [
        ("MARY_FOMT_US", 0x118, 0x119, 0x11A),
        ("MARY_FOMT_JP", 0x118, 0x119, 0x11A),
        ("MARY_MFOMT_US", 0x11B, 0x11C, 0x11D),
        ("MARY_MFOMT_JP", 0x11B, 0x11C, 0x11D),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["SetContestAnimal"].0 .0, set_id, "{target}");
        assert_eq!(map["ClearContestAnimal"].0 .0, clear_id, "{target}");
        assert_eq!(map["GetContestAnimalIndex"].0 .0, get_id, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestContest, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestContest(void) {\n\
             SetContestAnimal(1, 2);\n\
             switch (GetContestAnimalIndex(1)) {\n\
             case -1: return;\n\
             case 2: return;\n\
             }\n\
             ClearContestAnimal(1);\n\
             SetContestAnimal(3, 7);\n\
             switch (GetContestAnimalIndex(3)) {\n\
             case -1: return;\n\
             case 7: return;\n\
             }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestContest").unwrap();
        let source = format_named_script("TestContest", &raised).unwrap();
        assert!(
            source.contains("SetContestAnimal(ANIMAL_KIND_COW, ANIMAL_SLOT_3)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ClearContestAnimal(ANIMAL_KIND_COW)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case ANIMAL_SLOT_NOT_SELECTED:"),
            "{target}: {source}"
        );
        assert!(source.contains("case ANIMAL_SLOT_3:"), "{target}: {source}");
        assert!(
            source.contains("SetContestAnimal(ANIMAL_KIND_CHICKEN, CHICKEN_SLOT_8)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_NONE:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case CHICKEN_SLOT_8:"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn animal_growth_stage_results_use_the_selected_species_domain() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestGrowth, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestGrowth(void) {\n\
             if (GetAnimalGrowthStage(3, 0) == 0) { return; }\n\
             if (GetAnimalGrowthStage(1, 1) == 2) { return; }\n\
             if (GetAnimalGrowthStage(2, 2) == 1) { return; }\n\
             if (GetAnimalGrowthStage(0, 0) == 1) { return; }\n\
             GetAnimalName(0, 3, 7);\n\
             if (DoesAnimalExist(3, 7) == 1) { return; }\n\
             int kind = 3;\n\
             if (GetAnimalGrowthStage(kind, 7) == 0) { return; }\n\
             kind = 1;\n\
             if (GetAnimalGrowthStage(kind, 2) == 2) { return; }\n\
             if (RandomIntInclusive(0, 1) == 0) { kind = 3; } else { kind = 3; }\n\
             if (GetAnimalGrowthStage(kind, 6) == 0) { return; }\n\
             if (RandomIntInclusive(0, 1) == 0) { kind = 3; } else { kind = 1; }\n\
             if (GetAnimalGrowthStage(kind, 0) == 0) { return; }\n\
             kind = 3;\n\
             for (int i = 0; i < 1; i++) { kind = 1; }\n\
             if (GetAnimalGrowthStage(kind, 0) == 0) { return; }\n\
             do { kind = 3; } while (RandomIntInclusive(0, 1) == 0);\n\
             if (GetAnimalGrowthStage(kind, 5) == 0) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestGrowth").unwrap();
        let source = format_named_script("TestGrowth", &raised).unwrap();
        for expected in [
            "GetAnimalGrowthStage(ANIMAL_KIND_CHICKEN, CHICKEN_SLOT_1) == CHICKEN_GROWTH_STAGE_CHICK",
            "GetAnimalGrowthStage(ANIMAL_KIND_COW, ANIMAL_SLOT_2) == COW_GROWTH_STAGE_ADULT",
            "GetAnimalGrowthStage(ANIMAL_KIND_SHEEP, ANIMAL_SLOT_3) == SHEEP_GROWTH_STAGE_ADULT",
            "GetAnimalGrowthStage(ANIMAL_KIND_HORSE, ANIMAL_SLOT_1) == PET_GROWTH_STAGE_ADULT",
            "GetAnimalName(TEXT_VARIABLE_1, ANIMAL_KIND_CHICKEN, CHICKEN_SLOT_8)",
            "DoesAnimalExist(ANIMAL_KIND_CHICKEN, CHICKEN_SLOT_8) == TRUE",
            "GetAnimalGrowthStage(var_0, CHICKEN_SLOT_8) == CHICKEN_GROWTH_STAGE_CHICK",
            "GetAnimalGrowthStage(var_0, ANIMAL_SLOT_3) == COW_GROWTH_STAGE_ADULT",
            "GetAnimalGrowthStage(var_0, CHICKEN_SLOT_7) == CHICKEN_GROWTH_STAGE_CHICK",
            "GetAnimalGrowthStage(var_0, ANIMAL_SLOT_1) == 0",
            "GetAnimalGrowthStage(var_0, CHICKEN_SLOT_6) == CHICKEN_GROWTH_STAGE_CHICK",
        ] {
            assert!(source.contains(expected), "{target}: missing {expected}: {source}");
        }
        assert_eq!(
            source
                .matches("GetAnimalGrowthStage(var_0, ANIMAL_SLOT_1) == 0")
                .count(),
            2,
            "{target}: conflicting branch and optional-loop joins must both remain untyped: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn chicken_family_slot_metadata_covers_every_shared_livestock_callable() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let chicken_slot_type = mary::ir::ValueType::UserType(
            constants
                .user_type("MaryChickenSlotIndex")
                .expect("missing chicken slot type"),
        );
        for (callable, discriminator_index, target_index) in [
            ("DoesAnimalExist", 0, 1),
            ("GetAnimalName", 1, 2),
            ("HasAnimalBeenTalkedTo", 0, 1),
            ("SetAnimalTalkedTo", 0, 1),
            ("AddAnimalAffection", 0, 1),
            ("IsAnimalUnhappy", 0, 1),
            ("IsAnimalSick", 0, 1),
            ("GetAnimalAge", 0, 1),
            ("GetAnimalAffection", 0, 1),
            ("GetAnimalGrowthStage", 0, 1),
            ("SetContestAnimal", 0, 1),
        ] {
            assert_eq!(
                constants.dependent_callable_parameter_type(
                    callable,
                    discriminator_index,
                    3,
                    target_index,
                ),
                Some(chicken_slot_type),
                "{target}: {callable}"
            );
        }
        assert_eq!(
            constants.dependent_callable_return_type("GetContestAnimalIndex", 0, 3),
            Some(chicken_slot_type),
            "{target}"
        );
        assert_eq!(
            constants.dependent_callable_return_type("SelectFestivalAnimal", 0, 0),
            Some(chicken_slot_type),
            "{target}"
        );
    }
}

#[test]
fn caught_fish_state_callables_keep_their_target_ids() {
    for (target, name_id, size_id, maximum_id, king_id) in [
        ("MARY_FOMT_US", 0x11B, 0x11C, 0x11D, 0x11E),
        ("MARY_FOMT_JP", 0x11B, 0x11C, 0x11D, 0x11E),
        ("MARY_MFOMT_US", 0x11E, 0x11F, 0x120, 0x121),
        ("MARY_MFOMT_JP", 0x11E, 0x11F, 0x120, 0x121),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(
            map["SetTextVariableToCaughtFishName"].0 .0, name_id,
            "{target}"
        );
        assert_eq!(map["GetCaughtFishSize"].0 .0, size_id, "{target}");
        assert_eq!(map["IsCaughtFishMaximumSize"].0 .0, maximum_id, "{target}");
        assert_eq!(map["IsCaughtFishKing"].0 .0, king_id, "{target}");
    }
}

#[test]
fn spouse_nickname_callable_keeps_target_id_and_accepts_text_symbol() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x125),
        ("MARY_FOMT_JP", 0x125),
        ("MARY_MFOMT_US", 0x129),
        ("MARY_MFOMT_JP", 0x129),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SetPlayerNicknameForSpouse"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestNickname, };\n", &options).unwrap();
        parse_named_scripts(
            "mary_text_table { const char gText_Honey[] = \"Honey{Press}\"; };\n\
             void TestNickname(void) { SetPlayerNicknameForSpouse(gText_Honey); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn descend_mine_floor_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x135),
        ("MARY_FOMT_JP", 0x135),
        ("MARY_MFOMT_US", 0x139),
        ("MARY_MFOMT_JP", 0x139),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["DescendMineFloor"].0 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn interacting_animal_index_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x103),
        ("MARY_FOMT_JP", 0x103),
        ("MARY_MFOMT_US", 0x106),
        ("MARY_MFOMT_JP", 0x106),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["GetInteractingAnimalIndex"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn mountain_cottage_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x0C4),
        ("MARY_FOMT_JP", 0x0C4),
        ("MARY_MFOMT_US", 0x0C7),
        ("MARY_MFOMT_JP", 0x0C7),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["BuildMountainCottage"].0 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn livestock_life_state_symbols_print_and_round_trip_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x10F),
        ("MARY_FOMT_JP", 0x10F),
        ("MARY_MFOMT_US", 0x112),
        ("MARY_MFOMT_JP", 0x112),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["CountAnimalsByLifeState"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestLifeState, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestLifeState(void) { int count = CountAnimalsByLifeState(2, 2); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestLifeState")
                .unwrap();
        let source = format_named_script("TestLifeState", &raised).unwrap();
        assert!(
            source.contains(
                "CountAnimalsByLifeState(ANIMAL_KIND_SHEEP, \
                 LIVESTOCK_LIFE_STATE_DIED_FROM_NEGLECT)"
            ),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn livestock_death_summary_and_cleanup_callables_keep_target_ids() {
    for (target, neglect_summary, neglect_remove, natural_summary, natural_remove) in [
        ("MARY_FOMT_US", 0x110, 0x111, 0x112, 0x113),
        ("MARY_FOMT_JP", 0x110, 0x111, 0x112, 0x113),
        ("MARY_MFOMT_US", 0x113, 0x114, 0x115, 0x116),
        ("MARY_MFOMT_JP", 0x113, 0x114, 0x115, 0x116),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(
            map["ShowLivestockNeglectDeathSummary"].0 .0, neglect_summary,
            "{target}"
        );
        assert_eq!(
            map["RemoveLivestockDeadFromNeglect"].0 .0, neglect_remove,
            "{target}"
        );
        assert_eq!(
            map["ShowNaturalLivestockDeathSummary"].0 .0, natural_summary,
            "{target}"
        );
        assert_eq!(
            map["RemoveNaturallyDeadLivestock"].0 .0, natural_remove,
            "{target}"
        );
    }
}

#[test]
fn screen_color_flash_callable_keeps_its_target_id() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x127),
        ("MARY_FOMT_JP", 0x127),
        ("MARY_MFOMT_US", 0x12B),
        ("MARY_MFOMT_JP", 0x12B),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["FlashScreenColor"].0 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn global_farm_animal_and_shooting_star_callables_keep_target_ids() {
    for (target, cure_id, affection_id, shipping_id) in [
        ("MARY_FOMT_US", 0x129, 0x12A, 0x12B),
        ("MARY_FOMT_JP", 0x129, 0x12A, 0x12B),
        ("MARY_MFOMT_US", 0x12D, 0x12E, 0x12F),
        ("MARY_MFOMT_JP", 0x12D, 0x12E, 0x12F),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["CureAllSickLivestock"].0 .0, cure_id, "{target}");
        assert_eq!(
            map["AddAffectionToAllFarmAnimals"].0 .0, affection_id,
            "{target}"
        );
        assert_eq!(
            map["EnableShootingStarShippingBonus"].0 .0, shipping_id,
            "{target}"
        );
    }
}

#[test]
fn cooking_festival_rating_symbols_print_and_round_trip_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x120),
        ("MARY_FOMT_JP", 0x120),
        ("MARY_MFOMT_US", 0x124),
        ("MARY_MFOMT_JP", 0x124),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["GetCookingFestivalDishRating"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestDishRating, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestDishRating(void) { \
             VarSet(VAR_COOKING_FESTIVAL_PLAYER_DISH_RATING, GetCookingFestivalDishRating()); \
             if (VarGet(VAR_COOKING_FESTIVAL_PLAYER_DISH_RATING) == 1) { return; } \
             switch (GetCookingFestivalDishRating()) { \
             case 0: return; case 5: return; default: break; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestDishRating")
                .unwrap();
        let source = format_named_script("TestDishRating", &raised).unwrap();
        assert!(
            source.contains("case COOKING_FESTIVAL_DISH_RATING_EXCELLENT:"),
            "{target}: {source}"
        );
        assert!(
            source.contains("case COOKING_FESTIVAL_DISH_RATING_INELIGIBLE:"),
            "{target}: {source}"
        );
        assert!(
            source.contains(
                "VarGet(VAR_COOKING_FESTIVAL_PLAYER_DISH_RATING) == \
                 COOKING_FESTIVAL_DISH_RATING_GREAT"
            ),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn random_spouse_gift_callable_keeps_target_id_and_article_type() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x122),
        ("MARY_FOMT_JP", 0x122),
        ("MARY_MFOMT_US", 0x126),
        ("MARY_MFOMT_JP", 0x126),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["GetRandomSpouseGiftArticleId"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestSpouseGift, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestSpouseGift(void) { \
             SetPlayerHeldArticle(GetRandomSpouseGiftArticleId()); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn create_player_child_entity_keeps_target_id_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x12D),
        ("MARY_FOMT_JP", 0x12D),
        ("MARY_MFOMT_US", 0x131),
        ("MARY_MFOMT_JP", 0x131),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["CreatePlayerChildEntity"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn cycle_backward_to_non_cursed_tool_keeps_target_id_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x146),
        ("MARY_FOMT_JP", 0x146),
        ("MARY_MFOMT_US", 0x14A),
        ("MARY_MFOMT_JP", 0x14A),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["CycleBackwardToNonCursedTool"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
    }
}

#[test]
fn moon_viewing_partner_symbols_follow_the_selected_game_and_round_trip() {
    for (target, callable_id, first_symbol, last_symbol) in [
        (
            "MARY_FOMT_US",
            0x11F,
            "MOON_VIEWING_PARTNER_KAREN",
            "MOON_VIEWING_PARTNER_ELLI",
        ),
        (
            "MARY_FOMT_JP",
            0x11F,
            "MOON_VIEWING_PARTNER_KAREN",
            "MOON_VIEWING_PARTNER_ELLI",
        ),
        (
            "MARY_MFOMT_US",
            0x122,
            "MOON_VIEWING_PARTNER_RICK",
            "MOON_VIEWING_PARTNER_DOCTOR",
        ),
        (
            "MARY_MFOMT_JP",
            0x122,
            "MOON_VIEWING_PARTNER_RICK",
            "MOON_VIEWING_PARTNER_DOCTOR",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SelectMoonViewingPartner"]
                .0
                 .0,
            callable_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestMoonViewing, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestMoonViewing(void) { switch (SelectMoonViewingPartner()) { \
             case 0: return; case 4: return; default: break; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMoonViewing")
                .unwrap();
        let source = format_named_script("TestMoonViewing", &raised).unwrap();
        assert!(source.contains(first_symbol), "{target}: {source}");
        assert!(source.contains(last_symbol), "{target}: {source}");
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn mfomt_duplicate_moon_viewing_slot_preserves_its_distinct_id() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["SelectMoonViewingPartner"].0 .0, 0x122, "{target}");
        assert_eq!(map["SelectMoonViewingPartnerAlias"].0 .0, 0x123, "{target}");
        let script_table =
            parse_script_table("mary_script_table { TestMoonAlias, };\n", &options).unwrap();
        parse_named_scripts(
            "void TestMoonAlias(void) { int partner = SelectMoonViewingPartnerAlias(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn van_album_progress_callables_follow_all_four_target_tables_and_round_trip() {
    for (target, unlock_id, all_available_id) in [
        ("MARY_FOMT_US", 0x123, 0x124),
        ("MARY_FOMT_JP", 0x123, 0x124),
        ("MARY_MFOMT_US", 0x127, 0x128),
        ("MARY_MFOMT_JP", 0x127, 0x128),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["UnlockNextVanAlbum"].0 .0,
            unlock_id,
            "{target}"
        );
        assert_eq!(
            callables.scope.callable_map()["AreAllVanAlbumsAvailable"]
                .0
                 .0,
            all_available_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestVanAlbums, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestVanAlbums(void) { if (UnlockNextVanAlbum() && \
             AreAllVanAlbumsAvailable()) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestVanAlbums")
                .unwrap();
        let source = format_named_script("TestVanAlbums", &raised).unwrap();
        assert!(
            source.contains("UnlockNextVanAlbum()"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AreAllVanAlbumsAvailable()"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn record_player_album_callables_use_article_symbols_and_round_trip() {
    for (target, swap_id, remove_id) in [
        ("MARY_FOMT_US", 0x0B1, 0x0B2),
        ("MARY_FOMT_JP", 0x0B1, 0x0B2),
        ("MARY_MFOMT_US", 0x0B4, 0x0B5),
        ("MARY_MFOMT_JP", 0x0B4, 0x0B5),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SwapRecordPlayerAlbum"].0 .0,
            swap_id,
            "{target}"
        );
        assert_eq!(
            callables.scope.callable_map()["RemoveRecordPlayerAlbum"]
                .0
                 .0,
            remove_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestRecordPlayer, };\n", &options).unwrap();
        let empty_id = if target.contains("MFOMT") { 0x6A } else { 0x5F };
        let numeric = parse_named_scripts(
            &format!(
                "void TestRecordPlayer(void) {{ \
                 if (SwapRecordPlayerAlbum(64) == 64) {{ return; }} \
                 if (RemoveRecordPlayerAlbum() == {empty_id}) {{ return; }} }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestRecordPlayer")
                .unwrap();
        let source = format_named_script("TestRecordPlayer", &raised).unwrap();
        assert!(
            source.contains("SwapRecordPlayerAlbum(ARTICLE_ALBUM_1) == ARTICLE_ALBUM_1"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RemoveRecordPlayerAlbum() == ARTICLE_NONE"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn shipment_box_deposit_animation_follows_all_four_target_tables() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x0C0),
        ("MARY_FOMT_JP", 0x0C0),
        ("MARY_MFOMT_US", 0x0C3),
        ("MARY_MFOMT_JP", 0x0C3),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["StartShipmentBoxDepositAnimation"]
                .0
                 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestShipment, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestShipment(void) { StartShipmentBoxDepositAnimation(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestShipment").unwrap();
        let source = format_named_script("TestShipment", &raised).unwrap();
        assert!(
            source.contains("StartShipmentBoxDepositAnimation()"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn thomas_stocking_gift_callable_and_enum_follow_all_four_target_tables() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x121),
        ("MARY_FOMT_JP", 0x121),
        ("MARY_MFOMT_US", 0x125),
        ("MARY_MFOMT_JP", 0x125),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SelectThomasStockingGift"]
                .0
                 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestThomasGift, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestThomasGift(void) { \
             VarSet(VAR_THOMAS_STOCKING_GIFT_SELECTION, SelectThomasStockingGift()); \
             if (VarGet(VAR_THOMAS_STOCKING_GIFT_SELECTION) == THOMAS_STOCKING_GIFT_ALEXANDRITE) { \
                 VarSet(VAR_THOMAS_STOCKING_GIFT_SELECTION, THOMAS_STOCKING_GIFT_NONE); \
             } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestThomasGift")
                .unwrap();
        let source = format_named_script("TestThomasGift", &raised).unwrap();
        assert!(
            source.contains("SelectThomasStockingGift()")
                && source.contains("THOMAS_STOCKING_GIFT_ALEXANDRITE")
                && source.contains("THOMAS_STOCKING_GIFT_NONE"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn berry_acquisition_callables_follow_all_four_target_tables_and_round_trip() {
    for (target, power_id, mystic_id) in [
        ("MARY_FOMT_US", 0x061, 0x062),
        ("MARY_FOMT_JP", 0x061, 0x062),
        ("MARY_MFOMT_US", 0x062, 0x063),
        ("MARY_MFOMT_JP", 0x062, 0x063),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["ObtainPowerBerry"].0 .0,
            power_id,
            "{target}"
        );
        assert_eq!(
            callables.scope.callable_map()["ObtainMysticBerry"].0 .0,
            mystic_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestBerries, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestBerries(void) { ObtainPowerBerry(); ObtainMysticBerry(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestBerries").unwrap();
        let source = format_named_script("TestBerries", &raised).unwrap();
        assert!(source.contains("ObtainPowerBerry();"), "{target}: {source}");
        assert!(
            source.contains("ObtainMysticBerry();"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn preserved_overnight_location_callables_follow_all_four_target_tables() {
    for (target, preserve_id, clear_id, get_map_id, map_none) in [
        ("MARY_FOMT_US", 0x066, 0x067, 0x068, 0x0234),
        ("MARY_FOMT_JP", 0x066, 0x067, 0x068, 0x0234),
        ("MARY_MFOMT_US", 0x067, 0x068, 0x069, 0x023A),
        ("MARY_MFOMT_JP", 0x067, 0x068, 0x069, 0x023A),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["PreservePlayerLocationForNextDay"].0 .0, preserve_id);
        assert_eq!(map["ClearPreservedPlayerLocation"].0 .0, clear_id);
        assert_eq!(map["GetPreservedPlayerMapId"].0 .0, get_map_id);
        let map_type = constants.user_type("MaryMapId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(map_type, map_none),
            Some("MAP_NONE"),
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestOvernight, };\n", &options).unwrap();
        let numeric_source = format!(
            "void TestOvernight(void) {{ PreservePlayerLocationForNextDay(); \
             if (GetPreservedPlayerMapId() == {map_none}) {{ \
             ClearPreservedPlayerLocation(); }} }}\n"
        );
        let parsed =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestOvernight")
                .unwrap();
        let source = format_named_script("TestOvernight", &raised).unwrap();
        assert!(
            source.contains("PreservePlayerLocationForNextDay();"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GetPreservedPlayerMapId() == MAP_NONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ClearPreservedPlayerLocation();"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn entity_location_and_fireplace_callables_share_the_map_id_domain() {
    for (target, get_location_id, light_id, is_lit_id) in [
        ("MARY_FOMT_US", 0x014, 0x0B3, 0x0B4),
        ("MARY_FOMT_JP", 0x014, 0x0B3, 0x0B4),
        ("MARY_MFOMT_US", 0x014, 0x0B6, 0x0B7),
        ("MARY_MFOMT_JP", 0x014, 0x0B6, 0x0B7),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["GetEntityLocation"].0 .0, get_location_id, "{target}");
        assert_eq!(map["LightFireplaceAtLocation"].0 .0, light_id, "{target}");
        assert_eq!(map["IsFireplaceLitAtLocation"].0 .0, is_lit_id, "{target}");

        let script_table =
            parse_script_table("mary_script_table { TestFireplace, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestFireplace(void) { if (GetEntityLocation(0) == MAP_MOUNTAIN_COTTAGE) { \
             LightFireplaceAtLocation(MAP_MOUNTAIN_COTTAGE); } \
             if (IsFireplaceLitAtLocation(MAP_TOWN_COTTAGE)) { return; } }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestFireplace")
                .unwrap();
        let source = format_named_script("TestFireplace", &raised).unwrap();
        assert!(
            source.contains("GetEntityLocation(ENTITY_PLAYER) == MAP_MOUNTAIN_COTTAGE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("LightFireplaceAtLocation(MAP_MOUNTAIN_COTTAGE)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("IsFireplaceLitAtLocation(MAP_TOWN_COTTAGE)"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn player_scripted_animation_callables_follow_all_four_target_tables() {
    for (target, eat_id, prepare_id, restore_id) in [
        ("MARY_FOMT_US", 0x069, 0x06A, 0x06B),
        ("MARY_FOMT_JP", 0x069, 0x06A, 0x06B),
        ("MARY_MFOMT_US", 0x06A, 0x06B, 0x06C),
        ("MARY_MFOMT_JP", 0x06A, 0x06B, 0x06C),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["EatRandomMeal"].0 .0, eat_id);
        assert_eq!(map["PreparePlayerForScriptedAnimation"].0 .0, prepare_id);
        assert_eq!(map["RestorePlayerAfterScriptedAnimation"].0 .0, restore_id);

        let script_table =
            parse_script_table("mary_script_table { TestPlayerState, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestPlayerState(void) { PreparePlayerForScriptedAnimation(); \
             EatRandomMeal(); RestorePlayerAfterScriptedAnimation(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestPlayerState")
                .unwrap();
        let source = format_named_script("TestPlayerState", &raised).unwrap();
        assert!(
            source.contains("PreparePlayerForScriptedAnimation();"),
            "{target}: {source}"
        );
        assert!(source.contains("EatRandomMeal();"), "{target}: {source}");
        assert!(
            source.contains("RestorePlayerAfterScriptedAnimation();"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn new_day_map_rebuild_callable_follows_all_four_target_tables() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x128),
        ("MARY_FOMT_JP", 0x128),
        ("MARY_MFOMT_US", 0x12C),
        ("MARY_MFOMT_JP", 0x12C),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["RebuildMapEntitiesForNewDay"]
                .0
                 .0,
            callable_id
        );

        let script_table =
            parse_script_table("mary_script_table { TestNewDay, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestNewDay(void) { RebuildMapEntitiesForNewDay(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestNewDay").unwrap();
        let source = format_named_script("TestNewDay", &raised).unwrap();
        assert!(
            source.contains("RebuildMapEntitiesForNewDay();"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn hot_spring_state_callables_follow_all_four_target_tables() {
    for (target, enter_id, exit_id) in [
        ("MARY_FOMT_US", 0x064, 0x065),
        ("MARY_FOMT_JP", 0x064, 0x065),
        ("MARY_MFOMT_US", 0x065, 0x066),
        ("MARY_MFOMT_JP", 0x065, 0x066),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["EnterHotSpringBathingState"].0 .0, enter_id);
        assert_eq!(map["ExitHotSpringBathingState"].0 .0, exit_id);

        let script_table =
            parse_script_table("mary_script_table { TestHotSpring, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestHotSpring(void) { EnterHotSpringBathingState(); \
             ExitHotSpringBathingState(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestHotSpring")
                .unwrap();
        let source = format_named_script("TestHotSpring", &raised).unwrap();
        assert!(
            source.contains("EnterHotSpringBathingState();"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ExitHotSpringBathingState();"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn sunrise_and_star_effect_callables_follow_all_four_target_tables() {
    for (target, create_id, play_id, destroy_id, sparkle_id) in [
        ("MARY_FOMT_US", 0x12F, 0x130, 0x131, 0x132),
        ("MARY_FOMT_JP", 0x12F, 0x130, 0x131, 0x132),
        ("MARY_MFOMT_US", 0x133, 0x134, 0x135, 0x136),
        ("MARY_MFOMT_JP", 0x133, 0x134, 0x135, 0x136),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let map = callables.scope.callable_map();
        assert_eq!(map["CreateNewYearSunriseEffect"].0 .0, create_id);
        assert_eq!(map["PlayNewYearSunriseEffect"].0 .0, play_id);
        assert_eq!(map["DestroyNewYearSunriseEffect"].0 .0, destroy_id);
        assert_eq!(map["PlayStarSparkleEffect"].0 .0, sparkle_id);

        let script_table =
            parse_script_table("mary_script_table { TestSunrise, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestSunrise(void) { CreateNewYearSunriseEffect(); \
             PlayNewYearSunriseEffect(); DestroyNewYearSunriseEffect(); \
             PlayStarSparkleEffect(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestSunrise").unwrap();
        let source = format_named_script("TestSunrise", &raised).unwrap();
        for name in [
            "CreateNewYearSunriseEffect();",
            "PlayNewYearSunriseEffect();",
            "DestroyNewYearSunriseEffect();",
            "PlayStarSparkleEffect();",
        ] {
            assert!(source.contains(name), "{target}: {source}");
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn cross_map_entity_relocation_uses_map_symbols_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x12E),
        ("MARY_FOMT_JP", 0x12E),
        ("MARY_MFOMT_US", 0x132),
        ("MARY_MFOMT_JP", 0x132),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["RelocateEntityToMap"].0 .0,
            callable_id
        );

        let script_table =
            parse_script_table("mary_script_table { TestRelocate, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestRelocate(void) { RelocateEntityToMap(0, MAP_NONE, 0, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestRelocate").unwrap();
        let source = format_named_script("TestRelocate", &raised).unwrap();
        assert!(
            source.contains("RelocateEntityToMap(ENTITY_PLAYER, MAP_NONE, X(0), Y(0));"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn mine_floor_generator_accepts_symbolic_mine_kinds_for_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x134),
        ("MARY_FOMT_JP", 0x134),
        ("MARY_MFOMT_US", 0x138),
        ("MARY_MFOMT_JP", 0x138),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["GenerateMineFloorLayout"]
                .0
                 .0,
            callable_id
        );

        let script_table =
            parse_script_table("mary_script_table { TestMine, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestMine(void) { GenerateMineFloorLayout(MINE_SPRING, 58); \
             GenerateMineFloorLayout(MINE_LAKE, 8); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestMine").unwrap();
        let source = format_named_script("TestMine", &raised).unwrap();
        assert!(
            source.contains("GenerateMineFloorLayout(MINE_SPRING, 58);"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GenerateMineFloorLayout(MINE_LAKE, 8);"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn held_actor_graphic_callable_preserves_target_specific_animation_ids() {
    for (target, callable_id, animation_id) in [
        ("MARY_FOMT_US", 0x142, 1844),
        ("MARY_FOMT_JP", 0x142, 1844),
        ("MARY_MFOMT_US", 0x146, 1892),
        ("MARY_MFOMT_JP", 0x146, 1892),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["BeginHoldingActorGraphic"]
                .0
                 .0,
            callable_id
        );

        let script_table =
            parse_script_table("mary_script_table { TestHoldActor, };\n", &options).unwrap();
        let input =
            format!("void TestHoldActor(void) {{ BeginHoldingActorGraphic({animation_id}); }}\n");
        let parsed =
            parse_named_scripts(&input, &options, &callables.scope, &script_table).unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestHoldActor")
                .unwrap();
        let source = format_named_script("TestHoldActor", &raised).unwrap();
        assert!(
            source.contains("BeginHoldingActorGraphic(ANIMATION_CHICKEN_HELD);"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        let legacy = parse_named_scripts(
            &format!(
                "void TestHoldActor(void) {{ BeginHoldingActorGraphic(ANIMATION_ID_{animation_id:04}); }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy alias"
        );
    }
}

#[test]
fn player_actor_update_suspension_callable_follows_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x076),
        ("MARY_FOMT_JP", 0x076),
        ("MARY_MFOMT_US", 0x077),
        ("MARY_MFOMT_JP", 0x077),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SetPlayerActorUpdateSuspended"]
                .0
                 .0,
            callable_id
        );

        let script_table =
            parse_script_table("mary_script_table { TestSuspend, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestSuspend(void) { SetPlayerActorUpdateSuspended(TRUE); \
             SetPlayerActorUpdateSuspended(FALSE); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestSuspend").unwrap();
        let source = format_named_script("TestSuspend", &raised).unwrap();
        assert!(
            source.contains("SetPlayerActorUpdateSuspended(TRUE);"),
            "{target}: {source}"
        );
        assert!(
            source.contains("SetPlayerActorUpdateSuspended(FALSE);"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn fixed_field_width_text_number_callable_follows_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x039),
        ("MARY_FOMT_JP", 0x039),
        ("MARY_MFOMT_US", 0x03A),
        ("MARY_MFOMT_JP", 0x03A),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SetTextVariableNumberFieldWidth"]
                .0
                 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestNumberWidth, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestNumberWidth(void) { SetTextVariableNumberFieldWidth(0, 7, 2); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestNumberWidth")
                .unwrap();
        let source = format_named_script("TestNumberWidth", &raised).unwrap();
        assert!(
            source.contains("SetTextVariableNumberFieldWidth(TEXT_VARIABLE_1, 7, 2);"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn relationship_numeric_boundaries_are_not_folded_or_truncated() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let table =
            parse_script_table("mary_script_table { TestRelationshipBoundary, };", &options)
                .unwrap();
        // The source uses an invalid character as well: round-tripping is not
        // permission to execute it, nor an excuse to optimize away the call.
        let values = [-2147483648i32, -1, 0, 255, 256, 65535, 65536, 2147483647];
        let mut calls = Vec::new();
        for callable in [
            "AddNpcFriendship",
            "SetNpcFriendship",
            "AddCharacterLove",
            "SetCharacterLove",
        ] {
            for value in values {
                calls.push(format!("{callable}(CHARACTER_KAREN, {value});"));
            }
            calls.push(format!("{callable}(65536, -1);"));
        }
        let parsed = parse_named_scripts(
            &format!(
                "void TestRelationshipBoundary(void) {{ {} }}",
                calls.join("\n")
            ),
            &options,
            &callables.scope,
            &table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &parsed.scripts[0].2,
            &callables.scope,
            "TestRelationshipBoundary",
        )
        .unwrap();
        let source = format_named_script("TestRelationshipBoundary", &raised).unwrap();
        for call in calls {
            assert!(source.contains(&call), "{target}: missing {call}: {source}");
        }
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn random_range_operands_preserve_native_boundaries() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let table =
            parse_script_table("mary_script_table { TestRandomBoundary, };", &options).unwrap();
        // Compilation is lossless, not execution or parameter sanitization:
        // retain inverted and overflowing ranges too, without folding calls
        // or adding the mask belonging only to RandomU15.
        let calls = [
            "RandomIntInclusive(-10, 10)",
            "RandomIntInclusive(0, 65535)",
            "RandomIntInclusive(7, 7)",
            "RandomIntInclusive(10, -10)",
            "RandomIntInclusive(0, 2147483647)",
            "RandomIntInclusive(-2147483648, 2147483647)",
            "RandomU15()",
        ];
        let body = calls
            .iter()
            .map(|call| format!("VarSet(0, {call});"))
            .collect::<Vec<_>>()
            .join("\n");
        let parsed = parse_named_scripts(
            &format!("void TestRandomBoundary(void) {{ {body} }}"),
            &options,
            &callables.scope,
            &table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestRandomBoundary")
                .unwrap();
        let source = format_named_script("TestRandomBoundary", &raised).unwrap();
        for call in calls {
            assert!(source.contains(call), "{target}: missing {call}: {source}");
        }
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn audio_out_of_domain_values_are_not_normalized() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let table =
            parse_script_table("mary_script_table { TestAudioBoundary, };", &options).unwrap();
        let parsed = parse_named_scripts("void TestAudioBoundary(void) { PlaySong(3, 65536); PlaySong(-1, -1); PlaySong(65536, 211); }", &options, &callables.scope, &table).unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestAudioBoundary")
                .unwrap();
        let source = format_named_script("TestAudioBoundary", &raised).unwrap();
        for expected in [
            "PlaySong(3, 65536);",
            "PlaySong(-1, -1);",
            "PlaySong(65536, 211);",
        ] {
            assert!(source.contains(expected), "{target}: {source}");
        }
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
        let reused = parse_named_scripts(
            "void TestAudioBoundary(void) { int value; if (IsPlayerHoldingNothing()) { value = 1; } else { value = 2; } WaitFrames(1); PlaySong(value, 6); value = 6; WaitFrames(1); PlaySong(0, value); value = 65536; PlaySong(0, value); }",
            &options, &callables.scope, &table,
        ).unwrap();
        let raised =
            decompile_script_named(&reused.scripts[0].2, &callables.scope, "TestAudioBoundary")
                .unwrap();
        let source = format_named_script("TestAudioBoundary", &raised).unwrap();
        for expected in [
            "= AUDIO_START_WEAK;",
            "= AUDIO_START_OR_CONTINUE;",
            "= AUDIO_BGM_WEDDING;",
            "= 65536;",
        ] {
            assert!(source.contains(expected), "{target}: {source}");
        }
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&reused.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
        let conflicted = parse_named_scripts(
            "void TestAudioBoundary(void) { int value; if (IsPlayerHoldingNothing()) { value = 1; } else { value = 2; } WaitFrames(1); PlaySong(value, value); value = 1; PlaySong(value, 6); }",
            &options, &callables.scope, &table,
        ).unwrap();
        let raised = decompile_script_named(
            &conflicted.scripts[0].2,
            &callables.scope,
            "TestAudioBoundary",
        )
        .unwrap();
        let source = format_named_script("TestAudioBoundary", &raised).unwrap();
        // Both incoming definitions have conflicting uses. The later fresh
        // definition has only mode uses and must regain its precise domain.
        for expected in ["= 1;", "= 2;", "= AUDIO_START_WEAK;"] {
            assert!(source.contains(expected), "{target}: {source}");
        }
        assert!(
            !source.contains("= AUDIO_START_OR_CONTINUE;"),
            "{target}: {source}"
        );
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&conflicted.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
        let switched = parse_named_scripts(
            "void TestAudioBoundary(void) { int mode; switch (GetEntityFacing(0)) { case 0: mode = 0; break; case 1: if (IsPlayerHoldingNothing()) { mode = 1; } else { mode = 2; } break; default: mode = 3; break; } WaitFrames(1); switch (mode) { case 1: WaitFrames(2); break; case 2: WaitFrames(3); break; default: break; } PlaySong(mode, 6); }",
            &options, &callables.scope, &table,
        ).unwrap();
        let raised = decompile_script_named(
            &switched.scripts[0].2,
            &callables.scope,
            "TestAudioBoundary",
        )
        .unwrap();
        let source = format_named_script("TestAudioBoundary", &raised).unwrap();
        for expected in [
            "= AUDIO_START;",
            "= AUDIO_START_WEAK;",
            "= AUDIO_START_OR_CONTINUE;",
            "= 3;",
            "case AUDIO_START_WEAK:",
            "case AUDIO_START_OR_CONTINUE:",
        ] {
            assert!(source.contains(expected), "{target}: {source}");
        }
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&switched.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn relative_entity_position_callable_follows_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["OffsetEntityPosition"].0 .0,
            0x015,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestOffset, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestOffset(void) { OffsetEntityPosition(36, -8, 16); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestOffset").unwrap();
        let source = format_named_script("TestOffset", &raised).unwrap();
        assert!(
            source.contains("OffsetEntityPosition(ENTITY_STAID, X(-8), Y(16));"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
        let staged = parse_named_scripts(
            "void TestOffset(void) { int delta; if (GetEntityFacing(0) == 0) { delta = -8; } else { delta = -16; } WaitFrames(1); OffsetEntityPosition(36, delta, 0); delta = 16; OffsetEntityPosition(36, 0, delta); }",
            &options, &callables.scope, &script_table,
        ).unwrap();
        let raised =
            decompile_script_named(&staged.scripts[0].2, &callables.scope, "TestOffset").unwrap();
        let source = format_named_script("TestOffset", &raised).unwrap();
        for expected in ["X(-8)", "X(-16)", "Y(16)"] {
            assert!(
                source.contains(expected),
                "{target}: missing {expected} after branch/reassignment:\n{source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&staged.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: delayed axis reuse"
        );
        let conflicted = parse_named_scripts(
            "void TestOffset(void) { int delta; delta = 16; OffsetEntityPosition(36, delta, delta); }",
            &options, &callables.scope, &script_table,
        ).unwrap();
        let raised =
            decompile_script_named(&conflicted.scripts[0].2, &callables.scope, "TestOffset")
                .unwrap();
        let source = format_named_script("TestOffset", &raised).unwrap();
        assert!(!source.contains("X(16)") && !source.contains("Y(16)"),
            "{target}: one definition shared by both axes must not acquire a guessed axis:\n{source}");
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&conflicted.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: conflicting axis uses"
        );
    }
}

#[test]
fn stamina_and_fatigue_callable_follows_target_physical_slots() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x05B),
        ("MARY_FOMT_JP", 0x05B),
        ("MARY_MFOMT_US", 0x05C),
        ("MARY_MFOMT_JP", 0x05C),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["ChangePlayerStaminaAndFatigue"]
                .0
                 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestRecovery, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestRecovery(void) { ChangePlayerStaminaAndFatigue(50, -20); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestRecovery").unwrap();
        let source = format_named_script("TestRecovery", &raised).unwrap();
        assert!(
            source.contains("ChangePlayerStaminaAndFatigue(50, -20);"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn farm_horse_lifecycle_callables_follow_all_targets() {
    for (target, create_id, remove_id) in [
        ("MARY_FOMT_US", 0x100, 0x101),
        ("MARY_FOMT_JP", 0x100, 0x101),
        ("MARY_MFOMT_US", 0x103, 0x104),
        ("MARY_MFOMT_JP", 0x103, 0x104),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["CreateFarmHorse"].0 .0,
            create_id,
            "{target}"
        );
        assert_eq!(
            callables.scope.callable_map()["RemoveFarmHorse"].0 .0,
            remove_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestHorse, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestHorse(void) { CreateFarmHorse(0, 1, 2, 196, 145); CreateFarmHorse(1, 1, 2, 196, 145); CreateFarmHorse(2, 1, 2, 196, 145); RemoveFarmHorse(0, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestHorse").unwrap();
        let source = format_named_script("TestHorse", &raised).unwrap();
        assert!(
            source.contains("HORSE_AGE_STAGE_ADULT"),
            "{target}: {source}"
        );
        assert!(source.contains("MAP_FARM"), "{target}: {source}");
        assert!(
            source.contains("CreateFarmHorse(FALSE,"),
            "{target}: {source}"
        );
        assert!(
            source.contains("CreateFarmHorse(TRUE,"),
            "{target}: {source}"
        );
        assert!(source.contains("CreateFarmHorse(2,"), "{target}: {source}");
        assert!(
            !source.contains("CreateFarmHorse(FACING_"),
            "{target}: {source}"
        );
        assert!(
            source.contains("RemoveFarmHorse(FALSE, 0)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn seaside_cottage_callable_follows_all_targets() {
    for (target, callable_id) in [
        ("MARY_FOMT_US", 0x0C5),
        ("MARY_FOMT_JP", 0x0C5),
        ("MARY_MFOMT_US", 0x0C8),
        ("MARY_MFOMT_JP", 0x0C8),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["BuildSeasideCottage"].0 .0,
            callable_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestCottage, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestCottage(void) { BuildSeasideCottage(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestCottage").unwrap();
        let source = format_named_script("TestCottage", &raised).unwrap();
        assert!(
            source.contains("BuildSeasideCottage();"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn complete_animation_table_preserves_every_physical_slot() {
    for (target, last_id) in [
        ("MARY_FOMT_US", 2551),
        ("MARY_FOMT_JP", 2551),
        ("MARY_MFOMT_US", 2635),
        ("MARY_MFOMT_JP", 2635),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestAnimationIds, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestAnimationIds(void) {{ SetEntityAnim(70, {last_id}); \
                 BeginHoldingActorGraphic({last_id}); }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAnimationIds")
                .unwrap();
        let source = format_named_script("TestAnimationIds", &raised).unwrap();
        let symbol = format!("ANIMATION_ID_{last_id:04}");
        assert_eq!(source.matches(&symbol).count(), 2, "{target}: {source}");
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        let out_of_range_symbol = format!("ANIMATION_ID_{:04}", last_id + 1);
        let error = parse_named_scripts(
            &format!(
                "void TestAnimationIds(void) {{ SetEntityAnim(70, {out_of_range_symbol}); }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains(&out_of_range_symbol),
            "{target}: {error}"
        );
    }
}

#[test]
fn player_fishing_rod_animation_group_is_canonical_on_all_targets() {
    let animations = [
        (82, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_1"),
        (86, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_2"),
        (90, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_3"),
        (94, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_4"),
        (98, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_5"),
        (102, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_6"),
        (106, "ANIMATION_PLAYER_FISHING_ROD_CHARGE_STAGE_7"),
        (110, "ANIMATION_PLAYER_FISHING_ROD_CATCH"),
        (114, "ANIMATION_PLAYER_FISHING_ROD_COLLECT_FISH"),
        (118, "ANIMATION_PLAYER_FISHING_ROD_WAITING"),
    ];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFishingAnimations, };\n", &options)
                .unwrap();
        let animation_type = constants.user_type("MaryAnimationId").unwrap();

        for (id, symbol) in animations {
            assert_eq!(
                constants.typed_int_const_name(animation_type, id),
                Some(symbol),
                "{target}: animation {id} canonical name"
            );

            let numeric = parse_named_scripts(
                &format!(
                    "void TestFishingAnimations(void) {{ SetEntityAnim(ENTITY_PLAYER, {id}); }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                "TestFishingAnimations",
            )
            .unwrap();
            let source = format_named_script("TestFishingAnimations", &raised).unwrap();
            assert!(source.contains(symbol), "{target}: {source}");
            let rebuilt =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&rebuilt.scripts[0].2),
                "{target}: {symbol}"
            );

            let legacy = parse_named_scripts(
                &format!(
                    "void TestFishingAnimations(void) {{ SetEntityAnim(ENTITY_PLAYER, ANIMATION_ID_{id:04}); }}\n"
                ),
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&legacy.scripts[0].2),
                "{target}: legacy animation alias {id}"
            );
        }
    }
}

#[test]
fn proven_player_hold_state_animations_are_canonical_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestAnimations, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestAnimations(void) { SetEntityAnim(0, 326); SetEntityAnim(0, 402); SetEntityAnim(0, 338); SetEntityAnim(0, 454); SetEntityAnim(0, 342); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAnimations")
                .unwrap();
        let source = format_named_script("TestAnimations", &raised).unwrap();
        for symbol in [
            "ANIMATION_PLAYER_PREPARE_TO_HOLD_ITEM",
            "ANIMATION_PLAYER_IDLE_EMPTY_HANDED",
            "ANIMATION_PLAYER_IDLE_HOLDING_ITEM",
            "ANIMATION_PLAYER_WALK_EMPTY_HANDED",
            "ANIMATION_PLAYER_WALK_HOLDING_ITEM",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        let legacy = parse_named_scripts(
            "void TestAnimations(void) { SetEntityAnim(0, ANIMATION_ID_0326); SetEntityAnim(0, ANIMATION_ID_0402); SetEntityAnim(0, ANIMATION_ID_0338); SetEntityAnim(0, ANIMATION_ID_0454); SetEntityAnim(0, ANIMATION_ID_0342); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy aliases"
        );
    }
}

#[test]
fn rick_idle_walk_animations_are_shared_by_all_targets() {
    let values = [531, 535];
    let symbols = ["ANIMATION_RICK_IDLE", "ANIMATION_RICK_WALK"];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestNpcAnimations, };\n", &options).unwrap();
        let numeric_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, {value});"))
            .collect::<String>();
        let numeric_source = format!("void TestNpcAnimations(void) {{ {numeric_body} }}\n");
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestNpcAnimations")
                .unwrap();
        let source = format_named_script("TestNpcAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy_source = format!("void TestNpcAnimations(void) {{ {legacy_body} }}\n");
        let legacy =
            parse_named_scripts(&legacy_source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy aliases"
        );
    }
}

#[test]
fn child_growth_animation_slots_follow_each_game_family() {
    for (target, values) in [
        ("MARY_FOMT_US", [8, 12, 615, 619, 623, 631]),
        ("MARY_FOMT_JP", [8, 12, 615, 619, 623, 631]),
        ("MARY_MFOMT_US", [8, 12, 627, 631, 635, 643]),
        ("MARY_MFOMT_JP", [8, 12, 627, 631, 635, 643]),
    ] {
        let symbols = [
            "ANIMATION_CHILD_NEWBORN",
            "ANIMATION_CHILD_INFANT_SLEEPING",
            "ANIMATION_CHILD_WALKING_IDLE",
            "ANIMATION_CHILD_PRE_WALKING_IDLE",
            "ANIMATION_CHILD_FIRST_STEPS_WALK",
            "ANIMATION_CHILD_SLEEPING",
        ];
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestChildAnimations, };\n", &options).unwrap();
        let body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_CHILD, {value});"))
            .collect::<String>();
        let parsed = parse_named_scripts(
            &format!("void TestChildAnimations(void) {{ {body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &parsed.scripts[0].2,
            &callables.scope,
            "TestChildAnimations",
        )
        .unwrap();
        let source = format_named_script("TestChildAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_CHILD, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy = parse_named_scripts(
            &format!("void TestChildAnimations(void) {{ {legacy_body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy aliases"
        );
    }
}

#[test]
fn chicken_idle_walk_animations_follow_each_game_family() {
    for (target, values) in [
        ("MARY_FOMT_US", [1820, 1824]),
        ("MARY_FOMT_JP", [1820, 1824]),
        ("MARY_MFOMT_US", [1868, 1872]),
        ("MARY_MFOMT_JP", [1868, 1872]),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestChickenAnimations, };\n", &options)
                .unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestChickenAnimations(void) {{ SetEntityAnim(ENTITY_70, {}); SetEntityAnim(ENTITY_70, {}); }}\n",
                values[0], values[1]
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestChickenAnimations",
        )
        .unwrap();
        let source = format_named_script("TestChickenAnimations", &raised).unwrap();
        assert!(
            source.contains("ANIMATION_CHICKEN_IDLE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ANIMATION_CHICKEN_WALK"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );

        let legacy = parse_named_scripts(
            &format!(
                "void TestChickenAnimations(void) {{ SetEntityAnim(ENTITY_70, ANIMATION_ID_{:04}); SetEntityAnim(ENTITY_70, ANIMATION_ID_{:04}); }}\n",
                values[0], values[1]
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2)
        );
    }
}

#[test]
fn farm_dog_idle_walk_animations_follow_each_game_family() {
    for (target, values, sick_symbol) in [
        ("MARY_FOMT_US", [824, 828, 942], None),
        ("MARY_FOMT_JP", [824, 828, 942], None),
        (
            "MARY_MFOMT_US",
            [860, 864, 978],
            Some("ANIMATION_FARM_DOG_SICK_IDLE"),
        ),
        (
            "MARY_MFOMT_JP",
            [860, 864, 978],
            Some("ANIMATION_FARM_DOG_SICK_IDLE"),
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestDogAnimations, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestDogAnimations(void) {{ SetEntityAnim(ENTITY_70, {}); SetEntityAnim(ENTITY_70, {}); SetEntityAnim(ENTITY_70, {}); }}\n",
                values[0], values[1], values[2]
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestDogAnimations")
                .unwrap();
        let source = format_named_script("TestDogAnimations", &raised).unwrap();
        assert!(
            source.contains("ANIMATION_FARM_DOG_IDLE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ANIMATION_FARM_DOG_WALK"),
            "{target}: {source}"
        );
        assert!(
            source.contains("ANIMATION_FARM_DOG_YOUNG_IDLE"),
            "{target}: {source}"
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_FARM_DOG_SICK_IDLE"),
            sick_symbol.map(|_| 892),
            "{target}: sickness slot must remain target-local"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );

        let legacy = parse_named_scripts(
            &format!(
                "void TestDogAnimations(void) {{ SetEntityAnim(ENTITY_70, ANIMATION_ID_{:04}); SetEntityAnim(ENTITY_70, ANIMATION_ID_{:04}); SetEntityAnim(ENTITY_70, ANIMATION_ID_{:04}); }}\n",
                values[0], values[1], values[2]
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2)
        );
    }
}

#[test]
fn cliff_hospital_and_collapse_animations_follow_each_game_family() {
    for (target, values) in [
        ("MARY_FOMT_US", [659, 663, 667]),
        ("MARY_FOMT_JP", [659, 663, 667]),
        ("MARY_MFOMT_US", [671, 675, 679]),
        ("MARY_MFOMT_JP", [671, 675, 679]),
    ] {
        let symbols = [
            "ANIMATION_CLIFF_HOSPITAL_BED_IDLE",
            "ANIMATION_CLIFF_HOSPITAL_BED_REACT",
            "ANIMATION_CLIFF_COLLAPSE_FORWARD",
        ];
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestCliffAnimations, };\n", &options).unwrap();
        let body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_CLIFF, {value});"))
            .collect::<String>();
        let numeric = parse_named_scripts(
            &format!("void TestCliffAnimations(void) {{ {body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestCliffAnimations",
        )
        .unwrap();
        let source = format_named_script("TestCliffAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );
    }
}

#[test]
fn cow_and_sheep_idle_walk_animations_follow_each_game_family() {
    for (target, values) in [
        ("MARY_FOMT_US", [684, 688, 2333, 2337]),
        ("MARY_FOMT_JP", [684, 688, 2333, 2337]),
        ("MARY_MFOMT_US", [708, 712, 2405, 2409]),
        ("MARY_MFOMT_JP", [708, 712, 2405, 2409]),
    ] {
        let symbols = [
            "ANIMATION_COW_IDLE",
            "ANIMATION_COW_WALK",
            "ANIMATION_SHEEP_IDLE",
            "ANIMATION_SHEEP_WALK",
        ];
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table = parse_script_table(
            "mary_script_table { TestLivestockAnimations, };\n",
            &options,
        )
        .unwrap();
        let body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_70, {value});"))
            .collect::<String>();
        let numeric = parse_named_scripts(
            &format!("void TestLivestockAnimations(void) {{ {body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestLivestockAnimations",
        )
        .unwrap();
        let source = format_named_script("TestLivestockAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );

        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_70, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy = parse_named_scripts(
            &format!("void TestLivestockAnimations(void) {{ {legacy_body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2)
        );
    }
}

#[test]
fn young_livestock_idle_animations_follow_each_game_family() {
    for (target, values) in [
        ("MARY_FOMT_US", [1845, 764, 2413]),
        ("MARY_FOMT_JP", [1845, 764, 2413]),
        ("MARY_MFOMT_US", [1893, 788, 2485]),
        ("MARY_MFOMT_JP", [1893, 788, 2485]),
    ] {
        let symbols = [
            "ANIMATION_CHICK_IDLE",
            "ANIMATION_CALF_IDLE",
            "ANIMATION_LAMB_IDLE",
        ];
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table = parse_script_table(
            "mary_script_table { TestYoungAnimalAnimations, };\n",
            &options,
        )
        .unwrap();
        let body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_97, {value});"))
            .collect::<String>();
        let numeric = parse_named_scripts(
            &format!("void TestYoungAnimalAnimations(void) {{ {body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestYoungAnimalAnimations",
        )
        .unwrap();
        let source = format_named_script("TestYoungAnimalAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );

        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(ENTITY_97, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy = parse_named_scripts(
            &format!("void TestYoungAnimalAnimations(void) {{ {legacy_body} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2)
        );
    }
}

#[test]
fn foal_idle_walk_animations_follow_each_game_family() {
    for (target, values) in [
        ("MARY_FOMT_US", [1946, 1950]),
        ("MARY_FOMT_JP", [1946, 1950]),
        ("MARY_MFOMT_US", [1994, 1998]),
        ("MARY_MFOMT_JP", [1994, 1998]),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFoalAnimations, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestFoalAnimations(void) {{ SetEntityAnim(ENTITY_FARM_HORSE, {}); SetEntityAnim(ENTITY_FARM_HORSE, {}); }}\n",
                values[0], values[1]
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestFoalAnimations",
        )
        .unwrap();
        let source = format_named_script("TestFoalAnimations", &raised).unwrap();
        assert!(source.contains("ANIMATION_FOAL_IDLE"), "{target}: {source}");
        assert!(source.contains("ANIMATION_FOAL_WALK"), "{target}: {source}");
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );

        let legacy = parse_named_scripts(
            &format!(
                "void TestFoalAnimations(void) {{ SetEntityAnim(ENTITY_FARM_HORSE, ANIMATION_ID_{:04}); SetEntityAnim(ENTITY_FARM_HORSE, ANIMATION_ID_{:04}); }}\n",
                values[0], values[1]
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2)
        );
    }
}

#[test]
fn sheared_sheep_idle_animation_follows_each_game_family() {
    for (target, value) in [
        ("MARY_FOMT_US", 2361),
        ("MARY_FOMT_JP", 2361),
        ("MARY_MFOMT_US", 2433),
        ("MARY_MFOMT_JP", 2433),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table = parse_script_table(
            "mary_script_table { TestShearedSheepAnimation, };\n",
            &options,
        )
        .unwrap();
        let numeric = parse_named_scripts(
            &format!(
                "void TestShearedSheepAnimation(void) {{ SetEntityAnim(ENTITY_96, {value}); }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestShearedSheepAnimation",
        )
        .unwrap();
        let source = format_named_script("TestShearedSheepAnimation", &raised).unwrap();
        assert!(
            source.contains("ANIMATION_SHEEP_SHEARED_IDLE"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );

        let legacy = parse_named_scripts(
            &format!(
                "void TestShearedSheepAnimation(void) {{ SetEntityAnim(ENTITY_96, ANIMATION_ID_{value:04}); }}\n"
            ),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2)
        );
    }
}

#[test]
fn gender_specific_npc_animation_pairs_follow_each_physical_table() {
    let symbols = [
        "ANIMATION_POPURI_IDLE",
        "ANIMATION_POPURI_WALK",
        "ANIMATION_LILLIA_IDLE",
        "ANIMATION_LILLIA_WALK",
        "ANIMATION_HARVEST_GODDESS_IDLE",
        "ANIMATION_HARVEST_GODDESS_LEVITATE",
        "ANIMATION_ELLEN_IDLE",
        "ANIMATION_SASHA_IDLE",
        "ANIMATION_SASHA_WALK",
        "ANIMATION_DOUG_IDLE",
        "ANIMATION_DOUG_WALK",
        "ANIMATION_ANNA_IDLE",
        "ANIMATION_ANNA_WALK",
        "ANIMATION_THOMAS_IDLE",
        "ANIMATION_THOMAS_WALK",
        "ANIMATION_DUKE_IDLE",
        "ANIMATION_DUKE_WALK",
        "ANIMATION_ZACK_IDLE",
        "ANIMATION_ZACK_WALK",
        "ANIMATION_DOCTOR_IDLE",
        "ANIMATION_DOCTOR_WALK",
        "ANIMATION_KAREN_IDLE",
        "ANIMATION_KAREN_WALK",
        "ANIMATION_ANN_IDLE",
        "ANIMATION_ANN_WALK",
        "ANIMATION_MARY_IDLE",
        "ANIMATION_MARY_WALK",
        "ANIMATION_ELLI_IDLE",
        "ANIMATION_ELLI_WALK",
        "ANIMATION_GOURMET_IDLE",
        "ANIMATION_GOURMET_WALK",
        "ANIMATION_GOURMET_TASTE_FOOD",
        "ANIMATION_JEFF_IDLE",
        "ANIMATION_JEFF_WALK",
        "ANIMATION_BASIL_IDLE",
        "ANIMATION_BASIL_WALK",
        "ANIMATION_STU_IDLE",
        "ANIMATION_STU_WALK",
        "ANIMATION_MANNA_IDLE",
        "ANIMATION_MANNA_WALK",
        "ANIMATION_CARTER_IDLE",
        "ANIMATION_CARTER_WALK",
        "ANIMATION_KAI_IDLE",
        "ANIMATION_KAI_WALK",
        "ANIMATION_GRAY_IDLE",
        "ANIMATION_GRAY_WALK",
        "ANIMATION_SAIBARA_IDLE",
        "ANIMATION_SAIBARA_WALK",
        "ANIMATION_CLIFF_IDLE",
        "ANIMATION_CLIFF_WALK",
        "ANIMATION_FARM_HORSE_IDLE",
        "ANIMATION_FARM_HORSE_WALK",
        "ANIMATION_WON_IDLE",
        "ANIMATION_WON_WALK",
        "ANIMATION_HARRIS_IDLE",
        "ANIMATION_HARRIS_WALK",
        "ANIMATION_GOTZ_IDLE",
        "ANIMATION_GOTZ_WALK",
        "ANIMATION_MAY_IDLE",
        "ANIMATION_MAY_WALK",
        "ANIMATION_BARLEY_IDLE",
        "ANIMATION_BARLEY_WALK",
        "ANIMATION_STAID_IDLE",
        "ANIMATION_TIMID_IDLE",
        "ANIMATION_NAPPY_IDLE",
        "ANIMATION_BOLD_IDLE",
        "ANIMATION_CHEF_IDLE",
        "ANIMATION_AQUA_IDLE",
        "ANIMATION_HOGGY_IDLE",
        "ANIMATION_TIMID_SLEEPING",
        "ANIMATION_AQUA_SINGING",
        "ANIMATION_KAPPA_IDLE",
        "ANIMATION_KAPPA_HANDS_TOGETHER",
    ];
    for (target, values) in [
        (
            "MARY_FOMT_US",
            [
                559, 563, 607, 611, 1641, 1653, 1669, 1733, 1737, 1970, 1974, 2123, 2127, 2143,
                2147, 2240, 2244, 792, 796, 800, 804, 1681, 1685, 1982, 1986, 2067, 2071, 2180,
                2184, 1657, 1661, 1665, 1673, 1677, 2059, 2063, 2232, 2236, 2256, 2260, 2284, 2288,
                2306, 2310, 2441, 2445, 2465, 2469, 635, 639, 1914, 1918, 2135, 2139, 2276, 2280,
                2515, 2519, 2535, 2539, 2543, 2547, 992, 1076, 1160, 1244, 1328, 1412, 1496, 1084,
                1424, 2046, 2050,
            ],
        ),
        (
            "MARY_FOMT_JP",
            [
                559, 563, 607, 611, 1641, 1653, 1669, 1733, 1737, 1970, 1974, 2123, 2127, 2143,
                2147, 2240, 2244, 792, 796, 800, 804, 1681, 1685, 1982, 1986, 2067, 2071, 2180,
                2184, 1657, 1661, 1665, 1673, 1677, 2059, 2063, 2232, 2236, 2256, 2260, 2284, 2288,
                2306, 2310, 2441, 2445, 2465, 2469, 635, 639, 1914, 1918, 2135, 2139, 2276, 2280,
                2515, 2519, 2535, 2539, 2543, 2547, 992, 1076, 1160, 1244, 1328, 1412, 1496, 1084,
                1424, 2046, 2050,
            ],
        ),
        (
            "MARY_MFOMT_US",
            [
                571, 575, 619, 623, 1677, 1689, 1717, 1781, 1785, 2018, 2022, 2171, 2175, 2203,
                2207, 2300, 2304, 816, 820, 824, 828, 1729, 1733, 2030, 2034, 2115, 2119, 2240,
                2244, 1693, 1697, 1701, 1721, 1725, 2107, 2111, 2292, 2296, 2316, 2320, 2344, 2348,
                2366, 2370, 2513, 2517, 2549, 2553, 647, 651, 1962, 1966, 2183, 2187, 2336, 2340,
                2599, 2603, 2619, 2623, 2627, 2631, 1028, 1112, 1196, 1280, 1364, 1448, 1532, 1120,
                1460, 2094, 2098,
            ],
        ),
        (
            "MARY_MFOMT_JP",
            [
                571, 575, 619, 623, 1677, 1689, 1717, 1781, 1785, 2018, 2022, 2171, 2175, 2203,
                2207, 2300, 2304, 816, 820, 824, 828, 1729, 1733, 2030, 2034, 2115, 2119, 2240,
                2244, 1693, 1697, 1701, 1721, 1725, 2107, 2111, 2292, 2296, 2316, 2320, 2344, 2348,
                2366, 2370, 2513, 2517, 2549, 2553, 647, 651, 1962, 1966, 2183, 2187, 2336, 2340,
                2599, 2603, 2619, 2623, 2627, 2631, 1028, 1112, 1196, 1280, 1364, 1448, 1532, 1120,
                1460, 2094, 2098,
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestShiftedAnimations, };\n", &options)
                .unwrap();
        let semantic_body = symbols
            .iter()
            .map(|symbol| format!("SetEntityAnim(0, {symbol});"))
            .collect::<String>();
        let semantic_source = format!("void TestShiftedAnimations(void) {{ {semantic_body} }}\n");
        let semantic =
            parse_named_scripts(&semantic_source, &options, &callables.scope, &script_table)
                .unwrap();
        let numeric_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, {value});"))
            .collect::<String>();
        let numeric_source = format!("void TestShiftedAnimations(void) {{ {numeric_body} }}\n");
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        assert_eq!(
            encode_script(&semantic.scripts[0].2),
            encode_script(&numeric.scripts[0].2),
            "{target} physical IDs"
        );
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestShiftedAnimations",
        )
        .unwrap();
        let source = format_named_script("TestShiftedAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy_source = format!("void TestShiftedAnimations(void) {{ {legacy_body} }}\n");
        let legacy =
            parse_named_scripts(&legacy_source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy aliases"
        );
    }
}

fn assert_animation_pair_round_trip(
    target: &str,
    idle: i64,
    walk: i64,
    idle_symbol: &str,
    walk_symbol: &str,
) {
    let options = Options::default().define(target).unwrap();
    let constants = parse_constant_header(
        &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
        &options,
    )
    .unwrap();
    assert_eq!(constants.const_int_value(idle_symbol), Some(idle));
    assert_eq!(constants.const_int_value(walk_symbol), Some(walk));
    let callables = parse_callable_table_with_scope(
        &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
        &options,
        &constants,
    )
    .unwrap();
    let script_table =
        parse_script_table("mary_script_table { TestAnimationPair, };\n", &options).unwrap();
    let numeric_source = format!(
        "void TestAnimationPair(void) {{ SetEntityAnim(0, {idle}); SetEntityAnim(0, {walk}); }}\n"
    );
    let numeric =
        parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table).unwrap();
    let raised =
        decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAnimationPair")
            .unwrap();
    let source = format_named_script("TestAnimationPair", &raised).unwrap();
    assert!(source.contains(idle_symbol), "{target}: {source}");
    assert!(source.contains(walk_symbol), "{target}: {source}");
    let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&rebuilt.scripts[0].2)
    );

    let legacy_source = format!(
        "void TestAnimationPair(void) {{ SetEntityAnim(0, ANIMATION_ID_{idle:04}); SetEntityAnim(0, ANIMATION_ID_{walk:04}); }}\n"
    );
    let legacy =
        parse_named_scripts(&legacy_source, &options, &callables.scope, &script_table).unwrap();
    assert_eq!(
        encode_script(&numeric.scripts[0].2),
        encode_script(&legacy.scripts[0].2)
    );
}

#[test]
fn van_idle_walk_animations_follow_each_game_family() {
    for (target, idle, walk) in [
        ("MARY_FOMT_US", 2264, 2268),
        ("MARY_FOMT_JP", 2264, 2268),
        ("MARY_MFOMT_US", 2324, 2328),
        ("MARY_MFOMT_JP", 2324, 2328),
    ] {
        assert_animation_pair_round_trip(
            target,
            idle,
            walk,
            "ANIMATION_VAN_IDLE",
            "ANIMATION_VAN_WALK",
        );
    }
}

#[test]
fn lou_or_ruby_idle_walk_animations_follow_each_game_family() {
    for (target, idle, walk) in [
        ("MARY_FOMT_US", 2034, 2038),
        ("MARY_FOMT_JP", 2034, 2038),
        ("MARY_MFOMT_US", 2082, 2086),
        ("MARY_MFOMT_JP", 2082, 2086),
    ] {
        assert_animation_pair_round_trip(
            target,
            idle,
            walk,
            "ANIMATION_LOU_OR_RUBY_IDLE",
            "ANIMATION_LOU_OR_RUBY_WALK",
        );
    }
}

#[test]
fn ann_and_elli_gestures_follow_each_game_family() {
    for (target, ann_gesture, elli_gesture) in [
        ("MARY_FOMT_US", 1990, 2188),
        ("MARY_FOMT_JP", 1990, 2188),
        ("MARY_MFOMT_US", 2038, 2248),
        ("MARY_MFOMT_JP", 2038, 2248),
    ] {
        assert_animation_pair_round_trip(
            target,
            ann_gesture,
            elli_gesture,
            "ANIMATION_ANN_GESTURE",
            "ANIMATION_ELLI_GESTURE",
        );
    }
}

#[test]
fn doctor_karen_mary_and_gray_gestures_follow_each_game_family() {
    for (target, doctor, karen, mary, gray) in [
        ("MARY_FOMT_US", 808, 1689, 2075, 2449),
        ("MARY_FOMT_JP", 808, 1689, 2075, 2449),
        ("MARY_MFOMT_US", 832, 1737, 2123, 2521),
        ("MARY_MFOMT_JP", 832, 1737, 2123, 2521),
    ] {
        assert_animation_pair_round_trip(
            target,
            doctor,
            karen,
            "ANIMATION_DOCTOR_GESTURE",
            "ANIMATION_KAREN_GESTURE",
        );
        assert_animation_pair_round_trip(
            target,
            mary,
            gray,
            "ANIMATION_MARY_GESTURE",
            "ANIMATION_GRAY_GESTURE",
        );
    }
}

#[test]
fn popuri_item_and_reaction_animations_are_not_conflated() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        assert_animation_pair_round_trip(
            target,
            567,
            579,
            "ANIMATION_POPURI_HAND_OVER_ITEM",
            "ANIMATION_POPURI_REACTION",
        );
    }
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_POPURI_HAND_OVER_ITEM"),
            None,
            "{target} must not assign Popuri semantics to Rick's animation 567"
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_POPURI_REACTION"),
            Some(579)
        );
    }
}

#[test]
fn spouse_in_bed_animations_follow_each_game_family() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        for (first, second, first_symbol, second_symbol) in [
            (
                575,
                1697,
                "ANIMATION_POPURI_IN_BED",
                "ANIMATION_KAREN_IN_BED",
            ),
            (2002, 2087, "ANIMATION_ANN_IN_BED", "ANIMATION_MARY_IN_BED"),
            (
                2204,
                575,
                "ANIMATION_ELLI_IN_BED",
                "ANIMATION_POPURI_IN_BED",
            ),
        ] {
            assert_animation_pair_round_trip(target, first, second, first_symbol, second_symbol);
        }
    }
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        for (first, second, first_symbol, second_symbol) in [
            (559, 848, "ANIMATION_RICK_IN_BED", "ANIMATION_DOCTOR_IN_BED"),
            (687, 2390, "ANIMATION_CLIFF_IN_BED", "ANIMATION_KAI_IN_BED"),
            (2537, 559, "ANIMATION_GRAY_IN_BED", "ANIMATION_RICK_IN_BED"),
        ] {
            assert_animation_pair_round_trip(target, first, second, first_symbol, second_symbol);
        }
    }
}

#[test]
fn spouse_bed_dialogue_and_settle_animations_follow_each_game_family() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        for (awake, settle, awake_symbol, settle_symbol) in [
            (
                579,
                603,
                "ANIMATION_POPURI_REACTION",
                "ANIMATION_POPURI_SETTLE_IN_BED",
            ),
            (
                1701,
                1729,
                "ANIMATION_KAREN_IN_BED_AWAKE",
                "ANIMATION_KAREN_SETTLE_IN_BED",
            ),
            (
                2006,
                2030,
                "ANIMATION_ANN_IN_BED_AWAKE",
                "ANIMATION_ANN_SETTLE_IN_BED",
            ),
            (
                2091,
                2115,
                "ANIMATION_MARY_IN_BED_AWAKE",
                "ANIMATION_MARY_SETTLE_IN_BED",
            ),
            (
                2208,
                2228,
                "ANIMATION_ELLI_IN_BED_AWAKE",
                "ANIMATION_ELLI_SETTLE_IN_BED",
            ),
        ] {
            assert_animation_pair_round_trip(target, awake, settle, awake_symbol, settle_symbol);
        }
    }
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        for (awake, settle, awake_symbol, settle_symbol) in [
            (
                563,
                567,
                "ANIMATION_RICK_IN_BED_AWAKE",
                "ANIMATION_RICK_SETTLE_IN_BED",
            ),
            (
                691,
                695,
                "ANIMATION_CLIFF_IN_BED_AWAKE",
                "ANIMATION_CLIFF_SETTLE_IN_BED",
            ),
            (
                852,
                856,
                "ANIMATION_DOCTOR_IN_BED_AWAKE",
                "ANIMATION_DOCTOR_SETTLE_IN_BED",
            ),
            (
                2394,
                2398,
                "ANIMATION_KAI_IN_BED_AWAKE",
                "ANIMATION_KAI_SETTLE_IN_BED",
            ),
            (
                2541,
                2545,
                "ANIMATION_GRAY_IN_BED_AWAKE",
                "ANIMATION_GRAY_SETTLE_IN_BED",
            ),
        ] {
            assert_animation_pair_round_trip(target, awake, settle, awake_symbol, settle_symbol);
        }
    }
}

#[test]
fn mfomt_mary_wedding_and_kai_gesture_animations_are_target_scoped() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        assert_animation_pair_round_trip(
            target,
            2151,
            2155,
            "ANIMATION_MARY_WEDDING_IDLE",
            "ANIMATION_MARY_WEDDING_WALK",
        );
        assert_animation_pair_round_trip(
            target,
            2159,
            2374,
            "ANIMATION_MARY_WEDDING_KISS",
            "ANIMATION_KAI_GESTURE",
        );
    }
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(constants.const_int_value("ANIMATION_KAI_GESTURE"), None);
        assert_eq!(
            constants.const_int_value("ANIMATION_MARY_WEDDING_IDLE"),
            Some(2103)
        );
    }
}

#[test]
fn mfomt_mary_bad_dream_idle_walk_pair_is_target_scoped() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        assert_animation_pair_round_trip(
            target,
            2143,
            2147,
            "ANIMATION_MARY_BAD_DREAM_IDLE",
            "ANIMATION_MARY_BAD_DREAM_WALK",
        );
    }
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_MARY_BAD_DREAM_IDLE"),
            None
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_THOMAS_IDLE"),
            Some(2143)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_THOMAS_WALK"),
            Some(2147)
        );
    }
}

#[test]
fn mfomt_transformation_effect_parts_are_target_scoped() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        assert_animation_pair_round_trip(
            target,
            1272,
            1608,
            "ANIMATION_TRANSFORMATION_EFFECT_PART_1",
            "ANIMATION_TRANSFORMATION_EFFECT_PART_2",
        );
        assert_animation_pair_round_trip(
            target,
            1188,
            1104,
            "ANIMATION_TRANSFORMATION_EFFECT_PART_3",
            "ANIMATION_TRANSFORMATION_EFFECT_PART_4",
        );
    }
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_TRANSFORMATION_EFFECT_PART_1"),
            None
        );
        assert_eq!(constants.const_int_value("ANIMATION_ID_1272"), Some(1272));
    }
}

#[test]
fn sick_livestock_idle_animations_follow_game_family_offsets() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        assert_animation_pair_round_trip(
            target,
            696,
            1836,
            "ANIMATION_COW_SICK_IDLE",
            "ANIMATION_CHICKEN_SICK_IDLE",
        );
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_SHEEP_SICK_IDLE"),
            Some(2345)
        );
    }
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        assert_animation_pair_round_trip(
            target,
            720,
            1884,
            "ANIMATION_COW_SICK_IDLE",
            "ANIMATION_CHICKEN_SICK_IDLE",
        );
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_SHEEP_SICK_IDLE"),
            Some(2417)
        );
    }
    assert_animation_pair_round_trip(
        "MARY_FOMT_US",
        2345,
        696,
        "ANIMATION_SHEEP_SICK_IDLE",
        "ANIMATION_COW_SICK_IDLE",
    );
    assert_animation_pair_round_trip(
        "MARY_MFOMT_US",
        2417,
        720,
        "ANIMATION_SHEEP_SICK_IDLE",
        "ANIMATION_COW_SICK_IDLE",
    );
}

#[test]
fn colliding_npc_animation_slots_are_target_scoped() {
    let values = [1733, 2240, 2244, 2248];
    let symbols = [
        "ANIMATION_SASHA_IDLE",
        "ANIMATION_DUKE_IDLE",
        "ANIMATION_DUKE_WALK",
        "ANIMATION_DUKE_COLLAPSED",
    ];
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFoMTAnimations, };\n", &options).unwrap();
        let numeric_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, {value});"))
            .collect::<String>();
        let numeric_source = format!("void TestFoMTAnimations(void) {{ {numeric_body} }}\n");
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestFoMTAnimations",
        )
        .unwrap();
        let source = format_named_script("TestFoMTAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy_source = format!("void TestFoMTAnimations(void) {{ {legacy_body} }}\n");
        let legacy =
            parse_named_scripts(&legacy_source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy aliases"
        );
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_SASHA_IDLE"),
            Some(1781)
        );
        assert_eq!(constants.const_int_value("ANIMATION_DUKE_IDLE"), Some(2300));
        assert_eq!(constants.const_int_value("ANIMATION_DUKE_WALK"), Some(2304));
        assert_eq!(
            constants.const_int_value("ANIMATION_DUKE_COLLAPSED"),
            Some(2308)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_KAREN_WALK"),
            Some(1733)
        );
        assert_eq!(constants.const_int_value("ANIMATION_ELLI_IDLE"), Some(2240));
        assert_eq!(constants.const_int_value("ANIMATION_ELLI_WALK"), Some(2244));
        assert_eq!(
            constants.const_int_value("ANIMATION_ELLI_GESTURE"),
            Some(2248)
        );
    }
}

#[test]
fn wedding_animation_groups_are_target_scoped() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_RICK_WEDDING_IDLE"),
            Some(547)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_RICK_WEDDING_WALK"),
            Some(551)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_RICK_WEDDING_KISS"),
            Some(555)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_PLAYER_WEDDING_IDLE"),
            Some(130)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_PLAYER_WEDDING_WALK"),
            Some(134)
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_PLAYER_WEDDING_KISS"),
            Some(126)
        );
        let mut values = vec![547, 551, 555, 130, 134, 126];
        let mut symbols = vec![
            "ANIMATION_RICK_WEDDING_IDLE",
            "ANIMATION_RICK_WEDDING_WALK",
            "ANIMATION_RICK_WEDDING_KISS",
            "ANIMATION_PLAYER_WEDDING_IDLE",
            "ANIMATION_PLAYER_WEDDING_WALK",
            "ANIMATION_PLAYER_WEDDING_KISS",
        ];
        if target.contains("FOMT_") && !target.contains("MFOMT_") {
            assert_eq!(
                constants.const_int_value("ANIMATION_CLIFF_WEDDING_IDLE"),
                Some(647)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_CLIFF_WEDDING_WALK"),
                Some(651)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_CLIFF_WEDDING_KISS"),
                Some(655)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_GRAY_WEDDING_IDLE"),
                Some(2453)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_GRAY_WEDDING_WALK"),
                Some(2457)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_GRAY_WEDDING_KISS"),
                Some(2461)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAREN_WEDDING_IDLE"),
                Some(1717)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAREN_WEDDING_WALK"),
                Some(1721)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAREN_WEDDING_KISS"),
                Some(1725)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_POPURI_WEDDING_WALK"),
                Some(595)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_POPURI_WEDDING_KISS"),
                Some(599)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_POPURI_WEDDING_IDLE"),
                Some(591)
            );
            values.extend([
                647, 651, 655, 2453, 2457, 2461, 1717, 1721, 1725, 591, 595, 599,
            ]);
            symbols.extend([
                "ANIMATION_CLIFF_WEDDING_IDLE",
                "ANIMATION_CLIFF_WEDDING_WALK",
                "ANIMATION_CLIFF_WEDDING_KISS",
                "ANIMATION_GRAY_WEDDING_IDLE",
                "ANIMATION_GRAY_WEDDING_WALK",
                "ANIMATION_GRAY_WEDDING_KISS",
                "ANIMATION_KAREN_WEDDING_IDLE",
                "ANIMATION_KAREN_WEDDING_WALK",
                "ANIMATION_KAREN_WEDDING_KISS",
                "ANIMATION_POPURI_WEDDING_IDLE",
                "ANIMATION_POPURI_WEDDING_WALK",
                "ANIMATION_POPURI_WEDDING_KISS",
            ]);
            assert_eq!(
                constants.const_int_value("ANIMATION_MARY_WEDDING_IDLE"),
                Some(2103)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_MARY_WEDDING_WALK"),
                Some(2107)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_MARY_WEDDING_KISS"),
                Some(2111)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLI_WEDDING_IDLE"),
                Some(2216)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLI_WEDDING_WALK"),
                Some(2220)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLI_WEDDING_KISS"),
                Some(2224)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAI_WEDDING_IDLE"),
                Some(2318)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAI_WEDDING_WALK"),
                Some(2322)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAI_WEDDING_KISS"),
                Some(2326)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_DOCTOR_WEDDING_IDLE"),
                Some(812)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_DOCTOR_WEDDING_WALK"),
                Some(816)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_DOCTOR_WEDDING_KISS"),
                Some(820)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ANN_WEDDING_IDLE"),
                Some(2018)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ANN_WEDDING_WALK"),
                Some(2022)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ANN_WEDDING_KISS"),
                Some(2026)
            );
            values.extend([
                2103, 2107, 2111, 2216, 2220, 2224, 2318, 2322, 2326, 812, 816, 820, 2018, 2022,
                2026,
            ]);
            symbols.extend([
                "ANIMATION_MARY_WEDDING_IDLE",
                "ANIMATION_MARY_WEDDING_WALK",
                "ANIMATION_MARY_WEDDING_KISS",
                "ANIMATION_ELLI_WEDDING_IDLE",
                "ANIMATION_ELLI_WEDDING_WALK",
                "ANIMATION_ELLI_WEDDING_KISS",
                "ANIMATION_KAI_WEDDING_IDLE",
                "ANIMATION_KAI_WEDDING_WALK",
                "ANIMATION_KAI_WEDDING_KISS",
                "ANIMATION_DOCTOR_WEDDING_IDLE",
                "ANIMATION_DOCTOR_WEDDING_WALK",
                "ANIMATION_DOCTOR_WEDDING_KISS",
                "ANIMATION_ANN_WEDDING_IDLE",
                "ANIMATION_ANN_WEDDING_WALK",
                "ANIMATION_ANN_WEDDING_KISS",
            ]);
        } else {
            assert_eq!(
                constants.const_int_value("ANIMATION_CLIFF_WEDDING_IDLE"),
                Some(659)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_CLIFF_WEDDING_WALK"),
                Some(663)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_CLIFF_WEDDING_KISS"),
                Some(667)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAREN_WEDDING_IDLE"),
                Some(1765)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAREN_WEDDING_WALK"),
                Some(1769)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAREN_WEDDING_KISS"),
                Some(1773)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLEN_IDLE"),
                Some(1717)
            );
            assert_eq!(constants.const_int_value("ANIMATION_JEFF_IDLE"), Some(1721));
            assert_eq!(constants.const_int_value("ANIMATION_JEFF_WALK"), Some(1725));
            assert_eq!(
                constants.const_int_value("ANIMATION_MARY_WEDDING_IDLE"),
                Some(2151)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_MARY_WEDDING_WALK"),
                Some(2155)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_MARY_WEDDING_KISS"),
                Some(2159)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_BASIL_IDLE"),
                Some(2107)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_BASIL_WALK"),
                Some(2111)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_GRAY_WEDDING_IDLE"),
                Some(2525)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_GRAY_WEDDING_WALK"),
                Some(2529)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_GRAY_WEDDING_KISS"),
                Some(2533)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLI_WEDDING_IDLE"),
                Some(2276)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLI_WEDDING_WALK"),
                Some(2280)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ELLI_WEDDING_KISS"),
                Some(2284)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAI_WEDDING_IDLE"),
                Some(2378)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAI_WEDDING_WALK"),
                Some(2382)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_KAI_WEDDING_KISS"),
                Some(2386)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_DOCTOR_WEDDING_IDLE"),
                Some(836)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_DOCTOR_WEDDING_WALK"),
                Some(840)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_DOCTOR_WEDDING_KISS"),
                Some(844)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ANN_WEDDING_IDLE"),
                Some(2066)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ANN_WEDDING_WALK"),
                Some(2070)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_ANN_WEDDING_KISS"),
                Some(2074)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_POPURI_WEDDING_WALK"),
                Some(607)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_POPURI_WEDDING_KISS"),
                Some(611)
            );
            assert_eq!(
                constants.const_int_value("ANIMATION_POPURI_WEDDING_IDLE"),
                Some(603)
            );
            values.extend([
                603, 659, 663, 667, 2525, 2529, 2533, 2276, 2280, 2284, 2378, 2382, 2386, 836, 840,
                844, 2066, 2070, 2074, 1765, 1769, 1773, 607, 611,
            ]);
            symbols.extend([
                "ANIMATION_POPURI_WEDDING_IDLE",
                "ANIMATION_CLIFF_WEDDING_IDLE",
                "ANIMATION_CLIFF_WEDDING_WALK",
                "ANIMATION_CLIFF_WEDDING_KISS",
                "ANIMATION_GRAY_WEDDING_IDLE",
                "ANIMATION_GRAY_WEDDING_WALK",
                "ANIMATION_GRAY_WEDDING_KISS",
                "ANIMATION_ELLI_WEDDING_IDLE",
                "ANIMATION_ELLI_WEDDING_WALK",
                "ANIMATION_ELLI_WEDDING_KISS",
                "ANIMATION_KAI_WEDDING_IDLE",
                "ANIMATION_KAI_WEDDING_WALK",
                "ANIMATION_KAI_WEDDING_KISS",
                "ANIMATION_DOCTOR_WEDDING_IDLE",
                "ANIMATION_DOCTOR_WEDDING_WALK",
                "ANIMATION_DOCTOR_WEDDING_KISS",
                "ANIMATION_ANN_WEDDING_IDLE",
                "ANIMATION_ANN_WEDDING_WALK",
                "ANIMATION_ANN_WEDDING_KISS",
                "ANIMATION_KAREN_WEDDING_IDLE",
                "ANIMATION_KAREN_WEDDING_WALK",
                "ANIMATION_KAREN_WEDDING_KISS",
                "ANIMATION_POPURI_WEDDING_WALK",
                "ANIMATION_POPURI_WEDDING_KISS",
            ]);
        }

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestWeddingAnimations, };\n", &options)
                .unwrap();
        let numeric_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, {value});"))
            .collect::<String>();
        let numeric_source = format!("void TestWeddingAnimations(void) {{ {numeric_body} }}\n");
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestWeddingAnimations",
        )
        .unwrap();
        let source = format_named_script("TestWeddingAnimations", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn proven_player_action_animations_are_canonical_on_all_targets() {
    let values = [
        66, 70, 78, 238, 242, 258, 262, 270, 282, 286, 290, 294, 298, 302, 306, 314, 398, 426, 434,
        438, 442, 510, 26, 30, 34, 38, 42, 46, 50, 54, 58, 158, 162, 166, 186, 170, 174, 178, 182,
        190, 194, 198, 202, 206, 210, 214, 218, 222, 250, 334, 350, 354, 358, 362, 366, 370, 374,
        378, 382, 386, 394, 414, 418, 430, 446, 462, 466, 470, 474, 478, 482, 486, 490, 494, 498,
        502,
    ];
    let symbols = [
        "ANIMATION_PLAYER_RAISE_ARMS",
        "ANIMATION_PLAYER_DRINK",
        "ANIMATION_PLAYER_EAT",
        "ANIMATION_PLAYER_RIDING_HORSE_WALK",
        "ANIMATION_PLAYER_BACK_PAIN",
        "ANIMATION_PLAYER_SWEAT",
        "ANIMATION_PLAYER_HIGH_JUMP",
        "ANIMATION_PLAYER_FAINT_FROM_EXHAUSTION",
        "ANIMATION_PLAYER_COLLAPSE_HOLDING_HEAD",
        "ANIMATION_PLAYER_HOLD_HEAD_IN_PAIN",
        "ANIMATION_PLAYER_SHAKE_HEAD_NO",
        "ANIMATION_PLAYER_BATHE_IN_HOT_SPRING",
        "ANIMATION_PLAYER_ENTER_OR_EXIT_HOT_SPRING",
        "ANIMATION_PLAYER_PLAY_OCARINA",
        "ANIMATION_PLAYER_MOVE_ANIMAL",
        "ANIMATION_PLAYER_RUCKSACK_DEPOSIT",
        "ANIMATION_PLAYER_SLEEPING_IN_BED",
        "ANIMATION_PLAYER_BRUSH_LIVESTOCK",
        "ANIMATION_PLAYER_USE_CLIPPERS",
        "ANIMATION_PLAYER_USE_MILKER",
        "ANIMATION_PLAYER_SOW_SEEDS",
        "ANIMATION_PLAYER_LOWER_HEAD",
        "ANIMATION_PLAYER_AXE_PREPARE",
        "ANIMATION_PLAYER_AXE_CHARGE_STAGE_1",
        "ANIMATION_PLAYER_AXE_CHARGE_STAGE_2",
        "ANIMATION_PLAYER_AXE_CHARGE_STAGE_3",
        "ANIMATION_PLAYER_AXE_CHARGE_STAGE_4",
        "ANIMATION_PLAYER_AXE_CHARGE_STAGE_5",
        "ANIMATION_PLAYER_AXE_CHARGE_STAGE_6",
        "ANIMATION_PLAYER_AXE_SWING",
        "ANIMATION_PLAYER_AXE_CHARGED_SWING",
        "ANIMATION_PLAYER_HAMMER_PREPARE",
        "ANIMATION_PLAYER_HAMMER_CHARGE_STAGE_1",
        "ANIMATION_PLAYER_HAMMER_CHARGE_STAGE_2",
        "ANIMATION_PLAYER_HAMMER_CHARGE_STAGE_3",
        "ANIMATION_PLAYER_HAMMER_CHARGE_STAGE_4",
        "ANIMATION_PLAYER_HAMMER_CHARGE_STAGE_5",
        "ANIMATION_PLAYER_HAMMER_CHARGE_STAGE_6",
        "ANIMATION_PLAYER_HAMMER_SWING",
        "ANIMATION_PLAYER_HAMMER_CHARGED_SWING",
        "ANIMATION_PLAYER_HOE_PREPARE",
        "ANIMATION_PLAYER_HOE_CHARGE_STAGE_1",
        "ANIMATION_PLAYER_HOE_CHARGE_STAGE_2",
        "ANIMATION_PLAYER_HOE_CHARGE_STAGE_3",
        "ANIMATION_PLAYER_HOE_CHARGE_STAGE_4",
        "ANIMATION_PLAYER_HOE_CHARGE_STAGE_5",
        "ANIMATION_PLAYER_HOE_CHARGE_STAGE_6",
        "ANIMATION_PLAYER_HOE_SWING",
        "ANIMATION_PLAYER_COLLAPSE",
        "ANIMATION_PLAYER_RUN_HOLDING_ITEM",
        "ANIMATION_PLAYER_RUN_EMPTY_HANDED",
        "ANIMATION_PLAYER_CLINIC_BED_SHADOW",
        "ANIMATION_PLAYER_SICKLE_PREPARE",
        "ANIMATION_PLAYER_SICKLE_CHARGE_STAGE_1",
        "ANIMATION_PLAYER_SICKLE_CHARGE_STAGE_2",
        "ANIMATION_PLAYER_SICKLE_CHARGE_STAGE_3",
        "ANIMATION_PLAYER_SICKLE_CHARGE_STAGE_4",
        "ANIMATION_PLAYER_SICKLE_CHARGE_STAGE_5",
        "ANIMATION_PLAYER_SICKLE_CHARGE_STAGE_6",
        "ANIMATION_PLAYER_SICKLE_CHARGED_SWING",
        "ANIMATION_PLAYER_SICKLE_SWING",
        "ANIMATION_PLAYER_PUT_AWAY_PRESENTED_ITEM",
        "ANIMATION_PLAYER_USE_ANIMAL_MEDICINE",
        "ANIMATION_PLAYER_USE_COW_MIRACLE_POTION",
        "ANIMATION_PLAYER_USE_SHEEP_MIRACLE_POTION",
        "ANIMATION_PLAYER_WATERING_CAN_PREPARE",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGE_STAGE_1",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGE_STAGE_2",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGE_STAGE_3",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGE_STAGE_4",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGE_STAGE_5",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGE_STAGE_6",
        "ANIMATION_PLAYER_REFILL_WATERING_CAN",
        "ANIMATION_PLAYER_WATERING_CAN_POUR",
        "ANIMATION_PLAYER_WATERING_CAN_CHARGED_POUR",
        "ANIMATION_PLAYER_WATERING_CAN_MAX_CHARGE_POUR",
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestPlayerActions, };\n", &options).unwrap();
        let numeric_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, {value});"))
            .collect::<String>();
        let numeric_source = format!("void TestPlayerActions(void) {{ {numeric_body} }}\n");
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestPlayerActions")
                .unwrap();
        let source = format_named_script("TestPlayerActions", &raised).unwrap();
        for symbol in symbols {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );

        let legacy_body = values
            .iter()
            .map(|value| format!("SetEntityAnim(0, ANIMATION_ID_{value:04});"))
            .collect::<String>();
        let legacy_source = format!("void TestPlayerActions(void) {{ {legacy_body} }}\n");
        let legacy =
            parse_named_scripts(&legacy_source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target} legacy aliases"
        );
    }
}

#[test]
fn harvest_goddess_appear_disappear_effect_is_fomt_scoped() {
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_HARVEST_GODDESS_APPEAR_DISAPPEAR_EFFECT"),
            Some(1068),
            "{target}"
        );
        assert_eq!(constants.const_int_value("ANIMATION_ID_1068"), Some(1068));
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_HARVEST_GODDESS_APPEAR_DISAPPEAR_EFFECT"),
            None,
            "{target} must retain the unproven numbered symbol"
        );
        assert_eq!(constants.const_int_value("ANIMATION_ID_1068"), Some(1068));
    }
}

#[test]
fn proven_audio_names_are_canonical_while_numbered_sources_remain_compatible() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestAudio, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestAudio(void) { PlayBGM(1, 6); PlayBGM(1, 16); PlaySong(1, 17); \
             PlaySong(1, 125); PlaySong(1, 126); PlaySong(1, 127); \
             PlaySong(1, 132); PlaySong(1, 133); PlaySong(1, 146); PlaySong(1, 188); \
             PlaySong(1, 157); PlaySong(1, 175); PlaySong(1, 179); PlaySong(1, 184); PlaySong(1, 192); \
             PlaySong(1, 147); PlaySong(1, 149); PlaySong(1, 193); PlaySong(1, 200); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAudio").unwrap();
        let source = format_named_script("TestAudio", &raised).unwrap();
        assert!(source.contains("AUDIO_BGM_WEDDING"), "{target}: {source}");
        assert!(
            source.contains("AUDIO_BGM_RIVAL_EVENT"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_BGM_LOVE_EVENT"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_HAMMER_SMALL_STONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_HAMMER_LARGE_STONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_HAMMER_HUGE_STONE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_AXE_BRANCH_CHOP"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_AXE_STUMP_CHOP"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_WATER_SPLASH"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_STAR_SPARKLE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_HEAL_OR_PURIFY"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_TIME_PASSES"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_CEREMONIAL_CHIME"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_ATTENTION_CHIME"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_QUESTION_EMOTE"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_BRUSH_LIVESTOCK"),
            "{target}: {source}"
        );
        assert!(
            source.contains("AUDIO_SFX_DOCTOR_EXAMINATION"),
            "{target}: {source}"
        );
        assert!(source.contains("AUDIO_SFX_APPLAUSE"), "{target}: {source}");
        assert!(
            source.contains("AUDIO_SFX_KAPPA_SURPRISE"),
            "{target}: {source}"
        );

        let legacy = parse_named_scripts(
            "void TestAudio(void) { PlayBGM(AUDIO_START_WEAK, AUDIO_SEQUENCE_006); \
             PlayBGM(AUDIO_START_WEAK, AUDIO_SEQUENCE_016); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_017); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_125); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_126); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_127); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SFX_AXE_CHOP); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_133); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_146); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_188); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_157); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_175); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_179); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_184); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_192); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_147); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_149); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_193); \
             PlaySong(AUDIO_START_WEAK, AUDIO_SEQUENCE_200); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&legacy.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn animal_state_to_counter_reuse_does_not_leak_boolean_symbols() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let table =
            parse_script_table("mary_script_table { TestAnimalReuse, };", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestAnimalReuse(void) { int value; \
             value = IsAnimalPregnant(1, 0); \
             switch (value) { case 0: TalkClose(); break; case 1: TalkOpen(); break; } \
             value = GetAnimalAge(1, 0); \
             switch (value) { case 0: TalkClose(); break; case 1: TalkOpen(); break; } \
             if (IsAnimalSick(1, 0)) { value = GetAnimalHealthyPregnancyDays(1, 0); } \
             else { value = GetAnimalAge(1, 0); } \
             switch (value) { case 0: TalkClose(); break; case 1: TalkOpen(); break; } }",
            &options,
            &callables.scope,
            &table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAnimalReuse")
                .unwrap();
        let source = format_named_script("TestAnimalReuse", &raised).unwrap();
        for (pattern, count) in [
            ("case FALSE:", 1),
            ("case TRUE:", 1),
            ("case 0:", 2),
            ("case 1:", 2),
        ] {
            assert_eq!(source.matches(pattern).count(), count, "{target}: {source}");
        }
        let rebuilt = parse_named_scripts(&source, &options, &callables.scope, &table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn livestock_facility_slot_domains_preserve_every_physical_selector() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFacilitySlots, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestFacilitySlots(void) { int chicken_slot; int animal_slot; \
             IsChickenFeedTroughFilled(7); FillChickenFeedTrough(7); \
             BeginEggIncubation(1); IsIncubatorOccupied(1); \
             IsEggReadyToHatch(1); chicken_slot = AttemptEggHatch(1); \
             switch (chicken_slot) { case 0: break; case 7: break; default: break; } \
             IsBarnFeedTroughFilled(17); FillBarnFeedTrough(17); \
             IsBarnAnimalReadyToGiveBirth(1); animal_slot = AttemptBarnAnimalBirth(1); \
             if (animal_slot != -1) { \
                 switch (animal_slot) { case 0: break; case 15: break; default: break; } \
             } else { TalkClose(); } \
             animal_slot = 1; IsBarnAnimalReadyToGiveBirth(animal_slot); \
             if (IsPlayerHoldingNothing()) { animal_slot = AttemptBarnAnimalBirth(0); } \
             else { animal_slot = AttemptEggHatch(0); } \
             switch (animal_slot) { case 1: TalkClose(); break; default: break; } \
             switch (IsPlayerHoldingNothing()) { \
                 case 0: animal_slot = 0; break; \
                 default: if (IsIncubatorOccupied(0)) { animal_slot = 1; } \
                          else { animal_slot = 0; } break; \
             } \
             IsBarnAnimalReadyToGiveBirth(animal_slot); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestFacilitySlots")
                .unwrap();
        let source = format_named_script("TestFacilitySlots", &raised).unwrap();
        assert!(
            source.contains("= BARN_PREGNANCY_STALL_SOUTH;"),
            "{target}: {source}"
        );
        // The merge has incompatible chicken/barn roster types. Do not choose
        // either family merely because both physical domains contain value 1.
        assert!(source.contains("case 1:"), "{target}: {source}");
        // The later use determines literal assignments across a switch and a
        // nested if/else, without contaminating the earlier conflicting merge.
        assert_eq!(
            source.matches("= BARN_PREGNANCY_STALL_NORTH;").count(),
            2,
            "{target}: {source}"
        );
        for symbol in [
            "CHICKEN_COOP_FEED_TROUGH_SLOT_08",
            "CHICKEN_INCUBATOR_NORTH",
            "CHICKEN_SLOT_1",
            "CHICKEN_SLOT_8",
            "BARN_PREGNANCY_FEED_TROUGH_SOUTH",
            "BARN_PREGNANCY_STALL_SOUTH",
            "ANIMAL_SLOT_1",
            "ANIMAL_SLOT_16",
            "ANIMAL_SLOT_NOT_SELECTED",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn chicken_coop_fixture_symbols_follow_physical_slots_on_all_targets() {
    let expected = [
        "CHICKEN_COOP_FEED_TROUGH_SLOT_01",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_02",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_03",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_04",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_05",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_06",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_07",
        "CHICKEN_COOP_FEED_TROUGH_SLOT_08",
        "CHICKEN_INCUBATOR_SOUTH",
        "CHICKEN_INCUBATOR_NORTH",
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestChickenFixtures, };\n", &options).unwrap();
        let calls = (0..8)
            .map(|value| format!("IsChickenFeedTroughFilled({value});"))
            .chain((0..2).map(|value| format!("IsIncubatorOccupied({value});")))
            .collect::<String>();
        let numeric = parse_named_scripts(
            &format!("void TestChickenFixtures(void) {{ {calls} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestChickenFixtures",
        )
        .unwrap();
        let source = format_named_script("TestChickenFixtures", &raised).unwrap();
        for symbol in expected {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
        parse_named_scripts(
            "void TestChickenFixtures(void) { FillChickenFeedTrough(CHICKEN_FEED_TROUGH_1); BeginEggIncubation(CHICKEN_INCUBATOR_2); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn barn_feed_trough_symbols_follow_the_physical_rows_on_all_targets() {
    let troughs = [
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_01",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_02",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_03",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_04",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_01",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_02",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_03",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_04",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_05",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_06",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_07",
        "BARN_FEED_TROUGH_NORTH_ROW_SLOT_08",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_05",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_06",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_07",
        "BARN_FEED_TROUGH_SOUTH_ROW_SLOT_08",
        "BARN_PREGNANCY_FEED_TROUGH_NORTH",
        "BARN_PREGNANCY_FEED_TROUGH_SOUTH",
    ];
    let pregnancy_stalls = ["BARN_PREGNANCY_STALL_NORTH", "BARN_PREGNANCY_STALL_SOUTH"];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestBarnTroughs, };\n", &options).unwrap();
        let calls = (0..18)
            .map(|value| format!("IsBarnFeedTroughFilled({value});"))
            .chain((0..2).map(|value| format!("IsBarnAnimalReadyToGiveBirth({value});")))
            .collect::<String>();
        let numeric = parse_named_scripts(
            &format!("void TestBarnTroughs(void) {{ {calls} }}\n"),
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestBarnTroughs")
                .unwrap();
        let source = format_named_script("TestBarnTroughs", &raised).unwrap();
        let trough_positions = troughs
            .iter()
            .map(|symbol| {
                source
                    .find(&format!("IsBarnFeedTroughFilled({symbol});"))
                    .unwrap_or_else(|| {
                        panic!("{target}: wrong feed-trough mapping for {symbol}: {source}")
                    })
            })
            .collect::<Vec<_>>();
        assert!(
            trough_positions.windows(2).all(|pair| pair[0] < pair[1]),
            "{target}: feed-trough symbols do not follow numeric selectors 0..17: {source}"
        );
        let pregnancy_positions = pregnancy_stalls
            .iter()
            .map(|symbol| {
                source
                    .find(&format!("IsBarnAnimalReadyToGiveBirth({symbol})"))
                    .unwrap_or_else(|| {
                        panic!("{target}: wrong pregnancy-stall mapping for {symbol}: {source}")
                    })
            })
            .collect::<Vec<_>>();
        assert!(
            pregnancy_positions[0] < pregnancy_positions[1],
            "{target}: pregnancy-stall symbols do not map north=0, south=1: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );
        parse_named_scripts(
            "void TestBarnTroughs(void) { IsBarnFeedTroughFilled(BARN_FEED_TROUGH_5); FillBarnFeedTrough(BARN_PREGNANCY_FEED_TROUGH_2); IsBarnAnimalReadyToGiveBirth(BARN_PREGNANCY_STALL_2); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
    }
}

#[test]
fn numbered_callables_are_an_explicit_audited_allowlist() {
    let header = fs::read_to_string("goodies/mary_callables.mary.h").unwrap();
    let mut actual = header
        .lines()
        .filter_map(|line| {
            let declaration = line.trim().split('(').next()?;
            let name = declaration.split_whitespace().last()?;
            ((name.starts_with("Func") || name.starts_with("Proc"))
                && name[4..]
                    .chars()
                    .all(|character| character.is_ascii_hexdigit()))
            .then(|| name.to_owned())
        })
        .collect::<Vec<_>>();
    actual.sort();
    actual.dedup();

    let mut expected: Vec<&str> = vec![];
    expected.sort();

    assert_eq!(actual, expected, "numbered callable audit changed");
}

#[test]
fn callable_tables_cover_every_non_internal_physical_slot_exactly_once() {
    for (target, expected_next_id) in [
        ("MARY_FOMT_US", 0x147),
        ("MARY_FOMT_JP", 0x147),
        ("MARY_MFOMT_US", 0x153),
        ("MARY_MFOMT_JP", 0x153),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();

        assert_eq!(callables.base, 0, "{target}: callable table base");
        assert_eq!(
            callables.next_id, expected_next_id,
            "{target}: callable table physical end"
        );

        let mut ids = callables
            .scope
            .callable_map()
            .values()
            .map(|(id, _)| id.0)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(
            ids,
            (2..expected_next_id).collect::<Vec<_>>(),
            "{target}: only physical slots 0 and 1 may remain VM-internal"
        );
    }
}

#[test]
fn vacation_villa_and_mythic_tool_checks_follow_all_four_target_tables() {
    for (target, villa_id, mythic_id) in [
        ("MARY_FOMT_US", 0x0E3, 0x0EA),
        ("MARY_FOMT_JP", 0x0E3, 0x0EA),
        ("MARY_MFOMT_US", 0x0E6, 0x0ED),
        ("MARY_MFOMT_JP", 0x0E6, 0x0ED),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let callable_map = callables.scope.callable_map();
        assert_eq!(
            callable_map["IsVacationVillaBuilt"].0 .0, villa_id,
            "{target}"
        );
        assert_eq!(
            callable_map["HasObtainedMythicTool"].0 .0, mythic_id,
            "{target}"
        );

        let script_table =
            parse_script_table("mary_script_table { TestAchievements, };\n", &options).unwrap();
        let parsed = parse_named_scripts(
            "void TestAchievements(void) { int villa = IsVacationVillaBuilt(); int mythic = HasObtainedMythicTool(); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestAchievements")
                .unwrap();
        let source = format_named_script("TestAchievements", &raised).unwrap();
        assert!(
            source.contains("IsVacationVillaBuilt()"),
            "{target}: {source}"
        );
        assert!(
            source.contains("HasObtainedMythicTool()"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&parsed.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2)
        );
    }
}

#[test]
fn mfomt_native_callable_tail_is_fully_represented() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        for (id, name) in [
            (0x14B, "IsChickenIncubatorOccupied"),
            (0x14C, "IsCowPregnancySlotOccupied"),
            (0x14D, "IsSheepPregnancySlotOccupied"),
            (0x14E, "GetFishCatchCount"),
            (0x14F, "GetLargestCaughtFishSize"),
            (0x150, "IsMapRegistered"),
            (0x151, "GetToolExperience"),
            (0x152, "CountFestivalWinningAnimals"),
        ] {
            assert_eq!(
                callables.scope.callable_map()[name].0 .0,
                id,
                "{target}: {name}"
            );
        }

        let script_table =
            parse_script_table("mary_script_table { TestMfomtTail, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestMfomtTail(void) { int caught = GetFishCatchCount(53); int size = GetLargestCaughtFishSize(53); int registered = IsMapRegistered(MAP_FARM); int experience = GetToolExperience(TOOL_KIND_FISHING_ROD); int winners = CountFestivalWinningAnimals(ANIMAL_KIND_DOG); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestMfomtTail")
                .unwrap();
        let source = format_named_script("TestMfomtTail", &raised).unwrap();
        assert!(
            source.contains("GetFishCatchCount(FISHING_RECORD_JAPANESE_HUCHEN)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GetLargestCaughtFishSize(FISHING_RECORD_JAPANESE_HUCHEN)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("IsMapRegistered(MAP_FARM)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("GetToolExperience(TOOL_KIND_FISHING_ROD)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("CountFestivalWinningAnimals(ANIMAL_KIND_DOG)"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert!(!callables
            .scope
            .callable_map()
            .contains_key("IsChickenIncubatorOccupied"));
    }
}

#[test]
fn complete_food_article_and_tool_id_tables_are_contiguous_on_all_four_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let food_type = constants.user_type("MaryFoodId").unwrap();

        for id in 0x00..=0xAA {
            assert!(
                constants.typed_int_const_name(food_type, id).is_some(),
                "{target}: missing food ID 0x{id:02X}"
            );
        }
        for (id, expected) in [
            (0x15, "FOOD_SPA_BOILED_EGG"),
            (0x38, "FOOD_QUEEN_OF_THE_NIGHT_OR_MYSTERY_FLOWER"),
            (0x42, "FOOD_FLOUR"),
            (0x44, "FOOD_MUFFIN_MIX_OR_RICE_FLOUR"),
            (0x6B, "FOOD_SCRAMBLED_EGGS_OR_JAPANESE_OMELET"),
            (0x75, "FOOD_APPLE_SOUFFLE"),
            (0x7B, "FOOD_DINNER_ROLL"),
            (0xA5, "FOOD_EGG_OVER_RICE_OR_EGG_BOWL"),
            (0xAA, "FOOD_POTATO_PANCAKES_OR_CROQUETTE"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(food_type, id),
                Some(expected),
                "{target}: official English food name for ID 0x{id:02X}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(food_type, 0xAB),
            Some("FOOD_NONE"),
            "{target}"
        );
        let article_type = constants.user_type("MaryArticleId").unwrap();
        let article_last = if target.contains("MFOMT") { 0x6A } else { 0x5F };
        for id in 0x00..=article_last {
            assert!(
                constants.typed_int_const_name(article_type, id).is_some(),
                "{target}: missing article ID 0x{id:02X}"
            );
        }
        if target.contains("MFOMT") {
            for (id, expected) in [
                (0x33, "ARTICLE_RECIPE_FRENCH_FRIES"),
                (0x3B, "ARTICLE_VANS_FAVORITE"),
                (0x4F, "ARTICLE_NEW_RECORD_1"),
                (0x5F, "ARTICLE_GOLDEN_LUMBER"),
                (0x63, "ARTICLE_DOG_DISK"),
                (0x68, "ARTICLE_DOCTORS_GOODIES"),
                (0x69, "ARTICLE_MYSTERY_TICKET"),
                (0x6A, "ARTICLE_NONE"),
            ] {
                assert_eq!(
                    constants.typed_int_const_name(article_type, id),
                    Some(expected),
                    "{target}: official English MFoMT article name for ID 0x{id:02X}"
                );
            }
        } else {
            assert_eq!(
                constants.typed_int_const_name(article_type, 0x4F),
                Some("ARTICLE_BAND_AID"),
                "{target}: official English FoMT article name"
            );
            assert_eq!(
                constants.typed_int_const_name(article_type, 0x60),
                None,
                "{target}: FoMT article table ends at 0x5F"
            );
        }

        let tool_type = constants.user_type("MaryToolId").unwrap();
        for id in 0x00..=0x51 {
            assert!(
                constants.typed_int_const_name(tool_type, id).is_some(),
                "{target}: missing tool ID 0x{id:02X}"
            );
        }
        for (id, expected) in [
            (0x05, "TOOL_SICKLE_CURSED"),
            (0x06, "TOOL_SICKLE_BLESSED"),
            (0x07, "TOOL_SICKLE_MYTHIC"),
            (0x2D, "TOOL_FISHING_ROD_CURSED"),
            (0x2E, "TOOL_FISHING_ROD_BLESSED"),
            (0x2F, "TOOL_FISHING_ROD_MYTHIC"),
            (0x48, "TOOL_CLIPPER_OR_CLIPPERS"),
            (0x4B, "TOOL_BLUE_FEATHER"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(tool_type, id),
                Some(expected),
                "{target}: tool ID 0x{id:02X}"
            );
        }
    }
}

#[test]
fn script_facing_item_absence_sentinels_are_distinct_from_physical_none_ids() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let food_type = constants.user_type("MaryFoodId").unwrap();
        let article_type = constants.user_type("MaryArticleId").unwrap();
        let tool_type = constants.user_type("MaryToolId").unwrap();
        assert_eq!(
            constants.typed_int_const_name(food_type, -1),
            Some("FOOD_NOT_PRESENT"),
            "{target}"
        );
        assert_eq!(
            constants.typed_int_const_name(article_type, -1),
            Some("ARTICLE_NOT_PRESENT"),
            "{target}"
        );
        assert_eq!(
            constants.typed_int_const_name(tool_type, -1),
            Some("TOOL_NOT_PRESENT"),
            "{target}"
        );
        assert_ne!(
            constants.const_int_value("ARTICLE_NONE"),
            Some(-1),
            "{target}: physical ARTICLE_NONE must not be conflated with the VM sentinel"
        );
        assert_ne!(
            constants.const_int_value("TOOL_NONE"),
            Some(-1),
            "{target}: physical TOOL_NONE must not be conflated with the VM sentinel"
        );
    }
}

#[test]
fn named_negative_sentinels_preserve_unary_negation_bytes_on_all_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestNegativeSentinels, };\n", &options)
                .unwrap();
        let numeric = parse_named_scripts(
            "void TestNegativeSentinels(void) {\n\
             if (GetPlayerHeldFoodId() == -1) { return; }\n\
             if (GetPlayerHeldArticleId() == -1) { return; }\n\
             if (GetPlayerHeldToolId() == -1) { return; }\n\
             if (GetPlayerHeldToolStackCount() == -1) { return; }\n\
             if (FindFoodInRucksack(0) == -1) { return; }\n\
             if (SelectFestivalAnimal(0) == -1) { return; }\n\
             if (RunHorseRace(0) == -1) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestNegativeSentinels",
        )
        .unwrap();
        let source = format_named_script("TestNegativeSentinels", &raised).unwrap();
        for symbol in [
            "FOOD_NOT_PRESENT",
            "ARTICLE_NOT_PRESENT",
            "TOOL_NOT_PRESENT",
            "HELD_TOOL_STACK_NOT_PRESENT",
            "RUCKSACK_SLOT_NOT_FOUND",
            "CHICKEN_SLOT_NONE",
            "HORSE_RACE_INTERFACE_CANCELLED",
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol}: {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}"
        );

        // A negative value produced by a positive push followed by Neg is a
        // distinct physical encoding. Keep it through the explicit Mary-C
        // form rather than confusing it with the default direct negative push.
        let mut negated = parse_named_scripts(
            "void TestNegativeSentinels(void) {\n\
             if (GetPlayerHeldFoodId() == -1) { return; }\n\
             }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let instructions = &mut negated.scripts[0].2.instructions;
        let direct = instructions
            .iter()
            .position(|instruction| *instruction == Ins::PushInt(-1))
            .unwrap();
        instructions[direct] = Ins::PushInt(1);
        instructions.insert(direct + 1, Ins::Neg);
        let raised = decompile_script_named(
            &negated.scripts[0].2,
            &callables.scope,
            "TestNegativeSentinels",
        )
        .unwrap();
        let source = format_named_script("TestNegativeSentinels", &raised).unwrap();
        assert!(
            source.contains("== mary_negated_int(FOOD_NOT_PRESENT)"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&negated.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: positive push followed by Neg"
        );
    }
}

#[test]
fn complete_shipping_product_table_uses_physical_product_semantics() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let product_type = constants.user_type("MaryProductId").unwrap();
        for id in 0x00..=0x66 {
            assert!(
                constants.typed_int_const_name(product_type, id).is_some(),
                "{target}: missing product ID 0x{id:02X}"
            );
        }
        for (id, expected) in [
            (0x03, "PRODUCT_CABBAGE"),
            (0x08, "PRODUCT_PINEAPPLE"),
            (0x0D, "PRODUCT_GREEN_PEPPER"),
            (0x5A, "PRODUCT_MYTHIC_STONE"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(product_type, id),
                Some(expected),
                "{target}: product ID 0x{id:02X}"
            );
        }

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestProducts, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestProducts(void) { int amount = GetAmountShipped(3); SetTextVariableToProductName(0, 90); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestProducts")
                .unwrap();
        let source = format_named_script("TestProducts", &raised).unwrap();
        assert!(
            source.contains("GetAmountShipped(PRODUCT_CABBAGE)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("SetTextVariableToProductName(TEXT_VARIABLE_1, PRODUCT_MYTHIC_STONE)"),
            "{target}: {source}"
        );
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}"
        );
    }
}

#[test]
fn complete_map_table_covers_every_physical_map_and_target_specific_none_sentinel() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let map_type = constants.user_type("MaryMapId").unwrap();

        let is_fomt = target.starts_with("MARY_FOMT_");
        let last_map = if is_fomt { 0x0233 } else { 0x0239 };
        for id in 0x0000..=last_map {
            assert!(
                constants.typed_int_const_name(map_type, id).is_some(),
                "{target}: missing map ID 0x{id:04X}"
            );
        }
        let mine_shift = if is_fomt { 0 } else { 6 };
        for floor in 0..=255 {
            let spring_name = format!("MAP_SPRING_MINE_FLOOR_{floor}");
            let lake_name = format!("MAP_LAKE_MINE_FLOOR_{floor}");
            assert_eq!(
                constants.typed_int_const_name(map_type, 0x0034 + mine_shift + floor),
                Some(spring_name.as_str()),
                "{target}: Spring Mine floor {floor}"
            );
            assert_eq!(
                constants.typed_int_const_name(map_type, 0x0134 + mine_shift + floor),
                Some(lake_name.as_str()),
                "{target}: Lake Mine floor {floor}"
            );
        }
        if !is_fomt {
            for (scene, name) in [
                "MAP_MFOMT_OPENING_PLAYER_HOME",
                "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_1",
                "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_2",
                "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_3",
                "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_4",
                "MAP_MFOMT_OPENING_PHONE_CALL_SCENE",
            ]
            .into_iter()
            .enumerate()
            {
                assert_eq!(
                    constants.typed_int_const_name(map_type, 0x0034 + scene as i64),
                    Some(name),
                    "{target}: MFoMT opening scene {scene}"
                );
            }
        }
        let map_none = if is_fomt { 0x0234 } else { 0x023A };
        assert_eq!(
            constants.typed_int_const_name(map_type, map_none),
            Some("MAP_NONE"),
            "{target}"
        );
    }
}

#[test]
fn legacy_mfomt_opening_map_names_compile_but_decompile_semantically() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestOpeningMaps, };\n", &options).unwrap();
        let legacy = parse_named_scripts(
            "void TestOpeningMaps(void) { ChangeMap(MAP_MFOMT_OPENING_SCENE_0, 0, 0); ChangeMap(MAP_MFOMT_OPENING_SCENE_1, 0, 0); ChangeMap(MAP_MFOMT_OPENING_SCENE_2, 0, 0); ChangeMap(MAP_MFOMT_OPENING_SCENE_3, 0, 0); ChangeMap(MAP_MFOMT_OPENING_SCENE_4, 0, 0); ChangeMap(MAP_MFOMT_OPENING_SCENE_5, 0, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&legacy.scripts[0].2, &callables.scope, "TestOpeningMaps")
                .unwrap();
        let source = format_named_script("TestOpeningMaps", &raised).unwrap();
        for name in [
            "MAP_MFOMT_OPENING_PLAYER_HOME",
            "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_1",
            "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_2",
            "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_3",
            "MAP_MFOMT_OPENING_ADVERTISEMENT_MONTAGE_4",
            "MAP_MFOMT_OPENING_PHONE_CALL_SCENE",
        ] {
            assert!(
                source.contains(name),
                "{target}: missing {name} in {source}"
            );
        }
        let semantic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&legacy.scripts[0].2),
            encode_script(&semantic.scripts[0].2),
            "{target}: compatibility aliases changed bytecode"
        );
    }
}

#[test]
fn constant_arithmetic_in_typed_arguments_preserves_bytecode_and_types_the_base() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestTypedArithmetic, };\n", &options).unwrap();
        let lake_floor_zero = if target.starts_with("MARY_FOMT_") {
            0x0134
        } else {
            0x013A
        };
        let numeric_source = format!(
            "void TestTypedArithmetic(void) {{ ChangeMap({lake_floor_zero} + 9, 160, 24); }}\n"
        );
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestTypedArithmetic",
        )
        .unwrap();
        let source = format_named_script("TestTypedArithmetic", &raised).unwrap();
        assert!(
            source.contains("ChangeMap(MAP_LAKE_MINE_FLOOR_0 + 9, X(160), Y(24))"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: folding typed constant arithmetic changed bytecode"
        );
    }
}

#[test]
fn portrait_table_covers_all_0xb8_physical_slots_on_all_four_targets() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let portrait_type = constants.user_type("MaryTalkPortraitId").unwrap();
        let character_type = constants.user_type("MaryCharacterId").unwrap();

        for id in 1..=42 {
            assert!(
                constants.typed_int_const_name(character_type, id).is_some(),
                "{target}: missing character ID {id}"
            );
        }
        for id in 0..0xB8 {
            assert!(
                constants.typed_int_const_name(portrait_type, id).is_some(),
                "{target}: missing portrait ID 0x{id:02X}"
            );
        }
        assert!(
            constants
                .typed_int_const_name(portrait_type, 0xB8)
                .is_none(),
            "{target}: portrait table must stop after 0xB7"
        );
        for (id, expected) in [
            (0x00, "TALK_PORTRAIT_RICK_NORMAL"),
            (0x06, "TALK_PORTRAIT_RICK_WEDDING"),
            (0x12, "TALK_PORTRAIT_BABY_NORMAL"),
            (0x2A, "TALK_PORTRAIT_STAID_NORMAL"),
            (0x37, "TALK_PORTRAIT_TIMID_SCARED"),
            (0x47, "TALK_PORTRAIT_JEFF_HURT"),
            (0x84, "TALK_PORTRAIT_STU_CRYING"),
            (0xA5, "TALK_PORTRAIT_GRAY_SHY"),
            (0xB7, "TALK_PORTRAIT_BARLEY_AFRAID"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(portrait_type, id),
                Some(expected),
                "{target}: portrait ID 0x{id:02X}"
            );
        }
        assert_eq!(
            constants.typed_int_const_name(character_type, 34),
            Some("CHARACTER_LOU_OR_RUBY"),
            "{target}: canonical guest character ID"
        );
        assert_eq!(
            constants.typed_int_const_name(portrait_type, 94),
            Some("TALK_PORTRAIT_LOU_OR_RUBY_NORMAL"),
            "{target}: canonical guest portrait"
        );
        assert_eq!(
            constants.typed_int_const_name(character_type, 33),
            Some("CHARACTER_VAN"),
            "{target}: traveling merchant character"
        );
        assert_eq!(
            constants.typed_int_const_name(portrait_type, 141),
            Some("TALK_PORTRAIT_VAN_NORMAL"),
            "{target}: traveling merchant portrait"
        );
    }
}

#[test]
fn lou_or_ruby_character_symbols_do_not_regress_to_one_localization_name() {
    let constants = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let scripts = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    for forbidden in [
        "CHARACTER_LOU =",
        "CHARACTER_RUBY =",
        "ENTITY_LOU =",
        "ENTITY_RUBY =",
        "TALK_PORTRAIT_LOU_NORMAL",
        "TALK_PORTRAIT_LOU_HAPPY",
        "TALK_PORTRAIT_LOU_ANGRY",
        "TALK_PORTRAIT_LOU_AFRAID",
        "TALK_PORTRAIT_RUBY_NORMAL",
        "TALK_PORTRAIT_RUBY_HAPPY",
        "TALK_PORTRAIT_RUBY_ANGRY",
        "TALK_PORTRAIT_RUBY_AFRAID",
        "VAR_LOU_INTRODUCTION_EVENT_STATE",
        "VAR_RUBY_INTRODUCTION_EVENT_STATE",
        "VAR_GAMECUBE_LINK_LOU_DIALOGUE_PENDING",
        "VAR_GAMECUBE_LINK_LOU_RECIPES_LEARNED",
        "VAR_GAMECUBE_LINK_LOU_REWARD_DIALOGUES_PENDING",
        "VAR_GAMECUBE_LINK_LOU_RECIPE_UNLOCKS_PENDING",
        "VAR_GAMECUBE_LINK_LOU_INTRODUCTION_AVAILABLE",
        "VAR_GAMECUBE_LINK_RUBY_DIALOGUE_PENDING",
        "VAR_GAMECUBE_LINK_RUBY_RECIPES_LEARNED",
        "VAR_GAMECUBE_LINK_RUBY_REWARD_DIALOGUES_PENDING",
        "VAR_GAMECUBE_LINK_RUBY_RECIPE_UNLOCKS_PENDING",
        "VAR_GAMECUBE_LINK_RUBY_INTRODUCTION_AVAILABLE",
    ] {
        assert!(
            !constants.contains(forbidden),
            "person-related constant identifier regressed to one localization: {forbidden}"
        );
    }

    for forbidden in [
        "EventScript_NPCEvent_Lou_",
        "EventScript_NPCEvent_Ruby_",
        "gText_NPCEvent_Lou_",
        "gText_NPCEvent_Ruby_",
    ] {
        assert!(
            !scripts.contains(forbidden),
            "person-related script/text identifier regressed to one localization: {forbidden}"
        );
    }

    // Ruby the gemstone and the ruby wedding anniversary are separate concepts
    // and intentionally retain their official English names.
    assert!(constants.contains("ARTICLE_RUBY ="));
    assert!(constants.contains("REFERENCE_PAGE_ANNIVERSARY_40_RUBY ="));
}

#[test]
fn audio_sequence_table_preserves_all_211_physical_slots() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let sequence_type = constants.user_type("MaryAudioSequenceId").unwrap();

        for id in 0..211 {
            let semantic = match id {
                1 => Some("AUDIO_BGM_SPRING"),
                2 => Some("AUDIO_BGM_SUMMER"),
                3 => Some("AUDIO_BGM_AUTUMN"),
                4 => Some("AUDIO_BGM_WINTER"),
                5 => Some("AUDIO_BGM_SADNESS"),
                6 => Some("AUDIO_BGM_WEDDING"),
                7 => Some("AUDIO_BGM_FESTIVAL"),
                8 => Some("AUDIO_BGM_MUSIC_FESTIVAL"),
                9 => Some("AUDIO_BGM_ANIMAL_FESTIVAL"),
                10 => Some("AUDIO_AMBIENCE_RAIN"),
                11 => Some("AUDIO_AMBIENCE_STORM"),
                12 => Some("AUDIO_AMBIENCE_BEACH"),
                13 => Some("AUDIO_BGM_MINERAL_TOWN"),
                14 => Some("AUDIO_AMBIENCE_NIGHT"),
                15 => Some("AUDIO_BGM_MUSIC_FESTIVAL_PERFORMANCE"),
                16 => Some("AUDIO_BGM_RIVAL_EVENT"),
                17 => Some("AUDIO_BGM_LOVE_EVENT"),
                18 => Some("AUDIO_RECORD_SPRING_SONG"),
                19 => Some("AUDIO_RECORD_TOWN_SPIRIT"),
                20 => Some("AUDIO_RECORD_FLOWER_BUD_FALL"),
                21 => Some("AUDIO_RECORD_64_MEMORIES"),
                22 => Some("AUDIO_RECORD_MARINE_JAZZ"),
                23 => Some("AUDIO_RECORD_BUTTERFLY"),
                24 => Some("AUDIO_RECORD_SUMMER_MEMORIES"),
                25 => Some("AUDIO_RECORD_AUTUMN_JOY"),
                26 => Some("AUDIO_RECORD_QUIET_WINTER"),
                27 => Some("AUDIO_RECORD_GRIFFIN_BLUE"),
                33 => Some("AUDIO_BGM_HORSE_RACE_MINIGAME"),
                34 => Some("AUDIO_BGM_HARVEST_SPRITE_MINIGAME"),
                35 => Some("AUDIO_BGM_TITLE_SCREEN"),
                36 => Some("AUDIO_BGM_CREDITS"),
                37 => Some("AUDIO_BGM_CHILDHOOD"),
                38 if target.contains("MFOMT") => Some("AUDIO_RECORD_NEW_RECORD_1"),
                39 if target.contains("MFOMT") => Some("AUDIO_RECORD_NEW_RECORD_2"),
                40 if target.contains("MFOMT") => Some("AUDIO_RECORD_NEW_RECORD_3"),
                41 if target.contains("MFOMT") => Some("AUDIO_RECORD_NEW_RECORD_4"),
                42 if target.contains("MFOMT") => Some("AUDIO_RECORD_NEW_RECORD_5"),
                101 => Some("AUDIO_SFX_EAT"),
                102 => Some("AUDIO_SFX_DRINK"),
                105 => Some("AUDIO_SFX_ENTER_HOT_SPRING"),
                106 => Some("AUDIO_SFX_PICK_UP_ITEM"),
                108 => Some("AUDIO_SFX_THROW_ITEM"),
                109 => Some("AUDIO_SFX_THROWN_ITEM_LANDS"),
                110 => Some("AUDIO_SFX_SHIPMENT_DEPOSIT"),
                111 => Some("AUDIO_SFX_SICKLE_CUT"),
                118 => Some("AUDIO_SFX_HOE_TILL"),
                121 => Some("AUDIO_SFX_LIGHT_FIREPLACE"),
                124 => Some("AUDIO_SFX_ADD_ITEM_TO_FIRE"),
                125 => Some("AUDIO_SFX_HAMMER_SMALL_STONE"),
                126 => Some("AUDIO_SFX_HAMMER_LARGE_STONE"),
                127 => Some("AUDIO_SFX_HAMMER_HUGE_STONE"),
                132 => Some("AUDIO_SFX_AXE_BRANCH_CHOP"),
                133 => Some("AUDIO_SFX_AXE_STUMP_CHOP"),
                139 => Some("AUDIO_SFX_WATERING_CAN_POUR"),
                146 => Some("AUDIO_SFX_WATER_SPLASH"),
                147 => Some("AUDIO_SFX_BRUSH_LIVESTOCK"),
                148 => Some("AUDIO_SFX_SOW_SEEDS"),
                149 => Some("AUDIO_SFX_DOCTOR_EXAMINATION"),
                156 => Some("AUDIO_SFX_OPEN_DOOR"),
                157 => Some("AUDIO_SFX_HEAL_OR_PURIFY"),
                159 => Some("AUDIO_SFX_MILK_COW"),
                160 => Some("AUDIO_SFX_SHEAR_SHEEP"),
                161 => Some("AUDIO_SFX_COW_MOO"),
                163 => Some("AUDIO_SFX_SHEEP_BLEAT"),
                167 => Some("AUDIO_SFX_FOAL_NEIGH"),
                169 => Some("AUDIO_SFX_DOG_BARK"),
                172 => Some("AUDIO_SFX_BABY_CRY"),
                175 => Some("AUDIO_SFX_TIME_PASSES"),
                176 => Some("AUDIO_SFX_SUCCESS"),
                177 => Some("AUDIO_SFX_FIREWORK_LAUNCH"),
                178 => Some("AUDIO_SFX_FIREWORK_EXPLOSION"),
                179 => Some("AUDIO_SFX_CEREMONIAL_CHIME"),
                181 => Some("AUDIO_SFX_VICTORY"),
                182 => Some("AUDIO_SFX_ITEM_OBTAINED"),
                184 => Some("AUDIO_SFX_ATTENTION_CHIME"),
                188 => Some("AUDIO_SFX_STAR_SPARKLE"),
                190 => Some("AUDIO_SFX_HARVEST_GODDESS_APPEARS"),
                192 => Some("AUDIO_SFX_QUESTION_EMOTE"),
                193 => Some("AUDIO_SFX_APPLAUSE"),
                199 => Some("AUDIO_SFX_INCORRECT_ANSWER"),
                200 => Some("AUDIO_SFX_KAPPA_SURPRISE"),
                204 => Some("AUDIO_SFX_CHICKEN_CLUCK"),
                205 => Some("AUDIO_SFX_CLOSE_DOOR"),
                _ => None,
            };
            let numbered = format!("AUDIO_SEQUENCE_{id:03}");
            let canonical = semantic.unwrap_or(&numbered);
            assert_eq!(
                constants.typed_int_const_name(sequence_type, id),
                Some(canonical),
                "{target}: missing or displaced audio sequence ID {id}"
            );
            assert_eq!(
                constants.const_int_value(&numbered),
                Some(id),
                "{target}: numbered audio compatibility name {numbered}"
            );
        }
        assert!(
            constants.typed_int_const_name(sequence_type, 211).is_none(),
            "{target}: audio sequence table must stop after ID 210"
        );

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestAudio, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestAudio(void) { PlayBGM(0, 0); PlaySong(2, 210); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestAudio").unwrap();
        let source = format_named_script("TestAudio", &raised).unwrap();
        assert!(
            source.contains("PlayBGM(AUDIO_START, AUDIO_SEQUENCE_000)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("PlaySong(AUDIO_START_OR_CONTINUE, AUDIO_SEQUENCE_210)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: symbolic audio IDs changed emitted bytecode"
        );
    }
}

#[test]
fn ambiguous_reused_audio_sequences_remain_numbered() {
    const VANILLA_REFERENCED_NUMBERED_AUDIO: [i64; 7] = [131, 138, 144, 145, 170, 173, 174];

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let sequence_type = constants.user_type("MaryAudioSequenceId").unwrap();

        // These slots have useful call-time evidence, but not one reliable,
        // scene-independent English identity. Keep their physical names until
        // the sequence data and every reuse can support a narrower name.
        for id in VANILLA_REFERENCED_NUMBERED_AUDIO {
            let numbered = format!("AUDIO_SEQUENCE_{id:03}");
            assert_eq!(
                constants.typed_int_const_name(sequence_type, id),
                Some(numbered.as_str()),
                "{target}: ambiguous audio sequence {id} acquired a misleading semantic name"
            );
        }
    }
}

#[test]
fn fishing_record_table_preserves_all_59_physical_slots() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let record_type = constants.user_type("MaryFishingRecordId").unwrap();

        for id in 0..59 {
            assert!(
                constants.typed_int_const_name(record_type, id).is_some(),
                "{target}: missing fishing-record ID {id}"
            );
        }
        assert!(
            constants.typed_int_const_name(record_type, 59).is_none(),
            "{target}: fishing-record table must stop after ID 58"
        );
        for (id, expected) in [
            (0, "FISHING_RECORD_PIRATE_FORTUNE"),
            (7, "FISHING_RECORD_BOOTS"),
            (8, "FISHING_RECORD_ROCK_TROUT"),
            (22, "FISHING_RECORD_SILVER_CARP_22"),
            (37, "FISHING_RECORD_SILVER_CARP_37"),
            (52, "FISHING_RECORD_LAKE_SMELT"),
            (53, "FISHING_RECORD_JAPANESE_HUCHEN"),
            (58, "FISHING_RECORD_SQUID"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(record_type, id),
                Some(expected),
                "{target}: fishing-record ID {id}"
            );
        }
    }
}

#[test]
fn link_milestone_domains_preserve_direction_and_target_ranges() {
    let local_names = [
        "LOCAL_LINK_MILESTONE_VAN_ALBUM_PROGRESS",
        "LOCAL_LINK_MILESTONE_LOU_OR_RUBY_RECIPE_PROGRESS",
        "LOCAL_LINK_MILESTONE_DOCTOR_FORGET_ME_NOT_VALLEY_DIALOGUE",
        "LOCAL_LINK_MILESTONE_MANNA_AJA_UPDATE_DIALOGUE",
        "LOCAL_LINK_MILESTONE_LILLIA_ROD_LETTER_DIALOGUE",
        "LOCAL_LINK_MILESTONE_ZACK_FISHING_ROD_DIALOGUE",
        "LOCAL_LINK_MILESTONE_THOMAS_KANO_DIALOGUE",
        "LOCAL_LINK_MILESTONE_GOTZ_LOUIS_DIALOGUE",
        "LOCAL_LINK_MILESTONE_MAY_JOANNA_PHONE_UPDATE_DIALOGUE",
        "LOCAL_LINK_MILESTONE_DOUG_GOURMET_DIALOGUE",
        "LOCAL_LINK_MILESTONE_VACATION_VILLA_BUILT",
        "LOCAL_LINK_MILESTONE_ALL_VILLAGERS_MAX_FRIENDSHIP",
        "LOCAL_LINK_MILESTONE_ALL_CROPS_SHIPPED",
        "LOCAL_LINK_MILESTONE_ALL_FARM_ANIMALS_MAX_AFFECTION",
        "LOCAL_LINK_MILESTONE_ALL_MINERALS_SHIPPED",
        "LOCAL_LINK_MILESTONE_ALL_FISH_SPECIES_CAUGHT",
        "LOCAL_LINK_MILESTONE_ONE_MILLION_STEPS",
        "LOCAL_LINK_MILESTONE_FIFTY_YEARS_PLAYED",
        "LOCAL_LINK_MILESTONE_TEN_THOUSAND_FISH_CAUGHT",
        "LOCAL_LINK_MILESTONE_GOLDEN_LUMBER_ON_FARM",
        "LOCAL_LINK_MILESTONE_MOUNTAIN_AND_SEASIDE_COTTAGES_BUILT",
        "LOCAL_LINK_MILESTONE_MYTHIC_TOOL_OBTAINED",
        "LOCAL_LINK_MILESTONE_TEN_FRISBEE_TOURNAMENT_WINS",
        "LOCAL_LINK_MILESTONE_TEN_HORSE_RACE_WINS",
        "LOCAL_LINK_MILESTONE_TEN_CHICKEN_FESTIVAL_WINS",
        "LOCAL_LINK_MILESTONE_TEN_COW_FESTIVAL_WINS",
        "LOCAL_LINK_MILESTONE_TEN_SHEEP_FESTIVAL_WINS",
        "LOCAL_LINK_MILESTONE_TEN_COOKING_FESTIVAL_WINS",
    ];
    let received_names = [
        "RECEIVED_LINK_MILESTONE_VAN_INTRODUCTION_AND_ALBUM_PROGRESS",
        "",
        "RECEIVED_LINK_MILESTONE_AWL_PLAYER_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_RESERVED_03",
        "RECEIVED_LINK_MILESTONE_AWL_TAKAKURA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_ROMANA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_LUMINA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_SEBASTIAN_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_WALLY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_CHRIS_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_HUGH_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_GRANT_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_SAMANTHA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_KATE_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_GALEN_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_NINA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_DARYL_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_GUSTAFA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_CODY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_KASSEY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_PATRICK_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_MURREY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_TIM_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_LOU_OR_RUBY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_NAMI_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_ROCK_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_GRIFFIN_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_MUFFY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_CARTER_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_FLORA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_VESTA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_MARLIN_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_CELIA_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_HARDY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_VAN_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_MOOKY_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_NAK_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_NIC_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_AWL_FLAK_PROFILE_PROGRESS",
        "RECEIVED_LINK_MILESTONE_VAN_DIALOGUE_UPDATE",
        "",
        "RECEIVED_LINK_MILESTONE_ELLEN_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_BARLEY_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_RICK_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_THOMAS_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_JEFF_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_CARTER_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_GOTZ_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_HARVEST_SPRITES_DIALOGUE_UPDATE",
        "RECEIVED_LINK_MILESTONE_THOMAS_COOKING_RECORD_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_HARRIS_MURREY_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_CARTER_STRANGE_FOREST_CREATURE_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_STU_VALLEY_LIZARD_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_BASIL_TARTAN_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_SASHA_HUGH_TRACK_RECORD_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_ZACK_TAKAKURA_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_JEFF_CODY_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_BARLEY_LAND_TURTLE_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_LILLIA_CHIHUAHUA_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_DOUG_GRIFFIN_DIALOGUE",
        "RECEIVED_LINK_MILESTONE_MAY_MOVING_TEDDY_BEAR_DIALOGUE",
    ];
    for (target, local_max, received_max) in [
        ("MARY_FOMT_US", 21, 48),
        ("MARY_FOMT_JP", 21, 48),
        ("MARY_MFOMT_US", 27, 60),
        ("MARY_MFOMT_JP", 27, 60),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let local_type = constants.user_type("MaryLocalLinkMilestoneId").unwrap();
        let received_type = constants.user_type("MaryReceivedLinkMilestoneId").unwrap();

        for id in 0..=local_max {
            assert_eq!(
                constants.typed_int_const_name(local_type, id),
                Some(local_names[id as usize]),
                "{target}: local milestone {id}"
            );
        }
        assert!(
            constants
                .typed_int_const_name(local_type, local_max + 1)
                .is_none(),
            "{target}: local milestone domain exceeds its physical range"
        );
        for id in 0..=received_max {
            let expected = match id {
                1 => "RECEIVED_LINK_MILESTONE_LOU_OR_RUBY_INTRODUCTION_AND_RECIPE_PROGRESS",
                40 => "RECEIVED_LINK_MILESTONE_LOU_OR_RUBY_DIALOGUE_UPDATE",
                _ => received_names[id as usize],
            };
            assert_eq!(
                constants.typed_int_const_name(received_type, id),
                Some(expected),
                "{target}: received milestone {id}"
            );
        }
        assert!(
            constants
                .typed_int_const_name(received_type, received_max + 1)
                .is_none(),
            "{target}: received milestone domain exceeds its physical range"
        );

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestLink, };\n", &options).unwrap();
        let numeric_source = format!(
            "void TestLink(void) {{ if (HasLocalLinkMilestone({local_max}) || HasReceivedLinkMilestone({received_max})) {{ ClearLocalLinkMilestone({local_max}); }} else {{ SetLocalLinkMilestone(0); }} }}\n"
        );
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestLink").unwrap();
        let source = format_named_script("TestLink", &raised).unwrap();
        assert!(
            source.contains(local_names[local_max as usize]),
            "{target}: {source}"
        );
        assert!(
            source.contains(received_names[received_max as usize]),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: symbolic milestone IDs changed emitted bytecode"
        );
    }
}

#[test]
fn screen_fade_domains_preserve_all_native_selectors() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let style_type = constants.user_type("MaryScreenFadeStyle").unwrap();
        let speed_type = constants.user_type("MaryScreenFadeSpeed").unwrap();
        for (id, name) in [
            (0, "SCREEN_FADE_STYLE_BLACK"),
            (1, "SCREEN_FADE_STYLE_WHITE"),
            (2, "SCREEN_FADE_STYLE_BLACK_MOSAIC"),
            (3, "SCREEN_FADE_STYLE_WHITE_MOSAIC"),
            (4, "SCREEN_FADE_STYLE_MOSAIC"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(style_type, id),
                Some(name),
                "{target}: fade style {id}"
            );
        }
        assert!(constants.typed_int_const_name(style_type, 5).is_none());
        for (id, name) in [
            (0, "SCREEN_FADE_SPEED_FAST"),
            (1, "SCREEN_FADE_SPEED_NORMAL"),
            (2, "SCREEN_FADE_SPEED_SLOW"),
        ] {
            assert_eq!(
                constants.typed_int_const_name(speed_type, id),
                Some(name),
                "{target}: fade speed {id}"
            );
        }
        assert!(constants.typed_int_const_name(speed_type, 3).is_none());

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFade, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestFade(void) { FadeOutScreen(4, 2); FadeInScreen(0, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestFade").unwrap();
        let source = format_named_script("TestFade", &raised).unwrap();
        assert!(
            source.contains("FadeOutScreen(SCREEN_FADE_STYLE_MOSAIC, SCREEN_FADE_SPEED_SLOW)"),
            "{target}: {source}"
        );
        assert!(
            source.contains("FadeInScreen(SCREEN_FADE_STYLE_BLACK, SCREEN_FADE_SPEED_FAST)"),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: symbolic fade selectors changed emitted bytecode"
        );
    }
}

#[test]
fn game_variable_domains_preserve_every_physical_slot() {
    let mut failures = Vec::new();
    for (target, last_slot) in [
        ("MARY_FOMT_US", 589),
        ("MARY_FOMT_JP", 589),
        ("MARY_MFOMT_US", 727),
        ("MARY_MFOMT_JP", 727),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let missing = (0..=last_slot)
            .filter(|id| constants.typed_int_const_name(variable_type, *id).is_none())
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            failures.push(format!("{target}: missing variable slots {missing:?}"));
        }
        assert!(
            constants
                .typed_int_const_name(variable_type, last_slot + 1)
                .is_none(),
            "{target}: variable domain exceeds its verified physical range"
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn game_variable_completeness_check_detects_a_removed_target_slot() {
    let source = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    for (target, declaration, id) in [
        (
            "MARY_FOMT_US",
            "    VAR_HARVEST_FESTIVAL_ACTIVE = 420,\n",
            420,
        ),
        (
            "MARY_MFOMT_US",
            "    VAR_HARVEST_FESTIVAL_ACTIVE = 464,\n",
            464,
        ),
    ] {
        let modified = source.replacen(declaration, "", 1).replacen(
            "mary_var_type(VAR_HARVEST_FESTIVAL_ACTIVE, MaryFestivalActivityPhase);\n",
            "",
            1,
        );
        assert_ne!(modified, source, "test fixture declaration was not found");
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&modified, &options).unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        assert!(
            constants.typed_int_const_name(variable_type, id).is_none(),
            "{target}: removing variable {id} was masked by another constant domain"
        );
    }
}

#[test]
fn text_variable_domain_preserves_all_four_charmap_slots() {
    for (target, string_setter_id) in [
        ("MARY_FOMT_US", 0x03A),
        ("MARY_FOMT_JP", 0x03A),
        ("MARY_MFOMT_US", 0x03B),
        ("MARY_MFOMT_JP", 0x03B),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let slot_type = constants.user_type("MaryTextVariableSlot").unwrap();
        for id in 0..4 {
            assert_eq!(
                constants.typed_int_const_name(slot_type, id),
                Some(format!("TEXT_VARIABLE_{}", id + 1).as_str()),
                "{target}: text variable slot {id}"
            );
        }
        assert!(constants.typed_int_const_name(slot_type, 4).is_none());

        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        assert_eq!(
            callables.scope.callable_map()["SetTextVariableString"].0 .0,
            string_setter_id,
            "{target}"
        );
        let script_table =
            parse_script_table("mary_script_table { TestTextVariables, };\n", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestTextVariables(void) { SetTextVariableNumber(0, 12); SetTextVariableNumberFieldWidth(1, 3, 2); SetTextVariableString(2, \"x\"); SetTextVariableToProductName(3, 0); }\n",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestTextVariables")
                .unwrap();
        let source = format_named_script("TestTextVariables", &raised).unwrap();
        for symbol in [
            "TEXT_VARIABLE_1",
            "TEXT_VARIABLE_2",
            "TEXT_VARIABLE_3",
            "TEXT_VARIABLE_4",
        ] {
            assert!(source.contains(symbol), "{target}: {source}");
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: symbolic text-variable slots changed emitted bytecode"
        );
    }
}

#[test]
fn newly_audited_game_variable_domains_print_symbols_and_round_trip() {
    for (target, cooking_var, hidden_candidate) in [
        (
            "MARY_FOMT_US",
            "VAR_COOKING_FESTIVAL_DISH_CATEGORY",
            "HIDDEN_MARRIAGE_CANDIDATE_HARVEST_GODDESS",
        ),
        (
            "MARY_FOMT_JP",
            "VAR_COOKING_FESTIVAL_DISH_CATEGORY",
            "HIDDEN_MARRIAGE_CANDIDATE_HARVEST_GODDESS",
        ),
        (
            "MARY_MFOMT_US",
            "VAR_COOKING_FESTIVAL_DISH_CATEGORY",
            "HIDDEN_MARRIAGE_CANDIDATE_KAPPA",
        ),
        (
            "MARY_MFOMT_JP",
            "VAR_COOKING_FESTIVAL_DISH_CATEGORY",
            "HIDDEN_MARRIAGE_CANDIDATE_KAPPA",
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestDomains, };\n", &options).unwrap();
        let numeric_source = format!(
            "void TestDomains(void) {{ VarSet({cooking_var}, 3); VarSet(VAR_STARRY_NIGHT_FESTIVAL_HOST_INDEX, 5); VarSet(VAR_HARVEST_GODDESS_REQUESTED_ITEM_INDEX, 8); VarSet(VAR_HARVEST_GODDESS_CHOSEN_AS_FAVORITE_VALUE, 5); }}\n"
        );
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestDomains").unwrap();
        let source = format_named_script("TestDomains", &raised).unwrap();
        for symbol in [
            "COOKING_FESTIVAL_DISH_CATEGORY_NOODLES",
            "STARRY_NIGHT_HOST_MARY_HOUSEHOLD",
            "HARVEST_GODDESS_REQUESTED_ITEM_AEPFE_APPLE",
            hidden_candidate,
        ] {
            assert!(
                source.contains(symbol),
                "{target}: missing {symbol} in {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: audited variable symbols changed emitted bytecode"
        );
    }
}

#[test]
fn harvest_adjacent_unknown_slot_uses_the_proven_boolean_domain_on_all_targets() {
    for (target, variable) in [
        ("MARY_FOMT_US", "VAR_UNKNOWN_SLOT_423"),
        ("MARY_FOMT_JP", "VAR_UNKNOWN_SLOT_423"),
        ("MARY_MFOMT_US", "VAR_UNKNOWN_SLOT_467"),
        ("MARY_MFOMT_JP", "VAR_UNKNOWN_SLOT_467"),
        ("MARY_MFOMT_US", "VAR_UNKNOWN_SLOT_697"),
        ("MARY_MFOMT_JP", "VAR_UNKNOWN_SLOT_697"),
        ("MARY_MFOMT_US", "VAR_UNKNOWN_SLOT_705"),
        ("MARY_MFOMT_JP", "VAR_UNKNOWN_SLOT_705"),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestHarvestResult, };\n", &options).unwrap();
        let numeric_source = format!(
            "void TestHarvestResult(void) {{ VarSet({variable}, 0); if (VarGet({variable}) == 1) {{ return; }} }}\n"
        );
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestHarvestResult")
                .unwrap();
        let source = format_named_script("TestHarvestResult", &raised).unwrap();
        assert!(
            source.contains(&format!("VarSet({variable}, FALSE)")),
            "{target}: {source}"
        );
        assert!(
            source.contains(&format!("VarGet({variable}) == TRUE")),
            "{target}: {source}"
        );
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: boolean result symbols changed emitted bytecode"
        );
    }
}

#[test]
fn unknown_two_bit_state_preserves_unnamed_value_and_local_reuse() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let scripts =
            parse_script_table("mary_script_table { TestUnknownState, };", &options).unwrap();
        let variables = if target.starts_with("MARY_MFOMT_") {
            ["VAR_UNKNOWN_SLOT_232", "VAR_UNKNOWN_SLOT_233"]
        } else {
            ["VAR_UNKNOWN_SLOT_224", "VAR_UNKNOWN_SLOT_225"]
        };
        for variable in variables {
            let input = format!("void TestUnknownState(void) {{ int state; state = VarGet({variable}); switch (state) {{ case 1: VarSet({variable}, 3); break; case 3: if (state == 3) {{ VarSet({variable}, 2); }} break; }} state = GetKnownRecipeCount(); if (state == 2) {{ return; }} }}");
            let parsed = parse_named_scripts(&input, &options, &callables.scope, &scripts).unwrap();
            let raised =
                decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestUnknownState")
                    .unwrap();
            let output = format_named_script("TestUnknownState", &raised).unwrap();
            assert!(output.contains("case 1:"), "{target}: {output}");
            assert!(output.contains("case 3:"), "{target}: {output}");
            assert!(
                output.contains(&format!("VarSet({variable}, 3)")),
                "{target}: {output}"
            );
            assert!(
                output.contains(&format!("VarSet({variable}, 2)")),
                "{target}: {output}"
            );
            assert!(!output.contains("EVENT_LIFECYCLE"), "{target}: {output}");
            let quantity_phase = output.split("GetKnownRecipeCount();").nth(1).unwrap();
            assert!(quantity_phase.contains("== 2"), "{target}: {output}");
            assert!(
                !quantity_phase.contains("EVENT_LIFECYCLE"),
                "{target}: {output}"
            );
            let rebuilt =
                parse_named_scripts(&output, &options, &callables.scope, &scripts).unwrap();
            assert_eq!(
                encode_script(&parsed.scripts[0].2),
                encode_script(&rebuilt.scripts[0].2)
            );
        }
    }
}

#[test]
fn mfomt_unknown_booleans_propagate_through_delayed_local_use() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let scripts =
            parse_script_table("mary_script_table { TestUnknownBool, };", &options).unwrap();
        for variable in ["VAR_UNKNOWN_SLOT_697", "VAR_UNKNOWN_SLOT_705"] {
            let input = format!("void TestUnknownBool(void) {{ int state; state = VarGet({variable}); if (state == 1) {{ VarSet({variable}, 0); }} else {{ VarSet({variable}, 1); }} if (state == 0) {{ VarSet({variable}, 1); }} state = GetKnownRecipeCount(); if (state == 1) {{ return; }} }}");
            let parsed = parse_named_scripts(&input, &options, &callables.scope, &scripts);
            if target.starts_with("MARY_FOMT_") {
                assert!(parsed.is_err(), "{target}: MFoMT-only symbol leaked");
                continue;
            }
            let parsed = parsed.unwrap();
            let raised =
                decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestUnknownBool")
                    .unwrap();
            let output = format_named_script("TestUnknownBool", &raised).unwrap();
            assert!(output.contains("== TRUE"), "{target}: {output}");
            assert!(output.contains("== FALSE"), "{target}: {output}");
            let quantity_phase = output.split("GetKnownRecipeCount();").nth(1).unwrap();
            assert!(quantity_phase.contains("== 1"), "{target}: {output}");
            assert!(!quantity_phase.contains("TRUE"), "{target}: {output}");
            assert!(
                output.contains(&format!("VarSet({variable}, FALSE)")),
                "{output}"
            );
            assert!(
                output.contains(&format!("VarSet({variable}, TRUE)")),
                "{output}"
            );
            let rebuilt =
                parse_named_scripts(&output, &options, &callables.scope, &scripts).unwrap();
            assert_eq!(
                encode_script(&parsed.scripts[0].2),
                encode_script(&rebuilt.scripts[0].2)
            );
        }
    }
}

#[test]
fn mfomt_unknown_boolean_quantity_merge_does_not_invent_boolean_symbols() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let scripts = parse_script_table("mary_script_table { TestMerge, };", &options).unwrap();
        for variable in ["VAR_UNKNOWN_SLOT_697", "VAR_UNKNOWN_SLOT_705"] {
            let input = format!("void TestMerge(void) {{ int state; state = VarGet({variable}); if (GetKnownRecipeCount() > 3) {{ state = GetKnownRecipeCount(); }} if (state == 1) {{ return; }} }}");
            let parsed = parse_named_scripts(&input, &options, &callables.scope, &scripts).unwrap();
            let raised =
                decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestMerge")
                    .unwrap();
            let output = format_named_script("TestMerge", &raised).unwrap();
            assert!(output.contains("== 1"), "{target}: {output}");
            assert!(!output.contains("== TRUE"), "{target}: {output}");
            let rebuilt =
                parse_named_scripts(&output, &options, &callables.scope, &scripts).unwrap();
            assert_eq!(
                encode_script(&parsed.scripts[0].2),
                encode_script(&rebuilt.scripts[0].2)
            );
        }
    }
}

#[test]
fn mfomt_raw_unknown_byte_does_not_inherit_normalizing_setter_type() {
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let scripts = parse_script_table("mary_script_table { TestRawByte, };", &options).unwrap();
        for body in [
            "int state; VarSet(VAR_UNKNOWN_SLOT_053, 1); state = VarGet(VAR_UNKNOWN_SLOT_053); if (state == 1) { return; }",
            "int state; state = VarGet(VAR_UNKNOWN_SLOT_053); switch (state) { case 0: TalkClose(); break; case 1: TalkOpen(); break; case 255: return; }",
        ] {
            let input = format!("void TestRawByte(void) {{ {body} }}");
            let parsed = parse_named_scripts(&input, &options, &callables.scope, &scripts).unwrap();
            let raised = decompile_script_named(&parsed.scripts[0].2, &callables.scope, "TestRawByte").unwrap();
            let output = format_named_script("TestRawByte", &raised).unwrap();
            assert!(!output.contains("TRUE") && !output.contains("FALSE"), "{target}: {output}");
            if body.contains("case 255") {
                assert!(output.contains("case 255:"), "{target}: {output}");
            } else {
                assert!(output.contains("== 1"), "{target}: {output}");
            }
            let rebuilt = parse_named_scripts(&output, &options, &callables.scope, &scripts).unwrap();
            assert_eq!(encode_script(&parsed.scripts[0].2), encode_script(&rebuilt.scripts[0].2));
        }
    }
}

#[test]
fn ellen_adjacent_unknown_slots_keep_only_the_proven_boolean_domain() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let (gate, state) = if target.starts_with("MARY_FOMT_") {
            ("VAR_UNKNOWN_SLOT_275", "VAR_UNKNOWN_SLOT_276")
        } else {
            ("VAR_UNKNOWN_SLOT_283", "VAR_UNKNOWN_SLOT_284")
        };
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestUnknownEvent, };\n", &options).unwrap();
        let numeric_source = format!(
            "void TestUnknownEvent(void) {{ VarSet({gate}, 0); VarSet({state}, 2); if (VarGet({gate}) == 1 && VarGet({state}) == 1) {{ return; }} }}\n"
        );
        let numeric =
            parse_named_scripts(&numeric_source, &options, &callables.scope, &script_table)
                .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestUnknownEvent")
                .unwrap();
        let source = format_named_script("TestUnknownEvent", &raised).unwrap();
        for expected in [
            format!("VarSet({gate}, FALSE)"),
            format!("VarSet({state}, 2)"),
            format!("VarGet({gate}) == TRUE"),
            format!("VarGet({state}) == 1"),
        ] {
            assert!(
                source.contains(&expected),
                "{target}: missing {expected} in {source}"
            );
        }
        let symbolic =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&symbolic.scripts[0].2),
            "{target}: physical-domain symbols changed emitted bytecode"
        );
    }
}

#[test]
fn backward_type_inference_decorates_switch_definitions_and_respects_conflicts() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();

        for (script_name, numeric_source, expected, forbidden) in [
            (
                "TestSwitchBackprop",
                "void TestSwitchBackprop(void) { int selector = RandomU15(); int value; switch (selector) { case 0: value = 0; break; case 1: value = 2; break; default: value = 15; break; } SetContestAnimal(ANIMAL_KIND_COW, value); }\n",
                vec!["ANIMAL_SLOT_1", "ANIMAL_SLOT_3", "ANIMAL_SLOT_16"],
                vec![],
            ),
            (
                "TestBooleanQuantityBranchConflict",
                "void TestBooleanQuantityBranchConflict(void) { int value; if (RandomU15()) { value = IsPlayerHoldingNothing(); } else { value = GetKnownRecipeCount(); } if (value == 1) { return; } }\n",
                vec!["== 1"],
                vec!["== TRUE", "== FALSE"],
            ),
            (
                "TestBooleanQuantityLoopReassignment",
                "void TestBooleanQuantityLoopReassignment(void) { int value = IsPlayerHoldingNothing(); do { value = GetKnownRecipeCount(); } while (RandomU15()); if (value == 1) { return; } }\n",
                vec!["== 1"],
                vec!["== TRUE", "== FALSE"],
            ),
            (
                "TestBooleanQuantitySwitchConflict",
                "void TestBooleanQuantitySwitchConflict(void) { int value; switch (RandomU15()) { case 0: value = IsPlayerHoldingNothing(); break; default: value = GetKnownRecipeCount(); break; } switch (value) { case 0: TalkOpen(); break; case 1: TalkClose(); break; default: break; } }\n",
                vec!["case 0:", "case 1:"],
                vec!["case TRUE:", "case FALSE:"],
            ),
            (
                "TestReassignmentBackprop",
                "void TestReassignmentBackprop(void) { int value = 0; SetContestAnimal(ANIMAL_KIND_COW, value); value = 2; SetPlayerHeldFood(value); }\n",
                vec!["ANIMAL_SLOT_1", "FOOD_CUCUMBER"],
                vec![],
            ),
            (
                "TestConflictingBackprop",
                "void TestConflictingBackprop(void) { int value = 0; SetContestAnimal(ANIMAL_KIND_COW, value); SetPlayerHeldFood(value); }\n",
                vec!["var_0 = 0"],
                vec!["ANIMAL_SLOT_1", "FOOD_TURNIP"],
            ),
            (
                "TestSwitchFallthroughBackprop",
                "void TestSwitchFallthroughBackprop(void) { int selector = RandomU15(); int value = 15; switch (selector) { case 0: value = 0; case 1: SetContestAnimal(ANIMAL_KIND_COW, value); break; default: break; } }\n",
                vec!["ANIMAL_SLOT_1", "ANIMAL_SLOT_16"],
                vec![],
            ),
            (
                "TestContextualReturnReassignment",
                "void TestContextualReturnReassignment(void) { int value = GetEventContextValue(); SetTextVariableToProductName(TEXT_VARIABLE_1, value); switch (value) { case 0: break; default: break; } value = GetEventContextValue(); SetEntityFacing(ENTITY_PLAYER, value); switch (value) { case 0: break; default: break; } }\n",
                vec!["case PRODUCT_TURNIP:", "case FACING_DOWN:"],
                vec![],
            ),
            (
                "TestConflictingContextualReturn",
                "void TestConflictingContextualReturn(void) { int value = GetEventContextValue(); SetTextVariableToProductName(TEXT_VARIABLE_1, value); SetEntityFacing(ENTITY_PLAYER, value); switch (value) { case 0: break; default: break; } }\n",
                vec!["case 0:"],
                vec!["PRODUCT_TURNIP", "FACING_DOWN"],
            ),
            (
                "TestCharacterEntitySubset",
                "void TestCharacterEntitySubset(void) { int value; if (RandomU15()) { value = 19; } else { value = 3; } SetEntityFacing(value, FACING_DOWN); SetTalkNameplateCharacter(value); switch (value) { case 19: break; case 3: break; default: break; } }\n",
                vec![
                    "var_0 = CHARACTER_KAREN",
                    "var_0 = CHARACTER_POPURI",
                    "case CHARACTER_KAREN:",
                    "case CHARACTER_POPURI:",
                ],
                vec!["case 19:", "case 3:", "ENTITY_KAREN", "ENTITY_POPURI"],
            ),
        ] {
            let script_table = parse_script_table(
                &format!("mary_script_table {{ {script_name}, }};\n"),
                &options,
            )
            .unwrap();
            let numeric = parse_named_scripts(
                numeric_source,
                &options,
                &callables.scope,
                &script_table,
            )
            .unwrap();
            let raised = decompile_script_named(
                &numeric.scripts[0].2,
                &callables.scope,
                script_name,
            )
            .unwrap();
            let source = format_named_script(script_name, &raised).unwrap();
            for symbol in expected {
                assert!(source.contains(symbol), "{target}/{script_name}: {source}");
            }
            for symbol in forbidden {
                assert!(
                    !source.contains(symbol),
                    "{target}/{script_name}: conflicting domain emitted {symbol}: {source}"
                );
            }
            let symbolic =
                parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
            assert_eq!(
                encode_script(&numeric.scripts[0].2),
                encode_script(&symbolic.scripts[0].2),
                "{target}/{script_name}: inferred constants changed emitted bytecode"
            );
        }
    }
}

#[test]
fn switch_fallthrough_joins_only_real_exit_paths_for_local_types() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestFallthroughExitTypes, };", &options)
                .unwrap();
        let numeric = parse_named_scripts(
            "void TestFallthroughExitTypes(void) { int value; switch (RandomU15()) { case 0: value = GetKnownRecipeCount(); case 1: value = IsPlayerHoldingNothing(); break; default: value = IsPlayerHoldingNothing(); break; } switch (value) { case 0: TalkOpen(); break; case 1: TalkClose(); break; default: break; } }",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestFallthroughExitTypes",
        )
        .unwrap();
        let source = format_named_script("TestFallthroughExitTypes", &raised).unwrap();
        for expected in ["case FALSE:", "case TRUE:"] {
            assert!(
                source.contains(expected),
                "{target}: missing {expected} after a fully overwritten fallthrough path: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: fallthrough type decoration changed emitted bytecode"
        );
    }
}

#[test]
fn grouped_switch_labels_merge_conflicting_related_return_domains() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestGroupedKinds, };", &options).unwrap();
        let numeric = parse_named_scripts(
            "void TestGroupedKinds(void) { int id = GetPresentedItemId(); switch (GetPresentedItemKind()) { case 0: case 1: switch (id) { case 0: TalkOpen(); break; default: break; } break; default: break; } }",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised =
            decompile_script_named(&numeric.scripts[0].2, &callables.scope, "TestGroupedKinds")
                .unwrap();
        let source = format_named_script("TestGroupedKinds", &raised).unwrap();
        for expected in ["case HELD_ITEM_KIND_FOOD:", "case HELD_ITEM_KIND_ARTICLE:"] {
            assert!(
                source.contains(expected),
                "{target}: missing {expected}: {source}"
            );
        }
        assert!(
            source.contains("case 0:"),
            "{target}: conflicting food/article entries must keep the shared item ID numeric: {source}"
        );
        for forbidden in ["case FOOD_TURNIP:", "case ARTICLE_FLOWER_MOON_DROP:"] {
            assert!(
                !source.contains(forbidden),
                "{target}: grouped conflicting entries selected one arbitrary domain ({forbidden}): {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: grouped-label refinement changed emitted bytecode"
        );
    }
}

#[test]
fn grouped_switch_labels_preserve_matching_related_return_domains() {
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let mut constants_source = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
        // This extra test-only relation models two discriminator values which
        // intentionally share one semantic ID domain. Production metadata has
        // no such pair yet, so construct it here instead of weakening the
        // conflict test above.
        constants_source.push_str(
            "\nmary_callable_return_type_when_callable(GetPresentedItemId, GetPresentedItemKind, HELD_ITEM_KIND_DOG, MaryFoodId);\n",
        );
        let constants = parse_constant_header(&constants_source, &options).unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let script_table =
            parse_script_table("mary_script_table { TestGroupedMatchingKinds, };", &options)
                .unwrap();
        let numeric = parse_named_scripts(
            "void TestGroupedMatchingKinds(void) { int id = GetPresentedItemId(); switch (GetPresentedItemKind()) { case 0: case 2: switch (id) { case 0: TalkOpen(); break; default: break; } break; default: break; } }",
            &options,
            &callables.scope,
            &script_table,
        )
        .unwrap();
        let raised = decompile_script_named(
            &numeric.scripts[0].2,
            &callables.scope,
            "TestGroupedMatchingKinds",
        )
        .unwrap();
        let source = format_named_script("TestGroupedMatchingKinds", &raised).unwrap();
        for expected in [
            "case HELD_ITEM_KIND_FOOD:",
            "case HELD_ITEM_KIND_DOG:",
            "case FOOD_TURNIP:",
        ] {
            assert!(
                source.contains(expected),
                "{target}: matching grouped-label domain lost {expected}: {source}"
            );
        }
        let rebuilt =
            parse_named_scripts(&source, &options, &callables.scope, &script_table).unwrap();
        assert_eq!(
            encode_script(&numeric.scripts[0].2),
            encode_script(&rebuilt.scripts[0].2),
            "{target}: matching grouped-label refinement changed emitted bytecode"
        );
    }
}

#[test]
fn ordered_callable_table_preserves_every_physical_slot_for_all_targets() {
    fn legacy_slots(path: &str) -> Vec<(usize, String, bool, usize)> {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                let is_func = match fields.next()? {
                    "func" => true,
                    "proc" => false,
                    _ => return None,
                };
                let id = usize::from_str_radix(fields.next()?.strip_prefix("0x")?, 16).ok()?;
                let signature = line.split_once(fields.next()?)?.1;
                let (name, parameters) = signature.split_once('(')?;
                let parameters = parameters.split_once(')')?.0.trim();
                let parameter_count = if parameters.is_empty() {
                    0
                } else {
                    parameters.split(',').count()
                };
                Some((id, name.to_owned(), is_func, parameter_count))
            })
            .collect()
    }

    for (target, legacy_path, expected_slot_count) in [
        ("MARY_FOMT_US", "goodies/lib_fomt.txt", 327),
        ("MARY_FOMT_JP", "goodies/lib_fomt.txt", 327),
        ("MARY_MFOMT_US", "goodies/lib_mfomt.txt", 339),
        ("MARY_MFOMT_JP", "goodies/lib_mfomt.txt", 339),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();

        assert_eq!(
            callables.next_id, expected_slot_count,
            "{target}: conditional callable entries shifted the physical table length"
        );
        for (expected_id, expected_name, expected_is_func, expected_parameter_count) in
            legacy_slots(legacy_path)
        {
            let (actual_id, actual_shape) = callables
                .scope
                .callable_map()
                .get(&expected_name)
                .unwrap_or_else(|| panic!("{target}: missing physical callable {expected_name}"));
            assert_eq!(
                actual_id.0, expected_id,
                "{target}: {expected_name} moved away from physical slot 0x{expected_id:03X}"
            );
            assert_eq!(
                actual_shape.is_func(),
                expected_is_func,
                "{target}: {expected_name} changed between a value-returning func and a proc"
            );
            assert_eq!(
                actual_shape.num_parameters(),
                expected_parameter_count,
                "{target}: {expected_name} changed its physical argument count"
            );
        }
    }
}

#[test]
fn every_callable_declaration_has_its_own_bilingual_documentation_block() {
    let source = fs::read_to_string("goodies/mary_callables.mary.h").unwrap();
    let declaration_section = source
        .split_once("Mary-C callable declarations")
        .expect("missing callable declaration section")
        .1;
    let lines = declaration_section.lines().collect::<Vec<_>>();

    for (line_index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let is_declaration = ["void ", "int ", "Mary"]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            && trimmed.contains('(')
            && !trimmed.starts_with("Mary-C");
        if !is_declaration {
            continue;
        }

        let mut cursor = line_index;
        while cursor > 0 && lines[cursor - 1].trim().is_empty() {
            cursor -= 1;
        }
        assert!(
            cursor > 0 && lines[cursor - 1].trim_end().ends_with("*/"),
            "callable declaration on header line {} does not have an independent preceding comment: {}",
            source[..source.find(line).unwrap_or(0)].lines().count() + 1,
            trimmed
        );

        let comment_end = cursor - 1;
        let comment_start = (0..=comment_end)
            .rev()
            .find(|index| lines[*index].trim_start().starts_with("/*"))
            .expect("unterminated callable comment");
        let comment = lines[comment_start..=comment_end].join("\n");
        assert!(
            comment
                .chars()
                .any(|character| character.is_ascii_alphabetic()),
            "callable declaration lacks English documentation: {trimmed}"
        );
        assert!(
            comment
                .chars()
                .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character)),
            "callable declaration lacks Chinese documentation: {trimmed}"
        );
    }
}

#[test]
fn every_constant_type_has_its_own_bilingual_documentation_block() {
    let source = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let lines = source.lines().collect::<Vec<_>>();

    for (line_index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let is_type = trimmed.starts_with("typedef enum Mary")
            || (trimmed.starts_with("typedef int Mary") && trimmed.ends_with(';'));
        if !is_type {
            continue;
        }

        let mut cursor = line_index;
        while cursor > 0 && lines[cursor - 1].trim().is_empty() {
            cursor -= 1;
        }
        assert!(
            cursor > 0 && lines[cursor - 1].trim_end().ends_with("*/"),
            "constant type on header line {} does not have an independent preceding comment: {}",
            line_index + 1,
            trimmed
        );

        let comment_end = cursor - 1;
        let comment_start = (0..=comment_end)
            .rev()
            .find(|index| lines[*index].trim_start().starts_with("/*"))
            .expect("unterminated constant-type comment");
        let comment = lines[comment_start..=comment_end].join("\n");
        assert!(
            comment
                .chars()
                .any(|character| character.is_ascii_alphabetic()),
            "constant type lacks English documentation: {trimmed}"
        );
        assert!(
            comment
                .chars()
                .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character)),
            "constant type lacks Chinese documentation: {trimmed}"
        );
    }
}

#[test]
fn audited_system_dispatcher_texts_follow_their_actual_control_flow() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let mut selected = Vec::new();
        for script_id in [75, 110, 350, 413] {
            selected.extend(
                symbols
                    .names(script_id, symbols.text_count(script_id))
                    .into_iter()
                    .flatten(),
            );
        }
        for symbol in [
            "gText_SystemEvent_FestivalHostInteractionDispatcher_CowFestivalJudgingStartPrompt",
            "gText_SystemEvent_FestivalHostInteractionDispatcher_SheepFestivalJudgingStartPrompt",
            "gText_SystemEvent_BathroomUseAndFamilyExitChoice_KaiAnniversaryExitQuestion",
            "gText_SystemEvent_BathroomUseAndFamilyExitChoice_Shared_CliffRickAndGrayAnniversaryExitQuestion",
            "gText_SystemEvent_BathroomUseAndFamilyExitChoice_Shared_KaiCliffDoctorRickAndGrayFamilyEventExitQuestion",
        ] {
            assert!(selected.iter().any(|name| name == symbol), "{target}: missing {symbol}");
        }
        if target == "MARY_MFOMT_JP" {
            assert!(selected.iter().any(|name| {
                name ==
                "gText_SystemEvent_BathroomUseAndFamilyExitChoice_GourmetFamilyEventExitQuestion"
            }));
            assert!(selected.iter().any(|name| name ==
                "gText_NPCEvent_Popuri_IntroductionReturnChickenChoice_Shared_PopuriTakesChickenHomeAfterEitherChoice"
            ));
        } else {
            assert!(!selected.iter().any(|name| {
                name ==
                "gText_SystemEvent_BathroomUseAndFamilyExitChoice_GourmetFamilyEventExitQuestion"
            }));
            for symbol in [
                "gText_NPCEvent_Popuri_IntroductionReturnChickenChoice_PopuriTakesChickenHomeAfterPlayerLikesChickens",
                "gText_NPCEvent_Popuri_IntroductionReturnChickenChoice_PopuriTakesChickenHomeAfterPlayerDislikesChickens",
                "gText_SystemEvent_InspectPlantsAndSnowman_UntranslatedJapaneseSnowmanBeforeSpringDay1At0600",
                "gText_SystemEvent_InspectPlantsAndSnowman_UntranslatedJapanesePlantsFromSpringDay1At0600",
            ] {
                assert!(selected.iter().any(|name| name == symbol), "{target}: missing {symbol}");
            }
        }
    }
}

#[test]
fn upstairs_access_guards_preserve_region_specific_physical_text_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, poultry_script_id, inn_script_id) in [
        ("MARY_FOMT_US", 123, Some(272)),
        ("MARY_FOMT_JP", 123, Some(272)),
        ("MARY_MFOMT_US", 131, None),
        ("MARY_MFOMT_JP", 131, None),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let poultry = symbols
            .names(poultry_script_id, symbols.text_count(poultry_script_id))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if target.ends_with("_US") {
            assert!(
                poultry
                    .iter()
                    .any(|name| name.ends_with("Shared_RickLilliaAndPopuriSayDoNotGoUpstairs")),
                "{target}: {poultry:?}"
            );
        } else {
            for speaker in ["Rick", "Lillia", "Popuri"] {
                assert!(
                    poultry
                        .iter()
                        .any(|name| name.ends_with(&format!("{speaker}SaysDoNotGoUpstairs"))),
                    "{target}: missing {speaker}: {poultry:?}"
                );
            }
        }

        if let Some(script_id) = inn_script_id {
            let inn = symbols
                .names(script_id, symbols.text_count(script_id))
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            if target.ends_with("_US") {
                assert_eq!(inn.len(), 1, "{target}: {inn:?}");
                assert!(inn[0].ends_with("Shared_AnnAndDougSayUpstairsIsNotCleaned"));
            } else {
                assert_eq!(inn.len(), 2, "{target}: {inn:?}");
                assert!(inn[0].ends_with("AnnSaysUpstairsIsNotCleaned"));
                assert!(inn[1].ends_with("DougSaysUpstairsIsNotCleaned"));
            }
        }
    }
}

#[test]
fn new_year_festival_choices_preserve_us_duplicates_and_jp_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 406),
        ("MARY_FOMT_JP", 406),
        ("MARY_MFOMT_US", 415),
        ("MARY_MFOMT_JP", 415),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols
            .names(script_id, symbols.text_count(script_id))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for suffix in [
            "RiceCakeFestivalStartPrompt",
            "NewYearsEveNoodleFestivalStartPrompt",
            "ThomasStartsRiceCakeFestival",
            "ThomasStartsNoodleFestivalMeal",
        ] {
            assert!(
                names.iter().any(|name| name.ends_with(suffix)),
                "{target}: missing {suffix}: {names:?}"
            );
        }
        if target.ends_with("_US") {
            assert_eq!(names.len(), 13, "{target}: {names:?}");
            for suffix in [
                "ChoiceStartRiceCakeFestival",
                "ChoiceStartNoodleFestivalMeal",
            ] {
                assert!(
                    names.iter().any(|name| name.ends_with(suffix)),
                    "{target}: missing {suffix}: {names:?}"
                );
            }
        } else {
            assert_eq!(names.len(), 11, "{target}: {names:?}");
            assert!(
                names
                    .iter()
                    .any(|name| name.ends_with("Shared_RiceCakeAndNoodleFestivalStartChoice")),
                "{target}: {names:?}"
            );
        }
    }
}

#[test]
fn horse_race_invitation_symbols_follow_entry_outcomes_across_both_games() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1079),
        ("MARY_FOMT_JP", 1079),
        ("MARY_MFOMT_US", 1159),
        ("MARY_MFOMT_JP", 1159),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols
            .names(script_id, symbols.text_count(script_id))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for suffix in [
            "ThomasAsksPlayerToEnterAdultHorse",
            "ChoiceEnterHorse",
            "ChoiceDeclineEntry",
            "ThomasAcceptsHorseRaceEntry",
            "ThomasSuggestsSpectatingAndBettingAfterDecline",
        ] {
            assert!(
                names.iter().any(|name| name.ends_with(suffix)),
                "{target}: missing {suffix}: {names:?}"
            );
        }
        let expected_count = if target == "MARY_FOMT_US" { 6 } else { 7 };
        assert_eq!(names.len(), expected_count, "{target}: {names:?}");
    }
}

#[test]
fn fomt_animal_contest_reception_preserves_us_judging_prompt_sharing() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols
            .names(404, symbols.text_count(404))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if target.ends_with("_US") {
            assert_eq!(names.len(), 21, "{target}: {names:?}");
            assert!(
                names
                    .iter()
                    .any(|name| name.ends_with("Shared_CowAndSheepContestJudgingStartPrompt")),
                "{target}: {names:?}"
            );
        } else {
            assert_eq!(names.len(), 22, "{target}: {names:?}");
            for suffix in [
                "CowContestJudgingStartPrompt",
                "SheepContestJudgingStartPrompt",
            ] {
                assert!(
                    names.iter().any(|name| name.ends_with(suffix)),
                    "{target}: missing {suffix}: {names:?}"
                );
            }
        }
    }
}

#[test]
fn winter_thanksgiving_full_rucksack_texts_are_scoped_by_visitor() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let names = symbols
            .names(1299, symbols.text_count(1299))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for visitor in ["Karen", "Mary", "Ann", "Popuri", "Elli"] {
            for outcome in [
                "CannotLeaveGiftBecauseRucksackIsFull",
                "LeavesWithoutGivingGift",
            ] {
                let suffix = format!("{visitor}{outcome}");
                assert!(
                    names.iter().any(|name| name.ends_with(&suffix)),
                    "{target}: missing {suffix}: {names:?}"
                );
            }
        }
        if target.ends_with("_US") {
            assert_eq!(names.len(), 16, "{target}: {names:?}");
            assert!(
                names
                    .iter()
                    .any(|name| name
                        .ends_with("Shared_AnnPopuriAndElliSearchSleepingPlayersRucksack")),
                "{target}: {names:?}"
            );
        } else {
            assert_eq!(names.len(), 18, "{target}: {names:?}");
            for visitor in ["Ann", "Popuri", "Elli"] {
                let suffix = format!("{visitor}SearchesSleepingPlayersRucksack");
                assert!(
                    names.iter().any(|name| name.ends_with(&suffix)),
                    "{target}: missing {suffix}: {names:?}"
                );
            }
        }
    }
}

#[test]
fn thomas_stocking_delivery_uses_the_festival_event_namespace() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, script_id) in [
        ("MARY_FOMT_US", 1307),
        ("MARY_FOMT_JP", 1307),
        ("MARY_MFOMT_US", 1393),
        ("MARY_MFOMT_JP", 1393),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(script_id),
            Some("EventScript_FestivalEvent_StockingGift_ThomasDelivery"),
            "{target} slot {script_id}"
        );
        let names = symbols.names(script_id, symbols.text_count(script_id));
        for suffix in [
            "NoStockingInstalled",
            "GiftPlacedInEmptyStocking",
            "ExistingGiftBlocksDelivery",
        ] {
            assert!(
                names
                    .iter()
                    .any(|name| name.as_deref().is_some_and(|name| name.ends_with(suffix))),
                "{target}: missing {suffix}: {names:?}"
            );
        }
    }
    assert!(!source.contains("EventScript_HolidayEvent_"));
    assert!(!source.contains("gText_HolidayEvent_"));
}

#[test]
fn family_and_festival_scripts_use_event_level_categories_not_dialogue_fragments() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, expectations) in [
        (
            "MARY_FOMT_US",
            vec![
                (784, "EventScript_FamilyEvent_Ann_MothersDeathAnniversary"),
                (
                    1293,
                    "EventScript_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration",
                ),
            ],
        ),
        (
            "MARY_FOMT_JP",
            vec![
                (784, "EventScript_FamilyEvent_Ann_MothersDeathAnniversary"),
                (
                    1293,
                    "EventScript_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration",
                ),
            ],
        ),
        (
            "MARY_MFOMT_US",
            vec![
                (693, "EventScript_FamilyEvent_Kai_ChildInjury"),
                (793, "EventScript_FamilyEvent_Ann_MothersDeathAnniversary"),
                (867, "EventScript_FamilyEvent_Kappa_AnniversaryGiftCucumber"),
                (
                    1373,
                    "EventScript_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration",
                ),
                (
                    1392,
                    "EventScript_FestivalEvent_StarryNight_SpouseAndChildCelebration",
                ),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            vec![
                (693, "EventScript_FamilyEvent_Kai_ChildInjury"),
                (793, "EventScript_FamilyEvent_Ann_MothersDeathAnniversary"),
                (867, "EventScript_FamilyEvent_Kappa_AnniversaryGiftCucumber"),
                (
                    1373,
                    "EventScript_FestivalEvent_PumpkinFestival_SpouseAndChildCelebration",
                ),
                (
                    1392,
                    "EventScript_FestivalEvent_StarryNight_SpouseAndChildCelebration",
                ),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (script_id, expected) in expectations {
            assert_eq!(
                symbols.script_name(script_id),
                Some(expected),
                "{target} slot {script_id}"
            );
        }
    }
    for stale in [
        "EventScript_FamilyEvent_AnnsMothersDeathAnniversary",
        "EventScript_FamilyEvent_ShesFineNoBrokenBonesJustChoice",
        "EventScript_FamilyEvent_Here",
        "EventScript_FamilyEvent_PumpkinFestivalWithSpouseAndChild",
        "EventScript_FamilyEvent_StarryNightWithSpouseReward",
    ] {
        assert!(
            !source.contains(stale),
            "stale script category remains: {stale}"
        );
    }
}

#[test]
fn event_category_suffixes_are_not_split_by_an_underscore() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for category in [
        "Achievement",
        "Family",
        "Farm",
        "Festival",
        "Love",
        "NPC",
        "Rival",
        "Shop",
        "System",
    ] {
        for prefix in ["EventScript", "gText"] {
            let stale = format!("{prefix}_{category}_Event_");
            assert!(
                !source.contains(&stale),
                "fixed Event category suffix was split: {stale}"
            );
        }
    }
}

#[test]
fn fomt_slot_356_is_the_television_channel_dispatcher_not_animal_interaction() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(356),
            Some("EventScript_TV_ChannelDispatcher"),
            "{target} slot 356"
        );
        let names = symbols
            .names(356, symbols.text_count(356))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for suffix in [
            "StormGoldenLumberSignalInterference",
            "StormRareSignalInterference",
            "LateNightGoldenLumberSignalInterference",
            "LateNightRareSignalInterference",
        ] {
            assert!(
                names.iter().any(|name| name.ends_with(suffix)),
                "{target}: missing {suffix}: {names:?}"
            );
        }
        if target.ends_with("_US") {
            assert_eq!(names.len(), 7, "{target}: {names:?}");
            assert!(
                names
                    .iter()
                    .any(|name| name.ends_with("Shared_LateNightCommonSignalInterferenceAAndB")),
                "{target}: {names:?}"
            );
        } else {
            assert_eq!(names.len(), 8, "{target}: {names:?}");
            for suffix in [
                "LateNightCommonSignalInterferenceA",
                "LateNightCommonSignalInterferenceB",
            ] {
                assert!(
                    names.iter().any(|name| name.ends_with(suffix)),
                    "{target}: missing {suffix}: {names:?}"
                );
            }
        }
    }
    assert!(!source.contains("EventScript_FarmEvent_AnimalInteraction"));
}

#[test]
fn mfomt_television_dispatcher_interference_texts_follow_their_producers() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(365),
            Some("EventScript_TV_MainMenuAndProgramDispatcher"),
            "{target} slot 365"
        );
        let names = symbols
            .names(365, symbols.text_count(365))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for suffix in [
            "GoldenLumberInterferenceDotFill",
            "GoldenLumberInterferenceDashFill",
            "StormGoldenLumberSignalInterference",
            "LateNightGoldenLumberSignalInterference",
            "LateNightRareSignalInterference",
        ] {
            assert!(
                names.iter().any(|name| name.ends_with(suffix)),
                "{target}: missing {suffix}: {names:?}"
            );
        }
        if target.ends_with("_US") {
            assert!(
                names
                    .iter()
                    .any(|name| name.ends_with("Shared_LateNightCommonSignalInterferenceAAndB")),
                "{target}: {names:?}"
            );
        } else {
            for suffix in [
                "LateNightCommonSignalInterferenceA",
                "LateNightCommonSignalInterferenceB",
            ] {
                assert!(
                    names.iter().any(|name| name.ends_with(suffix)),
                    "{target}: missing {suffix}: {names:?}"
                );
            }
        }
    }
}

#[test]
fn festival_dialogue_symbols_describe_the_event_role_instead_of_copying_the_text() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, chicken_slot, cow_slot, sheep_slot) in [
        ("MARY_FOMT_US", 1153, 1177, 1289),
        ("MARY_FOMT_JP", 1153, 1177, 1289),
        ("MARY_MFOMT_US", 1233, 1257, 1369),
        ("MARY_MFOMT_JP", 1233, 1257, 1369),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(chicken_slot),
            Some("EventScript_FestivalEvent_ChickenFestival_PreTournamentDialogue_Barley"),
            "{target} slot {chicken_slot}"
        );
        assert_eq!(
            symbols.script_name(cow_slot),
            Some("EventScript_FestivalEvent_CowFestival_PostJudgingSceneSetup"),
            "{target} slot {cow_slot}"
        );
        assert_eq!(
            symbols.script_name(sheep_slot),
            Some("EventScript_FestivalEvent_SheepFestival_PlayerVictory"),
            "{target} slot {sheep_slot}"
        );

        let chicken_names = symbols.names(chicken_slot, symbols.text_count(chicken_slot));
        for expected in [
            "gText_FestivalEvent_ChickenFestival_PreTournamentDialogue_Barley_EnjoysChickenFestival",
            "gText_FestivalEvent_ChickenFestival_PreTournamentDialogue_Barley_AsksWhetherPlayersChickenIsStrong",
        ] {
            assert!(
                chicken_names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected)),
                "{target} slot {chicken_slot}: missing {expected}"
            );
        }

        let cow_names = symbols.names(cow_slot, symbols.text_count(cow_slot));
        let expected = "gText_FestivalEvent_CowFestival_PostJudgingSceneSetup_ThomasAnnouncesFestivalClosingAndCowReturn";
        assert!(
            cow_names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected)),
            "{target} slot {cow_slot}: missing {expected}"
        );

        let sheep_names = symbols.names(sheep_slot, symbols.text_count(sheep_slot));
        let expected =
            "gText_FestivalEvent_SheepFestival_PlayerVictory_BarleyAnnouncesPlayersWinningSheep";
        assert!(
            sheep_names
                .iter()
                .any(|actual| actual.as_deref() == Some(expected)),
            "{target} slot {sheep_slot}: missing {expected}"
        );
    }

    for stale in [
        "EventScript_FestivalEvent_ChickenFestival_ILoveTheChickenFestival",
        "EventScript_FestivalEvent_CowFestival_TheCowFestivalIsNowOver",
        "EventScript_FarmEvent_AndTheWinningSheepIsVar1",
    ] {
        assert!(
            !source.contains(stale),
            "stale text-derived script name remains: {stale}"
        );
    }
}

#[test]
fn animal_and_crop_demonstrations_are_named_as_tutorials_in_all_four_targets() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, first_slot) in [
        ("MARY_FOMT_US", 1060),
        ("MARY_FOMT_JP", 1060),
        ("MARY_MFOMT_US", 1133),
        ("MARY_MFOMT_JP", 1133),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (offset, semantic_name) in [
            (0, "CowCareInstructions"),
            (1, "ChickenCareInstructions"),
            (2, "SheepCareInstructions"),
            (3, "CropFarmingInstructions"),
        ] {
            let slot = first_slot + offset;
            let script_name = format!("EventScript_TutorialEvent_{semantic_name}");
            assert_eq!(
                symbols.script_name(slot),
                Some(script_name.as_str()),
                "{target} slot {slot}"
            );
            let text_prefix = format!("gText_TutorialEvent_{semantic_name}_");
            let names = symbols.names(slot, symbols.text_count(slot));
            assert!(
                !names.is_empty(),
                "{target} slot {slot}: missing tutorial text"
            );
            assert!(
                names.iter().all(|name| name
                    .as_deref()
                    .is_some_and(|name| name.starts_with(&text_prefix))),
                "{target} slot {slot}: text outside {text_prefix}: {names:?}"
            );
        }
    }

    for stale in [
        "EventScript_FarmEvent_UseACowMiraclePotionNext",
        "EventScript_TutorialEvent_AnimalCareInstructions",
        "EventScript_FarmEvent_AfterPickingAnEggUpFace",
        "EventScript_FarmEvent_UseASheepMiraclePotionNext",
        "EventScript_NPCEvent_UseSeedsOnTilledSoilTo",
    ] {
        assert!(
            !source.contains(stale),
            "stale tutorial script name remains: {stale}"
        );
    }
}

#[test]
fn achievement_scripts_use_the_event_category_and_preserve_version_specific_rewards() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, expectations) in [
        (
            "MARY_FOMT_US",
            [
                (822, "Pedometer_10000Steps"),
                (825, "Pedometer_10000000Steps"),
                (842, "SpringMine_ReachedB100"),
                (976, "Fishing_MilestoneReward"),
            ],
        ),
        (
            "MARY_FOMT_JP",
            [
                (822, "Pedometer_10000Steps"),
                (825, "Pedometer_10000000Steps"),
                (842, "SpringMine_ReachedB100"),
                (976, "Fishing_MilestoneReward"),
            ],
        ),
        (
            "MARY_MFOMT_US",
            [
                (831, "Pedometer_10000Steps"),
                (834, "Pedometer_10000000StepsRingReward"),
                (851, "SpringMine_ReachedB100"),
                (1044, "Fishing_MilestoneReward"),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            [
                (831, "Pedometer_10000Steps"),
                (834, "Pedometer_10000000StepsRingReward"),
                (851, "SpringMine_ReachedB100"),
                (1044, "Fishing_MilestoneReward"),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (slot, suffix) in expectations {
            let expected = format!("EventScript_AchievementEvent_{suffix}");
            assert_eq!(
                symbols.script_name(slot),
                Some(expected.as_str()),
                "{target} slot {slot}"
            );
            let names = symbols.names(slot, symbols.text_count(slot));
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with("gText_AchievementEvent_")),
                "{target} slot {slot}: {names:?}"
            );
        }
    }
    for legacy in [
        "EventScript_Achievement_Pedometer_",
        "EventScript_Achievement_SpringMine_",
        "EventScript_Achievement_Fishing_",
        "gText_Achievement_Pedometer_",
        "gText_Achievement_SpringMine_",
        "gText_Achievement_Fishing_",
    ] {
        assert!(
            !source.contains(legacy),
            "legacy category survived: {legacy}"
        );
    }
}

#[test]
fn npc_and_festival_script_names_keep_their_subject_hierarchy() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, expectations) in [
        (
            "MARY_FOMT_US",
            [
                (465, "EventScript_NPCEvent_Van_ShopCounter"),
                (694, "EventScript_NPCEvent_Doctor_PlayerCollapseRecovery"),
                (745, "EventScript_NPCEvent_Ellen_GrandfathersLetter"),
                (
                    780,
                    "EventScript_NPCEvent_Carter_PlayerSneaksPastSleepingCarter",
                ),
                (821, "EventScript_FestivalEvent_ShootingStar_WishChoice"),
                (
                    1065,
                    "EventScript_FestivalEvent_NewYearsDayRiceCakeFestival_Opening",
                ),
                (
                    1066,
                    "EventScript_FestivalEvent_NewYearRiceCakeFestival_Meal",
                ),
                (1079, "EventScript_FestivalEvent_HorseRace_EntryInvitation"),
            ],
        ),
        (
            "MARY_MFOMT_US",
            [
                (474, "EventScript_NPCEvent_Van_ShopCounter"),
                (924, "EventScript_NPCEvent_MineralTownGirls_BigBedSleepover"),
                (830, "EventScript_FestivalEvent_ShootingStar_WishChoice"),
                (
                    1139,
                    "EventScript_FestivalEvent_NewYearsDayRiceCakeFestival_Opening",
                ),
                (
                    1140,
                    "EventScript_FestivalEvent_NewYearRiceCakeFestival_Meal",
                ),
                (1159, "EventScript_FestivalEvent_HorseRace_EntryInvitation"),
                (
                    415,
                    "EventScript_FestivalEvent_NewYearCelebration_StartChoice",
                ),
                (1044, "EventScript_AchievementEvent_Fishing_MilestoneReward"),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (slot, expected) in expectations {
            assert_eq!(
                symbols.script_name(slot),
                Some(expected),
                "{target} slot {slot}"
            );
        }
    }
}

#[test]
fn every_shop_event_name_groups_the_shop_before_the_action() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let shop_prefixes = [
        "EventScript_ShopEvent_BeachCafe_",
        "EventScript_ShopEvent_Blacksmith_",
        "EventScript_ShopEvent_Carpenter_",
        "EventScript_ShopEvent_Clinic_",
        "EventScript_ShopEvent_Inn_",
        "EventScript_ShopEvent_Rucksack_",
        "EventScript_ShopEvent_Supermarket_",
        "EventScript_ShopEvent_Winery_",
        "EventScript_ShopEvent_Won_",
        "EventScript_ShopEvent_YodelRanch_",
    ];
    for (target, slot_count) in [
        ("MARY_FOMT_US", 1329),
        ("MARY_FOMT_JP", 1329),
        ("MARY_MFOMT_US", 1416),
        ("MARY_MFOMT_JP", 1416),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for slot in 0..slot_count {
            let Some(name) = symbols.script_name(slot) else {
                continue;
            };
            if name.starts_with("EventScript_ShopEvent_") {
                assert!(
                    shop_prefixes.iter().any(|prefix| name.starts_with(prefix)),
                    "{target} slot {slot}: shop event does not group its subject before its action: {name}"
                );
            }
        }
    }
}

#[test]
fn mfomt_reference_guide_and_mailbox_reminder_use_consistent_categories() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (slot, script_name, text_prefix) in [
            (
                317,
                "EventScript_SystemMenu_ReferenceGuideViewer",
                "gText_SystemMenu_ReferenceGuideViewer_",
            ),
            (
                919,
                "EventScript_TutorialEvent_MailboxReminder",
                "gText_TutorialEvent_MailboxReminder",
            ),
        ] {
            assert_eq!(
                symbols.script_name(slot),
                Some(script_name),
                "{target} slot {slot}"
            );
            let names = symbols.names(slot, symbols.text_count(slot));
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with(text_prefix)),
                "{target} slot {slot}: {names:?}"
            );
        }
    }
    for (target, first_slot) in [
        ("MARY_FOMT_US", 309),
        ("MARY_FOMT_JP", 309),
        ("MARY_MFOMT_US", 318),
        ("MARY_MFOMT_JP", 318),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (offset, suffix) in [
            (0, "ShowFestivalMinigames"),
            (1, "ShowControls"),
            (2, "ShowRaisingCrops"),
            (3, "ShowSeasonalCrops"),
            (4, "ShowMines"),
            (5, "ShowFishing"),
            (6, "ShowCaringForAnimals"),
        ] {
            let slot = first_slot + offset;
            let expected = format!("EventScript_SystemMenu_ReferenceGuide_{suffix}");
            assert_eq!(
                symbols.script_name(slot),
                Some(expected.as_str()),
                "{target} slot {slot}"
            );
        }
    }
    for stale in [
        "EventScript_System_ReferenceBookViewer",
        "EventScript_ReferenceGuide_",
        "gText_ReferenceGuide_",
        "EventScript_Tutorial_MailboxReminder",
        "gText_Tutorial_MailboxReminder",
        "EventScript_TVShopping_",
        "gText_TVShopping_",
        "EventScript_Television_",
        "gText_Television_",
    ] {
        assert!(!source.contains(stale), "stale category remains: {stale}");
    }
}

#[test]
fn multi_npc_event_names_group_participants_before_the_event_role() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for stale in [
        "NPCEvent_Barley_AndDoug",
        "NPCEvent_Gray_AndKai",
        "NPCEvent_Mary_AndGray",
        "NPCEvent_Lillia_AndSasha",
        "NPCEvent_Karen_AndDuke",
        "NPCEvent_Doug_AndDuke",
        "NPCEvent_Ann_AndCliff",
        "NPCEvent_Gotz_AndHarris",
        "NPCEvent_Elli_AndStu",
    ] {
        assert!(
            !source.contains(stale),
            "split participant group remains: {stale}"
        );
    }

    for (target, expectations) in [
        (
            "MARY_FOMT_US",
            [
                (
                    710,
                    "EventScript_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall",
                ),
                (786, "EventScript_NPCEvent_DougAndDuke_ArgumentChoice"),
            ],
        ),
        (
            "MARY_FOMT_JP",
            [
                (
                    710,
                    "EventScript_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall",
                ),
                (786, "EventScript_NPCEvent_DougAndDuke_ArgumentChoice"),
            ],
        ),
        (
            "MARY_MFOMT_US",
            [
                (
                    719,
                    "EventScript_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall",
                ),
                (795, "EventScript_NPCEvent_DougAndDuke_ArgumentChoice"),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            [
                (
                    719,
                    "EventScript_NPCEvent_BarleyAndDoug_DiscussJoannasPhoneCall",
                ),
                (795, "EventScript_NPCEvent_DougAndDuke_ArgumentChoice"),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        for (slot, expected) in expectations {
            assert_eq!(
                symbols.script_name(slot),
                Some(expected),
                "{target} slot {slot}"
            );
        }
    }
}

#[test]
fn every_text_symbol_has_an_explicit_semantic_domain() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let allowed_prefixes = [
        "gText_AchievementEvent_",
        "gText_CollectibleEvent_",
        "gText_FamilyEvent_",
        "gText_FarmEvent_",
        "gText_FestivalEvent_",
        "gText_LocationInteraction_",
        "gText_LocationTransition_",
        "gText_LoveEvent_",
        "gText_MineEvent_",
        "gText_Minigame_",
        "gText_NPCEvent_",
        "gText_RivalEvent_",
        "gText_RivalMarriageEvent_",
        "gText_ShopEvent_",
        "gText_SystemEvent_",
        "gText_SystemMenu_",
        "gText_SystemMessage_",
        "gText_TutorialEvent_",
        "gText_TV_",
        "gText_WeddingEvent_",
    ];
    let mut count = 0;
    for (line_index, line) in source.lines().enumerate() {
        let symbol = line.trim();
        if !symbol.starts_with("gText_") {
            continue;
        }
        count += 1;
        assert!(
            allowed_prefixes
                .iter()
                .any(|prefix| symbol.starts_with(prefix)),
            "line {}: text symbol lacks a semantic domain: {symbol}",
            line_index + 1
        );
    }
    assert!(
        count > 10_000,
        "unexpectedly small text-symbol table: {count}"
    );
}

#[test]
fn a_text_symbol_never_silently_belongs_to_different_scripts() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let mut current_script = None;
    let mut owners = HashMap::<String, String>::new();
    for (line_index, line) in source.lines().enumerate() {
        let token = line.trim();
        if token.starts_with("EventScript_")
            && token
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            current_script = Some(token.to_owned());
            continue;
        }
        if token == "}," {
            current_script = None;
            continue;
        }
        let Some(script) = current_script.as_ref() else {
            continue;
        };
        let Some(text) = token
            .strip_suffix(',')
            .filter(|token| token.starts_with("gText_"))
        else {
            continue;
        };
        if let Some(previous) = owners.insert(text.to_owned(), script.clone()) {
            assert_eq!(
                previous,
                *script,
                "line {}: {text} belongs to both {previous} and {script}",
                line_index + 1
            );
        }
    }
    assert!(owners.len() > 10_000, "unexpectedly small text-owner map");
}

#[test]
fn animal_birth_and_death_events_share_semantic_names_across_game_families() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, birth_slot, death_slot) in [
        ("MARY_FOMT_US", 984, 985),
        ("MARY_FOMT_JP", 984, 985),
        ("MARY_MFOMT_US", 1052, 1053),
        ("MARY_MFOMT_JP", 1052, 1053),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(birth_slot),
            Some("EventScript_FarmEvent_Livestock_NewBirthAndNaming"),
            "{target} slot {birth_slot}"
        );
        assert_eq!(
            symbols.script_name(death_slot),
            Some("EventScript_FarmEvent_Livestock_DeathConsequences"),
            "{target} slot {death_slot}"
        );
        let birth_names = symbols.names(birth_slot, symbols.text_count(birth_slot));
        for suffix in [
            "BirthExpectedSoon",
            "PromptNameNewChick",
            "PromptNameNewCalf",
            "PromptNameNewLamb",
            "PlayerWelcomesNewAnimal",
        ] {
            let expected = format!("gText_FarmEvent_Livestock_NewBirthAndNaming_{suffix}");
            assert!(
                birth_names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected.as_str())),
                "{target} slot {birth_slot}: missing {expected}"
            );
        }
        let death_names = symbols.names(death_slot, symbols.text_count(death_slot));
        for suffix in [
            "RickWarnsAnimalsDependOnPlayer",
            "CarterExplainsDeathIsInevitable",
            "CarterEncouragesPlayer",
        ] {
            let expected = format!("gText_FarmEvent_Livestock_DeathConsequences_{suffix}");
            assert!(
                death_names
                    .iter()
                    .any(|actual| actual.as_deref() == Some(expected.as_str())),
                "{target} slot {death_slot}: missing {expected}"
            );
        }
    }
    for stale in [
        "EventScript_NPCEvent_ACalfHasBeenBornGive",
        "gText_NPCEvent_ACalfHasBeenBornGive_",
        "EventScript_FarmEvent_AnimalDeathConsequences",
        "gText_FarmEvent_AnimalDeathConsequences_",
    ] {
        assert!(
            !source.contains(stale),
            "stale cross-family name remains: {stale}"
        );
    }
}

#[test]
fn huge_stone_inspection_names_the_engine_restriction_in_all_targets() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let script_name = "EventScript_LocationInteraction_InspectHugeStoneUnbreakableWithLevel5Hammer";
    let text_name = "gText_LocationInteraction_InspectHugeStoneUnbreakableWithLevel5Hammer";
    for (target, slot) in [
        ("MARY_FOMT_US", 969),
        ("MARY_FOMT_JP", 969),
        ("MARY_MFOMT_US", 1037),
        ("MARY_MFOMT_JP", 1037),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(slot),
            Some(script_name),
            "{target} slot {slot}"
        );
        let names = symbols.names(slot, symbols.text_count(slot));
        assert!(
            names
                .iter()
                .any(|actual| actual.as_deref() == Some(text_name)),
            "{target} slot {slot}: missing {text_name}"
        );
    }
    for stale in [
        "InspectHugeStoneYourHammerIsLV5",
        "InspectHugeStoneEvenALevel5",
    ] {
        assert!(
            !source.contains(stale),
            "localized sentence remains in symbol: {stale}"
        );
    }
}

#[test]
fn shared_location_inspections_include_their_actual_location_in_both_game_families() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let cases = [
        (99, 107, "InspectHorseStableWoodBox"),
        (100, 108, "InspectBeachBench"),
        (116, 124, "InspectLilliaHouse1FChair"),
        (117, 125, "InspectLilliaHouse1FKitchen"),
        (118, 126, "InspectLilliaHouse1FRefrigerator"),
        (119, 127, "InspectLilliaHouse1FKitchenCabinet"),
        (120, 128, "InspectLilliaHouse1FSofa"),
        (121, 129, "InspectLilliaHouse1FFireplace"),
        (122, 130, "InspectLilliaHouse1FTable"),
        (126, 134, "InspectLilliaHouse2FBed"),
        (127, 135, "InspectLilliaHouse2FBookshelf"),
        (129, 137, "InspectFlowerPainting"),
        (130, 138, "InspectLilliaHouse2FLamp"),
        (131, 139, "InspectMountainPainting"),
        (144, 152, "InspectChurchWindow"),
        (145, 153, "InspectZackHouseBarrel"),
        (150, 158, "InspectZackHouseLilliaPhoto"),
        (197, 206, "InspectKappaPainting"),
        (208, 217, "InspectSupermarketPoster"),
        (262, 271, "InspectInnBackRoomCookingUtensils"),
        (419, 428, "InspectAjaWineryBasementBarrelShelf"),
    ];
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        let use_mfomt_slot = target.contains("MFOMT");
        for (fomt_slot, mfomt_slot, suffix) in cases {
            let slot = if use_mfomt_slot {
                mfomt_slot
            } else {
                fomt_slot
            };
            let expected = format!("EventScript_LocationInteraction_{suffix}");
            assert_eq!(
                symbols.script_name(slot),
                Some(expected.as_str()),
                "{target} slot {slot}"
            );
            let text_prefix = format!("gText_LocationInteraction_{suffix}");
            let names = symbols.names(slot, symbols.text_count(slot));
            assert!(
                !names.is_empty(),
                "{target} slot {slot}: missing object label"
            );
            assert!(
                names
                    .iter()
                    .flatten()
                    .all(|name| name.starts_with(&text_prefix)),
                "{target} slot {slot}: text outside {text_prefix}: {names:?}"
            );
        }
    }
}

#[test]
fn mothers_hill_summit_transition_exposes_its_event_dispatch_role() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let expected =
        "EventScript_LocationTransition_EnterMothersHillSummitWithFestivalAndCharacterEventDispatch";
    for (target, slot) in [
        ("MARY_FOMT_US", 93),
        ("MARY_FOMT_JP", 93),
        ("MARY_MFOMT_US", 101),
        ("MARY_MFOMT_JP", 101),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(slot),
            Some(expected),
            "{target} slot {slot}"
        );
    }
    assert!(!source
        .lines()
        .any(|line| { line.trim() == "EventScript_LocationTransition_EnterMothersHillSummit" }));
}

#[test]
fn inn_upstairs_access_names_preserve_the_gender_version_event_difference() {
    let source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, slot, expected, expected_text_count) in [
        (
            "MARY_FOMT_US",
            272,
            "EventScript_NPCEvent_AnnAndDoug_BlockInnUpstairsAccess",
            1,
        ),
        (
            "MARY_FOMT_JP",
            272,
            "EventScript_NPCEvent_AnnAndDoug_BlockInnUpstairsAccess",
            2,
        ),
        (
            "MARY_MFOMT_US",
            281,
            "EventScript_NPCEvent_Ann_BlocksInnUpstairsAccess",
            1,
        ),
        (
            "MARY_MFOMT_JP",
            281,
            "EventScript_NPCEvent_Ann_BlocksInnUpstairsAccess",
            1,
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let symbols = parse_text_name_table(&source, &options).unwrap();
        assert_eq!(
            symbols.script_name(slot),
            Some(expected),
            "{target} slot {slot}"
        );
        assert_eq!(
            symbols.text_count(slot),
            expected_text_count,
            "{target} slot {slot}"
        );
    }
}

#[test]
fn script_local_purchase_and_blacksmith_domains_are_target_correct() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let symbols_source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    for (target, tv_slot, blacksmith_slot, mirror_slot) in [
        ("MARY_FOMT_US", 285, 487, None),
        ("MARY_FOMT_JP", 285, 487, None),
        ("MARY_MFOMT_US", 294, 496, Some(359)),
        ("MARY_MFOMT_JP", 294, 496, Some(359)),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        let symbols = parse_text_name_table(&symbols_source, &options).unwrap();

        let purchase_type = constants
            .constant_value_type("TV_SHOPPING_PHONE_ORDER_PURCHASED")
            .unwrap();
        let product_type = constants
            .constant_value_type("BLACKSMITH_PRODUCT_CATEGORY_TOOL")
            .unwrap();
        assert_eq!(
            symbols
                .local_types(tv_slot, &constants)
                .unwrap()
                .get("var_1"),
            Some(&purchase_type),
            "{target} TV Shopping outcome"
        );
        assert_eq!(
            symbols
                .local_types(blacksmith_slot, &constants)
                .unwrap()
                .get("var_0"),
            Some(&product_type),
            "{target} blacksmith product category"
        );
        if let Some(mirror_slot) = mirror_slot {
            assert_eq!(
                symbols
                    .local_types(mirror_slot, &constants)
                    .unwrap()
                    .get("var_6"),
                Some(&product_type),
                "{target} Shopping Master product category"
            );
        }
    }
}

#[test]
fn mfomt_local_event_result_domains_are_complete_and_target_correct() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let symbols_source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        let symbols = parse_text_name_table(&symbols_source, &options).unwrap();

        let response_type = constants
            .constant_value_type("GIRLS_COOKING_REQUEST_RESPONSE_PENDING")
            .unwrap();
        for slot in 926..=930 {
            assert_eq!(
                symbols.local_types(slot, &constants).unwrap().get("var_2"),
                Some(&response_type),
                "{target} cooking-request slot {slot}"
            );
        }

        let lottery_type = constants
            .constant_value_type("WON_LOTTERY_RESULT_NO_MATCH")
            .unwrap();
        let lottery_type_id = constants.user_type("MaryWonLotteryResult").unwrap();
        assert_eq!(
            symbols.local_types(933, &constants).unwrap().get("var_6"),
            Some(&lottery_type),
            "{target} Won lottery result"
        );
        for (name, value) in [
            ("WON_LOTTERY_RESULT_NO_MATCH", 0),
            ("WON_LOTTERY_RESULT_SECOND_AND_THIRD_MATCH", 1),
            ("WON_LOTTERY_RESULT_FIRST_AND_THIRD_MATCH", 2),
            ("WON_LOTTERY_RESULT_FIRST_AND_SECOND_MATCH", 3),
            ("WON_LOTTERY_RESULT_SEQUENTIAL_DIGITS", 4),
            ("WON_LOTTERY_RESULT_THREE_MATCHING_DIGITS", 5),
        ] {
            assert_eq!(
                constants.typed_int_const_name(lottery_type_id, value),
                Some(name),
                "{target} {name}"
            );
        }

        let dumpling_type = constants
            .constant_value_type("MOON_DUMPLING_GIFT_UNAVAILABLE")
            .unwrap();
        assert_eq!(
            symbols.local_types(1323, &constants).unwrap().get("var_1"),
            Some(&dumpling_type),
            "{target} Moon Viewing dumpling source"
        );
    }
}

#[test]
fn mfomt_family_event_completion_locals_are_boolean_in_both_regions() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let symbols_source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();
    let script_names = [
        "EventScript_FamilyEvent_Cliff_BabyBirthdayDateDialogueChoice",
        "EventScript_FamilyEvent_Cliff_ChildInjury",
        "EventScript_FamilyEvent_Cliff_FamilyDateDialogueChoice",
        "EventScript_FamilyEvent_Cliff_FamilyDateDialogueChoiceAlternate",
        "EventScript_FamilyEvent_Doctor_BabyBirthdayDateDialogueChoice",
        "EventScript_FamilyEvent_Doctor_FamilyDateDialogueChoice",
        "EventScript_FamilyEvent_Doctor_FamilyDateDialogueChoiceAlternate",
        "EventScript_FamilyEvent_Gourmet_ChildInjury",
        "EventScript_FamilyEvent_Gourmet_FamilyCelebrationAndAnniversaryChoice",
        "EventScript_FamilyEvent_Gourmet_FamilyCelebrationWithBirthdayGiftChoice",
        "EventScript_FamilyEvent_Gray_BabyBirthdayDateDialogueChoice",
        "EventScript_FamilyEvent_Gray_ChildInjury",
        "EventScript_FamilyEvent_Gray_FamilyDateDialogueChoice",
        "EventScript_FamilyEvent_Gray_FamilyDateDialogueChoiceAlternate",
        "EventScript_FamilyEvent_Kai_BabyBirthdayDateDialogueChoice",
        "EventScript_FamilyEvent_Kai_ChildInjury",
        "EventScript_FamilyEvent_Kai_FamilyDateDialogueChoice",
        "EventScript_FamilyEvent_Kai_FamilyDateDialogueChoiceAlternate",
        "EventScript_FamilyEvent_Kappa_DateChoiceWithChild",
        "EventScript_FamilyEvent_Kappa_FamilyCelebrationWithBirthdayGiftChoice",
        "EventScript_FamilyEvent_Kappa_SpouseBirthdayDialogue",
        "EventScript_FamilyEvent_Rick_BabyBirthdayDateDialogueChoice",
        "EventScript_FamilyEvent_Rick_ChildInjury",
        "EventScript_FamilyEvent_Rick_FamilyDateDialogueChoice",
        "EventScript_FamilyEvent_Rick_FamilyDateDialogueChoiceAlternate",
        "EventScript_FamilyEvent_Won_ChildInjury",
        "EventScript_FamilyEvent_Won_FamilyCelebrationWithBirthdayGiftChoice",
        "EventScript_FamilyEvent_Won_FamilyDateDialogueChoice",
    ];

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        let symbols = parse_text_name_table(&symbols_source, &options).unwrap();
        let bool_type = constants.constant_value_type("TRUE").unwrap();

        for script_name in script_names {
            let slot = (0..1416)
                .find(|slot| symbols.script_name(*slot) == Some(script_name))
                .unwrap_or_else(|| panic!("{target}: missing {script_name}"));
            assert_eq!(
                symbols.local_types(slot, &constants).unwrap().get("var_1"),
                Some(&bool_type),
                "{target} slot {slot} {script_name}"
            );
        }
    }
}

#[test]
fn minigame_round_and_apple_shuffle_local_domains_are_target_correct() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let symbols_source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    for (target, rps_slot, number_slot) in [
        ("MARY_FOMT_US", 1040, 1041),
        ("MARY_FOMT_JP", 1040, 1041),
        ("MARY_MFOMT_US", 1111, 1112),
        ("MARY_MFOMT_JP", 1111, 1112),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        let symbols = parse_text_name_table(&symbols_source, &options).unwrap();
        let round_type = constants
            .constant_value_type("HARVEST_GODDESS_GAME_ROUND_WIN")
            .unwrap();
        for slot in [rps_slot, number_slot] {
            assert_eq!(
                symbols.local_types(slot, &constants).unwrap().get("var_2"),
                Some(&round_type),
                "{target} Harvest Goddess game slot {slot}"
            );
        }

        if target.starts_with("MARY_MFOMT") {
            let apple_type = constants
                .constant_value_type("APPLE_SHUFFLE_APPLE_SUPER")
                .unwrap();
            let position_type = constants
                .constant_value_type("APPLE_SHUFFLE_POSITION_LEFT")
                .unwrap();
            let locals = symbols.local_types(932, &constants).unwrap();
            assert_eq!(
                locals.get("var_4"),
                Some(&apple_type),
                "{target} apple identity"
            );
            for local in ["var_5", "var_6", "var_7"] {
                assert_eq!(
                    locals.get(local),
                    Some(&position_type),
                    "{target} {local} apple position"
                );
            }
        }
    }
}

#[test]
fn clinic_diagnosis_and_awl_profile_menu_domains_are_target_correct() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let symbols_source = fs::read_to_string("goodies/mary_scripts_text.mary.sym").unwrap();

    for (target, profile_slot, clinic_slot) in [
        ("MARY_FOMT_US", 316, 483),
        ("MARY_FOMT_JP", 316, 483),
        ("MARY_MFOMT_US", 325, 492),
        ("MARY_MFOMT_JP", 325, 492),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        let symbols = parse_text_name_table(&symbols_source, &options).unwrap();
        let profile_type = constants
            .constant_value_type("AWL_PROFILE_MENU_CONTINUE_BROWSING")
            .unwrap();
        let diagnosis_type = constants
            .constant_value_type("CLINIC_EXAM_RESULT_HEALTHY")
            .unwrap();
        assert_eq!(
            symbols
                .local_types(profile_slot, &constants)
                .unwrap()
                .get("var_2"),
            Some(&profile_type),
            "{target} AWL profile menu result"
        );
        assert_eq!(
            symbols
                .local_types(clinic_slot, &constants)
                .unwrap()
                .get("var_1"),
            Some(&diagnosis_type),
            "{target} clinic diagnosis"
        );
    }
}

#[test]
fn music_festival_performance_animations_follow_each_game_family() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();

    for (target, expected) in [
        (
            "MARY_FOMT_US",
            [
                ("ANIMATION_KAREN_MUSIC_FESTIVAL_PERFORMANCE", 1705),
                ("ANIMATION_ANN_MUSIC_FESTIVAL_PERFORMANCE", 2014),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_PERFORMANCE", 2083),
                ("ANIMATION_ELLI_MUSIC_FESTIVAL_PERFORMANCE", 2212),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_IDLE", 2119),
            ],
        ),
        (
            "MARY_FOMT_JP",
            [
                ("ANIMATION_KAREN_MUSIC_FESTIVAL_PERFORMANCE", 1705),
                ("ANIMATION_ANN_MUSIC_FESTIVAL_PERFORMANCE", 2014),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_PERFORMANCE", 2083),
                ("ANIMATION_ELLI_MUSIC_FESTIVAL_PERFORMANCE", 2212),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_IDLE", 2119),
            ],
        ),
        (
            "MARY_MFOMT_US",
            [
                ("ANIMATION_KAREN_MUSIC_FESTIVAL_PERFORMANCE", 1753),
                ("ANIMATION_ANN_MUSIC_FESTIVAL_PERFORMANCE", 2062),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_PERFORMANCE", 2131),
                ("ANIMATION_ELLI_MUSIC_FESTIVAL_PERFORMANCE", 2272),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_IDLE", 2167),
            ],
        ),
        (
            "MARY_MFOMT_JP",
            [
                ("ANIMATION_KAREN_MUSIC_FESTIVAL_PERFORMANCE", 1753),
                ("ANIMATION_ANN_MUSIC_FESTIVAL_PERFORMANCE", 2062),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_PERFORMANCE", 2131),
                ("ANIMATION_ELLI_MUSIC_FESTIVAL_PERFORMANCE", 2272),
                ("ANIMATION_MARY_MUSIC_FESTIVAL_IDLE", 2167),
            ],
        ),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in expected {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
            let mary::ir::ValueType::UserType(type_id) =
                constants.constant_value_type(name).unwrap()
            else {
                panic!("{target} {name} must have an animation constant type");
            };
            assert_eq!(
                constants.typed_int_const_name(type_id, value),
                Some(name),
                "{target} canonical {name}"
            );
        }
    }
}

#[test]
fn fomt_spouse_newborn_presentation_animations_are_target_scoped() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let expected = [
        ("ANIMATION_POPURI_WITH_NEWBORN", 583),
        ("ANIMATION_KAREN_WITH_NEWBORN", 1709),
        ("ANIMATION_ANN_WITH_NEWBORN", 1994),
        ("ANIMATION_MARY_WITH_NEWBORN", 2095),
        ("ANIMATION_ELLI_WITH_NEWBORN", 2196),
    ];

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in expected {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
        }
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, _) in expected {
            assert_eq!(
                constants.const_int_value(name),
                None,
                "{target} leaked {name}"
            );
        }
        assert_eq!(constants.const_int_value("ANIMATION_FOAL_IDLE"), Some(1994));
    }
}

#[test]
fn harvest_sprite_wedding_ceremony_animations_follow_each_game_family() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let names = ["STAID", "TIMID", "NAPPY", "BOLD", "CHEF", "AQUA", "HOGGY"];

    for (target, values) in [
        ("MARY_FOMT_US", [1008, 1092, 1176, 1260, 1344, 1428, 1512]),
        ("MARY_FOMT_JP", [1008, 1092, 1176, 1260, 1344, 1428, 1512]),
        ("MARY_MFOMT_US", [1044, 1128, 1212, 1296, 1380, 1464, 1548]),
        ("MARY_MFOMT_JP", [1044, 1128, 1212, 1296, 1380, 1464, 1548]),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in names.into_iter().zip(values) {
            let symbol = format!("ANIMATION_{name}_WEDDING_CEREMONY");
            assert_eq!(
                constants.const_int_value(&symbol),
                Some(value),
                "{target} {symbol}"
            );
        }
    }
}

#[test]
fn harvest_sprite_tea_party_animations_follow_each_game_family() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let names = ["STAID", "TIMID", "NAPPY", "BOLD", "CHEF", "AQUA", "HOGGY"];

    for (target, values) in [
        ("MARY_FOMT_US", [1072, 1156, 1240, 1324, 1408, 1492, 1576]),
        ("MARY_FOMT_JP", [1072, 1156, 1240, 1324, 1408, 1492, 1576]),
        ("MARY_MFOMT_US", [1108, 1192, 1276, 1360, 1444, 1528, 1612]),
        ("MARY_MFOMT_JP", [1108, 1192, 1276, 1360, 1444, 1528, 1612]),
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in names.into_iter().zip(values) {
            let symbol = format!("ANIMATION_{name}_TEA_PARTY");
            assert_eq!(
                constants.const_int_value(&symbol),
                Some(value),
                "{target} {symbol}"
            );
        }
    }
}

#[test]
fn mfomt_rick_and_cliff_item_handover_animations_do_not_collide_with_fomt_wedding_slots() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in [
            ("ANIMATION_RICK_HAND_OVER_ITEM", 539),
            ("ANIMATION_CLIFF_HAND_OVER_ITEM", 655),
            ("ANIMATION_CLIFF_WEDDING_KISS", 667),
        ] {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
            let mary::ir::ValueType::UserType(type_id) =
                constants.constant_value_type(name).unwrap()
            else {
                panic!("{target} {name} must have an animation constant type");
            };
            assert_eq!(
                constants.typed_int_const_name(type_id, value),
                Some(name),
                "{target} canonical {name}"
            );
        }
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let scripts =
            parse_script_table("mary_script_table { TestHandOverItem, };", &options).unwrap();
        let legacy = parse_named_scripts(
            "void TestHandOverItem(void) { SetEntityAnim(0, ANIMATION_ID_0539); SetEntityAnim(0, ANIMATION_ID_0655); }",
            &options,
            &callables.scope,
            &scripts,
        )
        .unwrap();
        let semantic = parse_named_scripts(
            "void TestHandOverItem(void) { SetEntityAnim(0, ANIMATION_RICK_HAND_OVER_ITEM); SetEntityAnim(0, ANIMATION_CLIFF_HAND_OVER_ITEM); }",
            &options,
            &callables.scope,
            &scripts,
        )
        .unwrap();
        assert_eq!(
            encode_script(&legacy.scripts[0].2),
            encode_script(&semantic.scripts[0].2),
            "{target} legacy aliases"
        );
    }

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        assert_eq!(
            constants.const_int_value("ANIMATION_CLIFF_WEDDING_KISS"),
            Some(655)
        );
        let callables = parse_callable_table_with_scope(
            &fs::read_to_string("goodies/mary_callables.mary.h").unwrap(),
            &options,
            &constants,
        )
        .unwrap();
        let scripts =
            parse_script_table("mary_script_table { TestWeddingKiss, };", &options).unwrap();
        let legacy = parse_named_scripts(
            "void TestWeddingKiss(void) { SetEntityAnim(0, ANIMATION_ID_0655); }",
            &options,
            &callables.scope,
            &scripts,
        )
        .unwrap();
        let semantic = parse_named_scripts(
            "void TestWeddingKiss(void) { SetEntityAnim(0, ANIMATION_CLIFF_WEDDING_KISS); }",
            &options,
            &callables.scope,
            &scripts,
        )
        .unwrap();
        assert_eq!(
            encode_script(&legacy.scripts[0].2),
            encode_script(&semantic.scripts[0].2),
            "{target} legacy alias"
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_RICK_HAND_OVER_ITEM"),
            None
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_CLIFF_HAND_OVER_ITEM"),
            None
        );
    }
}

#[test]
fn mfomt_big_bed_sleep_over_animation_stages_are_target_scoped() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let expected = [
        ("ANIMATION_POPURI_BIG_BED_INITIAL_POSE", 587),
        ("ANIMATION_POPURI_BIG_BED_DIALOGUE_POSE", 591),
        ("ANIMATION_KAREN_BIG_BED_INITIAL_POSE", 1745),
        ("ANIMATION_KAREN_BIG_BED_DIALOGUE_POSE", 1749),
        ("ANIMATION_ANN_BIG_BED_INITIAL_POSE", 2050),
        ("ANIMATION_ANN_BIG_BED_DIALOGUE_POSE", 2054),
        ("ANIMATION_MARY_BIG_BED_INITIAL_POSE", 2135),
        ("ANIMATION_MARY_BIG_BED_DIALOGUE_POSE", 2139),
        ("ANIMATION_ELLI_BIG_BED_INITIAL_POSE", 2264),
        ("ANIMATION_ELLI_BIG_BED_DIALOGUE_POSE", 2268),
    ];

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in expected {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
            let mary::ir::ValueType::UserType(type_id) =
                constants.constant_value_type(name).unwrap()
            else {
                panic!("{target} {name} must have an animation constant type");
            };
            assert_eq!(
                constants.typed_int_const_name(type_id, value),
                Some(name),
                "{target} canonical {name}"
            );
        }
    }

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, _) in expected {
            assert_eq!(
                constants.const_int_value(name),
                None,
                "{target} leaked {name}"
            );
        }
        for (name, value) in [
            ("ANIMATION_POPURI_WEDDING_IDLE", 591),
            ("ANIMATION_KAPPA_HANDS_TOGETHER", 2050),
            ("ANIMATION_WON_IDLE", 2135),
            ("ANIMATION_WON_WALK", 2139),
            ("ANIMATION_VAN_IDLE", 2264),
            ("ANIMATION_VAN_WALK", 2268),
        ] {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
        }
    }
}

#[test]
fn special_spouse_and_harvest_goddess_event_animations_are_target_scoped() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();

    for target in ["MARY_FOMT_US", "MARY_FOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in [
            ("ANIMATION_HARVEST_GODDESS_WEDDING_WALK", 1645),
            ("ANIMATION_HARVEST_GODDESS_HAND_OVER_ITEM", 1649),
        ] {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
            let mary::ir::ValueType::UserType(type_id) =
                constants.constant_value_type(name).unwrap()
            else {
                panic!("{target} {name} must have an animation constant type");
            };
            assert_eq!(constants.typed_int_const_name(type_id, value), Some(name));
        }
        for name in [
            "ANIMATION_GOURMET_IN_BED",
            "ANIMATION_GOURMET_IN_BED_AWAKE",
            "ANIMATION_GOURMET_SETTLE_IN_BED",
            "ANIMATION_WON_IN_BED",
            "ANIMATION_WON_IN_BED_AWAKE",
            "ANIMATION_WON_SETTLE_IN_BED",
        ] {
            assert_eq!(
                constants.const_int_value(name),
                None,
                "{target} leaked {name}"
            );
        }
        assert_eq!(
            constants.const_int_value("ANIMATION_KAREN_MUSIC_FESTIVAL_PERFORMANCE"),
            Some(1705)
        );
        assert_eq!(constants.const_int_value("ANIMATION_ID_2191"), Some(2191));
    }

    for target in ["MARY_MFOMT_US", "MARY_MFOMT_JP"] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        for (name, value) in [
            ("ANIMATION_GOURMET_IN_BED", 1705),
            ("ANIMATION_GOURMET_IN_BED_AWAKE", 1709),
            ("ANIMATION_GOURMET_SETTLE_IN_BED", 1713),
            ("ANIMATION_WON_IN_BED", 2191),
            ("ANIMATION_WON_IN_BED_AWAKE", 2195),
            ("ANIMATION_WON_SETTLE_IN_BED", 2199),
        ] {
            assert_eq!(
                constants.const_int_value(name),
                Some(value),
                "{target} {name}"
            );
            let mary::ir::ValueType::UserType(type_id) =
                constants.constant_value_type(name).unwrap()
            else {
                panic!("{target} {name} must have an animation constant type");
            };
            assert_eq!(constants.typed_int_const_name(type_id, value), Some(name));
        }
        assert_eq!(
            constants.const_int_value("ANIMATION_HARVEST_GODDESS_WEDDING_WALK"),
            None
        );
        assert_eq!(
            constants.const_int_value("ANIMATION_HARVEST_GODDESS_HAND_OVER_ITEM"),
            None
        );
        assert_eq!(constants.const_int_value("ANIMATION_ID_1645"), Some(1645));
        assert_eq!(constants.const_int_value("ANIMATION_ID_1649"), Some(1649));
    }
}

#[test]
fn unresolved_variable_inventory_is_explicit_for_every_target() {
    const FOMT_UNKNOWN_IDS: [i64; 26] = [
        224, 225, 236, 237, 242, 275, 276, 314, 316, 317, 319, 320, 327, 333, 334, 335, 336, 343,
        344, 381, 382, 383, 385, 423, 449, 536,
    ];
    const MFOMT_UNKNOWN_IDS: [i64; 44] = [
        53, 86, 94, 96, 115, 232, 233, 244, 245, 250, 283, 284, 322, 324, 325, 327, 328, 335, 341,
        343, 344, 351, 352, 411, 412, 413, 415, 467, 482, 484, 486, 488, 490, 584, 585, 586, 587,
        588, 589, 590, 591, 628, 697, 705,
    ];
    let mut inventories = Vec::new();
    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(
            &fs::read_to_string("goodies/mary_constants.mary.h").unwrap(),
            &options,
        )
        .unwrap();
        let variable_type = constants.user_type("MaryVarId").unwrap();
        let unknowns = (0..=1024)
            .filter_map(|id| {
                constants
                    .typed_int_const_name(variable_type, id)
                    .filter(|name| name.starts_with("VAR_UNKNOWN_SLOT_"))
                    .map(|name| (id, name.to_owned()))
            })
            .collect::<Vec<_>>();
        let expected_ids = if target.contains("MFOMT") {
            MFOMT_UNKNOWN_IDS.as_slice()
        } else {
            FOMT_UNKNOWN_IDS.as_slice()
        };
        assert_eq!(
            unknowns.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            expected_ids,
            "{target}: physical unknown-slot inventory"
        );
        for (id, name) in &unknowns {
            assert_eq!(*name, format!("VAR_UNKNOWN_SLOT_{id:03}"), "{target}");
        }
        inventories.push((target, unknowns));
    }
    assert_eq!(inventories[0].1, inventories[1].1, "FoMT US/JP");
    assert_eq!(inventories[2].1, inventories[3].1, "MFoMT US/JP");
}

#[test]
fn every_physical_callable_has_a_semantic_name_for_every_target() {
    let header = fs::read_to_string("goodies/mary_constants.mary.h").unwrap();
    let callables_source = fs::read_to_string("goodies/mary_callables.mary.h").unwrap();

    for target in [
        "MARY_FOMT_US",
        "MARY_FOMT_JP",
        "MARY_MFOMT_US",
        "MARY_MFOMT_JP",
    ] {
        let options = Options::default().define(target).unwrap();
        let constants = parse_constant_header(&header, &options).unwrap();
        let callables =
            parse_callable_table_with_scope(&callables_source, &options, &constants).unwrap();

        let numbered = callables
            .scope
            .callable_map()
            .keys()
            .filter(|name| {
                ["Func", "Proc"].iter().any(|prefix| {
                    name.strip_prefix(prefix).is_some_and(|suffix| {
                        !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_hexdigit())
                    })
                })
            })
            .cloned()
            .collect::<Vec<_>>();

        assert!(
            numbered.is_empty(),
            "{target}: numbered callable placeholders remain: {numbered:?}"
        );
    }
}
