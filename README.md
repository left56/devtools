# Devtools

原生、离线优先的开发者工具箱。用 [GPUI](https://gpui.rs/) 做界面，把 JSON、编码、时间戳、JWT 这类日常操作留在本机，而不是贴进随机网页。

它不是 it-tools、DevUtils 或 DevKnife 的移植。工具广度参考 it-tools，唤起方式参考 DevUtils，检查器的完成度参考 DevKnife。第一版只做高频、可离线完成的工具。

研发计划、已做的决策和未决问题在 [docs/plan.md](docs/plan.md)。

## 当前能做什么

M1 工作台已落盘，目前只注册 JSON：

- 自研多行输入：选择、复制/剪切/粘贴、水平/垂直滚动、等宽字体、最多 128 步线性撤销（无 redo）。
- macOS 原生三色按钮与暗色内容区融合；紧凑侧边栏按「分类 → 工具」显示可折叠文件夹，原文与结果左右分栏，中间可拖动。
- 侧边栏可用顶部箭头或 `cmd-b` 收起/展开；收起后保留工具和搜索入口，输入、结果、查询和文件夹状态保留。
- JSON 树支持展开/收起、节点选择、聚焦子树和返回根；显示并复制选中节点的 JSONPath、jq 表达式和节点值。结果可切换为只读文本。
- 格式化、压缩、清空和复制结果。严格解析并保留键顺序；运行不改输入原文，错误显示行列并定位到输入中的对应行。
- 侧边栏搜索和 `cmd-k` 命令面板都使用注册表搜索。支持中文「格式化」和英文 `json`。
- 系统剪贴板仅在用户粘贴、剪切或复制时读写，没有后台监听。

`cargo test --workspace`（26 个单测）和全工作区 clippy `-D warnings` 已通过。已启动窗口并检查 JSON、剪贴板、撤销、搜索和错误滚动；真实系统 IME 的候选、取消和提交仍需人工验收。详见 [M1 验证记录](docs/m1-validation.md) 和 [JSON 检查器验证记录](docs/json-inspector-validation.md)。

## 环境

本机初始化时使用：

- macOS Apple Silicon
- rustc 1.93.0
- `gpui` 0.2.2（crates.io，只开 `font-kit`）
- Apple metal 32023.921

GPUI 在 macOS 上用 Metal 画界面，需要本机装有 Xcode。Xcode 27 的命令行工具里没有 `metal`。没装 Metal Toolchain 时，`gpui` 0.2.2 编译会报 `cannot execute tool 'metal'`。安装：

```bash
xcodebuild -downloadComponent MetalToolchain
```

## 命令

```bash
cargo test --workspace
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo run -p devtools
```

本机已有依赖缓存时，可以这样临时启动；配置只对这条命令生效：

```bash
cargo run -p devtools --offline --config 'source.crates-io.registry="sparse+https://rsproxy.cn/index/"'
```

关闭窗口后进程退出。`cmd-w` 关闭窗口，`cmd-m` 最小化，`ctrl-cmd-f` 进入/退出全屏，`cmd-b` 收起或展开侧边栏，`cmd-k` 打开或关闭命令面板。面板用上下键选择、回车打开、Esc 返回原焦点；回车只在多行编辑器中插入换行。`cmd-z` 撤销编辑，`cmd-a/c/x/v` 全选、复制、剪切、粘贴，`cmd-left/right` 到行首/行尾，配合 Shift 扩展选区。结果区只读，可选择复制。

格式化后默认展示树，压缩后展示文本；「复制结果」始终复制最近一次运行生成的完整字符串。在树中用上下键选择、左右键折叠/展开，回车聚焦选中容器，Esc 返回根，`cmd-c` 复制选中节点的 JSON 值。路径栏可选取和复制，聚焦子树后仍显示从根开始的绝对路径；jq 栏生成过滤表达式，不执行 jq，也不包含 shell 引号。

格式化或压缩之后继续编辑输入，会保留上次运行的结果或错误，并提示输入已更改，直到再次运行或清空。清空同时移除输入、结果和错误；撤销只恢复输入。

## 布局

```text
crates/devtools/        GPUI 壳：窗口、文本缓冲、输入组件、JSON 工作台、搜索面板
crates/devtools-core/   分类、工具元数据、注册表、搜索。不依赖 GPUI
crates/devtools-tools/  纯函数实现和剪贴板猜测
docs/plan.md            研发计划
```

依赖方向是 `devtools -> devtools-tools -> devtools-core`。算法不能引用 GPUI 类型。
