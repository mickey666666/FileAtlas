# RustFinder

RustFinder 是一个使用 Rust 编写的本地文件搜索与内容索引工具。它可以扫描目录、建立索引、搜索文件名、搜索文件内容、按条件过滤、生成统计报告，并把结果导出为 CSV 或 JSON。

## 功能

- 扫描本地目录并建立索引
- 保存和加载索引缓存
- 按文件名或路径搜索
- 在文本文件内容中搜索关键词
- 列出索引中的文件
- 按扩展名、大小、修改日期、文本/二进制类型过滤
- 生成统计报告
- 导出 CSV 或 JSON

## 编译

```powershell
cargo build
```

## 基本使用

先扫描目录：

```powershell
cargo run -- scan .
```

搜索文件名：

```powershell
cargo run -- find README
```

搜索文件内容：

```powershell
cargo run -- grep Rust --text --context 1
```

并发搜索文件内容：

```powershell
cargo run -- grep Result --ext rs --jobs 4 --limit 10
```

查看统计：

```powershell
cargo run -- stats
```

导出结果：

```powershell
cargo run -- export csv result.csv --ext rs
```

## 模块结构

```text
src/main.rs                        程序入口、命令分发和 shell 模式
src/cli.rs                         命令行参数解析
src/config.rs                      默认配置
src/error.rs                       统一错误类型
src/model.rs                       核心数据结构
src/util.rs                        通用工具函数
src/core/scanner.rs                目录扫描与文本识别
src/core/index.rs                  索引保存和加载
src/query/filter.rs                过滤条件
src/query/search.rs                文件名搜索和内容搜索
src/analysis/stats.rs              统计报告
src/analysis/tree.rs               目录树展示
src/analysis/code_structure.rs     源码结构分析
src/presentation/export.rs         JSON/CSV 导出
src/presentation/output.rs         终端输出格式化
```

## 测试与规范

```powershell
cargo fmt
cargo clippy
cargo test
```
