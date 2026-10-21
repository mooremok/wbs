# wbs 三项需求实施计划（品牌改名 → 导出增强 → 中文默认体验）

三个阶段依序进行，每阶段独立提交，提交前跑 `cargo xtask ci`（格式化、clippy、测试、资产、分层、wasm 全部通过才算完成）。

## 阶段 1：品牌改为 WBS（只改用户可见层，内部 crate 名 `wordcraft-*` 保留以保持上游可合并）

1. **应用名与界面文案**：窗口标题 `- WordCraft` → `- WBS`（`crates/ui-egui/src/lib.rs:2248,2250`）；About 对话框（`dialogs.rs:2686-2700`）及约 14 处含品牌的 `tl!` 字符串；23 个 i18n 目录 `.tsv` 中共约 910 个 `WordCraft` 词条（含 zh-hans）脚本替换为 `WBS` 并更新 `i18n/audit.rs:175` 的品牌白名单；`apps/wordcraft-web/index.html` 标题。
2. **二进制名**：`wordcraft`→`wbs`、`wordcraft-cli`→`wbs-cli`、`wordcraft-web`→`wbs-web`（只改各 `Cargo.toml` 的 package/bin 名，目录名不动）；同步 `CARGO_BIN_EXE_*` 测试常量、`xtask/src/parity.rs` 的 `-p wordcraft-cli`、打包脚本引用。`WORDCRAFT_*` 环境变量本期保留（对外兼容接口）。
3. **配置目录**：`crates/control-key/src/lib.rs` 的 `settings_dir()`（macOS `~/Library/Application Support/WBS`，其余 `wbs`）及其文件夹名测试断言；web localStorage 键 `wordcraft.ui`→`wbs.ui`；AGENTS.md/CLAUDE.md 里的控制密钥路径同步更新。
4. **导出文件元数据**：PDF Creator/Producer（`crates/pdf/src/lib.rs:402`）、HTML/ODT/RTF/LaTeX 的 generator 串、MCP serverInfo 名称。
5. **打包标识**：macOS `Info.plist.in` bundle id `ai.storyteller.wordcraft`→`com.mooremok.wbs` 及显示名；`package.sh`/dmg 脚本；Linux desktop/metainfo/mime/nfpm/flatpak 的 id；Windows WiX 产品名并换新 GUID（fork 无存量安装，直接换新身份）；`build.rs` winres；`release.yml` 的 `app-name: wbs / app-title: WBS`。
6. **图标与品牌资产（上游许可强制要求 fork 换标）**：删除 `docs/brand/` 的 ArtCraft 标志；新绘一个原创 WBS 标志（SVG 起稿，生成 icns/ico/png 全套，复用 `xtask/src/ico.rs`、`packaging/icons.sh`），替换 `assets/app-icon/` 下 13 个文件并更新 `apps/wordcraft/src/main.rs:260-262`、`build.rs` 引用；`ATTRIBUTION.md` 新增原创资产行，`cargo xtask assets` 必须通过。
7. **README.md** 标题与简介改为 wbs。

验证：`cargo xtask ci` 全绿；`cargo run -p wbs --release -- --sample` 跑起来截图确认标题栏与 About；`wbs-cli --help` 正常。

## 阶段 2：导出 PDF/图片增强（PDF 全页导出已存在，补图片短板和选项）

1. **引擎命令**：`file.exportPng`（`crates/engine/src/cmd/file.rs:341-359`）扩展参数：`pages`（`"all"` 或 `"1-3,5"`）、`format`（`png`/`jpeg`）、`quality`（1-100，默认 90）；多页时输出 `name-1.png … name-N.png`（仅一页不加后缀），返回文件列表。`io_ext::save` 新增 `jpg`/`jpeg` 分支接上已实现但无人调用的 `Rendered::to_jpeg`（`crates/render/src/lib.rs:77`），`SAVE_EXTS` 增加 jpg。
2. **CLI**：`convert` 增加 `--pages/--scale/--format/--quality`（`render` 已有 `--page/--scale`，补 `--format/--quality`）；帮助文本与测试同步。
3. **界面**：Backstage 导出页（`print_page.rs:335`）从"直接弹文件对话框"改为先弹导出选项对话框（页码范围、格式 PNG/JPEG、分辨率 1x/2x/4x、JPEG 质量），沿用现有对话框框架；PDF 复用 `file.printPdf` 的页码范围逻辑。
4. **MCP**：`save_document` 支持 `.jpg` 扩展名。
5. **测试**：多页命名规则、JPEG 魔数校验、quality 边界、CLI 参数校验、UI 对话框往返。

验证：用中文样例文档实际导出 PDF/PNG/JPEG（全页、不同分辨率），检查页数、文字渲染、文件大小。

## 阶段 3：中文默认体验（界面语言为中文时）

1. **新建文档中文默认样式**：正文 Normal 东亚字体=宋体、西文=Times New Roman；标题 1-3 东亚字体=黑体；正文首行缩进 2 字符（用 `w:firstLineChars` 字符单位，列表样式自身缩进优先级更高，跑全量 typing 测试确认不破坏列表行为）。
2. **中文字体替换链**（`crates/fonts/src/word.rs` 目前没有任何中文条目，这是硬缺口）：宋体/SimSun→Songti SC→Noto Serif CJK SC；黑体/SimHei→Heiti SC→Noto Sans CJK SC；微软雅黑→PingFang SC；楷体→Kaiti SC；仿宋→STFangsong；等线→PingFang SC，覆盖 macOS/Windows/Linux 三平台候选。
3. **纸张默认**：验证中国区域检测已给 A4（`i18n/mod.rs:325` 的区域逻辑），补测试固定行为。
4. **状态栏字数**：核对中文统计口径（汉字按字、西文按词）与标签本地化（字数/字符数）。
5. **界面字体实测**：中文界面 CJK 字体已优先加载（prefer_hans 链存在），截图检查中英文混排基线。

验证：中文语言下新建文档截图对照 WPS 观感（字体、缩进、纸型、状态栏）；`cargo xtask ci`。

## 本轮不做
内部 crate 名与环境变量改名、WPS 式标签页重排、中文模板中心、主题视觉刷新（作为后续阶段）、内部 parity 文档的大规模改名。

## 风险
- tsv 批量替换需保持 tab 分隔格式——用精确匹配脚本，靠 i18n 现有 1,562 行测试兜底。
- 首行缩进默认值波及所有段落——以全量 typing/layout 测试验证。
- bundle id 与 MSI GUID 变更意味着新安装身份——fork 尚无用户，安全。