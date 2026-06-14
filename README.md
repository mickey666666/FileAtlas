# FileAtlas

FileAtlas 是一个基于 Rust 实现的本地文件索引、内容检索与元数据分析工具。它面向本地学习资料、代码目录、文档目录和普通文件夹，提供从目录扫描、索引保存、文件搜索、内容搜索、统计分析到结果导出的完整命令行流程。

本项目不是简单地每次查询都重新遍历目录，而是采用“先扫描建立索引，再基于索引查询分析”的方式。用户先使用 `scan` 命令扫描指定目录，程序会提取文件路径、文件名、扩展名、大小、修改时间、文本类型、行数和校验值等信息，并保存到本地索引文件中。之后，`find`、`grep`、`list`、`stats`、`tree`、`export` 等命令都可以复用这份索引。

项目主要使用 Rust 标准库完成，覆盖了文件系统操作、路径处理、缓冲读取、字符串处理、集合统计、错误处理、模块化设计、单元测试和标准库并发等内容，适合作为本地文件检索、目录分析和 Rust 命令行工具开发示例。

## 项目特点

- 本地运行，不依赖网络服务
- 支持扫描目录并生成本地索引
- 支持文件名和路径关键词搜索
- 支持文本内容搜索，类似简化版 grep
- 支持 `--jobs` 并发内容检索
- 支持扩展名、路径、大小、日期、文本/二进制等过滤条件
- 支持按路径、文件名、大小、修改时间、扩展名排序
- 支持目录统计报告和扩展名分布分析
- 支持目录树展示，帮助理解项目结构
- 支持 Rust、Python 等源码文件的轻量结构分析
- 支持 CSV 和 JSON 导出
- 支持交互式 shell 模式，方便连续演示
- 输出格式针对长路径做了换行处理，避免终端表格挤压错位

## 功能概览

| 命令      | 功能                                     |
| --------- | ---------------------------------------- |
| `scan`    | 扫描目录，建立文件索引                   |
| `find`    | 按文件名或路径搜索文件                   |
| `grep`    | 在文本文件内容中搜索关键词               |
| `list`    | 列出索引中的文件，可配合过滤和排序       |
| `stats`   | 生成文件数量、大小、扩展名分布等统计信息 |
| `tree`    | 根据索引展示目录树                       |
| `inspect` | 分析单个源码文件的结构信息               |
| `export`  | 将文件记录导出为 CSV 或 JSON             |
| `shell`   | 进入交互式命令模式                       |
| `help`    | 查看帮助信息和命令示例                   |

## 运行环境

需要安装 Rust 工具链，建议使用稳定版 Rust。

检查 Rust 是否可用：

```powershell
rustc --version
cargo --version
```

编译项目：

```powershell
cargo build
```

运行项目：

```powershell
cargo run -- help
```

如果已经编译完成，也可以直接运行生成的可执行文件：

```powershell
target\debug\rust_finder.exe help
```

## 依赖说明

本项目主要基于 Rust 标准库实现，没有额外第三方 crate 依赖。目录扫描、路径处理、文件读写、缓冲读取、集合统计和并发搜索主要使用 `std::fs`、`std::path`、`std::io`、`std::collections` 和 `std::thread` 等标准库模块完成。

## 快速开始

第一次使用时，需要先扫描目录并建立索引：

```powershell
cargo run -- scan .
```

扫描完成后，默认索引会保存到：

```text
.rust_finder/index.rfidx
```

之后就可以执行搜索、统计和导出命令：

```powershell
cargo run -- stats
cargo run -- find README
cargo run -- grep Rust --text
cargo run -- tree src
```

## 命令写法说明

README 中大多数示例都使用 `cargo run -- ...`，这是开发阶段最方便的运行方式。`cargo run --` 后面的内容才是真正传给 FileAtlas 的命令参数。

例如：

```powershell
cargo run -- grep Result --ext rs --jobs 4
```

等价于编译后直接运行：

```powershell
target\debug\rust_finder.exe grep Result --ext rs --jobs 4
```

也可以进入交互式 shell 模式：

```powershell
cargo run -- shell
```

进入 shell 后，就不需要再写 `cargo run --`，可以直接输入命令。文档中如果出现以 `#` 开头的示例，`#` 只是 shell 提示符，不需要手动输入。

