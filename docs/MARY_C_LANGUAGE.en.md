# Mary-C Language and Lossless Workflow

[简体中文](MARY_C_LANGUAGE.md) | [English](MARY_C_LANGUAGE.en.md)

Mary-C is the C-shaped frontend for Mary's stack VM. Constructs with genuine C semantics use standard C spelling; VM-only behavior that affects the stack or byte layout uses an explicit `mary_` prefix. `.mary.c` and `.mary.h` receive normal C highlighting, but they are not ISO C and must be compiled by `mary`.

A successful decompilation means ROM bytecode can be printed as Mary-C, parsed again, and rebuilt to the exact original RIFF bytes. The printer rejects residual low-level `ir` and `jump next` instead of presenting them as high-level success.

## Files and sources of truth

```text
mary_callables.mary.h          ordered native-callable IDs and prototypes
mary_constants.mary.h          target-selected fixed ID constants and parameter types
mary_scripts_text.mary.sym     decompiler-only script/text symbol database
mary_scripts.mary.h            target script slots generated from ROM + symbols
EventScript_NNNN.mary.c        one script, its local text table, and its body
```

The `.mary.sym` file is never included by generated source and is not required for recompilation. Batch decompilation copies the fixed constants and callable headers and generates a local `mary_scripts.mary.h` from physical ROM slots and selected symbol names, keeping header and function names synchronized. A single-script decompile written to a file copies the fixed headers as well; when script symbols or a script table are supplied, it also generates the local script-table header.

## Target selection

Every `.mary.c` explicitly selects exactly one target:

```c
#define MARY_FOMT_JP
#include "mary_callables.mary.h"
#include "mary_scripts.mary.h"
```

Targets are `MARY_FOMT_US`, `MARY_MFOMT_US`, `MARY_FOMT_JP`, and `MARY_MFOMT_JP`. The compiler automatically derives the corresponding family and region macros: for example, `MARY_MFOMT_JP` also satisfies `MARY_MFOMT` and `MARY_JP`. Script-level ranges normally use `MARY_FOMT`/`MARY_MFOMT`, while language-specific text ranges use `MARY_US`/`MARY_JP`. Exact target macros remain available for combinations that cannot be expressed on one axis.

`mary_callables.mary.h` and the standalone `.mary.sym` spell this relationship out at the beginning of each file, for example:

```c
#if defined(MARY_MFOMT_JP)
#define MARY_MFOMT
#define MARY_JP
#endif
```

## Ordered IDs

The post-preprocessing order of `mary_callable_table` is the physical `Call(id)` order. Prototypes are declared separately; `NULL` preserves VM-internal or unavailable slots:

```c
mary_callable_table
{
    SetEntityPosition,
#if defined(MARY_FOMT)
    FomtOnlyNative,
#elif defined(MARY_MFOMT)
    MfomtOnlyNative,
#endif
    TalkMessage,
    NULL,
};

void SetEntityPosition(int entity, int x, int y, int facing);
void TalkMessage(const char *message);
```

`void` declares a procedure and `int` a value-returning function. Parameters may be `int`, `const char *`, or a fixed-ID type. `NULL` preserves a VM-internal, unavailable, or otherwise non-exportable slot and must not be removed. A named `NoOp*` is different from `NULL`: retail scripts really call that slot, but the native handlers in all four ROMs have been verified to produce no effect. Its declaration retains the exact number of operands pushed by the original bytecode so strict byte-for-byte round trips remain possible. For example, `NoOpTutorialFieldTile(...)` carries coordinate- and object-shaped operands, yet its native entry returns immediately; it must not be presented as a fake high-level field-update operation.

The physical callable table also preserves native aliases. `FadeInScreenAlias(...)` has the same handler body and parameter domains as `FadeInScreen(...)`, but each occupies a real `Call(id)` slot; retaining the alias lets decompiled source compile back to the original ID. The ROM script-launching callable has been verified through the source-level `ScriptEngine::LoadById` path and is declared as `CallScript(MaryScriptId script_id)`.

Fixed-ID domains use C-shaped enums with explicit brace boundaries:

```c
typedef enum MaryCharacterId
{
    CHARACTER_KAREN = 19,
    CHARACTER_DOCTOR = 20,
} MaryCharacterId;
```

The enum type is only a compile-time label for an argument's numeric domain. Members and raw integers are both erased to the same VM integer before bytecode emission, so `SetTalkNameplateCharacter(CHARACTER_KAREN)` and `SetTalkNameplateCharacter(19)` produce identical bytes. Types whose symbols come from another ordered table and have no fixed members of their own, such as `MaryScriptId`, remain `typedef int`.

