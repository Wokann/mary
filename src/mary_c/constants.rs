use std::collections::HashMap;

use thiserror::Error;

use crate::const_scope::ConstScope;

use super::{preprocessor::preprocess_constant_header, Options, PreprocessError};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConstantHeaderError {
    #[error(transparent)]
    Preprocess(#[from] PreprocessError),
    #[error("line {line}: malformed constant header: {message}")]
    Malformed { line: usize, message: String },
}

/// Parse the deliberately small fixed-ID header format used by Mary-C:
///
/// `typedef enum MaryItemId {`
/// `    ITEM_TURNIP = 0,`
/// `} MaryItemId;`
///
/// Enum braces provide explicit, C-compatible constant-group boundaries.
pub fn parse_constant_header(
    source: &str,
    options: &Options,
) -> Result<ConstScope, ConstantHeaderError> {
    let source = preprocess_constant_header(source, options)?;
    let mut scope = ConstScope::new();
    let mut current_type: Option<String> = None;
    let mut awaiting_enum_brace = false;
    let mut in_block_comment = false;
    let mut typed_values: HashMap<(String, i64), String> = HashMap::new();
    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        let mut text = raw.trim();
        if in_block_comment {
            let Some((_, rest)) = text.split_once("*/") else {
                continue;
            };
            in_block_comment = false;
            text = rest.trim();
        }
        while let Some(rest) = text.strip_prefix("/*") {
            if let Some((_, tail)) = rest.split_once("*/") {
                text = tail.trim();
            } else {
                in_block_comment = true;
                text = "";
                break;
            }
        }
        if text.is_empty() || text.starts_with("//") {
            continue;
        }
        if let Some(rest) = text.strip_prefix("typedef int ") {
            let Some(name) = rest.strip_suffix(';').map(str::trim) else {
                return malformed(line, "typedef must end with ';'");
            };
            if current_type.is_some() {
                return malformed(line, "cannot declare a typedef inside an enum");
            }
            if !is_identifier(name) {
                return malformed(line, "invalid typedef name");
            }
            scope.add_or_get_user_type(name.to_owned());
            continue;
        }
        if let Some(rest) = text.strip_prefix("typedef enum ") {
            if current_type.is_some() {
                return malformed(line, "cannot begin an enum before the previous enum ends");
            }
            let (name, has_brace) = rest
                .strip_suffix('{')
                .map_or((rest.trim(), false), |name| (name.trim(), true));
            if !is_identifier(name) {
                return malformed(line, "invalid enum type name");
            }
            scope.add_or_get_user_type(name.to_owned());
            current_type = Some(name.to_owned());
            awaiting_enum_brace = !has_brace;
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_type_subset(") {
            if current_type.is_some() {
                return malformed(line, "cannot declare a type subset inside an enum");
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(line, "mary_type_subset must end with ');'");
            };
            let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            if arguments.len() != 2 || arguments.iter().any(|name| !is_identifier(name)) {
                return malformed(
                    line,
                    "mary_type_subset expects a subset type and a superset type",
                );
            }
            match scope.add_type_subset(arguments[0], arguments[1]) {
                None => return malformed(line, "mary_type_subset references an unknown type"),
                Some(Ok(false)) => {
                    return malformed(line, "mary_type_subset declares the same relation twice")
                }
                Some(Err(())) => {
                    return malformed(line, "mary_type_subset would create a type cycle")
                }
                Some(Ok(true)) => {}
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_typed_identity(") {
            if current_type.is_some() {
                return malformed(line, "cannot declare a typed identity inside an enum");
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(line, "mary_typed_identity must end with ');'");
            };
            let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            if arguments.len() != 2 || arguments.iter().any(|name| !is_identifier(name)) {
                return malformed(
                    line,
                    "mary_typed_identity expects a value type and a wrapper name",
                );
            }
            match scope.add_typed_identity(arguments[0], arguments[1].to_owned()) {
                None => return malformed(line, "mary_typed_identity references an unknown type"),
                Some(false) => {
                    return malformed(line, "mary_typed_identity redeclares a type or name")
                }
                Some(true) => {}
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_const_alias(") {
            if current_type.is_some() {
                return malformed(line, "cannot declare a constant alias inside an enum");
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(line, "mary_const_alias must end with ');'");
            };
            let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            if arguments.len() != 2 || arguments.iter().any(|name| !is_identifier(name)) {
                return malformed(
                    line,
                    "mary_const_alias expects an alias and a canonical constant name",
                );
            }
            let Some(value) = scope.const_int_value(arguments[1]) else {
                return malformed(line, "mary_const_alias references an unknown constant");
            };
            let Some(crate::ir::ValueType::UserType(type_id)) =
                scope.constant_value_type(arguments[1])
            else {
                return malformed(
                    line,
                    "mary_const_alias target is not a typed integer constant",
                );
            };
            if !scope.add_typed_int_alias(arguments[0].to_owned(), value, type_id) {
                return malformed(line, "mary_const_alias redeclares an existing name");
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_callable_return_type_when(") {
            if current_type.is_some() {
                return malformed(
                    line,
                    "cannot bind a conditional callable return type inside an enum",
                );
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(line, "mary_callable_return_type_when must end with ');'");
            };
            let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            if arguments.len() != 4 {
                return malformed(
                    line,
                    "mary_callable_return_type_when expects callable, parameter index, parameter value, and enum type",
                );
            }
            let callable_name = arguments[0];
            if !is_identifier(callable_name) {
                return malformed(
                    line,
                    "mary_callable_return_type_when has an invalid callable name",
                );
            }
            let Some(parameter_index) = parse_integer(arguments[1]) else {
                return malformed(
                    line,
                    "mary_callable_return_type_when parameter index must be an integer",
                );
            };
            let Ok(parameter_index) = usize::try_from(parameter_index) else {
                return malformed(
                    line,
                    "mary_callable_return_type_when parameter index must be nonnegative",
                );
            };
            let parameter_value = scope
                .const_int_value(arguments[2])
                .or_else(|| parse_integer(arguments[2]));
            let Some(parameter_value) = parameter_value else {
                return malformed(
                    line,
                    "mary_callable_return_type_when references an unknown parameter value",
                );
            };
            match scope.add_dependent_callable_return_type(
                callable_name,
                parameter_index,
                parameter_value,
                arguments[3],
            ) {
                None => {
                    return malformed(
                        line,
                        "mary_callable_return_type_when references an unknown value type",
                    )
                }
                Some(false) => {
                    return malformed(
                        line,
                        "mary_callable_return_type_when binds the same callable condition more than once",
                    )
                }
                Some(true) => {}
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_callable_return_type_when_callable(") {
            if current_type.is_some() {
                return malformed(
                    line,
                    "cannot bind a related callable return type inside an enum",
                );
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(
                    line,
                    "mary_callable_return_type_when_callable must end with ');'",
                );
            };
            let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            if arguments.len() != 4 {
                return malformed(
                    line,
                    "mary_callable_return_type_when_callable expects value callable, discriminator callable, discriminator value, and enum type",
                );
            }
            if !is_identifier(arguments[0]) || !is_identifier(arguments[1]) {
                return malformed(
                    line,
                    "mary_callable_return_type_when_callable has an invalid callable name",
                );
            }
            let discriminator_value = scope
                .const_int_value(arguments[2])
                .or_else(|| parse_integer(arguments[2]));
            let Some(discriminator_value) = discriminator_value else {
                return malformed(
                    line,
                    "mary_callable_return_type_when_callable references an unknown discriminator value",
                );
            };
            match scope.add_related_callable_return_type(
                arguments[1],
                discriminator_value,
                arguments[0],
                arguments[3],
            ) {
                None => {
                    return malformed(
                        line,
                        "mary_callable_return_type_when_callable references an unknown value type",
                    )
                }
                Some(false) => {
                    return malformed(
                        line,
                        "mary_callable_return_type_when_callable binds the same callable relation more than once",
                    )
                }
                Some(true) => {}
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_callable_parameter_type_when(") {
            if current_type.is_some() {
                return malformed(
                    line,
                    "cannot bind a conditional callable parameter type inside an enum",
                );
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(line, "mary_callable_parameter_type_when must end with ');'");
            };
            let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            if arguments.len() != 5 {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when expects callable, discriminator parameter index, discriminator value, target parameter index, and enum type",
                );
            }
            if !is_identifier(arguments[0]) {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when has an invalid callable name",
                );
            }
            let Some(discriminator_parameter_index) = parse_integer(arguments[1]) else {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when discriminator parameter index must be an integer",
                );
            };
            let Ok(discriminator_parameter_index) = usize::try_from(discriminator_parameter_index)
            else {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when discriminator parameter index must be nonnegative",
                );
            };
            let discriminator_value = scope
                .const_int_value(arguments[2])
                .or_else(|| parse_integer(arguments[2]));
            let Some(discriminator_value) = discriminator_value else {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when references an unknown discriminator value",
                );
            };
            let Some(target_parameter_index) = parse_integer(arguments[3]) else {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when target parameter index must be an integer",
                );
            };
            let Ok(target_parameter_index) = usize::try_from(target_parameter_index) else {
                return malformed(
                    line,
                    "mary_callable_parameter_type_when target parameter index must be nonnegative",
                );
            };
            match scope.add_dependent_callable_parameter_type(
                arguments[0],
                discriminator_parameter_index,
                discriminator_value,
                target_parameter_index,
                arguments[4],
            ) {
                None => {
                    return malformed(
                        line,
                        "mary_callable_parameter_type_when references an unknown value type",
                    )
                }
                Some(false) => {
                    return malformed(
                        line,
                        "mary_callable_parameter_type_when binds the same callable condition more than once",
                    )
                }
                Some(true) => {}
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("mary_var_type(") {
            if current_type.is_some() {
                return malformed(line, "cannot bind a variable value type inside an enum");
            }
            let Some(arguments) = rest.strip_suffix(");") else {
                return malformed(line, "mary_var_type must end with ');'");
            };
            let Some((variable_name, type_name)) = arguments.split_once(',') else {
                return malformed(
                    line,
                    "mary_var_type expects a variable symbol and an enum type",
                );
            };
            let variable_name = variable_name.trim();
            let type_name = type_name.trim();
            let Some(variable_id) = scope.const_int_value(variable_name) else {
                return malformed(line, "mary_var_type references an unknown variable symbol");
            };
            if scope.has_variable_value_type(variable_id) {
                return malformed(line, "mary_var_type binds the same variable more than once");
            }
            if scope
                .add_variable_value_type(variable_id, type_name)
                .is_none()
            {
                return malformed(line, "mary_var_type references an unknown value type");
            }
            continue;
        }
        if text == "{" {
            if current_type.is_none() || !awaiting_enum_brace {
                return malformed(line, "unexpected enum opening brace");
            }
            awaiting_enum_brace = false;
            continue;
        }
        if let Some(rest) = text.strip_prefix('}') {
            let Some(open_name) = current_type.take() else {
                return malformed(line, "enum closing brace has no matching enum");
            };
            if awaiting_enum_brace {
                return malformed(line, "enum is missing its opening brace");
            }
            let Some(close_name) = rest.strip_suffix(';').map(str::trim) else {
                return malformed(line, "enum closing declaration must end with ';'");
            };
            if close_name != open_name {
                return malformed(
                    line,
                    &format!("enum closes as '{close_name}', but opened as '{open_name}'"),
                );
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("#define") {
            let name = rest.split_whitespace().next().unwrap_or("");
            if current_type.is_none() && name.starts_with("MARY_") {
                continue;
            }
            return malformed(line, "ID constants must be enum members, not '#define'");
        }
        let Some(type_name) = current_type.as_deref() else {
            return malformed(
                line,
                "expected 'typedef enum', 'typedef int', 'mary_const_alias', 'mary_type_subset', 'mary_typed_identity', 'mary_var_type', 'mary_callable_return_type_when', 'mary_callable_return_type_when_callable', or 'mary_callable_parameter_type_when'",
            );
        };
        if awaiting_enum_brace {
            return malformed(line, "expected enum opening brace");
        }
        let Some((name, value)) = text.strip_suffix(',').and_then(|item| item.split_once('='))
        else {
            return malformed(line, "enum member must be 'NAME = integer,'");
        };
        let name = name.trim();
        let value = value.trim();
        if !is_identifier(name) || value.is_empty() {
            return malformed(
                line,
                "enum member must contain one identifier and one integer",
            );
        }
        let value = parse_integer(value).ok_or_else(|| ConstantHeaderError::Malformed {
            line,
            message: format!("'{value}' is not a supported integer"),
        })?;
        let key = (type_name.to_owned(), value);
        if let Some(previous) = typed_values.insert(key, name.to_owned()) {
            return malformed(
                line,
                &format!(
                    "enum '{type_name}' assigns value {value} to both '{previous}' and '{name}'"
                ),
            );
        }
        scope.add_typed_int_const(type_name, name.to_owned(), value);
    }
    if let Some(name) = current_type {
        return malformed(
            source.lines().count().max(1),
            &format!("enum '{name}' is missing its closing declaration"),
        );
    }
    Ok(scope)
}

fn malformed<T>(line: usize, message: &str) -> Result<T, ConstantHeaderError> {
    Err(ConstantHeaderError::Malformed {
        line,
        message: message.to_owned(),
    })
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn parse_integer(value: &str) -> Option<i64> {
    let (negative, digits) = value
        .strip_prefix('-')
        .map_or((false, value), |digits| (true, digits));
    let parsed = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16).ok()?
    } else {
        digits.parse().ok()?
    };
    negative.then_some(-parsed).or(Some(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ConstVal, NameAccess, NameRef};

    #[test]
    fn parses_targeted_typed_enum_members() {
        let source = r#"
typedef enum MaryPortraitId
{
PORTRAIT_COMMON = 1,
#if defined(MARY_FOMT_JP)
PORTRAIT_TARGET = 2,
#else
PORTRAIT_TARGET = 3,
#endif
} MaryPortraitId;
"#;
        let scope =
            parse_constant_header(source, &Options::default().define("MARY_FOMT_JP").unwrap())
                .unwrap();
        let ty = scope.user_type("MaryPortraitId").unwrap();
        assert_eq!(scope.typed_int_const_name(ty, 2), Some("PORTRAIT_TARGET"));
        assert!(matches!(
            scope.lookup_name("PORTRAIT_COMMON"),
            Some(NameRef::Const(ConstVal::Int(1)))
        ));
    }

    #[test]
    fn rejects_an_enum_without_an_explicit_end() {
        let error = parse_constant_header(
            "typedef enum MaryItemId {\nITEM_TURNIP = 0,\n",
            &Options::default(),
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("missing its closing declaration"));
    }

    #[test]
    fn rejects_a_member_outside_an_enum() {
        let error = parse_constant_header("ITEM_TURNIP = 0,\n", &Options::default()).unwrap_err();
        assert!(error.to_string().contains("expected 'typedef enum'"));
    }

    #[test]
    fn rejects_a_mismatched_enum_end_name() {
        let error = parse_constant_header(
            "typedef enum MaryItemId {\nITEM_TURNIP = 0,\n} MaryOtherId;\n",
            &Options::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("opened as 'MaryItemId'"));
    }

    #[test]
    fn rejects_duplicate_values_inside_one_enum() {
        let error = parse_constant_header(
            "typedef enum MaryItemId {\nITEM_A = 1,\nITEM_B = 1,\n} MaryItemId;\n",
            &Options::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("assigns value 1 to both"));
    }

    #[test]
    fn constant_aliases_compile_but_do_not_replace_the_canonical_output_name() {
        let source = r#"
typedef enum MaryAudioSequenceId {
AUDIO_BGM_WEDDING = 6,
} MaryAudioSequenceId;
mary_const_alias(AUDIO_SEQUENCE_006, AUDIO_BGM_WEDDING);
"#;
        let scope = parse_constant_header(source, &Options::default()).unwrap();
        let ty = scope.user_type("MaryAudioSequenceId").unwrap();
        assert_eq!(scope.const_int_value("AUDIO_SEQUENCE_006"), Some(6));
        assert_eq!(
            scope.constant_value_type("AUDIO_SEQUENCE_006"),
            Some(crate::ir::ValueType::UserType(ty))
        );
        assert_eq!(scope.typed_int_const_name(ty, 6), Some("AUDIO_BGM_WEDDING"));
    }

    #[test]
    fn constant_aliases_reject_unknown_targets_and_duplicate_names() {
        let unknown = parse_constant_header(
            "mary_const_alias(OLD_NAME, MISSING_NAME);\n",
            &Options::default(),
        )
        .unwrap_err();
        assert!(unknown.to_string().contains("unknown constant"));

        let duplicate = parse_constant_header(
            "typedef enum MaryId {\nVALUE = 1,\n} MaryId;\n\
             mary_const_alias(VALUE, VALUE);\n",
            &Options::default(),
        )
        .unwrap_err();
        assert!(duplicate
            .to_string()
            .contains("redeclares an existing name"));
    }

    #[test]
    fn binds_a_game_variable_to_its_semantic_value_type() {
        let source = r#"
typedef enum MaryVarId {
VAR_RACE_RESULT = 12,
} MaryVarId;
typedef enum MaryRaceResult {
RACE_NOT_WON = 0,
RACE_WON = 1,
} MaryRaceResult;
mary_var_type(VAR_RACE_RESULT, MaryRaceResult);
"#;
        let scope = parse_constant_header(source, &Options::default()).unwrap();
        let race_type = scope.user_type("MaryRaceResult").unwrap();
        assert_eq!(
            scope.variable_value_type(12),
            Some(crate::ir::ValueType::UserType(race_type))
        );
    }

    #[test]
    fn parses_explicit_enum_subset_relations() {
        let source = r#"
typedef enum MaryCharacterId {
CHARACTER_KAREN = 19,
} MaryCharacterId;
typedef enum MaryEntityId {
ENTITY_KAREN = 19,
ENTITY_DOG = 43,
} MaryEntityId;
mary_type_subset(MaryCharacterId, MaryEntityId);
"#;
        let scope = parse_constant_header(source, &Options::default()).unwrap();
        let character = crate::ir::ValueType::UserType(scope.user_type("MaryCharacterId").unwrap());
        let entity = crate::ir::ValueType::UserType(scope.user_type("MaryEntityId").unwrap());
        assert_eq!(
            scope.narrower_value_type(character, entity),
            Some(character)
        );
        assert_eq!(
            scope.narrower_value_type(entity, character),
            Some(character)
        );

        let duplicate = format!("{source}mary_type_subset(MaryCharacterId, MaryEntityId);\n");
        assert!(parse_constant_header(&duplicate, &Options::default())
            .unwrap_err()
            .to_string()
            .contains("same relation twice"));
        let cycle = format!("{source}mary_type_subset(MaryEntityId, MaryCharacterId);\n");
        assert!(parse_constant_header(&cycle, &Options::default())
            .unwrap_err()
            .to_string()
            .contains("type cycle"));
        let self_cycle = format!("{source}mary_type_subset(MaryCharacterId, MaryCharacterId);\n");
        assert!(parse_constant_header(&self_cycle, &Options::default())
            .unwrap_err()
            .to_string()
            .contains("type cycle"));
        assert!(parse_constant_header(
            "typedef int MaryCharacterId;\nmary_type_subset(MaryCharacterId, Missing);\n",
            &Options::default(),
        )
        .unwrap_err()
        .to_string()
        .contains("unknown type"));
    }

    #[test]
    fn rejects_duplicate_game_variable_value_type_bindings() {
        let source = r#"
typedef enum MaryVarId {
VAR_RACE_RESULT = 12,
} MaryVarId;
typedef enum MaryRaceResult {
RACE_NOT_WON = 0,
RACE_WON = 1,
} MaryRaceResult;
typedef enum MaryBool {
FALSE = 0,
TRUE = 1,
} MaryBool;
mary_var_type(VAR_RACE_RESULT, MaryRaceResult);
mary_var_type(VAR_RACE_RESULT, MaryBool);
"#;
        let error = parse_constant_header(source, &Options::default()).unwrap_err();
        assert!(error.to_string().contains("same variable more than once"));
    }

    #[test]
    fn binds_callable_return_type_to_a_parameter_value() {
        let source = r#"
typedef enum MaryAnimalKind {
ANIMAL_KIND_COW = 1,
} MaryAnimalKind;
typedef enum MaryCowGrowthStage {
COW_GROWTH_STAGE_ADULT = 2,
} MaryCowGrowthStage;
mary_callable_return_type_when(GetGrowthStage, 0, ANIMAL_KIND_COW, MaryCowGrowthStage);
"#;
        let scope = parse_constant_header(source, &Options::default()).unwrap();
        let cow_growth_type = scope.user_type("MaryCowGrowthStage").unwrap();
        assert_eq!(
            scope.dependent_callable_return_type("GetGrowthStage", 0, 1),
            Some(crate::ir::ValueType::UserType(cow_growth_type))
        );
        assert_eq!(
            scope.dependent_callable_return_type("GetGrowthStage", 0, 0),
            None
        );
    }

    #[test]
    fn binds_callable_return_type_to_a_related_callable_result() {
        let source = r#"
typedef enum MaryHeldItemKind {
HELD_ITEM_KIND_FOOD = 0,
} MaryHeldItemKind;
typedef enum MaryFoodId {
FOOD_TURNIP = 0,
} MaryFoodId;
mary_callable_return_type_when_callable(GetPresentedItemId, GetPresentedItemKind, HELD_ITEM_KIND_FOOD, MaryFoodId);
"#;
        let scope = parse_constant_header(source, &Options::default()).unwrap();
        let food_type = scope.user_type("MaryFoodId").unwrap();
        assert_eq!(
            scope.related_callable_return_types("GetPresentedItemKind", 0),
            vec![(
                "GetPresentedItemId",
                crate::ir::ValueType::UserType(food_type)
            )]
        );
        assert!(scope
            .related_callable_return_types("GetPresentedItemKind", 1)
            .is_empty());
    }

    #[test]
    fn binds_callable_parameter_type_to_another_parameter_value() {
        let source = r#"
typedef enum MaryNameEntryKind {
NAME_ENTRY_COW = 1,
} MaryNameEntryKind;
typedef enum MaryAnimalSlotIndex {
ANIMAL_SLOT_1 = 0,
} MaryAnimalSlotIndex;
mary_callable_parameter_type_when(OpenNameEntry, 0, NAME_ENTRY_COW, 1, MaryAnimalSlotIndex);
"#;
        let scope = parse_constant_header(source, &Options::default()).unwrap();
        let slot_type = scope.user_type("MaryAnimalSlotIndex").unwrap();
        assert_eq!(
            scope.dependent_callable_parameter_type("OpenNameEntry", 0, 1, 1),
            Some(crate::ir::ValueType::UserType(slot_type))
        );
        assert_eq!(
            scope.dependent_callable_parameter_type("OpenNameEntry", 0, 0, 1),
            None
        );
    }
}
