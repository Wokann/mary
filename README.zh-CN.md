# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` 是《牧场物语：矿石镇的伙伴们》（FoMT）和《牧场物语：矿石镇的伙伴们 女孩版》（MFoMT）GBA 日版、美版事件脚本的编译器与反编译器。

```text
ROM 中的 RIFF 字节码
        ↓ 反编译
可编辑的结构化高级语言脚本（UTF-8）
        ↓ 编译
与原始 RIFF/HEX 逐字节一致的字节码
```

四个支持版本的全部 5486 个原版有效脚本均已通过严格源码往返测试：解码、结构化反编译、输出源码、重新解析、编译、编码，最后与原始 RIFF 逐字节比较。当前没有脚本需要低级 `ir`，也没有残留 `jump next`。

| 版本 | 有效脚本 | 结构化高级语言 | 逐字节往返 |
| --- | ---: | ---: | ---: |
| FoMT 美版 | 1328 | 1328/1328 | 1328/1328 |
| MFoMT 美版 | 1415 | 1415/1415 | 1415/1415 |
| FoMT 日版 | 1328 | 1328/1328 | 1328/1328 |
| MFoMT 日版 | 1415 | 1415/1415 | 1415/1415 |

> 这里的 100% 指当前四个原版 ROM 的完整测试语料。任意手写程序仍须使用虚拟机支持的结构，并通过编译器检查。

## 构建

安装 Rust 工具链后执行：

```console
cargo build --release
```

正式版程序位于 `target/release/mary.exe`。

## Mary-C 工作流

Mary-C 是新的 C 形标准前端。文件使用 `.mary.c` 和 `.mary.h` 后缀，编辑器可以直接提供 C 语法高亮，同时后缀也明确表示其中的 `mary_` 扩展必须交给本编译器。完整语法见 [Mary-C 语言说明](docs/MARY_C_LANGUAGE.zh-CN.md)（[English](docs/MARY_C_LANGUAGE.md)）。

以 FoMT 日版为例，批量反编译全部脚本：

```console
mary decompile rom/fomtjp.gba goodies/fomt_callables.mary.h --all --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP --charmap charmap.txt -o decompiled_text/fomt_jp
```

输出目录按指针表的每个槽生成 `.mary.c`，空指针槽也会生成明确标注 `NULL`、不含伪造脚本正文的占位文件；此外还会生成本地 `fomt_constants.mary.h`、`fomt_callables.mary.h`、`fomt_scripts.mary.h`。`.mary.sym` 只供反编译命名使用，不会被复制或 include，空槽同时以 `NULL` 保留在脚本表中。单脚本 Mary-C 反编译写入文件时也会把固定常量头和 callable 头复制到输出文件旁；提供脚本符号表或脚本表时，还会一并生成本地脚本表头。

每个生成脚本都会显式选择 ROM 目标：

```c
#define MARY_FOMT_JP
#include "fomt_callables.mary.h"
#include "fomt_scripts.mary.h"
```

因此，只要把单独脚本与两个头文件放在一起，就可以独立回编：

```console
mary compile decompiled_text/fomt_jp/EventScript_0867.mary.c --mary-c --charmap charmap.txt --binary -o EventScript_0867.riff
```

可用目标为 `MARY_FOMT_US`、`MARY_MFOMT_US`、`MARY_FOMT_JP` 和 `MARY_MFOMT_JP`。FoMT 使用 `fomt_*` 表，MFoMT 使用 `mfomt_*` 表；每个作品表内部只维护自己的 US/JP 本地化差异，不再把男女版差异很大的物理 ID 序列交织在同一文件中。

## 命令行用法

### 解包全部脚本

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D TARGET --charmap charmap.txt -o OUTPUT_DIRECTORY
```

MFoMT 请改用 `goodies/mfomt_callables.mary.h` 与 `goodies/mfomt_scripts_text.mary.sym`，输出头文件也相应使用 `mfomt_*.mary.h`。

`--all` 按指针表槽位输出 `.mary.c`；有语义名的文件使用脚本名，空指针生成明确占位文件，后续脚本 ID 不会偏移。

### 解包单个脚本

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id ID --mary-c --symbols goodies/fomt_scripts_text.mary.sym -D TARGET --charmap charmap.txt -o OUTPUT.mary.c
```

也可以从任意二进制偏移读取 RIFF：

```console
mary decompile BINARY goodies/fomt_callables.mary.h --offset OFFSET --mary-c -D TARGET --charmap charmap.txt -o OUTPUT.mary.c
```

