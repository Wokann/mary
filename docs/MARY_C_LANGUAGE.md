# Mary-C Language and Lossless Workflow

[English](MARY_C_LANGUAGE.md) | [简体中文](MARY_C_LANGUAGE.zh-CN.md)

Mary-C is the C-shaped frontend for Mary's stack VM. Constructs with genuine C semantics use standard C spelling; VM-only behavior that affects the stack or byte layout uses an explicit `mary_` prefix. `.mary.c` and `.mary.h` receive normal C highlighting, but they are not ISO C and must be compiled by `mary`.

A successful decompilation means ROM bytecode can be printed as Mary-C, parsed again, and rebuilt to the exact original RIFF bytes. Batch, named, single-script, and public default entry points all require strict structural recovery; residual low-level `ir`, `jump next`, and control flow that cannot be recovered uniquely are errors rather than silent fallback. `--print-ir` only appends explicitly marked low-level audit comments to otherwise successful output; those comments are not recompiled and never substitute for high-level recovery.

## Files and sources of truth

```text
fomt_callables.mary.h / mfomt_callables.mary.h
                               per-game ordered native-callable IDs and prototypes
fomt_constants.mary.h / mfomt_constants.mary.h
                               per-game fixed constants and parameter types
fomt_scripts_text.mary.sym / mfomt_scripts_text.mary.sym
                               per-game decompiler-only script/text symbols
fomt_scripts.mary.h / mfomt_scripts.mary.h
                               per-game script slots generated from ROM + symbols
EventScript_NNNN.mary.c        one script, its local text table, and its body
```

The `.mary.sym` file is never included by generated source and is not required for recompilation. Batch decompilation copies the fixed constants and callable headers and generates a local `fomt_scripts.mary.h` from physical ROM slots and selected symbol names, keeping header and function names synchronized. A single-script decompile written to a file copies the fixed headers as well; when script symbols or a script table are supplied, it also generates the local script-table header.

`fomt_constants.mary.h` and `mfomt_constants.mary.h` are also valid native C headers. Their typedefs, enums, and numeric symbols remain visible when included by the FoMT decompilation project. Mary-only type-propagation declarations erase to empty fixed-arity macros for a native C compiler. Mary defines `MARY_C` internally; under that guard the callable and script headers expose their Mary-only ordered tables and prototypes. Native FoMT builds leave `MARY_C` undefined, so those two headers contribute no declarations and remain owned exclusively by the Mary toolchain. Region-dependent constants use the shared `REGION_JP`, `REGION_US`, `REGION_EU`, and `REGION_DE` macros.

## Target selection

Every `.mary.c` explicitly selects exactly one target:

```c
#define MARY_FOMT_JP
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"
```

Targets, in canonical order, are `MARY_FOMT_JP`, `MARY_FOMT_US`, `MARY_FOMT_EU`, `MARY_FOMT_DE`, `MARY_MFOMT_JP`, and `MARY_MFOMT_US`. FoMT source uses the `fomt_*` files and MFoMT source uses `mfomt_*`. The parser derives `REGION_JP`, `REGION_US`, `REGION_EU`, or `REGION_DE` from the selected formal target. Cross-game layouts are separated by file, so the tables contain no repeated target validation, derived-family boilerplate, or interleaved family conditionals.

Pointer-table modes (`--all` and `--script-id`) verify that this target agrees with the ROM header title and game code. Arbitrary `--offset` input has no such requirement.

The corresponding callable, constants, and `.mary.sym` files test region macros only where localization data actually differs, for example:

```c
#if defined(REGION_JP)
    JapaneseLayoutEntry,
#elif defined(REGION_US)
    EnglishLayoutEntry,
#endif
```

## Ordered IDs

The post-preprocessing order of `mary_callable_table` is the physical `Call(id)` order. Prototypes are declared separately; `NULL` preserves VM-internal or unavailable slots:

```c
mary_callable_table
{
    /* 0x000 */ NULL,
    /* 0x001 */ NULL,
    /* 0x002 */ SetEntityPosition,
    /* 0x003 */ GetEntityX,
};

void SetEntityPosition(int entity, int x, int y, int facing);
int GetEntityX(int entity);
```

