# mary

[English](README.md) | [简体中文](README.zh-CN.md)

`mary` 用于反编译、重新编译并安全导入以下 GBA 版本的事件脚本：

- 《牧场物语：矿石镇的伙伴们》（FoMT）：日版、美版、欧版英文和德版
- 《牧场物语：矿石镇的伙伴们 女孩版》（MFoMT）：日版和美版

Mary-C 是当前唯一的标准源码格式。`.mary.c` 与 `.mary.h` 可以获得 C 语法高亮，
但必须由 `mary` 编译，不能交给普通 C 编译器。与 C 语义一致的结构采用 C 写法；
虚拟栈或物理字节专用结构显式使用 `mary_` 前缀。

```text
ROM 指针表与 RIFF 脚本
          ↓ 反编译
具名 UTF-8 Mary-C 源码及本地头文件
          ↓ 编辑／导入
重新生成 RIFF 和指针并通过校验的 ROM 副本
```

六个受支持原版 ROM 的 8142 个非空脚本均通过严格的
ROM → Mary-C → RIFF 逐字节往返测试。这证明的是六套原版测试语料；手写源码
仍须遵守文档限定的语言和虚拟机能力边界。

| 目标 | 非空脚本 | 严格往返 |
| --- | ---: | ---: |
| FoMT 日版 | 1328 | 1328/1328 |
| FoMT 美版 | 1328 | 1328/1328 |
| FoMT 欧版英文 | 1328 | 1328/1328 |
| FoMT 德版 | 1328 | 1328/1328 |
| MFoMT 日版 | 1415 | 1415/1415 |
| MFoMT 美版 | 1415 | 1415/1415 |

准确的 C 子集、Mary 专用语法、函数/常量表、文本排版和 RIFF 结构请阅读
[Mary-C 语言说明](docs/MARY_C_LANGUAGE.zh-CN.md)（[English](docs/MARY_C_LANGUAGE.md)）。

## 构建

```console
cargo build --release
```

程序生成在 `target/release/mary.exe`。

## 目标版本与生成文件

反编译时必须选择且只能选择一个目标：

```text
MARY_FOMT_JP  MARY_FOMT_US  MARY_FOMT_EU  MARY_FOMT_DE
MARY_MFOMT_JP  MARY_MFOMT_US
```

FoMT 使用 `goodies/fomt_callables.mary.h`、
`goodies/fomt_constants.mary.h` 和
`goodies/fomt_scripts_text.mary.sym`；MFoMT 使用相应的 `mfomt_*` 文件。
`.mary.sym` 只提供反编译命名，不参与回编。

完整反编译会按物理指针表的每个槽位生成一个 `.mary.c`，同时生成常量、函数和
脚本表三份本地头文件。空指针会生成明确的占位文件，并在脚本表中保持 `NULL`，
所以后续 ID 不会偏移。每个生成源码都包含目标 `#define` 并 include 三份头文件。

## 反编译

FoMT 日版完整示例：

```console
mary decompile ROM goodies/fomt_callables.mary.h --all --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT_DIRECTORY
```

单个脚本 ID：

```console
mary decompile ROM goodies/fomt_callables.mary.h --script-id ID --mary-c \
  --symbols goodies/fomt_scripts_text.mary.sym -D MARY_FOMT_JP \
  --charmap charmap.txt -o OUTPUT.mary.c
```

任意文件偏移处的 RIFF 不要求输入具有可识别的 ROM 头：

```console
mary decompile BINARY goodies/fomt_callables.mary.h --offset OFFSET --mary-c \
  -D MARY_FOMT_JP --charmap charmap.txt -o OUTPUT.mary.c
```

`OFFSET` 接受十进制或带 `0x` 前缀的十六进制文件偏移，不能与 `--script-id`
同时使用。如果迁移表无法通过 Mary 元数据自动找到，可提供两个手动恢复值：

```console
mary decompile ROM CALLABLES --all --mary-c -D TARGET --charmap charmap.txt \
  --pointer-table TABLE_ADDRESS --pointer-count SLOT_COUNT -o OUTPUT_DIRECTORY
```

## 编辑并导入 ROM 副本

输入 ROM 始终只读，`-o` 必须指定另一个文件。

在原分配区替换单个脚本：

```console
mary import ROM INPUT.mary.c --script-id ID --charmap charmap.txt -o OUTPUT.gba
```