```text
# scan .
# stats
# grep Result --ext rs --jobs 4
# exit
```

## 基本使用流程

如果想快速测试功能，可以使用下面这一组命令：

```powershell
cargo run -- scan .
cargo run -- stats
cargo run -- tree src --depth 3
cargo run -- find README
cargo run -- grep Result --ext rs --jobs 4 --limit 10
cargo run -- list --ext rs --sort size --desc --limit 5
cargo run -- inspect src/main.rs
cargo run -- export csv rust_files.csv --ext rs
```

这组命令可以展示索引、搜索、并发内容检索、过滤排序、目录分析、源码分析和导出功能，适合快速测试并熟悉本项目功能。

## scan：扫描目录并建立索引

`scan` 命令会递归扫描指定目录，提取文件元数据，并保存到索引文件中。

扫描当前目录：

```powershell
cargo run -- scan .
```

扫描 `src` 目录：

```powershell
cargo run -- scan ./src
```

使用自定义索引文件：

```powershell
cargo run -- scan . --index demo_index.rfidx
```

跟随符号链接：

```powershell
cargo run -- scan . --follow-links
```

扫描结果包括：

- 扫描根目录
- 文件总数
- 文本文件数量
- 二进制文件数量
- 文件总大小
- 被跳过的路径数量

默认会跳过一些不适合纳入索引的目录，例如：

- `.git`
- `target`
- `node_modules`
- `.rust_finder`
- `.idea`
- `.vscode`
- `__pycache__`

这样可以避免构建产物、依赖目录和索引目录干扰搜索结果。

## find：搜索文件名或路径

`find` 用于按文件名或路径关键词搜索文件。它搜索的是文件名和路径，不搜索文件内容。

搜索 README：

```powershell
cargo run -- find README
```

搜索包含 main 的 Rust 文件：

```powershell
cargo run -- find main --ext rs
```

按修改时间排序，只显示前 5 个：

```powershell
cargo run -- find src --sort modified --desc --limit 5
```

大小写敏感搜索：

```powershell
cargo run -- find README --case-sensitive
```

使用自定义索引：

```powershell
cargo run -- find main --index demo_index.rfidx
```

## grep：搜索文件内容

`grep` 用于在文本文件内容中搜索关键词。它搜索的是文件内容，不是文件名。

搜索所有文本文件中的 Rust：

```powershell
cargo run -- grep Rust --text
```

只搜索 Rust 源码文件：

```powershell
cargo run -- grep Result --ext rs
```

只搜索某个路径下的文件：

```powershell
cargo run -- grep FileRecord --path src
```

显示上下文行：

```powershell
cargo run -- grep Result --ext rs --context 2
```

`--context 2` 表示显示命中行前后各 2 行内容。

大小写敏感搜索：

```powershell
cargo run -- grep FileRecord --ext rs --case-sensitive
```

限制输出数量：

```powershell
cargo run -- grep Result --ext rs --limit 10
```

## 并发内容搜索

`grep` 支持通过 `--jobs <n>` 或 `-j <n>` 开启并发内容搜索。

示例：

```powershell
cargo run -- grep Result --ext rs --jobs 4 --limit 10
```

含义：

- 在扩展名为 `.rs` 的文本文件中搜索 `Result`
- 使用 4 个工作线程并行搜索
- 最多输出 10 条匹配结果

默认情况下，`jobs = 1`，程序使用普通串行搜索。只有当 `jobs > 1` 时，程序才会进入并发搜索流程。

并发搜索的大致流程：

```text
加载索引
筛选符合条件的文本文件
按照 jobs 数量将文件记录分块
使用 std::thread::spawn 创建工作线程
每个线程独立搜索自己负责的文件块
主线程通过 join 收集结果
合并所有匹配结果
按文件原始顺序和行号重新排序
根据 --limit 截断结果
输出结果
```

这样设计的原因是，多线程完成顺序并不固定。如果直接按线程返回顺序输出，结果可能每次运行都不一样。FileAtlas 会保留文件在索引中的原始位置，在合并结果后重新排序，保证并发搜索后的输出仍然稳定。

## list：列出文件记录

`list` 用于列出索引中的文件，可配合过滤和排序使用。

列出所有文件：

```powershell
cargo run -- list
```

只列出 Rust 文件：

```powershell
cargo run -- list --ext rs
```