`void` declares a procedure and `int` a value-returning function. Parameters may be `int`, `const char *`, or a fixed-ID type. `NULL` preserves a VM-internal, unavailable, or otherwise non-exportable slot and must not be removed. A named `NoOp*` is different from `NULL`: retail scripts really call that slot, but the native handlers in all four ROMs have been verified to produce no effect. Its declaration retains the exact number of operands pushed by the original bytecode so strict byte-for-byte round trips remain possible. For example, `NoOpTutorialFieldTile(...)` carries coordinate- and object-shaped operands, yet its native entry returns immediately; it must not be presented as a fake high-level field-update operation.

The physical callable table also preserves native variants. `FadeInScreenWithoutSceneHook(...)` uses the same fade domains and core inward-transition path as `FadeInScreen(...)`, but skips the active-scene virtual hook executed by the latter before the transition. Each occupies a real `Call(id)` slot and must not be collapsed into an alias. The ROM script-launching callable has been verified through the source-level `ScriptEngine::LoadById` path and is declared as `CallScript(MaryScriptId script_id)`.

Fixed-ID domains use C-shaped enums with explicit brace boundaries:

```c
typedef enum MaryCharacterId
{
    CHARACTER_KAREN = 19,
    CHARACTER_DOCTOR = 20,
} MaryCharacterId;
```

The enum type is only a compile-time label for an argument's numeric domain. Members and raw integers are both erased to the same VM integer before bytecode emission, so `SetTalkNameplateCharacter(CHARACTER_KAREN)` and `SetTalkNameplateCharacter(19)` produce identical bytes. Types whose symbols come from another ordered table and have no fixed members of their own, such as `MaryScriptId`, remain `typedef int`.

In `fomt_constants.mary.h` and `mfomt_constants.mary.h`, food, articles, tools, maps, characters, talk portraits, products, reference pages, audio sequences, fishing records, link milestones, and screen-fade selectors cover their verified complete physical domains; vanilla script usage is not a filter. Every other ID domain must likewise cover its complete native range before it is considered finished—listing only values seen in scripts is insufficient. `MaryVarId` preserves every physical slot from 0 through 589 in FoMT and 0 through 727 in MFoMT. Established slots use semantic names; an unproven slot remains explicitly visible as `VAR_UNKNOWN_SLOT_nnn` until its native readers, writers, lifecycle, and value domain are independently established. The placeholder is evidence of an unresolved identity, not a claim that the slot is unused.

Not every integer parameter is a fixed ID domain. Quantities, times, and counters are selected by runtime context and have no stable cross-script global member table, so they deliberately remain `int`. The open coordinate domain is not a finite enum either, but `MaryMapSpaceX`, `MaryMapSpaceY`, and the identity wrappers `X(...)` and `Y(...)` distinguish its axes without changing values or emitting instructions. Family-local livestock indices are a different case: their physical ranges are proven, so barn and chicken slots use separate types selected from the animal-kind argument. A `Mary*Id`, `Mary*Kind`, or state enum is introduced only when a native ordered table, bit domain, or finite dispatcher proves the complete member set; the few values that happen to occur in vanilla scripts are not enough to justify an incomplete enum.

Some VM integers acquire a narrower domain only after another value is known. The constants header records these facts with compiler metadata: `mary_callable_return_type_when(...)` selects a return domain from one argument, `mary_callable_return_type_when_callable(...)` relates two captured callable results, `mary_callable_parameter_type_when(...)` selects one parameter's domain from another parameter, and `mary_type_subset(Sub, Super)` records a proven containment relation so a local used through both domains keeps the narrower semantic symbols. These declarations emit no bytecode. Mary validates their callable names, parameter indices, and type names when loading the callable table, propagates the selected type through locals and control flow, and keeps a raw number whenever paths conflict without a declared containment relation. Symbol replacement also preserves physical encoding. Ordinary negative literals and named negative constants, such as `-1` and `ITEM_TOOL_NOT_PRESENT`, directly push the negative value by default. If input bytecode instead pushes the positive magnitude and then executes `Neg`, decompilation emits `mary_negated_int(ITEM_TOOL_NOT_PRESENT)`; this explicit Mary-C form keeps the semantic name while requesting the distinct negate-instruction encoding.

