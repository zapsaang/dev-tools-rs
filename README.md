# dev-tools-rs (`dt`)

一个聚合常用开发小工具的 Rust CLI，支持普通命令行输出与 Alfred Workflow JSON 输出。

当前包含：

- `scc`：String Case Converter（字符串风格转换）
- `ucc`：Universal Code Converter（编码/JWT/HTML/Unicode 智能转换）
- `ts`：Timestamp Utility（时间戳/日期解析与格式化）

## 安装

### 从源码安装

```bash
cargo install --path .
```

安装后可执行文件名为 `dt`。

### 运行（开发态）

```bash
cargo run --bin dt -- --help
```

## 使用

### 1) `scc` 字符串风格转换

列出所有格式：

```bash
dt scc --list
```

指定格式（非 Alfred 模式建议显式指定 `-f/--format`）：

```bash
dt scc -f snake "HelloWorld"         # hello_world
dt scc -f kebab "hello_world"        # hello-world
dt scc -f pascal "hello-world"       # HelloWorld
dt scc -f snake_upper "helloWorld"   # HELLO_WORLD
```

Alfred JSON（当 stdout 不是 TTY，或加 `--alfred`）：

```bash
echo "helloWorld" | dt scc
```

### 2) `ucc` 编码/解码/智能识别

对纯文本输入，输出多种编码：

```bash
dt ucc "hello"
```

示例输出包含：Unicode 码点、UTF-8 十六进制、URL 编码、Base64、HTML 转义。

对可识别输入，进行“解码/解释”：

```bash
dt ucc "48656C6C6F"          # Hex -> Hello
dt ucc "aGVsbG8="            # Base64 -> hello
dt ucc "%E4%BD%A0%E5%A5%BD"  # URL -> 你好
dt ucc "&lt;div&gt;"           # HTML Entity -> <div>
dt ucc "U+4F60 U+597D"       # Unicode points -> 你好
```

JWT payload 解码（支持 URL-safe / 缺 padding）：

```bash
dt ucc "<header>.<payload>.<signature>"
```

安静输出（只打印结果，适合脚本）：

```bash
dt ucc -q "aGVsbG8="
```

JSON 输出：

```bash
dt ucc --json "aGVsbG8="
```

Alfred JSON：

```bash
echo "hello" | dt ucc
```

### 3) `ts` 时间戳与日期工具

输入可以是：

- `now`（或空输入）
- 时间戳（秒 10 位 / 毫秒 13 位）
- RFC3339（如 `2023-01-01T12:00:00Z`）
- 常见日期格式（如 `YYYY-MM-DD HH:mm:ss`）

示例：

```bash
dt ts now
dt ts 1700000000

dt ts "2023-01-01T12:00:00Z"
dt ts "2023-01-01 12:00:00"
```

Alfred JSON：

```bash
echo 1700000000 | dt ts
```

## Alfred 输出模式说明

当满足任一条件时会输出 Alfred Workflow 所需 JSON：

- 显式 `--alfred`
- stdout 不是交互式终端（例如被 pipe/重定向）

你也可以用 `--json`（仅 `ucc`）来强制 JSON 输出。

## 开发与测试

```bash
cargo fmt
cargo test
```

## CI/CD（GitHub Actions）

本仓库包含两个工作流：

- CI：push / PR 自动运行 `cargo fmt`、`cargo clippy -D warnings`、`cargo test`
- Release：合并到 `main` 后，如果检测到当前版本号 `vX.Y.Z` tag 不存在，则：
	- 构建 macOS `x86_64` 与 `aarch64` 产物并打包为 tar.gz
	- 创建 GitHub Release（tag 为 `vX.Y.Z`）并上传产物
	- 自动更新 Homebrew tap 仓库中的 formula 并 push

### 需要配置的 Secrets

在 GitHub 仓库 Settings → Secrets and variables → Actions 中添加：

- `HOMEBREW_TAP_REPO`：你的 tap 仓库，例如 `yourname/homebrew-tap`
- `HOMEBREW_TAP_TOKEN`：有写入 tap 仓库权限的 PAT（至少 `repo` 权限）
- `HOMEBREW_FORMULA_PATH`（可选）：formula 文件路径，默认 `Formula/dt.rb`

### 测试覆盖说明

- 单元测试：覆盖 `scc/ucc/ts` 的解析与转换核心逻辑
- 集成测试：通过 `assert_cmd` 调用 `dt` 验证 CLI 行为与关键输出