只列出文本文件：

```powershell
cargo run -- list --text
```

只列出二进制文件：

```powershell
cargo run -- list --binary
```

按大小降序排列：

```powershell
cargo run -- list --sort size --desc --limit 10
```

按修改时间降序排列：

```powershell
cargo run -- list --sort modified --desc --limit 10
```

## stats：查看统计报告

`stats` 会根据索引生成统计报告。

```powershell
cargo run -- stats
```

统计报告包括：

- 扫描根目录
- 索引创建时间
- 文件总数
- 文本文件数量
- 二进制文件数量
- 总大小
- 平均文件大小
- 已知文本总行数
- 跳过项数量
- 扩展名分布
- 最大文件
- 最新文件
- 最旧文件

使用自定义索引：

```powershell
cargo run -- stats --index demo_index.rfidx
```

## tree：显示目录树

`tree` 根据索引中的路径信息还原目录结构。

显示当前索引的目录树：

```powershell
cargo run -- tree
```

显示 `src` 子目录：

```powershell
cargo run -- tree src
```

限制深度：

```powershell
cargo run -- tree --depth 3
```

限制每层显示数量：

```powershell
cargo run -- tree --depth 4 --limit 8
```

## inspect：源码结构分析

`inspect` 用于分析单个源码文件的结构信息。

分析 `main.rs`：

```powershell
cargo run -- inspect src/main.rs
```

分析 `cli.rs`：

```powershell
cargo run -- inspect src/cli.rs
```

目前主要支持轻量级结构提示，例如：

- 函数数量
- 结构体数量
- 枚举数量
- trait 数量
- impl 块数量
- mod 声明数量
- use 引入数量
- 总行数
- 空行数量
- 注释行数量

该功能不是完整编译器级别的语义分析，而是面向代码目录和快速浏览源码的轻量分析。

## export：导出结果

`export` 可以将索引中的文件记录导出为 CSV 或 JSON。

导出 CSV：

```powershell
cargo run -- export csv result.csv
```

导出 JSON：

```powershell
cargo run -- export json result.json
```

只导出 Rust 文件：

```powershell
cargo run -- export csv rust_files.csv --ext rs
```

只导出文本文件：

```powershell
cargo run -- export json text_files.json --text
```

导出内容包括：

- path：文件路径
- name：文件名
- extension：扩展名
- size：文件大小，单位为字节
- human_size：可读大小
- modified：修改时间
- kind：文本或二进制类型
- lines：文本行数
- checksum：样本校验值

CSV 文件可以用 Excel 或 WPS 打开，JSON 文件适合被其他程序继续处理。

## shell：交互式模式

进入 shell 模式：

```powershell
cargo run -- shell
```

进入后可以连续输入命令，例如：

```text
# scan .
# stats
# tree src
# find README
# grep Result --ext rs --jobs 4
# list --ext rs --sort size --desc --limit 5
# inspect src/main.rs
# export csv rust_files.csv --ext rs
# exit
```

shell 模式适合连续操作，因为不需要每次都输入完整的 `cargo run --`。

## 常用过滤参数

这些过滤参数可用于 `find`、`grep`、`list` 和 `export` 等命令。

| 参数                       | 说明                           |
| -------------------------- | ------------------------------ |
| `--index <file>`           | 使用自定义索引文件             |
| `--path <text>`            | 只保留路径中包含指定文本的文件 |
| `--ext <ext>`              | 只保留指定扩展名，可重复使用   |
| `--min-size <size>`        | 最小文件大小，例如 `10kb`      |
| `--max-size <size>`        | 最大文件大小，例如 `2mb`       |
| `--modified-after <date>`  | 只保留指定日期之后修改的文件   |
| `--modified-before <date>` | 只保留指定日期之前修改的文件   |
| `--text`                   | 只保留文本文件                 |
| `--binary`                 | 只保留二进制文件               |
| `--case-sensitive`         | 区分大小写                     |
| `--sort <field>`           | 排序字段                       |
| `--asc`                    | 升序排序                       |
| `--desc`                   | 降序排序                       |
| `--limit <n>`              | 限制输出数量                   |
| `--jobs <n>`               | grep 专用，指定并发搜索线程数  |

支持的排序字段：

```text
path
name
size
modified
ext
```

支持的大小单位：

```text
b
kb
mb
gb
```

