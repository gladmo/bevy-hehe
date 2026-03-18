# 合合游戏 (HeHe Game) — Bevy 0.18.1

一款以古风茶楼为主题的合成消除游戏，使用 [Bevy](https://bevyengine.org/) v0.18.1 编写。

这是对 [gladmo/hehegame](https://github.com/gladmo/hehegame)（React + TypeScript 版本）的 Rust/Bevy 重写版本。

## 游戏截图

游戏运行后显示：
- **顶部栏**：等级、体力、铜板、宝石信息
- **7×9 棋盘**：63 格合成区域
- **右侧订单面板**：3 个活跃订单，可提交完成

## 游戏特色

- **7×9 合成棋盘**（63 格）
- **13 条棋子链**：禽类、鸡蛋、茶壶、凉茶、食篓、面团、手作盒、灯笼、妆奁、戒指、织布机、布匹、荷包
- **生成器棋子**：
  - 老母鸡🐔（自动定时生成）
  - 茶壶🫖、食篓🧺、手作盒📦、妆奁💄、织布机🪡（点击消耗1体力生成）
- **合成机制**：同类同级两件棋子 → 下一级棋子
- **订单系统**：3 个活跃订单，完成后获得铜板奖励
- **体力 / 经济系统**：体力每 2 分钟恢复 1 点，上限 100

## 棋子链一览

| 链名 | 生成器 | 最高等级 |
|------|--------|----------|
| 老母鸡链 | 老母鸡🐔（自动）| 10 |
| 鸡蛋链 | — | 7 |
| 茶壶链 | 茶壶🫖（点击）| 11 |
| 凉茶链 | — | 14 |
| 食篓链 | 食篓🧺（点击）| 11 |
| 面团链 | — | 15 |
| 手作盒链 | 手作盒📦（点击）| 11 |
| 灯笼链 | — | 12 |
| 妆奁链 | 妆奁💄（点击）| 11 |
| 戒指链 | — | 11 |
| 织布机链 | 织布机🪡（点击）| 11 |
| 布匹链 | — | 5 |
| 荷包链 | — | 10 |

## 操作说明

| 操作 | 效果 |
|------|------|
| 点击有棋子的格 | 选中棋子 |
| 再点同类同级格 | 合成升级 |
| 点击选中后点空格 | 移动棋子 |
| 双击（点击已选中的）生成器 | 消耗 1 体力，生成子棋 |
| 点击订单"提交"按钮 | 完成订单，获得铜板 |

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
├── main.rs      # 应用入口、UI 布局、所有 Bevy 系统
├── items.rs     # 棋子定义（链类型、生成器属性、合成关系）
├── board.rs     # 棋盘状态（7×9 格）、点击逻辑
├── economy.rs   # 体力 / 铜板 / 宝石 / 经验系统
└── orders.rs    # 订单模板、计时、履单逻辑
```

## 参考项目

原版 React + TypeScript 实现：[gladmo/hehegame](https://github.com/gladmo/hehegame)
