# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` is a script compiler and decompiler for the US and Japanese GBA releases of *Harvest Moon: Friends of Mineral Town* (FoMT) and *Harvest Moon: More Friends of Mineral Town* (MFoMT).

```text
RIFF bytecode in ROM
        ↓ decompile
Editable structured source (UTF-8)
        ↓ compile
Byte-exact RIFF/HEX
```

All 5,486 valid vanilla scripts from the four supported ROMs pass the strict source round trip: decode, structured decompile, print source, parse source, compile, encode, and compare byte-for-byte with the original RIFF. None currently requires low-level `ir` or retains `jump next`.

| Version | Valid scripts | Structured source | Byte-exact round trip |
| --- | ---: | ---: | ---: |
| FoMT US | 1,328 | 1,328/1,328 | 1,328/1,328 |
| MFoMT US | 1,415 | 1,415/1,415 | 1,415/1,415 |
| FoMT JP | 1,328 | 1,328/1,328 | 1,328/1,328 |
| MFoMT JP | 1,415 | 1,415/1,415 | 1,415/1,415 |

> This 100% figure describes the four tested vanilla corpora. Arbitrary handwritten programs must still use structures supported by the VM and pass compiler validation.

## Build

Install the Rust toolchain, then run:

```console
cargo build --release
```

The release executable is written to `target/release/mary.exe`.

## Mary-C workflow

Mary-C is the canonical C-shaped frontend. Files use `.mary.c` and `.mary.h`, so editors provide C highlighting while the names still make it clear that Mary extensions require this compiler. See the [Mary-C language reference](docs/MARY_C_LANGUAGE.md) ([简体中文](docs/MARY_C_LANGUAGE.zh-CN.md)).

Decompile all scripts from a Japanese FoMT ROM:

```console
mary decompile rom/fomtjp.gba goodies/fomt_callables.mary.h --all --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap.txt -o decompiled_text/fomt_jp
```

The output directory contains one `.mary.c` per pointer-table slot, including an explicit `NULL` placeholder with no fabricated script body for every empty pointer, plus local `fomt_constants.mary.h`, `fomt_callables.mary.h`, and `fomt_scripts.mary.h`. The `.mary.sym` database is decompiler-only and is never copied or included. Null slots also remain `NULL` in the generated script table. A single-script Mary-C decompile written to a file also copies the fixed constants and callable headers beside that file; when script symbols or a script table are supplied, it generates the local script-table header as well.

Each generated source explicitly selects its ROM target:

```c
#define MARY_FOMT_JP
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"
```

This makes an extracted script independently compilable after copying it together with the two headers:

```console
mary compile decompiled_text/fomt_jp/EventScript_0867.mary.c --mary-c --charmap charmap.txt --binary -o EventScript_0867.riff
```

Available targets are `MARY_FOMT_US`, `MARY_MFOMT_US`, `MARY_FOMT_JP`, and `MARY_MFOMT_JP`. FoMT uses the `fomt_*` tables and MFoMT uses the `mfomt_*` tables. Each family table contains only its own US/JP localization branches; the different FoMT/MFoMT physical ID layouts are no longer interleaved in one file.

## Command-line usage

### Decompile every script

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D TARGET --charmap charmap.txt -o OUTPUT_DIRECTORY
```

For MFoMT, use `goodies/mfomt_callables.mary.h` and `goodies/mfomt_scripts_text.mary.sym`; generated headers are correspondingly named `mfomt_*.mary.h`.

`--all` writes one `.mary.c` per pointer-table slot. Named scripts use their semantic symbol; null pointers become explicit placeholders, so later IDs never shift.

### Decompile one script

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id ID --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D TARGET --charmap charmap.txt -o OUTPUT.mary.c
```

An arbitrary RIFF offset can also be decoded:

```console
mary decompile BINARY goodies/fomt_callables.mary.h --offset OFFSET --mary-c -D TARGET --charmap charmap.txt -o OUTPUT.mary.c
```

`--script-id` and `--offset` are mutually exclusive.
`--offset` accepts either decimal or `0x`-prefixed hexadecimal file offsets. An offset beyond the input is diagnosed instead of panicking.