日期格式：

```text
YYYY-MM-DD
```

## 索引文件说明

默认索引路径是：

```text
.rust_finder/index.rfidx
```

这个文件保存的是扫描目录后得到的文件元数据，不会保存完整文件内容。它主要用于后续快速查询和统计。

如果重复执行：

```powershell
cargo run -- scan .
```

默认索引文件会被重新生成，不是无限追加增长。

如果想保留不同目录的索引，可以使用不同的索引文件：

```powershell
cargo run -- scan ./src --index src_index.rfidx
cargo run -- scan ./docs --index docs_index.rfidx
```

查询时指定对应索引：

```powershell
cargo run -- stats --index src_index.rfidx
cargo run -- find main --index src_index.rfidx
```

## 更多组合示例

扫描当前项目并查看统计：

```powershell
cargo run -- scan .
cargo run -- stats
```

搜索所有 Rust 源码文件中的错误处理代码：

```powershell
cargo run -- grep AppResult --ext rs --jobs 4 --limit 20
```

搜索 `src` 目录下包含 `FileRecord` 的代码：

```powershell
cargo run -- grep FileRecord --path src --ext rs --context 1
```

找出最大的 10 个文件：

```powershell
cargo run -- list --sort size --desc --limit 10
```

找出最近修改的 10 个 Rust 文件：

```powershell
cargo run -- list --ext rs --sort modified --desc --limit 10
```

只列出 Markdown 文件：

```powershell
cargo run -- list --ext md
```

只列出大于 10KB 的文本文件：

```powershell
cargo run -- list --text --min-size 10kb
```

导出所有 Markdown 文件信息：

```powershell
cargo run -- export csv markdown_files.csv --ext md
```

导出所有文本文件信息：

```powershell
cargo run -- export json text_files.json --text
```

使用独立索引扫描 `src`：

```powershell
cargo run -- scan ./src --index src_index.rfidx
cargo run -- stats --index src_index.rfidx
cargo run -- grep Result --index src_index.rfidx --jobs 4
```

## 项目结构

```text
src/
├── main.rs
├── cli.rs
├── config.rs
├── error.rs
├── model.rs
├── util.rs
├── core/
│   ├── mod.rs
│   ├── scanner.rs
│   └── index.rs
├── query/
│   ├── mod.rs
│   ├── filter.rs
│   └── search.rs
├── analysis/
│   ├── mod.rs
│   ├── stats.rs
│   ├── tree.rs
│   └── code_structure.rs
└── presentation/
    ├── mod.rs
    ├── export.rs
    └── output.rs
```

模块说明：

| 路径                             | 作用                                           |
| -------------------------------- | ---------------------------------------------- |
| `src/main.rs`                    | 程序入口、命令分发和 shell 模式                |
| `src/cli.rs`                     | 命令行参数解析                                 |
| `src/config.rs`                  | 默认配置、默认忽略目录、文本扩展名             |
| `src/error.rs`                   | 统一错误类型和结果类型                         |
| `src/model.rs`                   | 核心数据结构，例如文件记录、索引、搜索结果     |
| `src/util.rs`                    | 通用工具函数，例如大小解析、日期解析、字段转义 |
| `src/core/scanner.rs`            | 目录扫描、元数据提取、文本识别、行数统计       |
| `src/core/index.rs`              | 索引保存和加载                                 |
| `src/query/filter.rs`            | 过滤条件和排序逻辑                             |
| `src/query/search.rs`            | 文件名搜索、内容搜索、并发 grep                |
| `src/analysis/stats.rs`          | 统计报告生成                                   |
| `src/analysis/tree.rs`           | 目录树生成                                     |
| `src/analysis/code_structure.rs` | 源码结构分析                                   |
| `src/presentation/export.rs`     | CSV 和 JSON 导出                               |
| `src/presentation/output.rs`     | 终端输出和帮助信息                             |

## 设计思路说明

FileAtlas 的核心设计思路是把文件系统扫描和查询分析分开。

如果每次执行搜索都重新遍历目录，虽然实现起来比较直接，但会带来几个问题：

- 文件数量多时，每次查询都要重复访问磁盘
- `find`、`grep`、`stats`、`tree`、`export` 等功能会重复读取相同的文件元数据
- 查询逻辑和扫描逻辑容易混在一起，代码结构不清晰
- 后续扩展过滤、排序、统计和导出功能时会出现较多重复代码

