# Devtools

原生、离线优先的开发者工具箱。用 [GPUI](https://gpui.rs/) 做界面，把 JSON、编码、时间戳、哈希这类日常操作留在本机，而不是贴进随机网页。

它不是 it-tools、DevUtils 或 DevKnife 的移植。工具广度参考 it-tools，唤起方式参考 DevUtils，检查器的完成度参考 DevKnife。第一版只做高频、可离线完成的工具。

研发计划、已做的决策和未决问题在 [docs/plan.md](docs/plan.md)。

## 当前能做什么

M2 纯函数工具已接上通用变换工作台。侧边栏目前注册：

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

共用能力：

- 自研多行输入：选择、复制/剪切/粘贴、滚动、等宽字体、最多 128 步线性撤销（无 redo）。
- macOS 原生三色按钮与暗色内容区融合；侧边栏按「分类 → 工具」折叠；原文与结果左右分栏。
- 侧边栏可用顶部箭头或 `cmd-b` 收起/展开；`cmd-k` 命令面板。
- 系统剪贴板仅在用户粘贴、剪切或复制时读写，没有后台监听。不根据内容自动切换工具。

`cargo test -p devtools-core -p devtools-tools`（当前 48 个单测）与两 crate 的 clippy `-D warnings` 已在 Linux x86_64 上通过。GPUI 壳请在 macOS 上 `cargo run -p devtools` 做窗口点验。详见 [M1 验证记录](docs/m1-validation.md) 与 [JSON 检查器验证记录](docs/json-inspector-validation.md)。

## 环境

本机初始化时使用：

- macOS Apple Silicon（壳的第一验收平台）
- rustc 1.93+（`rust-toolchain.toml` 跟踪 stable）
- `gpui` 0.2.2（crates.io，只开 `font-kit`）
- Apple metal 32023.921

GPUI 在 macOS 上用 Metal 画界面，需要本机装有 Xcode。Xcode 27 的命令行工具里没有 `metal`。没装 Metal Toolchain 时，`gpui` 0.2.2 编译会报 `cannot execute tool 'metal'`。安装：

```bash
xcodebuild -downloadComponent MetalToolchain
```

### 跨平台意图

长期目标包含 **Windows x86** 与 **Linux x86**，不是永远只做 macOS。在那两套主机上壳真正编过、跑过之前，不声称跨平台。

当前阻碍：

- `gpui` 仍按 macOS / Metal + `font-kit` 锁定；Linux 的 wayland/x11、Windows 后端都还没接。
- 融合标题栏、三色按钮等窗口 API 需要 `cfg` 隔离。
- M4 本地数据目录候选是 macOS Application Support 路径，其他系统要另定。
- 工具逻辑（`devtools-core` / `devtools-tools`）已保持可移植，可在无显示服务器的 Linux 上单独测试。

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

关闭窗口后进程退出。`cmd-w` 关闭窗口，`cmd-m` 最小化，`ctrl-cmd-f` 进入/退出全屏，`cmd-b` 收起或展开侧边栏，`cmd-k` 打开或关闭命令面板。面板用上下键选择、回车打开、Esc 返回原焦点；回车只在多行编辑器中插入换行。`cmd-z` 撤销编辑，`cmd-a/c/x/v` 全选、复制、剪切、粘贴。结果区只读，可选择复制。

JSON：格式化后默认展示树，压缩后展示文本；「复制结果」始终复制最近一次运行生成的完整字符串。变换类工具（Base64 等）同样左右分栏，选项在标题下，动作在原文侧。

## 布局

```text
crates/devtools/        GPUI 壳：窗口、文本缓冲、输入组件、JSON / 变换工作台、搜索面板
crates/devtools-core/   分类、工具元数据、注册表、搜索。不依赖 GPUI
crates/devtools-tools/  纯函数实现和剪贴板猜测（可移植）
docs/plan.md            研发计划
```

依赖方向是 `devtools -> devtools-tools -> devtools-core`。算法不能引用 GPUI 类型。