重新生成的 RIFF 连同对齐填充不能越过下一条脚本；缩短后，原分配区未使用部分
会全部清为 `00`。

在原地址重建完整或不完整的源码目录：

```console
mary import ROM SOURCE_DIRECTORY --charmap charmap.txt -o OUTPUT.gba
```

目录中提供的 `.mary.c` 会重新编译；缺少原 ROM 已有 ID 时保留原 RIFF 字节；
新增槽位没有源码时保持 `NULL`。脚本以四字节零填充方式紧凑排列，必须容纳在
原连续脚本区中；整体缩短后，未使用的原区尾部全部清为 `00`。

迁移单脚本或整体目录：

```console
mary import ROM SOURCE --address SCRIPT_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

迁移单脚本时还要传入 `--script-id`。地址可以是文件偏移或 `0x08xxxxxx` GBA
地址，并且必须四字节对齐。Mary 检查完整目标范围；发现 `00`/`FF` 以外的数据
时，交互环境会先询问，非交互环境必须添加 `--force`。`--dry-run` 完成编译和
全部规划校验，但不生成输出 ROM。

### 扩容或迁移指针表

生成脚本表增加槽位后，必须同时迁移整体脚本块和指针表：

```console
mary import ROM SOURCE_DIRECTORY --address SCRIPT_ADDRESS \
  --relocate-pointer-table TABLE_ADDRESS --charmap charmap.txt -o OUTPUT.gba
```

例如，下面会新增空的 ID 1416 和有脚本的 ID 1417；空指针不会终止后续索引：

```c
    /* 0x0588 */ NULL,
    /* 0x0589 */ EventScript_ExpansionProbe,
```

`EventScript_ExpansionProbe` 必须由某个 `.mary.c` 文件定义。Mary 会写入扩容表，
修补三处已经验证的原生表引用，并在废弃的原始表中保存带校验的发现元数据。
后续反编译和导入会自动复用迁移表；元数据损坏或不可用时，使用
`--pointer-table` 与 `--pointer-count` 手动恢复。

脚本数据、指针表、原生引用和元数据之间禁止重叠。只有编译、边界、占用和必要
确认全部通过后，输出才会通过唯一临时文件原子安装。

## 只编译而不导入

生成原始 RIFF：

```console
mary compile INPUT.mary.c --mary-c --binary --charmap charmap.txt -o OUTPUT.riff
```

生成 C 字节数组定义：

```console
mary compile INPUT.mary.c --mary-c --charmap charmap.txt -o OUTPUT.c
```

`--print-ir` 可以在文本反编译输出或 C 字节数组输出中附加不参与编译的虚拟机
审计注释；它不能和原始二进制输出组合，也绝不会替代结构化 Mary-C 恢复。

使用 `mary decompile --help`、`mary import --help` 和
`mary compile --help` 查看全部选项。

## 字符码表

所有本地化文本均通过 UTF-8 `charmap.txt` 编码和解码：

```text
05={Press}
0A=\n
0D=\r
0C=\p
8140=　
FF21={Player}
```

左侧十六进制是文件顺序的字节序列，不是小端整数。编码和解码采用最长匹配；
控制符名称和长度来自码表。未映射或物理编码有歧义的原字节输出为 `\xNN`；
`HEX=` 保留未分配码位。允许空行和以 `#` 开头的整行注释，不支持行尾注释。
生成源码只在 `\n` 或 `\p` 后为可读性拆分字符串片段；`\r`、`{Press}` 和其他
控制符本身不会拆分源码行。

## 验证

```console
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

ROM 测试使用以下固定路径：

```text
rom/fomtjp.gba
rom/fomt.gba
rom/fomteu.gba
rom/fomtde.gba
rom/mfomtjp.gba
rom/mfomt.gba
```

```console
cargo test --all-targets --features test_with_roms
```

ROM 文件由 Git 忽略。`decompiled_text/` 是工作区生成内容，不属于提交源码；
失败转储保存在被忽略的 `test_failures/` 中。Windows 下，已提交的
`tests\decompile_all_roms.bat` 会构建调试版程序并重新生成六套脚本供检查。

## 相关项目

- [StanHash/mary_old](https://github.com/StanHash/mary_old)：早期 C++ 实现
- [StanHash/FOMT-DOC](https://github.com/StanHash/FOMT-DOC)：FoMT 事件脚本研究
- [StanHash/fomt](https://github.com/StanHash/fomt)：FoMT 反编译项目