### Compile source

Generate C data definitions:

```console
mary compile INPUT.mary.c --mary-c --charmap charmap.txt -o OUTPUT.c
```

Generate one binary RIFF directly:

```console
mary compile INPUT.mary.c --mary-c --binary --charmap charmap.txt -o OUTPUT.riff
```

Binary mode accepts exactly one script. Omitting `--binary` emits a C data definition for embedding; that output choice does not make `.mary.c` ordinary C. Mary-C reads supported local `#include` files itself, so no external `cpp` pass is needed.

```console
mary compile INPUT.mary.c --mary-c --print-ir --charmap charmap.txt -o OUTPUT.c
```

`--print-ir` appends stack-VM IR audit comments to textual decompiler output or C byte-array definitions; those comments do not participate in recompilation. A `--binary` result is a raw RIFF byte stream and cannot contain comments, so do not combine the two options.

Use `mary compile --help` and `mary decompile --help` for all options.

## Character map

All ordinary text—including US English and Japanese Shift-JIS text—is converted through the replaceable UTF-8 `charmap.txt` file.

```text
HEX=text or control token
```

Examples:

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
FF22={Horse}
```

Rules:

- The left side is a byte sequence in file order. `8140` means bytes `81 40`; it is never endian-swapped. Numeric ROM pointers and RIFF fields remain little-endian integers.
- Encoding and decoding use longest matching. Multi-byte entries such as `FF21={Player}` stay intact.
- Control names, byte prefixes, and lengths come exclusively from the loaded map. The parser does not hardcode byte prefixes such as `FF` as controls.
- If several byte sequences map to the same text, the first entry is the canonical encoding for handwritten source. Ambiguous original bytes are printed as `\xNN` so round trips cannot silently select another encoding.
- Unmapped bytes are also printed as `\xNN`. This syntax bypasses the map and writes one raw byte.
- `\"` and `\\` are source-level escapes required to place a quote or backslash inside a string. Their ROM bytes still come from the map.
- `HEX=` reserves an unassigned code point and is skipped when the map is loaded. This differs from an entry such as `20= `, whose mapped text is an actual space.

Blank lines and full-line comments beginning with `#` are allowed:

```text
# Message controls
0C=\p
```

Inline comments are not supported. `23=#` remains a valid mapping because the line does not begin with `#`.

## Generated text layout

Every string constant starts on the line after `=`. To match the game's display behavior, generated source splits after `\n` or `\p` and keeps fragments aligned. `\r` only returns the game's horizontal cursor to the start of the line and does not cause a layout break by itself; neither does `{Press}` or any other control:

```c
const MESSAGE_12 =
    "あんたも男なんだから、\r\n"
    "気前がいいところをみせてよ。{Press}"
const MESSAGE_13 =
    "わ…わかったよ。{Press}"
```

This uses C-style adjacent string concatenation. Physical line breaks and indentation outside the quotes add no ROM bytes; the compiler merges the fragments into one string.

## RIFF, CODE, JUMP, and STR

Each script is a `RIFF`/`SCR ` container that normally holds a `CODE` chunk, an optional `JUMP` chunk, and a `STR ` chunk. Lengths, counts, offsets, and instruction operands are read as little-endian integers.

- `CODE`: data is bounded by the chunk length and its internal code length is validated. Branch targets are byte offsets within CODE.
- `JUMP`: the decoder reads the switch-table count and each table offset, then resolves every case/default table through that offset. It does not assume tables are physically contiguous in ID order.
- `STR `: the decoder reads the string count and offset table. Every text ID resolves as “string-pool start + that ID's offset”; physical appearance order is not guessed.

Some modified ROMs may retain the original STR/RIFF declared lengths, place text beyond the RIFF, and point STR offsets at that relocated text. ROM-mode decompilation keeps a backing view from the current RIFF to the remainder of the ROM, so these external strings remain addressable; CODE and JUMP stay bounded by their own chunk lengths.