In `mary_constants.mary.h`, food, articles, tools, maps, characters, talk portraits, products, reference pages, audio sequences, fishing records, link milestones, and screen-fade selectors cover their verified complete physical domains; vanilla script usage is not a filter. Every other ID domain must likewise cover its complete native range before it is considered finished—listing only values seen in scripts is insufficient. `MaryVarId` preserves every physical slot from 0 through 589 in FoMT and 0 through 727 in MFoMT. Established slots use semantic names; an unproven slot remains explicitly visible as `VAR_UNKNOWN_SLOT_nnn` until its native readers, writers, lifecycle, and value domain are independently established. The placeholder is evidence of an unresolved identity, not a claim that the slot is unused.

Not every integer parameter is a fixed ID domain. Quantities, times, and counters are selected by runtime context and have no stable cross-script global member table, so they deliberately remain `int`. The open coordinate domain is not a finite enum either, but `MaryMapSpaceX`, `MaryMapSpaceY`, and the identity wrappers `X(...)` and `Y(...)` distinguish its axes without changing values or emitting instructions. Family-local livestock indices are a different case: their physical ranges are proven, so barn and chicken slots use separate types selected from the animal-kind argument. A `Mary*Id`, `Mary*Kind`, or state enum is introduced only when a native ordered table, bit domain, or finite dispatcher proves the complete member set; the few values that happen to occur in vanilla scripts are not enough to justify an incomplete enum.

Some VM integers acquire a narrower domain only after another value is known. The constants header records these facts with compiler metadata: `mary_callable_return_type_when(...)` selects a return domain from one argument, `mary_callable_return_type_when_callable(...)` relates two captured callable results, `mary_callable_parameter_type_when(...)` selects one parameter's domain from another parameter, and `mary_type_subset(Sub, Super)` records a proven containment relation so a local used through both domains keeps the narrower semantic symbols. These declarations emit no bytecode. Mary validates their callable names, parameter indices, and type names when loading the callable table, propagates the selected type through locals and control flow, and keeps a raw number whenever paths conflict without a declared containment relation. Symbol replacement also preserves physical encoding. Ordinary negative literals and named negative constants, such as `-1` and `TOOL_NOT_PRESENT`, directly push the negative value by default. If input bytecode instead pushes the positive magnitude and then executes `Neg`, decompilation emits `mary_negated_int(TOOL_NOT_PRESENT)`; this explicit Mary-C form keeps the semantic name while requesting the distinct negate-instruction encoding.

`mary_type_subset` relations must be strict and acyclic. Duplicate declarations, self-containment, and direct or transitive cycles are rejected while loading the constants header, preventing narrower-type selection from depending on expression order.

Script IDs likewise come from an ordered table:

```c
mary_script_table
{
    NULL,
    EventScript_OpeningEvent,
    EventScript_MayorExplainsFarm,
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

## Decompiler symbol database

`mary_scripts_text.mary.sym` is an ordered decompiler symbol table. After target preprocessing, outer positions become script IDs and inner positions become text IDs:

```c
mary_script_symbols
{
    NULL,
    EventScript_MayorExplainsFarm
    {
        mary_local_type(var_0, MaryBool),
        gText_MayorGreeting,
        gText_GrandfatherWill,
#if defined(MARY_JP)
        gText_JapaneseOnlyExplanation,
#endif
    },
#if defined(MARY_MFOMT)
    EventScript_GirlVersionOnlyEvent {
        gText_GirlVersionOnlyEvent_000,
#if defined(MARY_JP)
        gText_GirlVersionJapaneseOnly_001,
#endif
    },
    EventScript_GirlVersionNextEvent { gText_GirlVersionNextEvent_000, },
#endif
};
```

The file contains no numeric IDs. Inserting or removing an ordered script/text automatically shifts later IDs. `NULL` preserves an empty script slot. Consecutive target-exclusive scripts share one outer conditional range, while narrower text differences may be nested inside an individual script. Common entries occur once, and conditions wrap only actual differences. Duplicate symbols and malformed tables are errors.

`mary_local_type(var_N, Type)` is decompiler-only local-type metadata. It consumes no text slot and is never emitted into generated `.mary.c`. It covers scripts that first store bare integers in a local and later use that local only in conditions or switches, leaving no callable parameter or return type from which to recover the semantic domain. For example, the `MaryBool` annotation above makes a local whose values are `0/1` decompile as `FALSE/TRUE`. The type must exist in the selected target's constants header and the generated `var_N` must exist in that script; unknown types, misspelled locals, and duplicate annotations are errors. Use this only when the local retains one meaning for its entire lifetime. A local reused for unrelated purposes must remain numeric.

## Supported language

Mary-C accepts only syntax with a defined mapping to the Mary stack VM:

| Category | Supported forms |
| --- | --- |
| Types | local `int`, `const int`, text `const char []`, callable `const char *` parameters, and named ID types used by headers for callable/constant propagation |
| Literals | decimal/hexadecimal integers, charmap-encoded strings, and `\xNN` for retained bytes |
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
#if defined(MARY_JP)
    TalkMessage(gText_JapaneseMessage);
#elif defined(MARY_US)
    TalkMessage(gText_EnglishMessage);
#endif
```

