# Devtools 研发计划

状态：M0–M5 已落盘；**it-tools 复刻第三批已合入（目录约 66 个工具）**；工具栏按钮等高；Date-time / BatchWorkbench。M6 分发仍是计划。

读者：接下来实现工具或改界面的人。先看「目标和非目标」与「决策」，加工具直接看「新增一个工具」。

产品名 Devtools 是暂名，跟仓库目录一致。最终名字、bundle id 和许可证还没定。

## UI 设计（Refactoring UI）

人格：**developer tools = casual + reserved**。实现要点：

- 独立明/暗色板（冷灰中性 + 克制蓝 accent），跟随系统外观；不用纯黑/纯白，不用紫色默认。
- 间距刻度 `theme::space`（4/8/12/16/24/32）、圆角 `theme::radius`（4/6/8）。
- 共享组件在 `ui.rs`：主按钮 / 次按钮 /「粘贴」**同一固定高度**（28px）与 `text_sm`；安静 chip 可更密；pane header + `pane_title`、`workbench_title`、空状态 `empty_hint`。
- 侧栏分类常开、标签淡化；选中态用左侧强调条而非重边框。
- **BatchWorkbench**：一输入 → 多行标注结果（每行复制）。`timestamp` / `number-base` / 颜色 / chmod 等走此台。
- 行为：轻量工具实时跑；HTTP/DNS/UUID/ULID/Lorem 显式运行；输入框「粘贴」；无顶栏剪贴板条；无后台剪贴板监听。

参考技能：https://github.com/jaywilburn/refactoring-ui-skill

## 目的

开发者经常把 JSON、Base64、时间戳、JWT 贴到网页工具里。那些页面能用，但不该成为处理 token、cookie 和请求体的默认路径。

Devtools 是一个本机桌面工具箱：

- 默认离线。会联网的工具必须在界面上标明，并且由用户显式运行。
- 窗口打开就能搜索、粘贴、看结果、复制。不账号、不同步。
- 工具行为是可单测的 Rust 函数。界面只负责编辑、展示和快捷键。

三个参照，以及各自只取什么：

