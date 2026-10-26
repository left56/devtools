# Devtools

原生、离线优先的开发者工具箱。用 [GPUI](https://gpui.rs/) 做界面，把 JSON、编码、时间戳、哈希这类日常操作留在本机，而不是贴进随机网页。

它不是 it-tools、DevUtils 或 DevKnife 的移植。工具广度参考 it-tools，唤起方式参考 DevUtils，检查器的完成度参考 DevKnife。第一版只做高频、可离线完成的工具。

研发计划、已做的决策和未决问题在 [docs/plan.md](docs/plan.md)。

## 当前能做什么

M2 变换、M3 检查器、M4 效率、M5 第一批联网工具已接上。侧边栏目前注册：

| 工具 | 作用 |
| --- | --- |
| JSON | 严格格式化 / 压缩 / 树形检查与路径 |
| Base64 | 标准编解码；可切换 URL-safe |
| URL Encode | 组件编解码（不是整段 URL 解析） |
| HTML Escape | 实体转义与反转义 |
| Hash | MD5 / SHA-1 / SHA-256 / SHA-512（弱算法标明不用于密码或完整性安全） |
| UUID | 生成 v4 / v7 |
| Timestamp | 秒或毫秒 ↔ 日期时间；默认本机时区，可改 UTC |
| Number Base | 2 / 8 / 10 / 16，非法数字指出位置 |
| Case | snake / camel / Pascal / kebab / CONSTANT |
| JWT | 只解码：header / payload / 签名分栏；`alg:none` 标不受信任；`exp` 显示时间与是否过期；从不暗示已验签 |
| Regex | Rust `regex`（不是 PCRE）：匹配、捕获组、可选替换 |
| Cron | 5 段表达式 → 之后 5 次运行；本机时区或 UTC |
| URL Parse | 拆分各段与 query；不发请求 |
| Diff | 行级插入/删除；无外部 diff 进程 |
| HTTP Request | 显式发送 HTTP；打开不发请求；响应不进 prefs |
| DNS Lookup | 显式 A/AAAA；打开不查询 |

共用能力：

- 自研多行输入：选择、复制/剪切/粘贴、滚动、等宽字体、最多 128 步线性撤销（无 redo）。
- **UI 按 [Refactoring UI](https://github.com/jaywilburn/refactoring-ui-skill) 原则设计**：人格定为 developer tools = casual + reserved（克制蓝强调色、冷灰中性色、中等圆角、密集但有序的间距刻度 4/8/12/16/24…）。明/暗两套独立色板（非互相反相），跟随系统外观（`WindowAppearance`；验收可用 `DEVTOOLS_FORCE_THEME=light|dark`）。
- macOS 原生三色按钮与内容区融合；侧边栏「分类 → 工具」常开（分类标签更淡、不可折叠）；选中行左侧强调条；每工具独立字形图标；原文与结果分栏卡片。
- 侧边栏可用顶部箭头或 `cmd-b` 收起/展开；`cmd-k` 命令面板。
- 系统剪贴板：**无后台监听**。输入框标题「粘贴」或 `Ctrl+Shift+V`（macOS `Cmd+Shift+V`）读一次并写入该输入；有匹配时出现紧凑建议芯片（点击只切换工具）。
- 轻量工具（变换 / 检查器 / JSON 格式化）**输入即更新**；UUID「生成」与 HTTP/DNS「运行」仍为显式按钮。
- 最近使用 / 收藏：侧边栏 ★ 与 `Ctrl+Shift+F`；本地 `prefs.json`（只存工具 id 与时间戳）。Linux：`$XDG_DATA_HOME/devtools` 或 `~/.local/share/devtools`；macOS：`~/Library/Application Support/Devtools`。
- 联网工具打开默认离线；运行前横幅标明「会访问网络」。

`cargo test --workspace`（tools 层 74 个单测）与全工作区 clippy `-D warnings` 已在 Linux x86_64 上通过；M2–M5 壳与 Refactoring UI 重设计已在同机 X11 窗口点验（见 [Linux 验证记录](docs/linux-x11-verification.md)、`/workspace/devtools-linux-verify-rui/`）。macOS 真机点验仍待补（含系统浅色/深色切换）。另见 [M1 验证记录](docs/m1-validation.md) 与 [JSON 检查器验证记录](docs/json-inspector-validation.md)。

## 环境

**当前验收主机是 Linux x86_64（X11）**。macOS 适配留到真机点验后再收；Windows x86 更靠后。

- rustc：`rust-toolchain.toml` 跟踪 stable
- `gpui` 0.2.2：Linux 开 `x11`；macOS 开 `font-kit`（见 `crates/devtools/Cargo.toml`）
- 无独显时用 Mesa lavapipe：`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`

Linux 依赖与启动步骤见 [docs/linux-x11-verification.md](docs/linux-x11-verification.md)。

macOS 仍需要 Metal Toolchain：

```bash
xcodebuild -downloadComponent MetalToolchain
```

### 跨平台意图

| 平台 | 状态 |
| --- | --- |
| Linux x86_64 + X11 | **当前可 `cargo run` 验收**（本仓库已在 Xvfb + lavapipe 上跑通） |
| macOS Apple Silicon | 下一轮真机点验（透明标题栏 / traffic lights / 系统外观已 `cfg` 与 `WindowAppearance` 接线） |
| Windows x86 | 更后；`windows-manifest` feature 占位，未验证 |

已知剩余差异：macOS 融合标题栏与签名分发（Metal 本轮未复验）；Linux Wayland 未验；证书/CSV/SQL 仍为后续。`xattr` 0.2 与新 `libc` 的冲突见 `patches/`。

## 安装包（GitHub Actions）

工作流定义在 `ci/workflows/`；首次需拷入 `.github/workflows/` 启用（见 [docs/packaging.md](docs/packaging.md)）。启用后 Push/PR 编译各平台包，打 `v*` tag 发 GitHub Release。

摘要：

| 平台 | 产物 |
| --- | --- |
| Linux x86_64 | `.tar.gz` + `.deb` |
| macOS aarch64 / x86_64 | `.app` 的 `.zip` + 未签名 `.dmg` |
| Windows x86_64 | `.zip`（实验性，失败不阻断） |

## 命令

```bash
cargo test --workspace
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo run -p devtools
```

只验证可移植工具层（适合 Linux CI / 无 GPU 环境）：

```bash
cargo test -p devtools-core -p devtools-tools --all-targets
cargo clippy -p devtools-core -p devtools-tools --all-targets -- -D warnings
```

本机已有依赖缓存时，可以这样临时启动；配置只对这条命令生效：

```bash
cargo run -p devtools --offline --config 'source.crates-io.registry="sparse+https://rsproxy.cn/index/"'
```

关闭窗口后进程退出。`cmd-w` 关闭窗口，`cmd-m` 最小化，`ctrl-cmd-f` 进入/退出全屏，`cmd-b` 收起或展开侧边栏，`cmd-k` 打开或关闭命令面板。面板用上下键选择、回车打开、Esc 返回原焦点；回车只在多行编辑器中插入换行。`cmd-z` 撤销编辑，`cmd-a/c/x/v` 全选、复制、剪切、粘贴。结果区只读，可选择复制。轻量工具输入即更新；联网工具与 UUID 生成可用 **Ctrl+Enter**（macOS 上 Cmd+Enter）执行。

JSON：输入时自动格式化并展示树；「压缩」「格式化」仍可手动点。变换类工具左右分栏，方向/选项在标题下芯片切换。

## 布局

```text
crates/devtools/        GPUI 壳：窗口、文本缓冲、输入组件、JSON / 变换工作台、搜索面板
crates/devtools-core/   分类、工具元数据、注册表、搜索。不依赖 GPUI
crates/devtools-tools/  纯函数实现和剪贴板猜测（可移植）
docs/plan.md            研发计划
```

依赖方向是 `devtools -> devtools-tools -> devtools-core`。算法不能引用 GPUI 类型。
