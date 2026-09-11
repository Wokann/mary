# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` decompiles, recompiles, and safely imports event scripts for:

- *Harvest Moon: Friends of Mineral Town*: JP, US, European English, and German
- *Harvest Moon: More Friends of Mineral Town*: JP and US

Mary-C is the canonical source format. Its `.mary.c` and `.mary.h` suffixes
enable C highlighting, but the files must be compiled by `mary`, not a normal C
compiler. C-compatible constructs keep C semantics; VM or physical-byte forms
use an explicit `mary_` prefix.

```text
ROM pointer table and RIFF scripts
              ↓ decompile
Named UTF-8 Mary-C source and local headers
              ↓ edit / import
Validated ROM copy with rebuilt RIFFs and pointers
```

All 8,142 non-null scripts in the six vanilla ROMs pass strict
ROM → Mary-C → RIFF byte-for-byte round trips. This proves the tested vanilla
corpus; handwritten source must still obey the documented language and VM
limits.

| Target | Non-null scripts | Strict round trip |
| --- | ---: | ---: |
| FoMT JP | 1,328 | 1,328/1,328 |
| FoMT US | 1,328 | 1,328/1,328 |
| FoMT EU | 1,328 | 1,328/1,328 |
| FoMT DE | 1,328 | 1,328/1,328 |
| MFoMT JP | 1,415 | 1,415/1,415 |
| MFoMT US | 1,415 | 1,415/1,415 |

See the [Mary-C language reference](docs/MARY_C_LANGUAGE.md)
([简体中文](docs/MARY_C_LANGUAGE.zh-CN.md)) for the exact C subset, Mary-only
syntax, callable/constant tables, text layout, and RIFF structure.

## Build

```console
cargo build --release
```

The executable is `target/release/mary.exe`.

## Targets and generated files

Select exactly one target when decompiling:

```text
MARY_FOMT_JP  MARY_FOMT_US  MARY_FOMT_EU  MARY_FOMT_DE
MARY_MFOMT_JP  MARY_MFOMT_US
```

FoMT uses `goodies/fomt_callables.mary.h`,
`goodies/fomt_constants.mary.h`, and
`goodies/fomt_scripts_text.mary.sym`; MFoMT uses the corresponding `mfomt_*`
files. `.mary.sym` is decompiler-only naming metadata.

A complete decompile writes one `.mary.c` per physical pointer-table slot and
three local headers: constants, callables, and the generated script table.
Null pointers get explicit placeholder files and remain `NULL` in that table,
so later IDs never shift. Each generated source contains its target `#define`
and includes all three headers.

## Decompile

Complete FoMT-JP example:

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT_DIRECTORY
```

One script ID:

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id ID --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT.mary.c
```

An arbitrary RIFF file offset does not require a recognized ROM header:

```console
mary decompile BINARY goodies/fomt_callables.mary.h --offset OFFSET --mary-c \
  -D MARY_FOMT_JP --charmap charmap.txt -o OUTPUT.mary.c
```

`OFFSET` may be decimal or `0x`-prefixed hexadecimal and cannot be combined
with `--script-id`. For a relocated table that cannot be found from Mary
metadata, provide both manual recovery values:

```console
mary decompile ROM CALLABLES --all --mary-c -D TARGET --charmap charmap.txt \
  --pointer-table TABLE_ADDRESS --pointer-count SLOT_COUNT -o OUTPUT_DIRECTORY
```

## Edit and import into a ROM copy

The input ROM is always read-only and `-o` must name a different file.

Replace one script in its original allocation:

```console
mary import ROM INPUT.mary.c --script-id ID --charmap charmap.txt -o OUTPUT.gba
```

The rebuilt RIFF including alignment may not cross the next script. If it is
shorter, the whole unused remainder of its old allocation is cleared to `00`.

Rebuild a complete or partial source directory at the original address:

```console
mary import ROM SOURCE_DIRECTORY --charmap charmap.txt -o OUTPUT.gba
```

Provided `.mary.c` files are compiled. An existing ROM ID omitted from the
directory keeps its original RIFF bytes; a newly added slot without source
remains `NULL`. Scripts are packed with four-byte zero alignment and must fit
the original contiguous area. Its unused old tail is cleared to `00`.

Relocate a script or packed directory:

```console
mary import ROM SOURCE --address SCRIPT_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

For one script, also pass `--script-id`. Addresses may be file offsets or
`0x08xxxxxx` GBA addresses and must be four-byte aligned. Mary checks the whole
destination. Non-`00`/`FF` data requires interactive confirmation or `--force`
in non-interactive use. `--dry-run` validates the complete plan without output.

### Expand or relocate the pointer table

If the generated script table gains slots, relocate both data and table:

```console
mary import ROM SOURCE_DIRECTORY --address SCRIPT_ADDRESS \
  --relocate-pointer-table TABLE_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

For example, these entries add a null ID 1416 followed by a real ID 1417. A
null slot never terminates later indexing:

```c
    /* 0x0588 */ NULL,
    /* 0x0589 */ EventScript_ExpansionProbe,
```

The named script must be defined by a `.mary.c` file. Mary writes the enlarged
table, patches the three verified native table references, and stores checked
discovery metadata in the abandoned original table. Later commands reuse the
relocated table automatically; damaged or unavailable metadata can be bypassed
with `--pointer-table` and `--pointer-count`.

Script data, tables, native references, and metadata cannot overlap. Output is
installed atomically through a unique temporary file only after compilation,
boundary checks, occupancy checks, and any confirmation all succeed.

## Compile without importing

Raw RIFF:

```console
mary compile INPUT.mary.c --mary-c --binary --charmap charmap.txt -o OUTPUT.riff
```

C byte-array definition:

```console
mary compile INPUT.mary.c --mary-c --charmap charmap.txt -o OUTPUT.c
```

`--print-ir` may append non-compiling VM audit comments to textual decompiler
or C-array output. It cannot accompany raw binary output and never substitutes
for structured Mary-C recovery.

Use `mary decompile --help`, `mary import --help`, and `mary compile --help`
for every option.

## Character map

All localized text is encoded and decoded through UTF-8 `charmap.txt`:

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
```

Hexadecimal keys are byte sequences in file order, not little-endian integers.
Encoding and decoding use longest matching; controls and lengths come from the
map. Unmapped or physically ambiguous bytes become `\xNN`; `HEX=` reserves an
unmapped code. Blank lines and full-line `#` comments are accepted, but inline
comments are not. Generated strings split for readability after `\n` or `\p`;
`\r`, `{Press}`, and other controls do not split source lines by themselves.

## Verification

```console
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

For ROM-backed tests, place the files at:

```text
rom/fomtjp.gba
rom/fomt.gba
rom/fomteu.gba
rom/fomtde.gba
rom/mfomtjp.gba
rom/mfomt.gba
```

Then run:

```console
cargo test --all-targets --features test_with_roms
```

ROMs are ignored by Git. `decompiled_text/` is generated workspace output and
is not committed. Failures go to ignored `test_failures/`. On Windows, the
tracked `tests\decompile_all_roms.bat` rebuilds the debug executable and
regenerates all six output directories for inspection.

## Related projects

- [StanHash/mary_old](https://github.com/StanHash/mary_old) — earlier C++ implementation
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC) — FoMT event-script research
- [StanHash/fomt](https://github.com/StanHash/fomt) — FoMT decompilation project