`mary_type_subset` relations must be strict and acyclic. Duplicate declarations, self-containment, and direct or transitive cycles are rejected while loading the constants header, preventing narrower-type selection from depending on expression order.

Script IDs likewise come from an ordered table:

```c
mary_script_table
{
    /* 0x0000 */ NULL,
    /* 0x0001 */ EventScript_OpeningEvent,
    /* 0x0002 */ EventScript_MayorExplainsFarm,
};
```

Definitions carry no numeric annotation:

```c
void EventScript_MayorExplainsFarm(void)
{
    return;
}
```

Script symbols are integer constants, so a verified native parameter that really accepts a script ID can use either a symbol or a number. Mary does not infer script-call semantics from arbitrary integer arguments.

## Script-local text IDs

Text IDs are the order of the current script's STR table:

```c
mary_text_table
{
    const char gText_MayorGreeting[] =
        "First line\r\n"
        "Second line{Press}";

    const char gText_GrandfatherWill[] =
        "Next page\p"
        "Continue.{Press}";
};
```

Declaration order inside the table directly determines text IDs, so names are not repeated in a separate slot list. Reusing one symbol reuses one ID. Equal bytes in two physical slots remain two declarations with different names; the compiler never merges them automatically. The former name-list-plus-external-declaration form remains readable for compatibility, but new decompilation always emits declarations inside the table. Without symbol metadata, names are generated in memory as `gText_<script>_<text-id>` and appear only in `.mary.c`.

Text arguments must reference a `gText_*` symbol declared by the current script; an absent name is a compile-time error. The original `GetString` implementation in all four ROMs has an `id <= string_count` bounds bug, but Mary-C does not expose that unsafe extra index: valid IDs are always `0` through `string_count - 1`.

## Decompiler symbol database

`fomt_scripts_text.mary.sym` is an ordered decompiler symbol table. After target preprocessing, outer positions become script IDs and inner positions become text IDs:

```c
mary_script_symbols
{
    NULL,
    EventScript_MayorExplainsFarm
    {
        mary_local_type(var_0, MaryBool),
        gText_MayorGreeting,
        gText_GrandfatherWill,
#if defined(REGION_JP)
        gText_JapaneseOnlyExplanation,
#endif
    },
    EventScript_RegionSensitiveEvent
    {
#if defined(REGION_US)
        gText_EnglishOnlyMessage,
#elif defined(REGION_JP)
        gText_JapaneseOnlyMessage,
#endif
    },
};
```

The file contains no numeric IDs. Inserting or removing an ordered script/text automatically shifts later IDs. `NULL` preserves an empty script slot. Consecutive target-exclusive scripts share one outer conditional range, while narrower text differences may be nested inside an individual script. Common entries occur once, and conditions wrap only actual differences. Duplicate symbols and malformed tables are errors.

`mary_local_type(var_N, Type)` is decompiler-only local-type metadata. It consumes no text slot and is never emitted into generated `.mary.c`. It covers scripts that first store bare integers in a local and later use that local only in conditions or switches, leaving no callable parameter or return type from which to recover the semantic domain. For example, the `MaryBool` annotation above makes a local whose values are `0/1` decompile as `FALSE/TRUE`. The type must exist in the selected target's constants header and the generated `var_N` must exist in that script; unknown types, misspelled locals, and duplicate annotations are errors. Use this only when the local retains one meaning for its entire lifetime. A local reused for unrelated purposes must remain numeric.

### Mary metadata in headers and symbol databases

The following forms are not ISO C declarations. Mary-C reads them while loading tables and recovering semantic types; none emits VM instructions or RIFF bytes:

```c
mary_var_type(VAR_SEASON, MarySeason);
mary_typed_identity(MaryMapSpaceX, X);
mary_type_subset(MaryCharacterId, MaryEntityId);
mary_callable_return_type_when(GetAnimalGrowthStage, 0, ANIMAL_KIND_COW, MaryAnimalCowGrowthStage);
mary_callable_return_type_when_callable(GetPresentedItemId, GetPresentedItemKind, HELD_ITEM_KIND_FOOD, MaryItemFoodId);
mary_callable_parameter_type_when(OpenNameEntry, 0, NAME_ENTRY_COW, 1, MaryAnimalSlotIndex);
```

- `mary_var_type` binds a global game variable to a declared enum domain so reads, writes, comparisons, and `switch` cases can print semantic constants.
- `mary_typed_identity` declares a typed value-preserving wrapper such as `X(120)`; it emits neither a call nor an extra instruction.
- `mary_type_subset` records a proven strict containment relation and preserves the narrowest still-correct enum domain when control-flow paths merge.
- `mary_callable_return_type_when` selects a call's return type from one of that call's parameter values.
- `mary_callable_return_type_when_callable` selects a return type from another captured callable result.
- `mary_callable_parameter_type_when` selects one parameter's type from another parameter value.
- `mary_local_type` occurs only in `.mary.sym` and supplies one stable whole-script type when callable data flow cannot recover a local's domain.

Metadata must reference declared variables, callables, parameter positions, constants, and types. Duplicates, unknown references, invalid parameter indices, cyclic subset relations, and annotations inconsistent with a local's lifetime are diagnosed instead of guessed or silently printed with an incorrect symbol.

## Supported language

Mary-C accepts only syntax with a defined mapping to the Mary stack VM:

| Category | Supported forms |
| --- | --- |
| Types | local `int`, `const int`, text `const char []`, callable `const char *` parameters, and named ID types used by headers for callable/constant propagation |
| Literals | decimal/hexadecimal integers (encodable range `-2147483648` through `0xFFFFFFFF`), charmap-encoded strings, and `\xNN` for retained bytes |
| Expressions | `+ - * / %`, unary `-`/`!`, comparisons, eager `&& ||`, parentheses, and nested callable calls |
| Mutation | `= += -= *= /= %=`, plus prefix/postfix `++` and `--` |
| Statements | declarations, calls, assignments, `if`/`else`, `for`, `do`/`while`, `switch`, and `return;` |
| Switches | shared case bodies, explicit fallthrough, `default`, switch `break;`, and nested control flow |

```c
if (condition) { ... } else { ... }
for (int i = 0; i < 10; i++) { ... }
do { ... } while (condition);
switch (value) { case 1: ... break; default: ... break; }
return;
```

`return;` exits the current VM script; it is not a return from an ordinary C function. Omitting `break;` preserves C fallthrough. Standard `break;` is currently accepted only when it directly belongs to a switch.

Local `int` declarations have block scope. An inner block may shadow an outer
name, and leaving that block restores the outer binding. Variables whose
lifetimes overlap in ancestor and child blocks receive distinct VM slots; only
temporaries in nonoverlapping sibling blocks may reuse a slot. This applies to
nested blocks inside `if`, `for`, `do`/`while`, and `switch` constructs.

A named ID type such as `MaryCharacterId` is a static semantic annotation read from Mary-C headers. Script locals are still declared as `int character`, not `MaryCharacterId character`. The decompiler selects enum symbols from callable parameters, return types, and propagated data flow, while the physical VM value remains an integer.

Standard `break;` currently exits a switch. Loop `break`, `while`, `continue`, `goto`, and user labels are not yet modeled by the Mary-C frontend and structured decompiler, although the VM instruction set can encode all of them with labels and conditional or unconditional jumps. Supporting them requires both compilation and lossless control-flow recovery; one-way bytecode emission alone is not considered complete support. Value-returning `return`, pointer arithmetic, structs, and arbitrary ordinary C functions remain outside the supported subset.

The current `&&` and `||` operators evaluate both operands and then emit VM `LogicalAnd` or `LogicalOr`. They do not provide ISO C short-circuit side-effect semantics. They are C-like only when both operands are free of side effects.

The unsupported surface falls into three different categories:

- **Expressible by the existing IR but not yet modeled by the frontend/decompiler:** `while`, `continue`, loop `break`, `goto`, user labels, the conditional operator, the comma operator, and true short-circuit evaluation. Character literals, limited integer casts, and limited `sizeof` could also be compile-time features once Mary-C defines their exact meanings. Function-like macros are a missing preprocessor feature rather than a VM limitation.
- **No general direct support in the current IR:** script parameters and return values, user-defined callable C functions, pointer dereference and general memory access, runtime arrays/structs/unions, floating point, bitwise operations and shifts, dynamic memory, the C standard library, and ordinary object-file linkage. These require verified native callables or a VM/runtime extension.
- **Superficially C-like but semantically narrower:** addresses may be manipulated as integers but cannot be generally dereferenced; the text table is a build-time RIFF string table rather than a runtime C array; Mary-C headers accept only supported tables, enums, callable declarations, and target conditionals rather than arbitrary ISO C declarations.

Supported `#define`, `#if`, `#elif`, `#else`, `#endif`, and local `#include` forms select targets and read Mary-C tables/declarations; they are not a complete C preprocessor. Unsupported input is diagnosed rather than silently reinterpreted or lowered to pseudo-high-level IR.

### Recommended rewrites for unsupported syntax

#### `while`

Unsupported:

```c
while (HasWork())
{
    DoWork();
}
```

Preserve the pre-test behavior with `for`:

```c
for (int keep_running = HasWork(); keep_running; keep_running = HasWork())
{
    DoWork();
}
```

Use `do/while` only when the body must execute at least once; it is not generally equivalent.

#### `continue`

Unsupported:

```c
for (int i = 0; i < 10; i++)
{
    if (ShouldSkip(i))
    {
        continue;
    }
    Process(i);
}
```

Guard the remaining body instead:

```c
for (int i = 0; i < 10; i++)
{
    if (!ShouldSkip(i))
    {
        Process(i);
    }
}
```

#### Loop `break`

Unsupported:

```c
for (int i = 0; i < 10; i++)
{
    if (ShouldStop(i))
    {
        break;
    }
    Process(i);
}
```

Carry the exit state in the loop condition:

```c
int keep_running = 1;
for (int i = 0; i < 10 && keep_running; i++)
{
    if (ShouldStop(i))
    {
        keep_running = 0;
    }
    else
    {
        Process(i);
    }
}
```

`mary_break_switch;` is not a substitute for loop `break`; it represents a specific original-ROM jump from inside a loop to the end of an enclosing switch.

#### Lexically unreachable statements after control transfer

A statement cannot follow a direct `return;`, `break;`, or an `if/else` whose two branches both terminate within the same block:

```c
switch (state)
{
    case STATE_DONE:
        break;
        Cleanup(); /* Error: this can never execute. */
}
```

Mary-C reports `unreachable statement after return or break`. Accepting this input would produce trailing control flow that the structured decompiler cannot represent again, breaking source round trips. The source formatter also rejects legacy ASTs with such trailing statements instead of emitting Mary-C that cannot be compiled again. This restriction concerns code physically placed after a terminating statement; original-ROM branches that are unreachable only because of runtime conditions are still preserved exactly and are not optimized or rewritten.

#### `goto`, user labels, and arbitrary jumps

Currently unsupported:

```c
goto retry;
retry:
    RetryOperation();
```

Rewrite ordinary conditions and loops with supported `if`, `for`, `do/while`, or `switch` forms. If those structures cannot represent the original control flow losslessly, there is no safe high-level Mary-C spelling yet. `mary_dead_jump:` is not a substitute: it only preserves an unreachable physical tail jump inside a switch and is not a user label.

#### `&&` and `||` with short-circuit side effects

Do not rely on this ISO C behavior:

```c
if (IsReady() && ConsumeItem())
{
    ContinueEvent();
}
```

Nest the conditions so the second call executes only after the first succeeds:

```c
if (IsReady())
{
    if (ConsumeItem())
    {
        ContinueEvent();
    }
}
```

The current `&&` and `||` forms are safe only when both operands are side-effect-free values or queries.

#### Conditional operator