`--script-id` 和 `--offset` 不能同时使用。
`--offset` 接受十进制或带 `0x` 前缀的十六进制文件偏移；越过输入文件会返回错误，不会发生崩溃。

### 编译脚本

生成可嵌入 C 工程的数据定义：

```console
mary compile INPUT.mary.c --mary-c --charmap charmap.txt -o OUTPUT.c
```

直接生成单个 RIFF 二进制：

```console
mary compile INPUT.mary.c --mary-c --binary --charmap charmap.txt -o OUTPUT.riff
```

二进制模式只允许一个脚本。不使用 `--binary` 时输出可供 C 工程嵌入的数据定义；这只是输出容器，不表示 `.mary.c` 能交给普通 C 编译器。Mary-C 会读取源码中的本地 `#include`，无需外部 `cpp`。

```console
mary compile INPUT.mary.c --mary-c --print-ir --charmap charmap.txt -o OUTPUT.c
```

`--print-ir` 在文本形式的反编译源码或 C 字节数组定义中附加虚拟栈 IR 审计注释，不参与回编。`--binary` 输出是纯 RIFF 字节流，无法容纳注释，因此不要把两者组合使用。

使用 `mary compile --help` 和 `mary decompile --help` 查看完整参数。

## 字符码表

所有普通文本，包括美版英文和日版 Shift-JIS 文本，都通过可替换的 UTF-8 文件 `charmap.txt` 转换。

```text
HEX=文本或控制符
```

示例：

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
FF22={Horse}
```

规则：

- 等号左侧是按文件顺序书写的真实字节序列。`8140` 表示字节 `81 40`，不会进行大小端反转；ROM 指针、RIFF 长度等数值字段仍按小端整数读取。
- 编码和解码均采用最长匹配，因此 `FF21={Player}` 这样的多字节条目会被整体处理。
- 控制符名称、字节前缀和长度完全由当前码表决定。解析器不会把 `FF` 等字节前缀硬编码为控制符。
- 多个字节序列映射到同一文本时，第一项是手写源码的默认编码；反编译遇到歧义原始字节时会输出 `\xNN`，防止静默选择其他编码。
- 未映射字节同样输出为 `\xNN`。该写法绕过码表，直接写入一个原始字节。
- `\"` 和 `\\` 是在高级语言字符串中表示双引号与反斜杠所需的源码转义；它们在 ROM 中的字节仍由码表决定。
- `HEX=` 表示保留但未分配的码位，加载码表时会跳过；它与 `20= ` 这类映射到真实空格的条目不同。

允许空行以及以 `#` 开头的整行注释：

```text
# 文本控制符
0C=\p
```

当前不支持行尾注释。`23=#` 仍是合法映射，因为该行不是以 `#` 开头。

## 生成文本的排版

所有字符串常量统一在 `=` 后换行。为了匹配游戏中的显示行为，生成源码在 `\n` 或 `\p` 后分段并保持引号对齐；`\r` 只让游戏的横向光标回到行首，不会单独触发排版换行，`{Press}` 本身以及其他控制符也不会触发排版换行：

```c
const MESSAGE_12 =
    "あんたも男なんだから、\r\n"
    "気前がいいところをみせてよ。{Press}"
const MESSAGE_13 =
    "わ…わかったよ。{Press}"
```

这里使用 C 风格的相邻字符串拼接。引号外的物理换行和缩进不会产生 ROM 字节，编译器会把这些片段合并为同一个字符串。

## RIFF、CODE、JUMP 与 STR

每个脚本是一个 `RIFF`/`SCR ` 容器，内部通常包含 `CODE`、可选的 `JUMP` 和 `STR ` 块。所有长度、数量、偏移和指令操作数均按小端整数读取。

- `CODE`：按块长度取得数据，再校验内部代码长度；跳转目标是 CODE 内的字节偏移。
- `JUMP`：先读取 switch 表数量和每张表的偏移，再按偏移定位对应的 case/default 表；不会假定各表恰好按 ID 顺序连续排列。
- `STR `：先读取文本数量和文本偏移表。每个文本 ID 都通过“字符串池起点 + 该 ID 的偏移”定位，不按物理出现顺序猜测。

部分改版 ROM 可能会保留原 STR/RIFF 的声明长度，却把文本放在 RIFF 外部，并让 STR 偏移指向远处。ROM 模式反编译会保留从当前 RIFF 起点到 ROM 后部的后备视图，因此能够解析这种外置文本；CODE 和 JUMP 仍严格受各自块长度限制。