因此项目采用索引机制。用户先执行 `scan`，程序扫描真实文件系统并生成 `FileIndex`；之后其他命令都基于索引文件运行。

```text
真实文件系统
    ↓
scan 扫描目录
    ↓
提取文件元数据
    ↓
保存 FileIndex
    ↓
.rust_finder/index.rfidx
    ↓
find / grep / list / stats / tree / export 复用索引
```

这种方式的好处是，扫描只需要执行一次，后续多个命令都可以直接加载索引进行处理。对于本地索引工具来说，这种设计也能更好地体现数据建模、模块拆分、文件 IO 和错误处理能力。

## 核心数据模型

项目中最核心的数据结构是 `FileRecord` 和 `FileIndex`。

`FileRecord` 表示一个被扫描到的文件，它保存的不只是路径，还包括后续搜索、统计和导出所需要的信息：

```text
path          文件完整路径
name          文件名
extension     扩展名
size          文件大小
modified      修改时间
is_text       是否为文本文件
line_count    文本文件行数
checksum      文件样本校验值
```

`FileIndex` 表示一次扫描得到的完整索引：

```text
root          扫描根目录
created_at    索引创建时间
records       文件记录列表
skipped       扫描时跳过或无法读取的路径
```

后续所有功能基本都围绕这两个数据结构展开：

- `find` 使用 `FileRecord` 的文件名和路径进行匹配
- `grep` 使用 `FileRecord` 判断是否需要打开文本文件
- `list` 使用 `FileRecord` 进行过滤和排序
- `stats` 根据 `FileIndex.records` 统计总数、大小和扩展名分布
- `tree` 根据 `FileRecord.path` 还原目录层级
- `export` 将 `FileRecord` 转换为 CSV 或 JSON 字段

## 命令处理流程

项目不是把命令行参数直接散落在各个业务函数中，而是先在 `cli.rs` 中解析为结构化命令。

整体流程如下：

```text
用户输入命令
    ↓
cli.rs 解析参数
    ↓
生成 Command 枚举
    ↓
main.rs 根据命令类型分发
    ↓
调用 scanner / index / search / filter / stats / tree / export
    ↓
presentation/output.rs 统一输出
```

这种结构让命令解析、业务处理和终端展示相互分离。比如以后要修改输出格式，主要修改 `presentation/output.rs`；如果要增加新的过滤条件，主要修改 `cli.rs` 和 `query/filter.rs`；如果要增加新的分析功能，可以放到 `analysis/` 目录。

## 并发 grep 的实现细节

并发内容搜索位于 `src/query/search.rs`。

当用户执行：

```powershell
cargo run -- grep Result --ext rs --jobs 4
```

程序会先完成以下准备工作：

1. 加载索引文件
2. 根据过滤条件筛选文件
3. 跳过二进制文件
4. 保留符合条件的文本文件
5. 检查 `jobs` 是否大于 0

如果 `jobs = 1`，程序使用普通串行搜索。串行搜索会按照索引顺序逐个打开文本文件，使用 `BufReader` 逐行读取并匹配关键词。

如果 `jobs > 1`，程序会进入并发搜索：

```text
records = 符合条件的文本文件
jobs = min(用户指定线程数, records数量)
chunk_size = records数量 / jobs 向上取整
for 每个 chunk:
    thread::spawn 创建线程
    线程独立搜索这一组文件
主线程 join 所有线程
合并结果
排序
截断 limit
输出
```

并发搜索没有让多个线程同时修改同一个 `Vec`，而是让每个线程返回自己的结果集合。主线程统一合并结果。这样可以避免共享可变状态，也更符合 Rust 的所有权和线程安全思想。

并发搜索还需要保证输出稳定。因为不同线程完成顺序不固定，所以 FileAtlas 在线程返回结果时会保留文件在索引中的原始位置。主线程合并结果后，按照：

```text
文件索引位置
命中行号
```

重新排序。这样即使线程完成顺序不同，用户看到的结果顺序仍然稳定。

## 错误处理设计

真实文件系统中经常会遇到各种异常情况，例如：

- 扫描路径不存在
- 路径不是目录
- 某些文件没有读取权限
- 某些目录无法访问
- 索引文件不存在
- 索引格式不正确
- 用户输入了错误参数
- 导出文件无法创建
- 文本文件读取中途失败