Unsupported:

```c
int result = condition ? when_true : when_false;
```

Use an explicit branch:

```c
int result;
if (condition)
{
    result = when_true;
}
else
{
    result = when_false;
}
```

#### Comma operator, character literals, and limited compile-time operations

Currently unsupported:

```c
int result = (First(), Second());
int letter = 'A';
int width = sizeof(int);
```

Split sequential side effects into statements. Represent a verified byte or charmap character with an integer or named enum, and represent a fixed size with a verified constant:

```c
First();
int result = Second();
int letter = LETTER_A;
int width = MARY_VM_INTEGER_WIDTH;
```

Those example symbols may be used only when the project header actually defines them. Mary-C does not guess that a Unicode character or the host C compiler's `sizeof(int)` equals a ROM value.

#### Value-returning scripts and helper functions

Unsupported:

```c
int AddOne(int value)
{
    return value + 1;
}
```

Mary VM script entries are `void ScriptName(void)`. Inline simple calculations and call verified native callables for engine services:

```c
int result = value + 1;
int affection = GetCharacterLove(CHARACTER_KAREN);
```

Use `CallScript(EventScript_Name);` to start another event script, but a called script cannot return a C value.

#### Pointers, structs, runtime arrays, and allocation

There is no general lossless rewrite for:

```c
Actor *actor = &actors[index];
actor->position.x = 10;
```

Use an independently verified ROM callable when one exists:

```c
SetEntityPosition(entity_id, 10, 20, FACING_DOWN);
```

Without a corresponding callable, Mary-C source alone cannot add the operation. Script text is the only supported array-shaped special form and belongs in `mary_text_table`.

#### Bit operations, shifts, and casts

These have no general syntax at present:

```c
int masked = value & 0x0F;
int shifted = value << 2;
int narrowed = (unsigned char)value;
```

Rewrite with `+ - * / %` only when the input range proves exact equivalence. Otherwise use a verified native callable or wait for an explicit VM/compiler feature; an approximation is not a valid compilation.

Floating point likewise has no general replacement. A verified fixed-point protocol may use named scale constants and integer arithmetic; otherwise use a verified native callable. Mary-C never silently truncates a floating-point literal to an integer.

#### Function-like macros and complex preprocessing

Function-like C macros are unsupported:

```c
#define IS_WEEKEND(day) ((day) == 0 || (day) == 6)
```

Write the expression directly, and put fixed numeric domains in typed enums:

```c
if (day == DAY_OF_WEEK_SUNDAY || day == DAY_OF_WEEK_SATURDAY)
{
    CallScript(EventScript_WeekendEvent);
}
```

Target differences may still use supported conditionals:

```c
#if defined(REGION_JP)
    TalkMessage(gText_JapaneseMessage);
#elif defined(REGION_US)
    TalkMessage(gText_EnglishMessage);
#endif
```

## Mary-specific syntax

These extensions are used only where standard C cannot distinguish the original VM byte layout. They do not add new gameplay features; they preserve byte-exact reconstruction of otherwise identical high-level semantics.

### `mary_negated_int`

A normal negative literal or named constant directly pushes its negative value:

```c
SetHeldTool(ITEM_TOOL_NOT_PRESENT);
```

If the original bytecode first pushes the positive magnitude and then executes
`Neg`, the decompiler makes that physical distinction explicit:

```c
SetHeldTool(mary_negated_int(ITEM_TOOL_NOT_PRESENT));
```

`mary_negated_int` accepts exactly one compile-time integer constant. Both forms
have the high-level value `-1`; the ordinary form directly pushes `-1`, while
the explicit form pushes `1` and executes `Neg`. It exists only for byte-exact
reconstruction and is not general arithmetic or a cast.

VM immediates and `case` values occupy physical 32-bit fields. Mary-C accepts
the complete signed/unsigned bit-pattern range, from `-2147483648` through
`4294967295` (`0xFFFFFFFF`). Values outside that range are diagnosed instead
of being silently truncated to their low 32 bits. Integer text beyond `i64`,
constant-expression overflow, division by zero, and remainder by zero likewise
produce compilation diagnostics rather than crashing the compiler.

