# Devtools

原生、离线优先的开发者工具箱。界面基于 [GPUI](https://gpui.rs/)，把 JSON、编码、时间戳、哈希、JWT 这类日常操作留在本机完成，而不是贴进随机网页。

默认离线；会访问网络的工具在界面上标明，并由你显式运行。无账号、不同步、无后台剪贴板监听。

研发计划与工程决策见 [docs/plan.md](docs/plan.md)。

## 能力概览

侧边栏按分类列出约 80 个工具（以应用内注册表为准），常见类别：

| 分类 | 示例 |
| --- | --- |
| 数据 | JSON / 数据格式互转、进制、颜色、chmod、SQL 格式化、数学求值 |
| 编码 | Base64、URL / HTML、二进制与 Unicode 转义 |
| 文本 | 大小写 / Slug、正则、Diff、列表与邮箱规范化、速查备忘 |
| 时间 | Date-time（自动识别 + 多格式输出）、Cron |
| 生成 | UUID / ULID / Token、Lorem、二维码 / Wi‑Fi QR、SVG 占位 |
| 网络 | URL 解析、HTTP / DNS（显式联网）、IP / MIME / 状态码等 |
| 安全 | Hash、JWT 解码、HMAC / OTP、bcrypt / AES / RSA / BIP39（学习向文案） |
| 图形 | 颜色检查器与色板、Base64 图片、标尺等 |

共用交互：

- **侧边栏**：分类常开；顶部可搜索；箭头或 `Cmd/Ctrl+B` 收起整栏；★ 收藏与最近使用。
- **命令面板**：`Cmd/Ctrl+K`。
- **BatchWorkbench**：一输入 → 多行标注结果（每行复制）；单值转换器用上下紧凑布局。
- **模式芯片**：如 URL / HTML / Base64 / SQL / JSON 的格式选项。
- **颜色**：实时色板预览、色相条与快捷色块。
- **拖放**：Base64 Image / File 等可拖入文件。
- **剪贴板**：无后台监听；输入框「粘贴」或 `Cmd/Ctrl+Shift+V` 读一次；有匹配时出现建议芯片（点击只切换工具）。
- **主题**：明/暗独立色板，跟随系统外观（可用 `DEVTOOLS_FORCE_THEME=light|dark` 强制）。

轻量工具输入即更新；UUID / ULID / Token 等生成与 HTTP / DNS 等联网操作为显式按钮（可用 `Ctrl+Enter` / macOS `Cmd+Enter`）。

## 下载与安装

预编译包见 **[v0.1.0 Release](https://github.com/left56/devtools/releases/tag/v0.1.0)**（含校验和）：

| 平台 | 产物 |
| --- | --- |
| Linux x86_64 | `.tar.gz`、`.deb`（含桌面图标） |
| macOS aarch64 | `.zip`（内含带图标的 `Devtools.app`）、未签名 `.dmg` |
| Windows x86_64 | `.zip`（实验性） |

打包说明、触发方式与本地等价命令见 [docs/packaging.md](docs/packaging.md)。Windows 构建失败不阻断 Release。

### macOS

当前发布包**未签名、未公证**，从浏览器下载后系统可能加上隔离属性（quarantine），Gatekeeper 会拦截打开。

1. 把 `Devtools.app` 拖到 `/Applications`。
2. 若双击被拦，或提示「无法验证开发者」，在终端执行：

```bash
sudo xattr -cr /Applications/Devtools.app
```

3. 再双击打开；第一次也可用右键 →「打开」。

说明：这里的 `xattr` 是 macOS 自带的扩展属性命令，用来清除下载隔离标记，与仓库 `patches/` 里 Rust 的 `xattr` crate 补丁无关。应用包内已带 `AppIcon.icns` 图标。

## 从源码构建

```bash
cargo test --workspace
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo run -p devtools
```

只验证可移植工具层（适合无 GPU / CI）：

```bash
cargo test -p devtools-core -p devtools-tools --all-targets
cargo clippy -p devtools-core -p devtools-tools --all-targets -- -D warnings
```

快捷键摘要：`Cmd/Ctrl+W` 关窗，`Cmd/Ctrl+B` 侧栏，`Cmd/Ctrl+K` 命令面板，`Cmd/Ctrl+Z` 撤销，结果区只读可复制。

## 仓库布局

```text
crates/devtools/        GPUI 壳：窗口、工作台、主题、快捷键
crates/devtools-core/   分类、工具元数据、注册表、搜索（无 GPUI）
crates/devtools-tools/  纯函数实现与剪贴板猜测（可移植）
assets/icon/            应用图标（PNG / SVG / AppIcon.icns）
docs/                   研发计划、打包与验证记录
```

依赖方向：`devtools → devtools-tools → devtools-core`。算法层不引用 GPUI。

## 环境

- **Linux x86_64 + X11**：当前已验收主机（`cargo test` / `cargo run`；无独显可用 Mesa lavapipe）。步骤见 [docs/linux-x11-verification.md](docs/linux-x11-verification.md)。
- **macOS aarch64**：CI 已出包；本机构建仍需 Metal（`xcodebuild -downloadComponent MetalToolchain`）。真机点验与签名分发仍待完善。
- **Windows x86_64**：实验性产物；壳未做完整验收。
- rustc：`rust-toolchain.toml` 跟踪 stable；`gpui` 0.2.2（Linux 开 `x11`，macOS 开 `font-kit`）。

另见 [M1 验证](docs/m1-validation.md)、[JSON 检查器验证](docs/json-inspector-validation.md)。`xattr` 与新 `libc` 的冲突见 `patches/`。

## 致谢

工具覆盖面与「小工具默认离线、敏感数据不轻易出本机」的思路，受到 [it-tools](https://github.com/CorentinTh/it-tools)（[it-tools.tech](https://it-tools.tech)）启发。本项目为独立的原生桌面实现，并非其移植。