项目没有大量使用 `unwrap()`，而是定义了统一错误类型和 `AppResult<T>`。对于无法继续执行的错误，程序会返回明确提示；对于扫描过程中某个文件或目录无法读取的情况，程序会记录到 `skipped` 中，然后继续扫描其他文件。

这种设计让工具在真实目录中更稳定，不会因为一个文件失败就导致整个扫描任务崩溃。

## 文本文件识别

内容搜索只适合文本文件，所以项目需要区分文本和二进制文件。

FileAtlas 使用两种方式组合判断：

1. 根据扩展名判断
2. 根据文件内容样本判断

常见文本扩展名包括：

```text
rs md toml json yaml yml csv txt log html css js ts py java c cpp h hpp xml ini conf lock
```

对于未知扩展名，程序会读取文件前面一部分字节作为样本：

- 如果样本为空，认为是文本文件
- 如果样本包含空字节，倾向认为是二进制文件
- 如果样本可以按 UTF-8 解析，认为是文本文件
- 否则认为是二进制文件

这种方式不是完整的文件类型识别，但对代码目录、文档目录和常见开发项目已经比较实用。

## 输出结果说明

FileAtlas 的终端输出采用分块式展示，而不是把所有字段挤到一行表格里。这样做主要是为了处理 Windows 终端中的中文路径、长路径和不同宽度字体问题。

文件搜索输出示例结构：

```text
1 file(s) matched

README.md
path       D:\...\README.md
size       9.36 KB
type       text
ext        md
lines      571
modified   2026-06-04 20:30:00
```

内容搜索输出示例结构：

```text
3 match(es)

file       D:\...\src\main.rs
line       42
   40- previous context line
   41- previous context line
   42: matched [Result] line
   43+ after context line
```

统计输出示例结构：

```text
Index statistics
root             D:\...\FileAtlas
files            72
text files       49
binary files     23
total size       15.99 MB
known lines      4200
skipped entries  2

Top extensions
rs               13
md                3
toml              1
```

这种输出方式虽然不像传统表格那样紧凑，但在路径很长时更加稳定，不容易出现字段挤在一起的问题。

## CSV 和 JSON 导出说明

导出 CSV 时，程序会处理逗号、双引号、换行符等特殊字符，避免生成的 CSV 文件字段错位。

导出 JSON 时，程序会对字符串中的特殊字符进行转义，保证 JSON 格式合法。

CSV 更适合用 Excel 或 WPS 查看，例如：

```powershell
cargo run -- export csv rust_files.csv --ext rs
```

JSON 更适合给其他程序继续处理，例如：

```powershell
cargo run -- export json files.json --text
```

## 与普通文件搜索的区别

FileAtlas 和普通资源管理器搜索相比，主要区别是：

| 对比项       | 普通文件搜索         | FileAtlas                             |
| ------------ | -------------------- | ------------------------------------- |
| 文件名搜索   | 支持                 | 支持                                  |
| 内容搜索     | 通常较弱或依赖编辑器 | 支持 grep                             |
| 本地索引     | 不可控               | 用户自己扫描生成                      |
| 过滤条件     | 较少                 | 扩展名、路径、大小、日期、文本/二进制 |
| 排序         | 较基础               | 支持多字段排序                        |
| 统计分析     | 通常没有             | 支持文件数量、大小、扩展名分布        |
| 目录树       | 需要额外工具         | 内置 tree                             |
| 导出结果     | 通常不方便           | 支持 CSV/JSON                         |
| 并发内容搜索 | 不可控               | 支持 `--jobs`                         |

因此，FileAtlas 不只是“找文件”，而是围绕本地目录建立索引，并提供搜索、分析和导出的完整流程。

## Rust 特性体现

本项目主要体现了以下 Rust 语言和工程特性：

