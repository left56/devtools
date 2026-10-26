# M1 验证记录

日期：2026-10-23（Asia/Shanghai）。M1 已落盘，没有 commit；仓库仍没有任何提交，源文件全部未跟踪。本轮没有实现 Base64、JWT、建议条或收藏。

本文记录 M1 初次实现的验收。随后 UI 与 JSON 树迭代的增量验收见 [JSON 检查器验证记录](json-inspector-validation.md)；当前单测为 26 个。

## 自动检查

以下检查通过：

```sh
cargo test --workspace
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo fmt --all -- --check
```

本机用临时 `--config` registry 参数取依赖，随后在相同参数下离线验证。没有写仓库或用户级 Cargo 镜像配置。Clippy 的临时参数放在 `clippy` 子命令之后，确保内部 Cargo 调用也收到参数。所有 lockfile registry source 都为 `registry+https://github.com/rust-lang/crates.io-index`。GPUI 固定 `=0.2.2`、关闭 default features、仅启用 `font-kit`；`unicode-segmentation =1.10.1` 只由 `devtools` 直接依赖。

总计 21 个单测：壳 8、core 3、tools 10。测试不启动窗口。

| 覆盖 | 验证内容 |
| --- | --- |
| 缓冲区 | 中文、组合字素与 emoji 的删除边界；正向/反向选区；撤销恢复文本和选区 |
| 换行 | 多行 CRLF/CR → LF、保留末尾空行；单行去换行；纯换行粘贴不删选区；空串替换仍删除 |
| IME 状态 | 相对 UTF-16 选区、代理对边界；组合更新/提交共用一次撤销；取消比较独立 baseline；满栈取消和选区替换取消 |
| 历史 | 上限 128、无 redo；移动光标和错误定位使用的移动方法不增加或移除历史 |
| JSON 运行 | 输出与工具纯函数精确一致并保留键顺序；非法、空白和空输入没有 output；`[1,]` 第 4 列对应偏移 3；错误偏移落在字符边界 |
| 原有行为 | 注册表去重、分类/搜索、严格 JSON 和保序、现有 suggest 规则 |

## 窗口检查

`cargo run -p devtools` 成功启动，没有启动 panic。通过原生 UI 自动化做了下面的点击、粘贴和按键检查，并读取截图验证结果。裸二进制无法被 UI 工具直接识别，因此交互检查使用仓库外临时 `.app` 外壳，运行同一构建产物；这不代表完成正式打包、签名或分发。

| 场景 | 观察结果 |
| --- | --- |
| 启动焦点 | 窗口创建后再聚焦 JSON 编辑器；重启后无需点击即可粘贴 `{"startup":true}` |
| 普通输入 | 回车插入第二行；普通 `a`/`b` 按键、左移、退格和撤销正确，没有字符重复插入 |
| 格式化 | `{"b":1,"a":[true,null],"中文":"😀"}` 产生多行结果；输入保留单行，键顺序未变化 |
| 复制/粘贴 | 点击「复制结果」后，在输入框全选并粘贴，得到完整的多行结果；撤销回到原单行输入 |
| 压缩 | 点击「压缩」后显示紧凑结果 |
| 严格错误 | `{"a":1,}` 显示第 1 行第 8 列错误；结果为空，光标在 `}` 前；不产生修复结果 |
| 清空/空输入 | 清空同时移除输入和结果；随后运行显示第 1 行第 0 列错误，不崩溃 |
| 长输入滚动 | 粘贴 82 行，滚回顶部；运行后报告第 82 行第 1 列，并自动将末尾 `}` 和光标滚入视口 |
| 命令面板 | `cmd-k` 打开；中文「格式化」可找到 JSON；上下键保留搜索内容，回车选中并返回编辑器；Esc 恢复侧边栏原焦点 |
| 侧边栏搜索 | 不存在的查询显示空结果；`json` 找到 JSON；单行回车没有插入换行或运行 JSON |
| 关闭 | 从搜索框 `cmd-w` 关闭后进程退出；重启后点击窗口红灯也退出。均额外核对没有残留 devtools 进程 |

## 尚未完成的人工验收

- 真实 macOS 中文输入法的候选定位、组合更新、取消和提交。缓冲区状态单测与中文/emoji 粘贴不能替代系统 IME 的完整交互。
- 跨行鼠标拖选、长按拖出视口、横向超长行滚动及大文件压力。已接入对应编辑和滚动逻辑，未逐项做窗口验收。
- catalog 当前只有 JSON，因此未在窗口里验证多个工具之间切换。工作台实体由 Workspace 持有，过滤列表或关闭面板不会销毁输入。

## 后续开发入口

- `crates/devtools/src/buffer.rs`：无 GPUI 的文本、选区、IME 与撤销状态。
- `crates/devtools/src/text_field.rs`：逐行整形、绘制、鼠标、键盘、剪贴板、输入法接口和滚动。
- `crates/devtools/src/json_run.rs`：无 GPUI 的 JSON 运行状态和错误偏移。
- `crates/devtools/src/json_workbench.rs`：输入/结果实体、运行、清空和复制按钮。
- `crates/devtools/src/workspace.rs` / `actions.rs`：注册表搜索、命令面板及上下文键绑定。

格式化/压缩不写回输入。继续编辑保留上次运行结果或错误，直到再次运行或清空。清空后撤销只恢复输入，不恢复结果。M1 不提供 redo、自动折行或语法高亮。
