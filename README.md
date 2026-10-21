<p align="center">
  <img src="assets/app-icon/wbs.svg" alt="WBS" width="120">
</p>

<h1 align="center">WBS</h1>

<p align="center">
  <b>开源、中文优先的文字处理软件，纯 Rust 编写，fork 自 WordCraft。</b>
</p>

<p align="center">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-2b47b5?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on macOS, Windows, Linux, BSD and the web" src="https://img.shields.io/badge/runs%20on-macOS%20%C2%B7%20Windows%20%C2%B7%20Linux%20%C2%B7%20BSD%20%C2%B7%20Web-3b5bdb?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-2b47b5?style=flat-square">
  <img alt="Agent-drivable over MCP" src="https://img.shields.io/badge/agents-MCP%20%C2%B7%20CLI-3b5bdb?style=flat-square">
</p>

<br>

<p align="center">
  <img src="docs/images/hero.png" alt="WBS 的「开始」功能区打开在一份两页文档上" width="100%">
</p>

## 简介

WBS 是一个面向中文用户的开源文字处理软件，目标是提供接近 WPS 与 Microsoft Word 的使用体验：功能区（ribbon，窗口顶部按「开始 / 插入 / 页面布局…」分组的一排工具栏）、样式、表格、修订、引用和邮件合并一应俱全，读写 .docx 文件，界面提供 20 余种语言（含简体/繁体中文），并为中文排版做了专门处理：

- **中文排版**：中文断行与避头尾规则，中文/西文字体分槽设置（例如全文中文用宋体、数字和字母用 Times New Roman，互不覆盖），中文编号，全角标点，按字统计字数。
- **文件格式**：读写 .docx（OOXML（.docx 所遵循的开放文件格式标准）），导出带可选中文字和书签的 PDF；支持 .odt、.rtf、.html、.md、.tex（LaTeX）、.txt，并可导入 Word 97-2003 的旧版 .doc。
- **快**：段落排版有缓存，在 188 页的文档里打字，重新排版一段只要约 1.4 毫秒。
- **全平台**：一套 Rust 代码跑 macOS、Windows、Linux、BSD，也能通过 WebAssembly（让 Rust 代码直接在浏览器里运行的技术）跑在网页上；界面用原生 GPU 绘制，不用 Electron。
- **离线可用**：拼写、语法和其余一切功能都不联网。
- **为 AI 代理而生**：每个操作都是一条带编号的命令，功能区、快捷键、命令行、控制通道和 MCP（Model Context Protocol，一种让 AI 代理直接调用软件功能的标准协议）服务器共用同一套命令。

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/review.png" alt="审阅功能区：修订开启，插入和删除的词高亮显示，页边有批注气泡" width="100%"><p align="center"><sub><b>审阅。</b>修订、页边批注、接受/拒绝、边打字边拼写检查。</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/layout.png" alt="页面布局功能区：首字下沉三行、文字环绕图形、自动断字、行号和页面边框" width="100%"><p align="center"><sub><b>页面布局。</b>首字下沉、图文绕排、自动断字、行号、页面边框。</sub></p></td>
</tr>
</table>

## 快速开始

```sh
git clone https://github.com/mooremok/wbs
cd wbs
cargo run --release -p wbs -- --sample      # 桌面版，打开内置示例文档
cargo run --release -p wbs -- 报告.docx      # 或打开你自己的文档
```

命令行工具（`wbs-cli`）：

```sh
wbs-cli convert 报告.docx 报告.pdf           # 转换：docx、pdf、odt、rtf、html、md、tex、txt、png
wbs-cli text 报告.docx                      # 提取纯文本
wbs-cli run --template sample \
  --cmd 'select.text={"text":"会员"}' --cmd format.bold --save 输出.docx   # 无头批量处理
```

网页版：`cd apps/wordcraft-web && trunk serve`，然后打开 <http://127.0.0.1:8771/?sample>。