### `X(...)` and `Y(...)`

`MaryMapId` selects a map. `X(...)` and `Y(...)` mark axes in that map's local
pixel space, not a world coordinate formed by joining every map:

```c
ChangeMap(MAP_ZACK_HOUSE, X(120), Y(141));
SetEntityPosition(ENTITY_ZACK, X(130), Y(116), FACING_DOWN);
PanCameraTo(X(120), Y(208), CAMERA_MOVE_SPEED_NOMINAL_2_PIXELS_PER_UPDATE);
```

They are typed identity wrappers and emit neither a callable nor extra VM
instructions. Write an ordinary negative coordinate as `Y(-48)`. Only an
original positive-push-plus-`Neg` encoding is printed as
`Y(mary_negated_int(-48))`.

### `mary_nodisc`

```c
value = VarGet(VAR_HOUR);
mary_nodisc(value = VarGet(VAR_HOUR));
```

The first assignment discards its VM-stack result. The second preserves that result because the original bytecode omitted `Discard`. Their visible assignment semantics match, but their bytes and later stack state do not.

### `mary_switch_compact`

```c
mary_switch_compact (choice)
{
case 0:
    Accept();
    break;
default:
    Decline();
    break;
}
```

It has the same high-level behavior as `switch`, but omits the standard layout's dead tail jump before the VM switch dispatcher.

### `mary_implicit_default`

```c
switch (choice)
{
mary_implicit_default:
    Decline();
    break;
case 1:
    Accept();
    break;
}
```

Unlike `default:`, it emits executable default-path code without adding a default entry to the JUMP case table.

### `mary_dead_jump`

```c
switch (choice)
{
case 1:
    Accept();
    break;
mary_dead_jump:
}
```

This preserves an unreachable jump present in the physical switch body. It is not a user label and cannot be targeted by `goto`.

### `mary_break_switch`

```c
switch (choice)
{
case 1:
    do
    {
        if (ShouldStop())
        {
            mary_break_switch;
        }
        Process();
    } while (HasMoreWork());
    Finish();
    break;
default:
    break;
}
```

This jump crosses the inner loop and targets the end of the enclosing switch. It is distinct from an ordinary loop `break;`.

### Complete representative source

The following example combines target selection, local headers, script-local text, typed constants, nested calls, conditions, a loop, a switch, a symbolic cross-script call, a numeric cross-script call, and script exit. It assumes the ordered script table contains both named script symbols shown below:

```c
#define MARY_FOMT_JP
#include "fomt_constants.mary.h"
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"

mary_text_table
{
    const char gText_MorningGreeting[] =
        "おはよう。\r\n"
        "今日もがんばろう。{Press}";

    const char gText_EveningGreeting[] =
        "こんばんは。{Press}";
};

void EventScript_FarmMorningCheck(void)
{
    int hour = VarGet(VAR_HOUR);

    if (hour < 12)
    {
        TalkMessage(gText_MorningGreeting);
    }
    else
    {
        TalkMessage(gText_EveningGreeting);
    }

    for (int i = 0; i < 3; i++)
    {
        if (VarGet(VAR_WEATHER_TODAY) == WEATHER_RAIN)
        {
            TalkMessage(gText_EveningGreeting);
        }
    }

    switch (VarGet(VAR_DAY_OF_WEEK))
    {
        case DAY_OF_WEEK_SUNDAY:
        case DAY_OF_WEEK_SATURDAY:
            CallScript(EventScript_WeekendEvent);
            break;
        default:
            CallScript(638);
            break;
    }

    return;
}
```

The numeric and symbolic cross-script forms are byte-equivalent:

```c
CallScript(638);
CallScript(EventScript_WeekendEvent);
```

The first fixes physical script ID 638. The second resolves the name through the selected target's ordered `mary_script_table`, so it follows deliberate table insertions or target-specific slot changes.

## Commands