重新编译时会生成标准的自包含 RIFF：字符串按文本 ID 连续写入 STR 字符串池，偏移表及各块长度重新计算。因此采用外置文本布局的改版 ROM 可以正确读取和规范化重建，但规范化后的 RIFF 不保证与原改版 ROM 的特殊物理布局逐字节一致。四个原始 ROM 则继续通过逐字节严格往返测试。

## ROM 指针表

| 版本 | ROM 地址 | 文件偏移 | 槽位情况 |
| --- | ---: | ---: | --- |
| FoMT 美版 | `0x080F89D4` | `0x0F89D4` | ID 0 为空，之后为 1328 个脚本 |
| MFoMT 美版 | `0x081014BC` | `0x1014BC` | ID 0 为空，之后为 1415 个脚本 |
| FoMT 日版 | `0x080F8230` | `0x0F8230` | ID 0 为空，之后为 1328 个脚本 |
| MFoMT 日版 | `0x0810145C` | `0x10145C` | ID 0 为空，之后为 1415 个脚本 |

表长由指针合法性和 RIFF 数据确定，不会通过删除空项来推断；空槽位会保留。

## callable、常量与脚本语法

按作品拆分的 [FoMT callable 表](goodies/fomt_callables.mary.h) 与 [MFoMT callable 表](goodies/mfomt_callables.mary.h) 分别定义各自的真实物理 ID 序列，并为已确认的原生函数声明返回值、参数类型和中英文用途。固定编号和生成脚本表也使用对应的作品前缀。符号与原始整数会生成相同 VM 字节。

Mary-C 支持的 C 子集、未支持语法的等价改写建议，以及 `mary_nodisc`、`mary_switch_compact`、`mary_implicit_default`、`mary_dead_jump`、`mary_break_switch` 等无损专用语法，统一以 [Mary-C 语言说明](docs/MARY_C_LANGUAGE.zh-CN.md) 为准。打印器若残留低级 `ir` 或 `jump next` 会报错，不会伪装成高级语言成功。

## 验证

`tests/` 是 Rust 集成测试和人工验证辅助脚本目录：

- `structured_stress.rs`：构造嵌套 `switch`、循环、`if/else`、`break` 和栈密集表达式，连续执行三轮“编译 → 结构化反编译 → 输出源码 → 重编译”并比较字节；其中还包含两层和三层控制流组合矩阵。
- `mary_c_vanilla.rs`：对四个 ROM 执行“RIFF → Mary-C 文本 → RIFF”严格逐字节往返，同时验证脚本名、文本名和跨脚本调用符号。
- `vanilla_symmetry.rs`：读取四个原始 ROM 的全部指针表槽位。每个版本分别验证直接解码/编码和高级语言源码往返，要求 RIFF 逐字节一致，并禁止残留低级 `ir` 或 `jump next`。
- `common/mod.rs`：供集成测试共用的 AST 递归检查函数，不会作为独立测试执行。
- `decompile_all_roms.bat`：手动批量解包四个 ROM 的辅助脚本，不会被 `cargo test` 自动执行。

不依赖 ROM 的普通测试：

```console
cargo test --all-targets
```

完整 ROM 测试前，将四个文件放在以下固定位置：

```text
rom/fomt.gba
rom/mfomt.gba
rom/fomtjp.gba
rom/mfomtjp.gba
```

ROM 和解包输出目录均由 Git 忽略。Mary-C 测试直接使用仓库内 FoMT/MFoMT 两套表，不需要设置环境变量。运行：

```console
cargo test --all-targets --features test_with_roms
```

只执行四 ROM 的 Mary-C 严格测试并显示逐版本统计：

```console
cargo test --features test_with_roms --test mary_c_vanilla -- --nocapture
```

同时可单独运行原结构化 DSL 的对称性回归测试：

```console
cargo test --features test_with_roms --test vanilla_symmetry -- --nocapture
```

失败时，原始和重建的 RIFF 会保存在被忽略的 `test_failures/<版本>/` 中。完整 ROM 测试的成功标准不是“能够解析”，而是每个脚本重新生成的 RIFF 与 ROM 原始数据逐字节完全一致。

在 Windows 上批量生成四个 ROM 的全部反编译源码，可从仓库根目录运行：

```console
tests\decompile_all_roms.bat
```

脚本会先构建当前代码，再生成 `decompiled_text/fomt_us`、`mfomt_us`、`fomt_jp` 和 `mfomt_jp`。它只用于检查解包结果，不替代严格往返测试。

## 相关项目

- [StanHash/mary_old](https://github.com/StanHash/mary_old)：早期 C++ 实现。
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC)：FoMT 事件脚本研究资料。
- [StanHash/fomt](https://github.com/StanHash/fomt)：FoMT 反编译项目。