| 产品 | 形态 | 取 | 不取 |
| --- | --- | --- | --- |
| [it-tools](https://github.com/CorentinTh/it-tools) | 自托管网页，70+ 小工具 | 工具名单和「敏感数据不要出本机」的动机 | 不做网页版，不追求第一版覆盖 70 个工具 |
| [DevUtils](https://devutils.com/) | macOS 原生，40+ 工具 | 离线、按内容猜工具、键盘优先 | 不在后台持续读剪贴板 |
| [DevKnife](https://devknife.app/) | macOS 原生，约 33 个工具 | 检查器要做得能用（JSON、JWT、正则、cron），而不是只摆一个文本框 | 不把 HTTP 客户端、端口扫描放进第一版 |

## 目标和非目标

第一版要达成：

- macOS 上一个原生窗口。Linux / Windows 不在第一版验收里。
- 侧边栏、分类、搜索、命令面板都能定位工具。
- 下面「工具目录」里标为 P0 和 P1 的工具可以在窗口里完成一次完整操作：输入、出结果、复制、看到错误位置。
- 纯文本变换和识别规则有表驱动测试。测试不启动 GPUI。
- 关闭窗口即退出，不留后台监听。

第一版明确不做：

- 账号、同步、插件市场、动态库或 WASM 插件 ABI。
- 代码编辑器、语法高亮、语言服务器。
- 用一个 `fn(&str) -> String` 描述所有工具。
- JSON 自动修复、JSON5、默认按键名排序。
- 把 JWT 解码说成验签。`alg: none` 永远不能显示成可信。
- MD5 / SHA-1 当作密码哈希。界面上要写明它们只用于校验和兼容。
- 后台剪贴板监视、全局常驻状态栏。全局热键单独调研，不放进 M1。
- WHOIS、端口扫描。HTTP / DNS 已在 M5 第一批以 `NetworkUse::Explicit` 落地；端口扫描仍不做。
- AI 功能。

## 约束

这些是事实，不是偏好。

- 初始化机器是 macOS `aarch64-apple-darwin`，rustc 1.93.0（`254b59607`，2026-01-19），cargo 1.93.0。只装了 host target。
- crates.io 上的 `gpui` 最新版是 **0.2.2**，发布于 2025-10-22，edition 2024，Apache-2.0。0.2.2 的入口是 `Application::new().run`，示例在 docs.rs 的 `examples/hello_world.rs` 和 `examples/on_window_close_quit.rs`。
- 2026-10-22 查询 crates.io，没有 `gpui_platform` 这个 crate（API 404）。Zed 主分支 README 已经写成 `gpui_platform::application()`。那是上游未发布的拆分，不是这个仓库能依赖的版本。
- GPUI 仍是 pre-1.0。0.2.2 没有官方多行文本框。Zed 的编辑器不在 `gpui` crate 里。M1 必须自己做一个够用的文本输入，这是关键路径。
- macOS 渲染走 Metal，字形需要 `font-kit` feature。`gpui` 的 default features 会打开 `wayland` 和 `x11`。本仓库在 workspace 依赖里关掉 default features，只开 `font-kit`。不要为了「看起来完整」把 default features 打开。
- Xcode 27 自带的命令行工具里没有 `metal`。`gpui` 0.2.2 的 `build.rs` 会调用它编译 shader，缺少时报 `cannot execute tool 'metal'`。先执行 `xcodebuild -downloadComponent MetalToolchain`。本机装好后 `xcrun metal` 是 Apple metal 32023.921。
- 长期目标包含 Windows x86 与 Linux x86。在那两套主机上实际编译通过之前，不把 GPUI default features、数据目录路径或标题栏行为写进跨平台验收。工具 crate 保持无 GPUI、无 `unsafe`。
- `serde_json` 默认用 `BTreeMap`，对象键会被排序。workspace 依赖打开了 `preserve_order`。这是行为的一部分，不是实现细节。

## 当前状态

M0–M5（第一批联网）已实现。**当前验收主机是 Linux x86_64 + X11**（`cargo test --workspace`、`cargo run -p devtools`）。macOS 为下一轮真机适配；Windows x86 更后。

已实现：

- workspace 三 crate：`devtools`、`devtools-core`、`devtools-tools`。
- `ToolMeta`、固定分类、注册表、搜索。重复 id 会在构造注册表时 panic；空分类隐藏。
- JSON `prettify` / `minify`：严格解析、保留键顺序。错误行号从 1 开始，列号为 `serde_json` 的字节列号，空输入列号为 0。
- M2 纯函数工具（全部 `NetworkUse::Offline`，界面接好后 `Availability::Ready`）：
  - `base64`：标准编解码；URL-safe 为开关，默认标准。
  - `url-encode`：组件编解码（非整段 URL 解析）。
  - `html-escape`：实体转义/反转义，含数字字符引用。
  - `hash`：MD5 / SHA-1 / SHA-256 / SHA-512；弱算法旁标注「不用于密码或完整性安全」。
  - `uuid`：v4 与可选 v7（`uuid` crate，不自写 RNG）。
  - `timestamp`（显示名 Date-time）：自动识别 Unix 秒/毫秒、ISO、常见日期；默认填入当前本机时间；批量输出本机/ISO/RFC/Unix/Mongo/Excel 等（`BatchWorkbench`）。
  - `number-base`：同时显示 2/8/10/16；可识别 `0x`/`0b`/`0o`（`BatchWorkbench`）。
  - 新增（it-tools 第一批）：`ulid`、`slugify`、`lorem`、`text-stats`、`roman`、`percentage`、`chmod`、`binary`、`color`。
  - 新增（it-tools 第二批）：`basic-auth`、`hmac`、`token`、`otp`、`email-norm`、`list-conv`、`json-yaml`、`json-toml`、`sql-format`、`markdown-html`、IPv4×3、`ipv6-ula`、`mac-gen`、`mime`、`http-status`、`ua-parse`、`iban`、`temperature`、`eta`、`numeronym`、`obfuscate`、`nato`、`safelink`、`docker-run`；Hash 增 SHA-384。
  - 新增（it-tools 第三批）：`xml-tool`、`json-csv`、`yaml-toml`、`json-diff`、`password-strength`、`keycode`、`phone`、`ascii-draw`、`random-port`、`unicode-esc`、`qr-code`、`wifi-qr`、`meta-tags`、`svg-placeholder`、`cron-presets`。Chronometer 暂缓（需计时器 UI）。
  - `case`：snake / camel / Pascal / kebab / CONSTANT。
- 通用自研文本框：多行编辑器、单行搜索、只读结果。选择、复制/剪切/粘贴、滚动、Menlo 与中文字体回退、128 步线性撤销，无 redo。
- JSON 工作台：格式化、压缩、清空、复制结果；原文/结果左右分栏；JSON 树/文本切换与路径。
- **UI 视觉打磨**：明/暗双色板（`Theme::light` / `Theme::dark`），经 `WindowAppearance` + `observe_window_appearance` 跟随系统；`DEVTOOLS_FORCE_THEME=light|dark` 仅用于截图验收。侧栏每工具独立字形图标（`icons.rs`），分类标签用 `faint`、不可折叠。Linux Xvfb 无 portal 时默认偏 Light；截图见 `/workspace/devtools-linux-verify-theme/`。
- **通用变换工作台**（`TransformWorkbench`）：左右分栏；**轻量变换实时运行**（编码/解码用方向芯片；UUID「生成」仍为按钮）。切换变换类工具时保留输入并按新工具重算。不自动切工具。
- **M4 效率（UX 已收紧）**：无顶栏剪贴板条。输入框标题右侧「粘贴」= 读一次剪贴板并写入该输入；`Ctrl+Shift+V` / macOS `Cmd+Shift+V` 同效（兼更新建议芯片）。建议为标题下紧凑芯片行（点击只切换工具）。收藏：侧边栏 ★ + `Ctrl+Shift+F`。最近使用写入 `prefs.json`；命令面板空查询置顶最近。
- **本地数据目录**：Linux `$XDG_DATA_HOME/devtools` 或 `~/.local/share/devtools`；macOS `~/Library/Application Support/Devtools`。文件 `prefs.json`。
- **M5 联网（第一批）**：`http-request`、`dns-lookup`，`NetworkUse::Explicit`。打开不发请求；运行前横幅「会访问网络」。响应/URL 不进 prefs。`NetworkWorkbench`。
- macOS 透明原生标题栏融合内容区；侧边栏分类常开（不折叠）；`cmd-b` 收起整条侧栏；`cmd-k` 命令面板。
- `suggest` 纯函数：JSON / JWT 结构 / 时间戳 / 标准 Base64。猜中也不自动切换；未注册工具不会出现在 `applicable_suggestions` 里。
- `cmd-w` 和最后一个窗口关闭时退出。

未实现：

- ~~M3 检查器 / M4 效率 / M5 HTTP+DNS~~（已实现）。
- M5 后续：证书解码、CSV、SQL 格式化（仍为 P2，未分配 id）。
- 打包、签名、菜单栏、全局热键、状态栏图标、登录项、后台剪贴板监听。
- 语法高亮、自动折行、多光标和 redo。
- macOS 真机点验与签名分发；Linux Wayland；Windows 壳（见下方「跨平台意图」）。

跨平台意图：

- **Linux x86_64 + X11**：已可构建并窗口验收（见 [linux-x11-verification.md](linux-x11-verification.md)）。`gpui` feature 为 `x11`；无独显用 lavapipe。
- **macOS**：下一轮适配。透明标题栏 / traffic lights 已 `cfg(target_os = "macos")`。仍需 Metal Toolchain 与真机点验。
- **Windows x86**：更后。`windows-manifest` 仅占位。
- 工具逻辑保持可移植。`xattr` 0.2 与新 `libc` 的 ENOATTR 问题用 `patches/xattr-0.2.3` 解决。

验收命令：

```bash
cargo test --workspace
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo run -p devtools
```

在只有工具层的环境（无显示服务器 / 非 macOS）至少跑：

```bash
cargo test -p devtools-core -p devtools-tools --all-targets
cargo clippy -p devtools-core -p devtools-tools --all-targets -- -D warnings
```

2026-10-23：M2 在 Linux x86_64（rustc stable）上 `devtools-core` 3 + `devtools-tools` 45 个单测通过，两 crate clippy `-D warnings` 通过。壳需在 macOS 上 `cargo run -p devtools` 点验每个新工具的编码/解码/复制与错误展示。

2026-10-23：UI 视觉打磨落盘（主题分层、主/次按钮、JWT 堆叠卡、Diff 上下结构、URL KV）；Xvfb 截图 `/workspace/devtools-linux-verify-ui/`。

2026-10-23：M3 检查器（jwt / regex / cron / url-parse / diff）在 Linux x86_64 落盘并接 `InspectorWorkbench`。`devtools-tools` 单测扩到含 alg:none 与过期 exp；全工作区 `cargo test --workspace`（66 tools 层）与 clippy `-D warnings` 通过；Xvfb + lavapipe 交互截图见 `/workspace/devtools-linux-verify-m3/`。

2026-10-24：侧栏独立图标 + 分类常开淡化 + 明暗主题跟随系统（`observe_window_appearance`）；Xvfb 截图 `/workspace/devtools-linux-verify-theme/`。

2026-10-23：M4 效率 + M5 第一批联网落盘。prefs 只存 id/时间戳/收藏；tools 层 74 个单测；`cargo test --workspace` + clippy `-D warnings` 通过；Xvfb 截图见 `/workspace/devtools-linux-verify-m4/` 与 `/workspace/devtools-linux-verify-m5/`。


## 架构

```text
devtools            GPUI 窗口、主题、快捷键、每个工具的交互状态
    |
    v
devtools-tools      纯函数、错误类型、剪贴板猜测
    |
    v
devtools-core       分类、ToolMeta、注册表、搜索、稳定 id
```

- `devtools-core` 和 `devtools-tools` 禁止依赖 GPUI，也禁止 `unsafe`。
- 壳可以依赖工具 crate。工具 crate 不能依赖壳。
- 注册表是壳可以展示的工具的名单。`devtools_core::ids` 里可以提前放稳定 id，但没实现的工具不得放进 `catalog()`。
- 交互工具的状态放在壳里。例如正则的高亮、HTTP 的请求编辑、JSON 树。它们调用 tools crate 里的纯函数，不把 GPUI 类型传进 tools。
- 纯文本变换不必上 trait。一个工具一个模块，导出 `meta()` 和具体函数即可。等出现第三种相同形状再抽 trait，不提前抽。

数据流（M1–M5 + UX）：

1. 用户在编辑器里输入，或点输入框「粘贴」/ `Ctrl+Shift+V`（读一次剪贴板写入）。
2. 轻量工具：壳在输入变化时把 `&str` 交给对应函数（Regex 短防抖）；联网工具等用户点「运行」。
3. 函数返回结果或带位置的错误。
4. 壳展示结果。复制也由壳做。
5. 猜测走 `suggest`。壳只显示建议芯片。不自动切换工具，除非用户确认。

进程内日志以后如果要加，默认不得记录工具输入和输出。当前没有日志依赖。

## 决策

下面每一条都是当前决定。被否决的方案只在它很可能被再次提出时保留。

1. **锁定 `gpui` 0.2.2，不跟 Zed 主分支。**\
   理由：0.2.2 有 crates.io 版本和可引用的示例。主分支已经改成尚未发布的 `gpui_platform`。跟 `branch = "main"` 会让构建绑在编辑器仓库的任意提交上。\
   否决：把 GPUI 当 git submodule 或每天升级。\
   升级条件：0.2.2 缺少而且无法在应用内补上的 API，或有影响正确性的缺陷。升级时锁一个 commit，并在本文记上 commit 和原因。

2. **macOS 只开 `font-kit`。**\
   理由：default features 含 Wayland / X11，本机没有验证过，也不该进 Apple Silicon 构建。Linux 和 Windows 的 feature 等到真的在那两个系统上编译时再加。

3. **界面文案用中文，工具名保持开发者会搜索的英文。**\
   分类显示「数据」「编码」。工具显示 "JSON"、"JWT"。关键词同时包含中文和英文，搜索两边都能命中。\
   否决：整套界面先做英文再翻译。当前使用者是中文。等有第二个语言需求再把文案从源码里提出去。

4. **不做后台剪贴板监听。**\
   理由：DevUtils 的自动识别体验好，但常驻读取会把密码和 token 扫进进程。「数据不离开本机」并不等于「本进程可以随时读剪贴板」。\
   代替：用户按按钮或快捷键时读一次。识别结果是建议。\
   否决：启动后订阅剪贴板变更。以后如果要做，只能是默认关闭的设置，不在 M4 的验收里。

5. **识别结果不自动切换工具。**\
   理由：误判会换掉用户正在看的内容。时间戳规则的置信度是 Medium，因为 10 位数字太常见。JSON 对象/数组和 JWT 结构是 High，也仍然只作为建议。\
   规则宁可漏报。标准 Base64 还要求出现数字、`+`、`/` 或 `=`，所以纯字母文本不会被当成 Base64。URL-safe Base64 在 M0 不识别。

6. **JSON 严格解析，保留键顺序，不默认修复。**\
   理由：修复和排序都会改变用户拿去 diff 的字节。尾随逗号就是错误，并报告行列。\
   否决：默认 JSON5，或默认按键名排序。修复模式如果以后要做，必须是单独的显式动作。

7. **JWT 在 M3 只解码。**\
   解码展示 header、payload、签名的原文和 `alg`。过期时间如果能解析，就显示，但不代替验签。\
   验签是后一个独立动作：用户提供密钥，密钥只留在内存，不写入最近使用，不进日志。\
   `alg: none` 可以识别和解码，界面必须写成不受信任。没有「信任此 token」这种按钮。

8. **哈希不是密码学建议。**\
   M2 可以提供 MD5、SHA-1、SHA-256、SHA-512，因为查文件和调试需要它们。MD5 和 SHA-1 旁边要写「不用于密码或完整性安全」。密码哈希（bcrypt、argon2）不在 P0/P1。

9. **v1 不做插件 ABI。**\
   理由：工具作者就是这个仓库。内部模块足够。动态插件会把注册表、版本和崩溃边界提前复杂化。\
   否决：第一版就做 dylib / WASM。

10. **空分类不显示。**\
    分类枚举是固定的，避免每个工具自造分组。侧边栏只列出注册表里实际出现的分类。

## 工具目录

id 一旦发布给注册表就不要改。显示名可以改。

| 优先级 | id | 分类 | 阶段 | 行为 |
| --- | --- | --- | --- | --- |
| 已实现 | `json` | 数据 | M1 工作台，Ready | 严格格式化、压缩、树形检查与路径。保留键顺序 |
| 已实现 | `base64` | 编码 | M2，Ready | 标准 Base64 编解码。URL-safe 做成开关，默认标准 |
| 已实现 | `url-encode` | 编码 | M2，Ready | URL 组件编解码，不是整段 URL 解析 |
| 已实现 | `html-escape` | 编码 | M2，Ready | HTML 实体转义和反转义 |
| 已实现 | `hash` | 安全 | M2，Ready | MD5、SHA-1、SHA-256、SHA-512。见决策 8 |
| 已实现 | `uuid` | 生成 | M2，Ready | v4 + v7。使用 `uuid` crate 的随机数，不自己写 RNG |
| 已实现 | `timestamp` | 时间 | M2，Ready | **Date-time**：自动识别 + 批量格式（it-tools 风格）。默认当前时间。id 仍为 `timestamp` |
| 已实现 | `number-base` | 数据 | M2，Ready | 同时显示 2/8/10/16；可识别前缀。非法数字指出位置 |
| 已实现 | `ulid` | 生成 | it-tools 批，Ready | 显式生成 ULID |
| 已实现 | `slugify` | 文本 | it-tools 批，Ready | 标题 → URL slug |
| 已实现 | `lorem` | 生成 | it-tools 批，Ready | 占位段落，显式生成 |
| 已实现 | `text-stats` | 文本 | it-tools 批，Ready | 字符/词/行/字节 |
| 已实现 | `roman` | 数据 | it-tools 批，Ready | 罗马数字 ↔ 阿拉伯数字（1–3999） |
| 已实现 | `percentage` | 数据 | it-tools 批，Ready | 百分比常用算式（两数输入） |
| 已实现 | `chmod` | 数据 | it-tools 批，Ready | 755 ↔ rwxr-xr-x |
| 已实现 | `binary` | 编码 | it-tools 批，Ready | 文本 ↔ 二进制/十六进制/码点 |
| 已实现 | `color` | 数据 | it-tools 批，Ready | #hex ↔ rgb ↔ hsl |
| 已实现 | `case` | 文本 | M2，Ready | snake、camel、Pascal、kebab、CONSTANT。不处理自然语言复数 |
| 已实现 | `jwt` | 安全 | M3，Ready | 只解码。`alg` 原文；`alg:none` 标不受信任；可解析的 `exp` 显示时间与是否过期。文案强调未验签，见决策 7 |
| 已实现 | `regex` | 文本 | M3，Ready | Rust `regex` crate 语法（不是 PCRE）。匹配、捕获组、可选替换；编译错误直接展示 |
| 已实现 | `cron` | 时间 | M3，Ready | 5 段 cron，之后 5 次运行。时区可选本机或 UTC，界面写明 |
| 已实现 | `url-parse` | 网络 | M3，Ready | 只拆分，不发请求。`NetworkUse::Offline` |
| 已实现 | `diff` | 文本 | M3，Ready | 行级插入/删除。纯 Rust（`similar`），不调用外部 diff 进程 |
| 已实现 | `http-request` | 网络 | M5，Ready | 显式 HTTP。打开不发请求；`NetworkUse::Explicit`；响应不进 prefs |
| 已实现 | `dns-lookup` | 网络 | M5，Ready | 显式 A/AAAA（系统解析器）。打开不查询；`NetworkUse::Explicit` |
| P2 | 证书解码、CSV、SQL 格式化 | — | M5 之后 | 未分配 id。保持 Offline 或按需 Explicit |

未注册的工具没有 id，就是为了避免半成品出现在注册表里。

## 阶段

每个阶段结束时，上一阶段的行为还在。不把 M1 的编辑器做成只能服务 JSON、换一个工具就要重写的一次性代码。

### M0 骨架

已完成，范围见「当前状态」。不再往 M0 加工具。

### M1 工作台

已落盘，验证范围见 [M1 验证记录](m1-validation.md)。

目标：不看源码，用户能把一段 JSON 格式化并复制走。

范围：

- 一个多行文本输入。要求：输入、选择、复制、粘贴、滚动、等宽字体。线性撤销上限 128，无 redo；不要求撤销树、语法高亮、多光标。
- JSON 三个动作：格式化、压缩、清空。失败时显示 `JsonError` 的行列，输入框尽量滚到那一行。
- 结果区一键复制。复制的是结果，不是再解析一次。
- 搜索框接到 `Registry::search`。中文「格式化」和英文 `json` 都能找到 JSON。
- `cmd-k` 打开同一个搜索。选中后切工具。
- 明暗主题跟随系统夜间模式（`WindowAppearance`）。Linux 无 xdg-desktop-portal 时 GPUI 常保持 Light。

验收：

- 合法 JSON 格式化后的文本与 `json::prettify` 一致，包括键顺序。
- `{"a":1,}` 不会产出「修好」的 JSON。
- 空输入有明确错误，而不是 panic。
- 搜索和侧边栏看到的是同一份注册表。

M1 不做收藏持久化，不做剪贴板猜测条。

实现边界：

- `buffer.rs` 与 `json_run.rs` 不引用 GPUI，单测不启动窗口。`text_field.rs` 按 `split('\n')` 一行一个元素调用 `shape_line`，保留末尾空行；不把换行传给单行整形。
- 多行插入将 CRLF 和单独 CR 归一为 LF。单行去掉 CR/LF，纯换行粘贴是空操作，空字符串替换仍可删除选区。归一化发生时光标放插入末尾。
- 移光标不记撤销。IME 开始组合只记一次，后续 marked 替换和提交共用该快照；取消后用独立 baseline 比较，恢复原文时丢弃该条。组合内选区按插入文本的相对 UTF-16 范围解释。
- 键上下文为 `Editor`、`LineField`、`CommandPalette`；树新增独立 `JsonTree`。只有 Editor 的回车插换行；编辑器上下移动和面板上下选择使用不同 action。`cmd-k` / `cmd-w` 为窗口级。
- `JsonWorkbench` 由 Workspace 持有，切换不销毁。结果使用已存字符串复制，不再次解析；继续编辑保留上次运行状态并标记待更新，直到再次运行或清空。结果文本和路径栏只读，树只检查结果，不修改原文。
- `unicode-segmentation = 1.10.1` 仅在壳 crate 中；GPUI 精确锁定 crates.io 0.2.2，关默认特性，只开 `font-kit`。

实现提示：GPUI 0.2.2 的 `examples/input.rs` 是单行输入的参考，不是能直接拿来的组件。不要把 Zed 的 `editor` crate 依赖进来。那个 crate 绑定的是 Zed 本体，不是稳定库。

### M2 纯函数工具

已落盘。P0 全部接上通用变换工作台；P1 的 `number-base` 与 `case` 一并完成。

每个工具：

- `meta()` 放进 `catalog()`，`availability` 在界面接好之后改为 `Ready`。只写了函数、界面还是示例时保持 `LogicOnly`。
- 至少覆盖：空输入、典型合法输入、一个非法输入、一次往返（编码再解码）。
- 不读网络，不读环境变量里的密钥。

UUID 的测试不断言具体随机值，只断言版本位和可解析。

验收：侧边栏能打开每个 P0 工具，完成一次操作并复制。`cargo test -p devtools-tools` 不需要显示服务器。

### M3 检查器（已完成）

JWT、正则、cron、URL 拆分、diff。结果走 `InspectorWorkbench`（不是整块替换文本）。

- JWT：header / payload / 签名分栏。`alg` 用原文显示。`none` 用明确的警告样式。能解析的 `exp` 显示时间和是否已过期，文案是「未验签」。
- 正则：模式、被测文本、替换可选。错误来自 `regex` 的编译错误。界面标明 Rust regex 语法。
- cron：下 5 次运行。写明用的是本机时区还是 UTC。
- URL Parse：只拆分，不发请求。
- diff：两块输入，行级插入和删除。不调用外部 `diff` 进程。

验收：每个工具有不启动窗口的测试（含 JWT `alg: none` 与过期 `exp`）。Linux x86_64 上 `cargo test --workspace` + clippy `-D warnings` + Xvfb/lavapipe 交互点验通过。

### M4 效率（已完成；UX 收紧）

- 输入框「粘贴」+ `Ctrl+Shift+V` / `Cmd+Shift+V`：读一次并写入当前输入（无后台监听）。建议芯片行仅在有检测时出现；点击只切换工具。
- **实时运行**：变换类（除 UUID 生成）与检查器（JWT / Regex 防抖 / Cron / URL / Diff）及 JSON 格式化在输入时更新；HTTP / DNS 仍显式「运行」。
- 最近使用和收藏：`prefs.json`，只存工具 id、`used_at`、favorites。路径：Linux `$XDG_DATA_HOME/devtools`（或 `~/.local/share/devtools`）；macOS `~/Library/Application Support/Devtools`。
- 命令面板空查询时置顶最近使用；侧边栏「★ 收藏」；`Ctrl+Shift+F` 切换收藏。

不做：状态栏图标、登录项、后台监听、全局热键产品化。

### M5 联网工具（第一批已完成）

单独阶段，因为信任模型和前面不同。

- `http-request` / `dns-lookup`：`NetworkUse::Explicit`。界面横幅「会访问网络」；打开 ≠ 请求；只有「运行」/ `Ctrl+Enter` 才联网。
- 响应和 URL 不进收藏/最近使用文件，不进日志。
- HTTP：方法芯片 + URL / 头 / 体；`ureq` 阻塞发送。DNS：主机名 → A/AAAA（`dns-lookup` + 系统解析器）。
- 端口扫描不在范围。证书解码 / CSV / SQL 格式化仍为后续 P2。

### M6 macOS 分发

- `.app` 包、图标、关于窗口、版本号。
- 签名和 notarization 取决于还没定的 Apple 账号，不在代码骨架里假装完成。
- 在 Linux 或 Windows 上实际编译通过之前，不声明跨平台。届时再给 `gpui` 加对应 feature，并在本文改「约束」。

## it-tools 覆盖

对照 [it-tools](https://it-tools.tech/) / `CorentinTh/it-tools`（约 86 个）：

| 状态 | 工具 |
| --- | --- |
| 已有（≈66） | 第二批全部 + XML/JSON、JSON↔CSV、YAML↔TOML、JSON diff、口令强度、按键码、电话整理、ASCII 横幅、随机端口、Unicode 转义、QR（SVG/ASCII）、Wi‑Fi QR、Meta 标签、SVG 占位、Cron 模板 |
| 仍缺 / 下一批 | HTML WYSIWYG、JSON→CSV 以外的部分转换变体、Password 之外的部分 crypto 周边、完整 FIGlet、完整 libphonenumber、相机/设备信息等 |
| 暂缓及原因 | **Chronometer**：需 GPUI 定时器/动画 UI，本批未做。**Camera / Device info**：需硬件与浏览器/系统 API。**PDF 签名 / PDF 任意**：重解析与 UI。**html-wysiwyg**：富文本编辑器超出当前文本框。**Emoji picker**：资源与面板 UI。**RSA / Encryption / bcrypt / BIP39**：密钥管理与安全文案需单独设计。**MAC vendor lookup**：联网库。**Benchmark builder**：长时间任务与图表。Wi‑Fi/QR 已做离线 SVG/ASCII，不做系统分享面板。 |

设计原则：优先「一输入、多结果、每行可复制」；随机生成类显式按钮；日期时间默认当前时间。Linux-first，纯 Rust，Offline 除非 `NetworkUse::Explicit`。


## 新增一个工具

1. 如果没有稳定 id，加到 `devtools_core::ids`。用短横线小写英文，之后不改。
2. 在 `devtools-tools` 加模块。导出 `meta()` 和纯函数。错误类型带上用户能看懂的位置信息。
3. 先写失败测试，再写实现。不要测 GPUI。
4. 确认 `NetworkUse`。拿不准就当它会联网，不要默认成离线。
5. 把 `meta()` 放进 `catalog()`。这时界面会列出它。没接编辑器之前，`availability` 用 `LogicOnly`。
6. 在壳里接输入和输出。接完改为 `Ready`。
7. 如果这个工具适合剪贴板猜测，改 `suggest`。新规则要有漏报测试（看起来像、但不该猜中的输入）。猜中了也不要自动切换。
8. 更新本文的工具目录。

不要为了单个工具修改 `devtools-core` 的搜索语义。搜索只看元数据。

## 风险

| 风险 | 影响 | 处理 |
| --- | --- | --- |
| GPUI 0.2.2 停在 2025-10，主分支已经分叉 | 缺 API 时不能随手升级 | 按决策 1。能在应用里补的交互就在应用里补 |
| 没有现成多行编辑器 | M1 被卡住，后面的工具都只是函数 | M1 只做一个够用的文本框。高亮推到 M1 之后，而且不是第一版目标 |
| GPUI 编译慢 | 反馈变慢 | 算法放在 `devtools-tools`，日常用 `cargo test -p devtools-tools` |
| Xcode 27 没有 Metal Toolchain | `gpui` 构建失败，报 `cannot execute tool 'metal'` | `xcodebuild -downloadComponent MetalToolchain`，见「约束」 |
| 识别误判 | 用户不信任建议 | 不自动切换；规则宁可漏报；未注册工具不出现在建议里 |
| 窗口关闭后进程不退 | 看起来像后台驻留 | 代码按官方示例写了。`cmd-w` 与红灯关闭均已实测退出 |
| 有人打开 default features | macOS 构建卷进 Wayland / X11 | 依赖集中在 workspace。改 feature 时先看本文「约束」 |
| 工具输入被记进日志或最近使用 | 和离线工具箱的信任相反 | 最近使用只存工具 id。没有日志就不要先加日志 |

## 未决问题

- 最终产品名和 macOS bundle id。没有选定域名之前，窗口不设置 `app_id`。
- 许可证。仓库 `publish = false`，避免误发到 crates.io。
- ~~本地数据目录~~：已定（见「当前状态」与 M4）。
- 全局热键和状态栏要不要做。已调研范围外；不承诺。
- 亮色主题和英文界面。当前不做。

## 建议的下一步

1. 继续按 it-tools 名单补离线纯函数工具（见「it-tools 覆盖」下一批）。
2. 合并当前 PR 后，在 macOS 真机复验（标题栏融合、触控板/IME、Metal、剪贴板权限）。
3. M5 后续：证书解码、CSV、SQL 格式化。M6 分发取决于 Apple 账号。