## 给 AI 代理：命令行与 MCP

WBS 从设计上就同时服务人和 AI 代理。

```sh
claude mcp add wbs -- wbs-cli mcp                        # 无头模式，直接操作文档
wbs --control 7981 &                                     # 或者驱动正在运行的窗口
claude mcp add wbs-app -- wbs-cli mcp --connect 127.0.0.1:7981
```

工具包括 `list_commands`、`execute`、`batch`、`type_text`、`select_text`、`inspect_document`、`render_page`、`save_document`；驱动运行中的窗口时还有 `screenshot`、`click` 和 `key`。详见 [docs/mcp.md](docs/mcp.md) 和[控制协议文档](docs/control-protocol.md)。

## 架构

| 层 | crate | 职责 |
|---|---|---|
| L0 | `wordcraft-geom` | 单位与度量 |
| L1 | `wordcraft-doc`、`wordcraft-fonts`、`wordcraft-proof` | 文档模型与编辑；字体与整形；拼写、语法、断字 |
| L2 | `wordcraft-layout`、`wordcraft-docx`、`wordcraft-formats` | 断行、分页、表格、脚注；OOXML；ODT/RTF/HTML/Markdown/TXT |
| L3 | `wordcraft-render`、`wordcraft-pdf` | 光栅化（vello_cpu）；PDF（Krilla） |
| L4 | `wordcraft-engine` | 会话、撤销、命令注册表、功能目录 |
| L5 | `wordcraft-mcp` | MCP 服务器 |
| L6 | `wordcraft-ui-egui` | 界面（可替换） |
| 应用 | `wbs`、`wbs-cli`、`wbs-web` | 桌面版、命令行、网页版 |

`cargo xtask ci` 运行格式化、clippy、全部测试、资产署名检查、分层检查和 wasm 构建。参与开发请读 [AGENTS.md](AGENTS.md)。

## 下载

各平台安装包（Windows 的 MSI、macOS 的 DMG、Linux 的 AppImage/Flatpak/deb/rpm 等）见 [GitHub 发布页](https://github.com/mooremok/wbs/releases/latest)，每个文件都有 SHA256 校验和。也可以从源码自行编译，见上面的快速开始。

## 上游与致谢

本仓库 **fork 自 [WordCraft](https://github.com/storytold/wordcraft)**——[ArtCraft](https://getartcraft.com/) 团队的开源文字处理软件，也是其 Crafting Apps 系列（PhotoCraft、VectorCraft、FilmCraft、LightCraft、PdfCraft、EffectCraft、DesignCraft，全部用 Rust 从零写成）中的一员。WBS 跟随上游的引擎与文件格式层，同时打造中文优先的体验；上游的绝大部分功能与代码贡献都来自 WordCraft 团队和社区，在此致谢。

按照上游许可的要求，本 fork 移除了 ArtCraft 的品牌文件并更换了自己的名字与标志；WordCraft 和 ArtCraft 名称归 ArtCraft 团队所有。

## 许可证

WBS 采用 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可，可任选其一。版权 (c) 2026 ArtCraft Team 及 WBS 贡献者，必需声明见 [NOTICE](NOTICE)。

内置的字体、图标、图片等资产保留各自的开源许可，每一项的作者、来源和许可都列在 [ATTRIBUTION.md](ATTRIBUTION.md)。英文拼写词典与断字规则来自 Grady Ward 的公有领域词表；波兰语、俄语等拼写检查读取系统安装的词典（不内置，安装方法见 [docs/spelling-dictionaries.md](docs/spelling-dictionaries.md)）。示例文档和模板是为本项目原创撰写的。

<p align="center">
  <sub>WBS 由 <a href="https://github.com/mooremok">mooremok</a> 维护，建立在
  <a href="https://github.com/storytold/wordcraft">WordCraft</a> 团队与社区的工作之上。</sub>
</p>