### Decompile every pointer-table slot

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap.txt -o OUTPUT
```

The output preserves physical slot order and emits an explicit placeholder file for a null pointer. It also copies the constants/callable headers and generates the target script table.

The six macros `MARY_FOMT_JP`, `MARY_FOMT_US`, `MARY_FOMT_EU`,
`MARY_FOMT_DE`, `MARY_MFOMT_JP`, and `MARY_MFOMT_US` are the supported
targets; exactly one must be selected.

### Decompile one script ID

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id 867 --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap.txt -o EventScript_0867.mary.c
```

### Decompile an arbitrary binary offset

```console
mary decompile INPUT.bin goodies/fomt_callables.mary.h --offset 0x123456 --mary-c -D MARY_FOMT_JP --charmap charmap.txt -o Extracted.mary.c
```

This path does not require a recognized ROM or pointer table. `--offset` accepts decimal or `0x`-prefixed hexadecimal file offsets and must identify a valid RIFF script; an out-of-range or malformed location is diagnosed.

### Compile one generated source to RIFF

```console
mary compile OUTPUT/EventScript_0867.mary.c --mary-c --charmap charmap.txt --binary -o EventScript_0867.riff
```

### Compile as a C byte array

```console
mary compile EventScript_0867.mary.c --mary-c --charmap charmap.txt -o EventScript_0867.inc.c
```

Omit `--binary` to emit the C data definition; this chooses the output container and does not make `.mary.c` valid input to an ordinary C compiler.

### Import into a ROM copy

```console
mary import ROM EventScript.mary.c --script-id ID --charmap charmap.txt -o OUTPUT.gba
mary import ROM OUTPUT_DIRECTORY --charmap charmap.txt -o OUTPUT.gba
```

The first form replaces one slot in its original allocation; the RIFF may use
the original zero alignment padding but may not reach the following script.
Any unused remainder of that allocation is cleared to zero.
The second form recompiles supplied files, preserves original RIFF bytes for
existing IDs whose files are absent, leaves newly added unsupplied slots
`NULL`, and repacks all slots with zero-filled four-byte alignment. Without a
destination override, the packed result must fit the original contiguous
script area, whose unused tail is also cleared to zero.

Relocate the packed scripts and an expanded table with:

```console
mary import ROM OUTPUT_DIRECTORY --address SCRIPT_ADDRESS \
  --relocate-pointer-table TABLE_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

Both file offsets and GBA addresses are accepted. They must be four-byte
aligned, remain inside the ROM, and not overlap each other, native pointer
references, or Mary metadata. Non-`00`/`FF` destination bytes require an
interactive confirmation or `--force`; `--dry-run` validates without writing.
The relocated table is rediscovered through checked metadata stored in the
abandoned native table. `--pointer-table` and `--pointer-count` provide a
manual recovery path. Mary patches the three verified native table references;
the native `ScriptEngine::LoadById` indexes the table directly and imposes no
separate hard-coded slot-count limit.

### Print IR beside Mary-C

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id 867 --mary-c --print-ir -D MARY_FOMT_JP --charmap charmap.txt -o EventScript_0867.mary.c
mary compile EventScript_0867.mary.c --mary-c --print-ir --charmap charmap.txt -o EventScript_0867.c
```

`--print-ir` appends stack-VM audit comments to textual decompiler output or C byte-array definitions. The comments are not recompiled. A `--binary` result is a raw RIFF byte stream and cannot carry comments, so the two options should not be combined.

The source `#define` selects the target, and both quoted includes resolve relative to the `.mary.c` file. `-D` remains available for decompilation and automation.

## Verification

`tests/mary_c_vanilla.rs` performs a charmap-aware ROM → Mary-C text → RIFF byte-exact round trip. FoMT JP/US/EU/DE each preserve 1,329 physical slots (1,328 non-null RIFFs), while MFoMT JP/US each preserve 1,416 physical slots (1,415 non-null RIFFs). Every non-null script must match byte for byte, and every null slot must remain in place.

`tests/mary_c_stress.rs` covers unfamiliar nested switch/if/do-while combinations, compact and fallthrough switches, nested calls, retained stack results, and invalid-input diagnostics. The 100% claim applies to those six verified vanilla corpora, not arbitrary C programs.