VM-specific lossless forms distinguish byte layouts that ordinary C cannot express:

### `mary_negated_int`

A normal negative literal or named constant directly pushes its negative value:

```c
SetHeldTool(TOOL_NOT_PRESENT);
```

If the original bytecode first pushes the positive magnitude and then executes
`Neg`, the decompiler makes that physical distinction explicit:

```c
SetHeldTool(mary_negated_int(TOOL_NOT_PRESENT));
```

`mary_negated_int` accepts exactly one compile-time integer constant. Both forms
have the high-level value `-1`; the ordinary form directly pushes `-1`, while
the explicit form pushes `1` and executes `Neg`. It exists only for byte-exact
reconstruction and is not general arithmetic or a cast.

### `X(...)` and `Y(...)`

`MaryMapId` selects a map. `X(...)` and `Y(...)` mark axes in that map's local
pixel space, not a world coordinate formed by joining every map:

```c
ChangeMap(MAP_ZACK_HOUSE, X(120), Y(141));
SetEntityPosition(ENTITY_ZACK, X(130), Y(116), FACING_DOWN);
PanCameraTo(X(120), Y(208), CAMERA_MOVE_SPEED_2);
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
#include "mary_constants.mary.h"
#include "mary_callables.mary.h"
#include "mary_scripts.mary.h"

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
mary decompile ROM goodies/mary_callables.mary.h --all --mary-c --symbols goodies/mary_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap_jp.txt -o OUTPUT
```

The output preserves physical slot order and emits an explicit placeholder file for a null pointer. It also copies the constants/callable headers and generates the target script table.

### Decompile one script ID

```console
mary decompile ROM goodies/mary_callables.mary.h --script-id 867 --mary-c --symbols goodies/mary_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap_jp.txt -o EventScript_0867.mary.c
```

### Decompile an arbitrary binary offset

```console
mary decompile INPUT.bin goodies/mary_callables.mary.h --offset 0x123456 --mary-c -D MARY_FOMT_JP --charmap charmap_jp.txt -o Extracted.mary.c
```

This path does not require a recognized ROM or pointer table. `--offset` accepts decimal or `0x`-prefixed hexadecimal file offsets and must identify a valid RIFF script; an out-of-range or malformed location is diagnosed.

### Compile one generated source to RIFF

```console
mary compile OUTPUT/EventScript_0867.mary.c --mary-c --charmap charmap_jp.txt --binary -o EventScript_0867.riff
```

### Compile as a C byte array

```console
mary compile EventScript_0867.mary.c --mary-c --charmap charmap_jp.txt -o EventScript_0867.inc.c
```

Omit `--binary` to emit the C data definition; this chooses the output container and does not make `.mary.c` valid input to an ordinary C compiler.

### Print IR beside Mary-C

```console
mary decompile ROM goodies/mary_callables.mary.h --script-id 867 --mary-c --print-ir -D MARY_FOMT_JP --charmap charmap_jp.txt -o EventScript_0867.mary.c
mary compile EventScript_0867.mary.c --mary-c --print-ir --charmap charmap_jp.txt -o EventScript_0867.c
```

`--print-ir` appends stack-VM audit comments to textual decompiler output or C byte-array definitions. The comments are not recompiled. A `--binary` result is a raw RIFF byte stream and cannot carry comments, so the two options should not be combined.

The source `#define` selects the target, and both quoted includes resolve relative to the `.mary.c` file. `-D` remains available for decompilation and automation.

## Verification

`tests/mary_c_vanilla.rs` performs a charmap-aware ROM → Mary-C text → RIFF byte-exact round trip. FoMT US/JP each preserve 1,329 physical slots (1,328 non-null RIFFs), while MFoMT US/JP each preserve 1,416 physical slots (1,415 non-null RIFFs). Every non-null script must match byte for byte, and every null slot must remain in place.

`tests/mary_c_stress.rs` covers unfamiliar nested switch/if/do-while combinations, compact and fallthrough switches, nested calls, retained stack results, and invalid-input diagnostics. The 100% claim applies to those four verified vanilla corpora, not arbitrary C programs.
