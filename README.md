# 合合游戏 (HeHe Game) — Bevy 0.18.1

一款以古风茶楼为主题的合成消除游戏，使用 [Bevy](https://bevyengine.org/) v0.18.1 编写。

这是对 [gladmo/hehegame](https://github.com/gladmo/hehegame)（React + TypeScript 版本）的 Rust/Bevy 重写版本。

## 游戏界面

游戏运行后显示：
- **顶部栏**：等级、体力、铜板、宝石信息
- **左侧 7×9 棋盘**（63 格）：合成操作区域，底部显示当前选中棋子的详情
- **右侧订单面板**：最多 3 个活跃订单，可提交完成并获得铜板奖励

## 游戏特色

- **7×9 合成棋盘**（63 格），相邻格背景色交替以便辨识
- **18 条棋子链**：5 条主生成器链、9 条子链、3 条稀有子链、2 种奖励礼盒
- **生成器棋子**：
  - 老母鸡🐔 Lv6+（自动定时生成，每小时一次，无需体力）
  - 茶壶🫖 Lv4+、食篓🧺 Lv5+、手作盒📦 Lv5+、妆奁💄 Lv5+、织布机🪡 Lv5+（点击消耗 1 体力生成）
- **合成机制**：同链同级两件棋子合并为下一级棋子
- **拖拽支持**：
  - 拖到空格 → 移动
  - 拖到同链同级格 → 合成升级
  - 拖到不兼容格 → 两件棋子互换位置
- **订单系统**：最多 3 个活跃订单，限时完成可获铜板奖励
- **经济系统**：体力每 2 分钟恢复 1 点，上限 100；升级时体力全满

## 棋子链一览

### 主生成器链与子链

| 生成器链 | 生效等级 | 生成方式 | 子链 | 子链等级数 |
|----------|----------|----------|------|-----------|
| 老母鸡🐔（禽类链，10 级）| Lv6+ | 自动（每小时）| 鸡蛋链 | 7 |
| 茶壶🫖（茶壶链，11 级）| Lv4+ | 点击（1 体力）| 凉茶链 | 14 |
| 食篓🧺（食篓链，11 级）| Lv5+ | 点击（1 体力）| 面团链 | 15 |
| 手作盒📦（手作盒链，11 级）| Lv5+ | 点击（1 体力）| 灯笼链 | 12 |
| 妆奁💄（妆奁链，11 级）| Lv5+ | 点击（1 体力）| 戒指链 | 11 |
| 织布机🪡（织布机链，11 级）| Lv5+ | 点击（1 体力）| 布匹链 → 荷包链 | 5 → 10 |

### 稀有子链（生成概率随生成器等级提升）

| 稀有链 | 来源生成器 | 等级数 |
|--------|-----------|--------|
| 酒酝圆子🍡 | 茶壶🫖 | 7 |
| 西瓜🍉 | 食篓🧺 | 7 |
| 平安扣🔮 | 妆奁💄 | 7 |

### 奖励礼盒

| 礼盒 | 等级数 | 开出物品 |
|------|--------|----------|
| 红色漆盒 | 2 | 茶壶 / 食篓 / 织布机 |
| 绿色漆盒 | 2 | 老母鸡 / 妆奁 / 手作盒 |

## 操作说明

### 点击操作

| 操作 | 效果 |
|------|------|
| 点击有棋子的格 | 选中棋子（高亮显示） |
| 选中后点击同链同级格 | 合成升级 |
| 选中后点击空格 | 移动棋子 |
| 选中后点击其他棋子 | 切换选中到该棋子 |
| 再次点击已选中的生成器 | 消耗 1 体力，生成子棋（老母鸡无需体力）|
| 再次点击已选中的非生成器 | 取消选中 |
| 点击订单「提交」按钮 | 消耗棋盘上所需棋子，完成订单，获铜板 |

### 拖拽操作

| 操作 | 效果 |
|------|------|
| 拖拽棋子到空格 | 移动棋子 |
| 拖拽到同链同级格 | 合成升级 |
| 拖拽到不兼容格 | 两件棋子互换位置 |

## 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 1.80+ | 编程语言 |
| Bevy | 0.18.1 | 游戏引擎 |
| rand | 0.8 | 随机数生成 |

## 在线试玩

> 游戏已部署到 GitHub Pages，可直接在浏览器中游玩（无需安装任何软件）：
>
> **https://gladmo.github.io/bevy-hehe/**

## 本地运行

```bash
# 安装系统依赖（Linux）
sudo apt-get install -y libwayland-dev libxkbcommon-dev libx11-dev libasound2-dev

# 构建并运行
cargo run

# 优化构建
cargo run --release
```

## WASM 构建（本地）

```bash
# 添加 WASM 编译目标
rustup target add wasm32-unknown-unknown

# 安装 wasm-bindgen-cli（版本须与 Cargo.lock 中一致）
WASM_BINDGEN_VERSION=$(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | sed 's/.*"\(.*\)".*/\1/')
cargo install wasm-bindgen-cli --version "${WASM_BINDGEN_VERSION}" --locked

# 编译为 WASM
cargo build --profile wasm-release --target wasm32-unknown-unknown

# 生成 JS 绑定
wasm-bindgen --out-dir ./out --target web --no-typescript \
  target/wasm32-unknown-unknown/wasm-release/bevy-hehe.wasm

# 复制静态资源
cp index.html ./out/
cp -r assets ./out/

# 本地预览（需要 HTTP 服务器，浏览器限制 file:// 协议）
python3 -m http.server 8000 --directory ./out
# 然后访问 http://localhost:8000
```

## 项目结构

```
src/
├── main.rs      # 应用入口、UI 布局、所有 Bevy 系统（含拖拽逻辑）
├── items.rs     # 棋子定义（链类型、生成器属性、合成关系、加权生成表）
├── board.rs     # 棋盘状态（7×9 格）、点击 / 拖拽 / 合成逻辑
├── economy.rs   # 体力 / 铜板 / 宝石 / 经验 / 升级系统
└── orders.rs    # 订单模板、计时、履单逻辑
```

## 参考项目

原版 React + TypeScript 实现：[gladmo/hehegame](https://github.com/gladmo/hehegame)
