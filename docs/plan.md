# Devtools 研发计划

状态：**v0.2.0 已发布**（新增 Log Viewer）；M0–M5 已落盘；侧边栏注册约 80+ 个工具（含 bcrypt / AES / RSA / BIP39，界面为学习向文案）。计时器 UI、相机、PDF 等仍暂缓。当前验收主机是 **Linux x86_64 + X11**；macOS aarch64 / Windows 有 CI 安装包，真机点验与签名分发仍待完善。

读者：接下来实现工具或改界面的人。先看「目标和非目标」与「决策」，加工具直接看「新增一个工具」。

产品名 Devtools 是暂名，跟仓库目录一致。最终名字、bundle id 和许可证还没定。

## UI 设计（Refactoring UI）

**转换器 UX 规则**：凡「一种输入 → 多种等价/相关表示」的工具（编解码、进制、大小写、摘要、数据格式互转等）必须用 `BatchWorkbench`：单输入、能自动识别则识别、批量展示全部有用输出、每行复制。不要用 TransformWorkbench 的方向芯片墙。仅随机生成类保留 TransformWorkbench。Date-time 是标杆。

人格：**developer tools = casual + reserved**。实现要点：

- 独立明/暗色板（冷灰中性 + 克制蓝 accent），跟随系统外观；不用纯黑/纯白，不用紫色默认。
- 间距刻度 `theme::space`（4/8/12/16/24/32）、圆角 `theme::radius`（4/6/8）。
- 共享组件在 `ui.rs`：主按钮 / 次按钮 /「粘贴」**同一固定高度**（28px）与 `text_sm`；安静 chip 可更密；pane header + `pane_title`、`workbench_title`、空状态 `empty_hint`。
- 侧栏分类常开、标签淡化；选中态用左侧强调条而非重边框。
- **BatchWorkbench**：一输入 → 多行标注结果（每行复制）。**转换器统一走此台**（与 Date-time 同设计：能自动识别则识别，批量给出相关输出）。`timestamp` / `base64` / `hash` / `case` / 数据格式 / 颜色 / chmod 等。
- **单行输入**：语义上只接受单值的转换器（CIDR、IPv4、状态码、MIME、颜色、进制等，见 `is_batch_single_line_tool`）用 **上下布局**：顶栏单行输入 + 下方「标签 | 单行值 | 复制」列表，不用高多行文本框，也不用大「结果」面板框住；文档/列表/载荷/多字段协议仍用左右多行布局。
- **依赖告警**：`patches/xattr-0.2.3` 已改 `uint32_t`→`u32`；数学求值用 `fasteval`（去掉 `meval`→`nom` 1.2.4）。剩余 `proc-macro-error2` future-incompat 来自 GPUI→`stacksafe`，等上游，勿本地硬 patch。`patches/grid-1.0.1` 修补 CVE-2026-42199。
- 行为：轻量工具实时跑；HTTP/DNS/UUID/ULID/Lorem 显式运行；输入框「粘贴」；无顶栏剪贴板条；无后台剪贴板监听。

参考技能：https://github.com/jaywilburn/refactoring-ui-skill

## 目的

开发者经常把 JSON、Base64、时间戳、JWT 贴到网页工具里。那些页面能用，但不该成为处理 token、cookie 和请求体的默认路径。

Devtools 是一个本机桌面工具箱：

- 默认离线。会联网的工具必须在界面上标明，并且由用户显式运行。
- 窗口打开就能搜索、粘贴、看结果、复制。不账号、不同步。
- 工具行为是可单测的 Rust 函数。界面只负责编辑、展示和快捷键。
- 检查器（JSON、JWT、正则、cron 等）要能真正完成一次操作，而不是只摆一个文本框。
- 键盘优先：侧边栏搜索、命令面板、显式粘贴；不在后台持续读剪贴板。

## 目标和非目标

当前版本要达成：