Compilation produces a standard self-contained RIFF: strings are written consecutively in text-ID order, and the STR offsets and chunk lengths are rebuilt. A modified ROM that uses external strings can therefore be decoded and normalized, but its rebuilt RIFF is not guaranteed to reproduce the modified ROM's unusual physical layout byte-for-byte. The four vanilla ROMs continue to pass strict byte-exact round-trip tests.

## ROM pointer tables

| Version | ROM address | File offset | Slots |
| --- | ---: | ---: | --- |
| FoMT US | `0x080F89D4` | `0x0F89D4` | null ID 0, then 1,328 scripts |
| MFoMT US | `0x081014BC` | `0x1014BC` | null ID 0, then 1,415 scripts |
| FoMT JP | `0x080F8230` | `0x0F8230` | null ID 0, then 1,328 scripts |
| MFoMT JP | `0x0810145C` | `0x10145C` | null ID 0, then 1,415 scripts |

Table length is determined from pointer validity and RIFF data, not by dropping null entries. Empty slots are retained.

## Callables, constants, and script syntax

The family-specific [FoMT callable table](goodies/fomt_callables.mary.h) and [MFoMT callable table](goodies/mfomt_callables.mary.h) define their independent physical ID sequences and document each verified native function's return value, parameter types, and purpose in English and Chinese. The matching constants files hold fixed domains; generated family script-table order defines script slots. Symbols and raw integers compile to identical VM bytes.

The [Mary-C language reference](docs/MARY_C_LANGUAGE.md) is authoritative for the supported C subset, recommended rewrites for unsupported syntax, and the lossless `mary_nodisc`, `mary_switch_compact`, `mary_implicit_default`, `mary_dead_jump`, and `mary_break_switch` forms. The printer rejects residual low-level `ir` or `jump next` rather than reporting false high-level success.

## Verification

`tests/` contains Rust integration tests and a manual verification helper:

- `structured_stress.rs` constructs nested switches, loops, conditionals, breaks, and stack-heavy expressions. Each case performs three compile/decompile/print/recompile rounds and compares the resulting bytes; pairwise and three-level control-flow matrices are included.
- `mary_c_vanilla.rs` performs a strict `RIFF -> Mary-C text -> RIFF` byte round trip on all four ROMs and also verifies script names, text names, and symbolic cross-script calls.
- `vanilla_symmetry.rs` reads every pointer-table slot from all four vanilla ROMs. Each version gets both a direct decode/encode test and a printed high-level source round trip; RIFF bytes must match exactly, with no residual low-level `ir` or `jump next`.
- `common/mod.rs` provides recursive AST inspection shared by the integration tests and is not an independently executed test.
- `decompile_all_roms.bat` manually decompiles all four ROMs for inspection and is not run by `cargo test`.

Run ordinary tests that do not require ROMs with:

```console
cargo test --all-targets
```

Before running the complete ROM suite, place the four files at these fixed paths:

```text
rom/fomt.gba
rom/mfomt.gba
rom/fomtjp.gba
rom/mfomtjp.gba
```

The ROM and decompiled-output directories are ignored by Git. Mary-C tests read the tracked FoMT and MFoMT table families directly; no environment variables are required. Run:

```console
cargo test --all-targets --features test_with_roms
```

To run only the strict four-ROM Mary-C suite and show per-version statistics:

```console
cargo test --features test_with_roms --test mary_c_vanilla -- --nocapture
```

The original structured-DSL symmetry regression can be run separately:

```console
cargo test --features test_with_roms --test vanilla_symmetry -- --nocapture
```

On failure, original and rebuilt RIFF files are stored under the ignored `test_failures/<variant>/` directory. Success means byte-for-byte equality for every regenerated RIFF—not merely successful parsing.

On Windows, generate decompiled source for all four ROMs from the repository root with:

```console
tests\decompile_all_roms.bat
```

The helper builds the current code and writes `decompiled_text/fomt_us`, `mfomt_us`, `fomt_jp`, and `mfomt_jp`. It is intended for inspecting decompiled output and does not replace the strict round-trip suite.

## Related projects

- [StanHash/mary_old](https://github.com/StanHash/mary_old) — earlier C++ implementation.
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC) — FoMT event-script research.
- [StanHash/fomt](https://github.com/StanHash/fomt) — FoMT decompilation project.
