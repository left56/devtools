# Linux x86_64（X11）验证记录

日期：2026-10-23（Asia/Shanghai）  
主机：无独显 Linux 盒子，`DISPLAY=:2`（Xvfb），Vulkan 走 Mesa lavapipe。

## 怎么跑

```bash
# 系统依赖（Debian/Ubuntu）
sudo apt-get install -y \
  build-essential pkg-config \
  libxkbcommon-dev libxkbcommon-x11-dev \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxcb-keysyms1-dev libxcb-randr0-dev libxcb-image0-dev \
  libxcb-util-dev libxcb-icccm4-dev \
  libfontconfig1-dev libfreetype6-dev \
  libegl-dev libvulkan-dev mesa-vulkan-drivers \
  imagemagick xdotool x11-utils

export DISPLAY=:2
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json
export LIBGL_ALWAYS_SOFTWARE=1

cargo test --workspace --all-targets
cargo clippy -p devtools-core -p devtools-tools -p devtools --all-targets -- -D warnings
cargo run -p devtools
```

`gpui` 在 Linux 上开 `x11` feature（见 `crates/devtools/Cargo.toml`）。Wayland 未在本机验收。

`xattr` 0.2 与新版 `libc` 的 `ENOATTR` 冲突用仓库内 `patches/xattr-0.2.3` 修补（ENODATA）。

## 已点验

截图在 `/workspace/devtools-linux-verify/`（本机验证产物，未进 git）。

| 场景 | 结果 |
| --- | --- |
| 窗口启动、侧边栏分类、Ctrl+K 面板 | 通过 |
| JSON 输入 + 点击「格式化」→ 树保留键顺序 | 通过（`04-json-after-format-clicks.png`） |
| Base64 `hello world` → `aGVsbG8gd29ybGQ=`（Ctrl+Enter） | 通过（`50-base64-ctrlenter.png`） |
| Hash SHA-256 `abc` → NIST 向量（Ctrl+Enter） | 通过（`51-hash-ctrlenter.png`） |
| UUID v4 生成（Ctrl+Enter） | 通过（`52-uuid-ctrlenter.png`） |
| Timestamp `1700000000` → 本机时区日期时间（Ctrl+Enter） | 通过（`53-ts-ctrlenter.png`） |

变换类工具的主操作也可用 **Ctrl+Enter**（macOS 上为 Cmd+Enter）。侧边栏鼠标点选与 JSON「格式化」点击正常；变换工具顶栏按钮在 Xvfb + xdotool 下命中不稳定，以快捷键验收为准，真机鼠标仍建议再点一次。

## 相对 macOS 仍保留的差异

- 普通装饰标题栏（无透明融合 / 无 traffic lights）
- 快捷键提示写 Ctrl，并同时绑定 Cmd 别名
- 未开 `wayland`；未做签名 / `.app` 包


## M3 检查器点验（同日）

截图在 `/workspace/devtools-linux-verify-m3/`（本机验证产物，未进 git）。推荐看 `final-*.png`。

| 场景 | 结果 |
| --- | --- |
| JWT `alg:none` + 过期 `exp` → 未验签警告、`alg` 原文、`exp` 时间+已过期，header/payload/签名三栏 | 通过（`final-jwt-alg-none-expired.png`） |
| Regex 匹配/捕获组（标明 Rust regex，不是 PCRE）；可选替换 | 通过（`final-regex.png`） |
| Cron 5 段 → 之后 5 次；「本机时区」与「UTC」标注可切换 | 通过（`final-cron-local.png` / `final-cron-utc.png`） |
| URL Parse 拆分各段与 query（无网络） | 通过（`final-url-parse.png`） |
| Diff 行级插入/删除高亮（无外部进程） | 通过（`final-diff.png`） |

导航以 **Ctrl+K 命令面板** + **Ctrl+Enter 运行** 为主（与 M2 相同）；Regex/Diff 多输入框辅以鼠标点击切换字段。侧边栏「安全 / 网络」分类需下滚才看到 JWT、Hash、URL Parse。


## UI 视觉打磨（同日稍后）

截图在 `/workspace/devtools-linux-verify-ui/`（本机验证产物，未进 git）。推荐看 `final-*.png`。

改动摘要：

- 暗色主题分层：`bg` / `sidebar` / `panel` / `elevated`，独立 success / warning / error。
- 主按钮（运行 / 格式化 / 编码）用 accent 实心；清空 / 复制为次级描边按钮；状态用小 chip。
- 窗格标题高度约 36px，工作台内边距更紧。
- **JWT**：Token 顶栏压缩；Header / Payload / Signature 改为内容高度堆叠卡片；警告为紧凑 banner。
- **Diff**：原文/新文本并排在上（约 30% 高），全宽 Diff 结果在下；`+n` / `-m` 摘要 chip。
- **URL Parse**：统一 KV 行（固定标签宽 + mono 值盒）；空的 username/password/fragment 省略；Query Params 独立分区。
- 侧边栏选中行与搜索框跟新 chrome 对齐。

点验：Ctrl+K 切工具 + Ctrl+Enter 运行（与 M3 相同）。macOS Metal 本轮未测。
