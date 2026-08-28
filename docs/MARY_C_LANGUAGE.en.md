# Mary-C Language and Lossless Workflow

[简体中文](MARY_C_LANGUAGE.md) | [English](MARY_C_LANGUAGE.en.md)

Mary-C is the C-shaped frontend for Mary's stack VM. Constructs with genuine C semantics use standard C spelling; VM-only behavior that affects the stack or byte layout uses an explicit `mary_` prefix. `.mary.c` and `.mary.h` receive normal C highlighting, but they are not ISO C and must be compiled by `mary`.

A successful decompilation means ROM bytecode can be printed as Mary-C, parsed again, and rebuilt to the exact original RIFF bytes. The printer rejects residual low-level `ir` and `jump next` instead of presenting them as high-level success.

## Files and sources of truth

```text
mary_callables.mary.h          ordered native-callable IDs and prototypes
mary_scripts_text.mary.sym     decompiler-only script/text symbol database
mary_scripts.mary.h            target script slots generated from ROM + symbols
EventScript_NNNN.mary.c        one script, its local text table, and its body
```

The `.mary.sym` file is never included by generated source and is not required for recompilation. Batch decompilation generates a local `mary_scripts.mary.h` from physical ROM slots and selected symbol names, keeping header and function names synchronized.

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

The post-preprocessing order of `mary_callable_table` is the physical `Call(id)` order. Prototypes are declared separately; `NULL` preserves unknown or unavailable slots:

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

`void` declares a procedure and `int` a value-returning function. Parameters may be `int` or `const char *`. There is no invented `CallScript`: a script-launching callable may only be named after its real ROM behavior and ID have been verified.

Script IDs likewise come from an ordered table:

```c
mary_script_table
{
    NULL,
    OpeningEvent,
    MayorExplainsFarm,
};
```

Definitions carry no numeric annotation:

```c
void MayorExplainsFarm(void)
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
    gText_MayorGreeting,
    gText_GrandfatherWill,
};

const char gText_MayorGreeting[] =
    "First line\r\n"
    "Second line{Press}";
```

Reusing one symbol reuses one ID. Equal bytes in two physical slots remain two declarations; the compiler never merges them automatically. Without symbol metadata, names are generated in memory as `gText_<script>_<text-id>` and appear only in `.mary.c`.

## Decompiler symbol database

`mary_scripts_text.mary.sym` is an ordered decompiler symbol table. After target preprocessing, outer positions become script IDs and inner positions become text IDs:

```c
mary_script_symbols
{
    NULL,
    MayorExplainsFarm
    {
        gText_MayorGreeting,
        gText_GrandfatherWill,
#if defined(MARY_JP)
        gText_JapaneseOnlyExplanation,
#endif
    },
#if defined(MARY_MFOMT)
    GirlVersionOnlyEvent {
        gText_GirlVersionOnlyEvent_000,
#if defined(MARY_JP)
        gText_GirlVersionJapaneseOnly_001,
#endif
    },
    GirlVersionNextEvent { gText_GirlVersionNextEvent_000, },
#endif
};
```

The file contains no numeric IDs. Inserting or removing an ordered script/text automatically shifts later IDs. `NULL` preserves an empty script slot. Consecutive target-exclusive scripts share one outer conditional range, while narrower text differences may be nested inside an individual script. Common entries occur once, and conditions wrap only actual differences. Duplicate symbols and malformed tables are errors.

## Supported language

Mary-C supports integer variables/constants, string constants, typed calls, nested calls, assignments, increment/decrement, arithmetic, comparisons, logical expressions, `if`/`else`, `for`, `do`/`while`, `switch`, and `return;`.

Standard `break;` currently exits a switch. The VM backend has no equivalent general loop-break structure, so loop `break` is rejected. `while`, `continue`, value-returning `return`, pointer arithmetic, structs, and arbitrary ordinary C functions are outside the supported subset.

VM-specific lossless forms are:

- `mary_nodisc(expr);` preserves an expression result on the VM stack.
- `mary_switch_compact` preserves a compact switch layout.
- `mary_implicit_default:` preserves an implicit default path.
- `mary_dead_jump:` preserves a layout-affecting unreachable jump.
- `mary_break_switch;` exits an outer switch across an inner loop.

## Commands

Batch decompile:

```console
mary decompile ROM goodies/mary_callables.mary.h --all --mary-c --symbols goodies/mary_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap_jp.txt -o OUTPUT
```

Recompile one generated source:

```console
mary compile OUTPUT/EventScript_0867.mary.c --mary-c --charmap charmap_jp.txt --binary -o EventScript_0867.riff
```

The source `#define` selects the target, and both quoted includes resolve relative to the `.mary.c` file. `-D` remains available for decompilation and automation.

## Verification

`tests/mary_c_vanilla.rs` performs a charmap-aware ROM → Mary-C text → RIFF byte-exact round trip for all valid scripts in FoMT US (1,328), MFoMT US (1,415), FoMT JP (1,328), and MFoMT JP (1,415).

`tests/mary_c_stress.rs` covers unfamiliar nested switch/if/do-while combinations, compact and fallthrough switches, nested calls, retained stack results, and invalid-input diagnostics. The 100% claim applies to those four verified vanilla corpora, not arbitrary C programs.