- 可在 Linux x86_64（X11）上完整验收；CI 产出 Linux / macOS aarch64 / Windows 安装包（见 [packaging.md](packaging.md) 与 [v0.2.0](https://github.com/left56/devtools/releases/tag/v0.2.0)）。
- 侧边栏、分类、搜索、命令面板都能定位工具。
- 「工具目录」中已实现项可以在窗口里完成一次完整操作：输入、出结果、复制、看到错误位置。
- 纯文本变换和识别规则有表驱动测试。测试不启动 GPUI。
- 关闭窗口即退出，不留后台监听。

明确不做：

- 账号、同步、插件市场、动态库或 WASM 插件 ABI。
- 代码编辑器、语法高亮、语言服务器。
- 用一个 `fn(&str) -> String` 描述所有工具。
- JSON 自动修复、JSON5、默认按键名排序。
- 把 JWT 解码说成验签。`alg: none` 永远不能显示成可信。
- MD5 / SHA-1 当作密码哈希。界面上要写明它们只用于校验和兼容。
- 后台剪贴板监视、全局常驻状态栏。全局热键单独调研，不放进第一版验收。
- WHOIS 产品化、端口扫描。HTTP / DNS 已以 `NetworkUse::Explicit` 落地；端口扫描仍不做。
- AI 功能。

## 约束

这些是事实，不是偏好。

- crates.io 上的 `gpui` 锁定 **0.2.2**（2025-10-22），edition 2024，Apache-2.0。入口是 `Application::new().run`。不要跟 Zed 主分支未发布的 `gpui_platform` 拆分。
- GPUI 仍是 pre-1.0。0.2.2 没有官方多行文本框。壳内自研够用的文本输入。
- 平台 feature：Linux 开 `x11`；macOS 开 `font-kit`。workspace 关掉 GPUI default features（避免无意义卷进未验收的 Wayland 等）。
- macOS 本机构建需要 Metal：`xcodebuild -downloadComponent MetalToolchain`。GitHub `macos-14` runner 自带 Metal SDK，打包工作流不再下载 MetalToolchain。
- 工具 crate 保持无 GPUI、无 `unsafe`。跨平台壳行为以各平台实际编译/点验为准，不提前写进验收。
- `serde_json` workspace 依赖打开 `preserve_order`：对象键顺序是行为的一部分。

## 当前状态

已实现：

- workspace 三 crate：`devtools`、`devtools-core`、`devtools-tools`。
- `ToolMeta`、固定分类、注册表、搜索。重复 id 会在构造注册表时 panic；空分类隐藏。
- JSON `prettify` / `minify`：严格解析、保留键顺序；缩进与格式选项芯片。
- 大批量离线工具经 `BatchWorkbench` / `TransformWorkbench` / `InspectorWorkbench` / `NetworkWorkbench` 接入（详见「工具目录」与应用内侧边栏）。
- 通用自研文本框：多行编辑器、单行搜索、只读结果。选择、复制/剪切/粘贴、滚动、等宽字体与中文回退、128 步线性撤销，无 redo。
- **UI**：明/暗双色板，跟随系统；侧栏图标、模式芯片、颜色实时预览与取色、文件拖放（Base64 Image/File）。
- **M4 效率**：无顶栏剪贴板条；「粘贴」/快捷键读一次；建议芯片；收藏与最近使用写入 `prefs.json`。
- **本地数据目录**：Linux `$XDG_DATA_HOME/devtools` 或 `~/.local/share/devtools`；macOS `~/Library/Application Support/Devtools`。
- **M5 联网（第一批）**：`http-request`、`dns-lookup` 等，`NetworkUse::Explicit`。打开不发请求；响应/URL 不进 prefs。
- macOS 透明原生标题栏融合（`cfg`）；`cmd-b` 侧栏；`cmd-k` 命令面板。
- `suggest` 纯函数：JSON / JWT / 时间戳 / 标准 Base64 等。猜中也不自动切换。
- CI：`.github/workflows/` 中 `CI` + `Package`；`v*` tag 发 Release。

未实现 / 暂缓：

- 证书解码深化、CSV 专用台、更完整 SQL 格式化（部分已有启发式 SQL format）。
- 签名 / notarization、菜单栏、全局热键、状态栏图标、登录项、后台剪贴板监听。
- 语法高亮、自动折行、多光标和 redo。
- Chronometer（需定时器 UI）、相机 / 设备信息、PDF、富文本 HTML、完整表情面板等。
- macOS 真机点验与签名分发；Linux Wayland；Windows 壳完整验收。

跨平台意图：

- **Linux x86_64 + X11**：已可构建并窗口验收（见 [linux-x11-verification.md](linux-x11-verification.md)）。
- **macOS aarch64**：CI 出 `.app` zip / unsigned dmg；本机仍需 Metal；真机点验下一轮。
- **Windows x86_64**：实验性 zip；`windows-manifest` 占位。
- `xattr` 0.2 与新 `libc` 用 `patches/xattr-0.2.3`；`grid` CVE 用 `patches/grid-1.0.1`。

验收命令：

```bash
cargo test --workspace
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo run -p devtools
```

仅工具层：

```bash
cargo test -p devtools-core -p devtools-tools --all-targets
cargo clippy -p devtools-core -p devtools-tools --all-targets -- -D warnings
```

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
- 交互工具的状态放在壳里。它们调用 tools crate 里的纯函数，不把 GPUI 类型传进 tools。
- 纯文本变换不必上 trait。一个工具一个模块，导出 `meta()` 和具体函数即可。

数据流：

1. 用户在编辑器里输入，或点「粘贴」/ `Ctrl+Shift+V`（读一次剪贴板写入）。
2. 轻量工具：输入变化时把 `&str` 交给对应函数（Regex 短防抖）；联网工具等用户点「运行」。
3. 函数返回结果或带位置的错误。
4. 壳展示结果；复制由壳做。
5. 猜测走 `suggest`。壳只显示建议芯片。不自动切换工具，除非用户确认。

进程内日志以后如果要加，默认不得记录工具输入和输出。当前没有日志依赖。

## 决策

下面每一条都是当前决定。被否决的方案只在它很可能被再次提出时保留。

1. **锁定 `gpui` 0.2.2，不跟 Zed 主分支。**\
   理由：0.2.2 有 crates.io 版本和可引用的示例。主分支已经改成尚未发布的 `gpui_platform`。\
   否决：把 GPUI 当 git submodule 或每天升级。\
   升级条件：0.2.2 缺少而且无法在应用内补上的 API，或有影响正确性的缺陷。升级时锁一个 commit，并在本文记上 commit 和原因。

2. **按平台精确开 GPUI feature。**\
   macOS 只开 `font-kit`；Linux 开 `x11`。不要为了「看起来完整」打开 default features。

3. **界面文案用中文，工具名保持开发者会搜索的英文。**\
   分类显示「数据」「编码」。工具显示 "JSON"、"JWT"。关键词同时包含中文和英文。\
   否决：整套界面先做英文再翻译。等有第二个语言需求再把文案从源码里提出去。

4. **不做后台剪贴板监听。**\
   理由：常驻读取会把密码和 token 扫进进程。「数据不离开本机」并不等于「本进程可以随时读剪贴板」。\
   代替：用户按按钮或快捷键时读一次。识别结果是建议。\
   否决：启动后订阅剪贴板变更。以后如果要做，只能是默认关闭的设置。

5. **识别结果不自动切换工具。**\
   理由：误判会换掉用户正在看的内容。规则宁可漏报。

6. **JSON 严格解析，保留键顺序，不默认修复。**\
   理由：修复和排序都会改变用户拿去 diff 的字节。\
   否决：默认 JSON5，或默认按键名排序。修复模式如果以后要做，必须是单独的显式动作。

7. **JWT 只解码（可后续加显式验签）。**\
   解码展示 header、payload、签名的原文和 `alg`。过期时间如果能解析就显示，但不代替验签。\
   `alg: none` 可以识别和解码，界面必须写成不受信任。

8. **哈希不是密码学建议。**\
   提供 MD5、SHA-1、SHA-256、SHA-384、SHA-512 供校验与调试。MD5 / SHA-1 旁标注不用于密码或完整性安全。bcrypt 等为学习/调试向，文案写明。

9. **v1 不做插件 ABI。**\
   理由：工具作者就是这个仓库。内部模块足够。

10. **空分类不显示。**\
    分类枚举固定；侧边栏只列出注册表里实际出现的分类。

## 工具目录

id 一旦发布给注册表就不要改。显示名可以改。完整名单以 `devtools_tools::catalog()` / 应用侧边栏为准；下表保留早期核心与里程碑项，并概括后续批次。

| 优先级 | id | 分类 | 阶段 | 行为 |
| --- | --- | --- | --- | --- |
| 已实现 | `json` | 数据 | M1，Ready | 严格格式化、压缩、树形检查与路径；缩进/格式选项 |
| 已实现 | `base64` | 编码 | M2，Ready | Batch：标准/URL-safe；可解码时一并展示 |
| 已实现 | `url-encode` | 编码 | M2，Ready | Batch：组件编解码 |
| 已实现 | `html-escape` | 编码 | M2，Ready | Batch：实体转义/反转义 |
| 已实现 | `hash` | 安全 | M2，Ready | Batch：MD5…SHA-512；弱算法行标题标注 |
| 已实现 | `uuid` | 生成 | M2，Ready | v1/v3–v7（无 v2）；数量芯片 |
| 已实现 | `timestamp` | 时间 | M2，Ready | Date-time：自动识别 + 批量格式；默认当前时间 |
| 已实现 | `number-base` | 数据 | M2，Ready | Batch：2/8/10/16 |
| 已实现 | `case` | 文本 | M2，Ready | Batch：snake/camel/Pascal/kebab/CONSTANT |
| 已实现 | `jwt` / `regex` / `cron` / `url-parse` / `diff` | 各分类 | M3，Ready | InspectorWorkbench |
| 已实现 | `http-request` / `dns-lookup` | 网络 | M5，Ready | Explicit 联网 |
| 已实现 | 扩展批 | 各分类 | Ready | ULID、Slugify、Lorem、text-stats、roman、percentage、chmod、binary、color；basic-auth、hmac、token、otp、email-norm、list-conv、数据格式（JSON/YAML/TOML/XML/CSV）、sql-format、markdown-html、IPv4×3、ipv6-ula、mac-gen、mime、http-status、ua-parse、iban、temperature、eta、numeronym、obfuscate、nato、safelink、docker-run；xml/json-diff、password-strength、keycode、phone、ascii-draw、random-port、unicode-esc、qr/wifi-qr、meta-tags、svg-placeholder、cron-presets；math-eval、git-memo、regex-memo、base64-hex、bcrypt、encrypt、rsa-keygen、bip39；图形与部分网络扩展（颜色检查器、色板、Base64 Image/File、证书解码等） |
| P2 / 暂缓 | Chronometer、相机、PDF、完整 libphonenumber / FIGlet 等 | — | — | 需专用 UI 或重依赖 |

未注册的工具没有 id，避免半成品出现在注册表里。

## 阶段

每个阶段结束时，上一阶段的行为还在。

### M0–M5（已完成）

- **M0** 骨架：三 crate、注册表、窗口退出。
- **M1** JSON 工作台与自研编辑器；见 [m1-validation.md](m1-validation.md)、[json-inspector-validation.md](json-inspector-validation.md)。
- **M2** 纯函数变换 + BatchWorkbench。
- **M3** 检查器（JWT / Regex / Cron / URL Parse / Diff）。
- **M4** 效率：粘贴、建议、收藏、最近使用、实时运行。
- **M5** 第一批 Explicit 联网（HTTP / DNS 等）。端口扫描不在范围。

### M6 分发（进行中）

- CI 多平台 Package 与 `v0.2.0` Release 已通（Linux tar.gz/deb、macOS aarch64 zip/dmg、Windows zip）；`v0.1.0` 为首发。
- 签名和 notarization 取决于还没定的 Apple 账号。
- macOS Intel（`macos-13`）矩阵已因排队问题暂缓。

## 暂缓清单

| 状态 | 说明 |
| --- | --- |
| 已有大量离线小工具 | 编码/哈希/JWT/OTP/HMAC/bcrypt、AES、RSA、BIP39、UUID/ULID、Date-time、进制与颜色、JSON 全家桶、Markdown/SQL 启发式、网络与图形扩展等 |
| 暂缓及原因 | **Chronometer**：需 GPUI 定时器 UI。**Camera / Device info**：硬件 API。**PDF**：重解析。**html-wysiwyg**：富文本。**Emoji picker**：完整表情包面板。**MAC vendor lookup**：需 OUI 库/联网。**Benchmark builder**：长任务与图表。完整 FIGlet / libphonenumber：体积与数据。 |

安全类（bcrypt / encrypt / rsa / bip39）均 Offline；界面写明学习/调试用途。

设计原则：一输入多结果、每行可复制；随机/密钥类显式「生成」；Linux-first 验收。

## 新增一个工具

1. 如果没有稳定 id，加到 `devtools_core::ids`。用短横线小写英文，之后不改。
2. 在 `devtools-tools` 加模块。导出 `meta()` 和纯函数。错误类型带上用户能看懂的位置信息。
3. 先写失败测试，再写实现。不要测 GPUI。
4. 确认 `NetworkUse`。拿不准就当它会联网，不要默认成离线。
5. 把 `meta()` 放进 `catalog()`。这时界面会列出它。没接编辑器之前，`availability` 用 `LogicOnly`。
6. 在壳里接输入和输出。接完改为 `Ready`。转换器优先 `BatchWorkbench`。
7. 如果这个工具适合剪贴板猜测，改 `suggest`。新规则要有漏报测试。猜中了也不要自动切换。
8. 更新本文的工具目录或侧边栏说明。

不要为了单个工具修改 `devtools-core` 的搜索语义。搜索只看元数据。

## 风险

| 风险 | 影响 | 处理 |
| --- | --- | --- |
| GPUI 0.2.2 停在 2025-10，主分支已经分叉 | 缺 API 时不能随手升级 | 按决策 1。能在应用里补的交互就在应用里补 |
| 没有现成多行编辑器 | 交互能力受限于自研文本框 | 只做够用的输入；高亮不是第一版目标 |
| GPUI 编译慢 | 反馈变慢 | 算法放在 `devtools-tools`，日常用 `cargo test -p devtools-tools` |
| 本机缺少 Metal Toolchain | macOS `gpui` 构建失败 | `xcodebuild -downloadComponent MetalToolchain` |
| 识别误判 | 用户不信任建议 | 不自动切换；规则宁可漏报 |
| 窗口关闭后进程不退 | 看起来像后台驻留 | 按官方示例；关窗退出已测 |
| 有人打开 default features | 构建卷进未验收后端 | 依赖集中在 workspace；改 feature 先看「约束」 |
| 工具输入被记进日志或最近使用 | 和离线工具箱的信任相反 | 最近使用只存工具 id。没有日志就不要先加日志 |

## 未决问题

- 最终产品名和 macOS bundle id。没有选定域名之前，窗口不设置 `app_id`。
- 许可证。仓库 `publish = false`，避免误发到 crates.io。
- 全局热键和状态栏要不要做。已调研范围外；不承诺。
- 英文界面。当前不做。

## 建议的下一步

1. macOS 真机复验（标题栏融合、触控板/IME、Metal、剪贴板权限）与签名/公证调研。
2. 按产品需要补离线纯函数工具；暂缓项单独评估 UI/依赖成本。
3. 证书解码、CSV、SQL 体验继续打磨；Windows 壳验收。
