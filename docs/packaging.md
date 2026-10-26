# 各平台安装包（GitHub Actions）

仓库用 GitHub Actions 在 **Linux / macOS / Windows** 上编译 GPUI 壳，并上传可下载的安装包。Windows 仍为实验性（`windows-manifest` 占位，未真机验收）。

工作流 YAML 在 **`.github/workflows/`**（`ci.yml`、`package.yml`）。`ci/workflows/` 下保留同名副本，便于本地对照；以 `.github/workflows/` 为准。

## 怎么触发

| 触发 | 效果 |
| --- | --- |
| **Push** 到 `main` 或 `feat/m2-tools` | 跑 `CI`（测+clippy）与 `Package`（打安装包），产物在 Actions → Artifacts |
| **Pull Request** | 同上；在 PR 的 Checks 里打开对应 run → Artifacts |
| **打 tag** `v*`（例如 `v0.1.0`） | `Package` 额外创建 **GitHub Release**，附件为各平台包 + `SHA256SUMS.txt` |
| **Actions → Package → Run workflow** | 手动重打当前分支 |

示例打 tag：

```bash
git tag v0.1.0
git push origin v0.1.0
```

版本号来自根 `Cargo.toml` 的 `workspace.package.version`（打包脚本读取）；tag 名建议与之对齐。

已发布示例：[v0.1.0](https://github.com/left56/devtools/releases/tag/v0.1.0)。

## 产物矩阵

| 平台 | Runner | 产物 | 说明 |
| --- | --- | --- | --- |
| Linux x86_64 | `ubuntu-22.04` | `devtools-<ver>-linux-x86_64.tar.gz`、`devtools_<ver>_amd64.deb` | X11 GPUI；需本机有 libxkbcommon / libxcb / fontconfig / vulkan 等运行时 |
| macOS Apple Silicon | `macos-14` | `devtools-<ver>-macos-aarch64.zip`（内含 `Devtools.app`）、同名 **unsigned** `.dmg` | runner 自带 Xcode Metal SDK（工作流不下载 MetalToolchain）；未签名 / 未公证，Gatekeeper 会提示 |
| macOS Intel | — | （暂缓；`macos-13` runner 常排队不调度） | 后续补 |
| Windows x86_64 | `windows-latest` | `devtools-<ver>-windows-x86_64.zip`（`devtools.exe`） | **experimental**；构建失败不阻断 Release。暂无 MSI |

未做：**AppImage**（可用 tar.gz / deb）、**已签名 DMG / notarization**、**WiX MSI**。需要时再加 secrets 与打包步骤。

## 本地等价命令

```bash
# Linux
cargo build --release -p devtools
VERSION=0.1.0 bash script/package-linux.sh

# macOS（本机通常需先装 Metal：xcodebuild -downloadComponent MetalToolchain）
cargo build --release -p devtools --target aarch64-apple-darwin
VERSION=0.1.0 TARGET=aarch64-apple-darwin ARCH_LABEL=aarch64 bash script/package-macos.sh

# Windows (PowerShell)
cargo build --release -p devtools
powershell -ExecutionPolicy Bypass -File script/package-windows.ps1
```

### 若需重新推送 workflow 文件

GitHub 要求推送 `.github/workflows/**` 的 token 带 **`workflow` scope**。可用：

```bash
bash script/enable-github-actions.sh
git add .github/workflows
git commit -m "ci: enable GitHub Actions workflows"
git push   # 需 workflow scope
```