- 使用 `struct` 建模文件记录、索引、搜索结果和命令参数
- 使用 `enum` 表示不同命令、排序字段、文件类型等状态
- 使用 `Option<T>` 表示可能不存在的扩展名、修改时间和行数
- 使用 `Result<T, E>` 和 `?` 进行错误传播
- 使用 `PathBuf` 和 `Path` 处理跨平台文件路径
- 使用 `BufReader` 和 `BufWriter` 进行缓冲读写
- 使用 `Vec`、`VecDeque`、`BTreeMap` 等集合组织数据
- 使用 trait 抽象文件名匹配和行内容匹配
- 使用模块化结构拆分扫描、索引、搜索、分析和展示逻辑
- 使用 `std::thread::spawn`、`move` 闭包和 `join` 实现并发内容搜索
- 使用单元测试验证过滤、搜索、导出、日期、大小解析等逻辑

## 工程能力说明

FileAtlas 体现了以下工程能力：

| 能力方向       | 项目体现                                             |
| -------------- | ---------------------------------------------------- |
| 完整可运行工具 | 可通过 Cargo 编译运行，提供多个 CLI 命令             |
| 模块化设计     | `core`、`query`、`analysis`、`presentation` 分层清晰 |
| Rust 语言特性  | 使用结构体、枚举、trait、Option、Result、match 等    |
| 文件 IO        | 扫描目录、读取文件、保存索引、导出结果               |
| 错误处理       | 自定义错误类型和 `AppResult<T>`                      |
| 数据处理       | 文件元数据统计、过滤、排序、导出                     |
| 并发能力       | `grep --jobs` 使用标准库线程并发搜索                 |
| 测试           | 包含搜索、过滤、导出、工具函数等单元测试             |
| 文档           | README 和功能示例文档                                |
| 实用性         | 可用于本地项目目录搜索和分析                         |

## 测试与代码检查

格式化代码：

```powershell
cargo fmt
```

检查格式：

```powershell
cargo fmt -- --check
```

运行 Clippy：

```powershell
cargo clippy
```

运行测试：

```powershell
cargo test
```

可以直接使用下面的命令检查代码是否正确：

```powershell
cargo fmt -- --check
cargo clippy
cargo test
```

## 常见问题

### 为什么运行 find 时报索引不存在？

需要先执行扫描：

```powershell
cargo run -- scan .
```

然后再执行：

```powershell
cargo run -- find README
```

### `.rust_finder/index.rfidx` 会一直变大吗？

默认不会无限追加。重新执行 `scan` 时，默认索引文件会重新生成。

如果你扫描的是不同目录，并且使用不同 `--index` 文件，那么会生成多个独立索引。

### `target` 和 `.rust_finder` 为什么会被跳过？

`target` 是 Rust 编译产物目录，里面文件很多，但通常不是用户真正想搜索的源码内容。

`.rust_finder` 是 FileAtlas 自己生成索引的目录，如果把它也扫描进去，会造成结果混乱。

所以项目默认跳过这些目录。

### `grep Rust --text` 是搜索文件名还是内容？

这是搜索文件内容。

如果要搜索文件名或路径，使用：

```powershell
cargo run -- find Rust
```

如果要搜索文本文件内容，使用：

```powershell
cargo run -- grep Rust --text
```

### `export csv result.csv` 导出的是什么？

导出的是索引中的文件信息，不是文件正文。

CSV 中包含文件路径、文件名、扩展名、大小、修改时间、文本类型、行数和校验值等字段。

### `--jobs` 必须使用吗？

不是必须。

普通搜索可以不写：

```powershell
cargo run -- grep Result --ext rs
```

如果文件较多，可以写：

```powershell
cargo run -- grep Result --ext rs --jobs 4
```

### 为什么并发搜索不一定每次都明显更快？

并发搜索适合文件较多、内容搜索范围较大的情况。如果只搜索几个很小的文件，创建线程和合并结果本身也有开销，速度提升可能不明显。这个功能的意义主要是展示项目具备并发处理能力，并且在较大目录下有实际价值。

## 后续可扩展方向

如果继续完善 FileAtlas，可以考虑：

- 增量索引：只更新发生变化的文件
- 正则表达式搜索：支持更复杂的内容匹配
- 倒排索引：提高大规模文本搜索速度
- 多线程扫描：扫描阶段也进行并发目录遍历
- HTML 报告导出：生成更适合展示的可视化报告
- 更完整的源码分析：结合语法解析库分析函数、类型和依赖关系
- TUI 界面：提供终端交互式界面
- 配置文件：允许用户自定义忽略目录和文本扩展名

当前版本已经实现了本地文件索引与内容检索工具的核心功能，后续扩展可以在现有模块结构上继续增加。
