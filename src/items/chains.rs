/// Item chain definitions for 合合游戏 (HeHe Game).
use super::types::{ChainType, GenerationOption, ItemDef};

// ── Static generation option tables ──────────────────────────────────────────
// These live as static arrays so they can be referenced by &'static [GenerationOption].

/// 老母鸡 auto-gen: 鸡蛋 only
static POULTRY_GEN: &[GenerationOption] = &[GenerationOption::new("egg_1", 100, 0)];

/// 茶壶 gen: 凉茶 (main) + 菊花茶 (level-scaled) + 酒酝圆子 (rare, scales with level)
static TEA_GEN: &[GenerationOption] = &[
    GenerationOption::new("coolTea_1", 85, 0),
    GenerationOption::new("coolTea_2", 0, 2),   // Lv4→8, Lv11→22
    GenerationOption::new("riceBall_1", 5, 1),  // Lv4→9, Lv11→16
];

/// 食篓 gen: 面团 (main) + 汤圆 (level-scaled) + 西瓜 (rare, scales with level)
static BASKET_GEN: &[GenerationOption] = &[
    GenerationOption::new("dough_1", 85, 0),
    GenerationOption::new("dough_2", 0, 2),
    GenerationOption::new("watermelon_1", 5, 1),
];

/// 手作盒 gen: 纸糊灯笼 (main) + 圆灯笼 (level-scaled)
static CRAFTBOX_GEN: &[GenerationOption] = &[
    GenerationOption::new("lantern_1", 88, 0),
    GenerationOption::new("lantern_2", 0, 2),
];

/// 妆奁 gen: 金戒指 (main) + 雕花耳坠 (level-scaled) + 平安扣 (rare, scales with level)
static DRESSER_GEN: &[GenerationOption] = &[
    GenerationOption::new("ring_1", 85, 0),
    GenerationOption::new("ring_2", 0, 2),
    GenerationOption::new("peaceLock_1", 5, 1),
];

/// 织布机 gen: 白色布匹 (main) + 彩色布匹 (level-scaled)
static LOOM_GEN: &[GenerationOption] = &[
    GenerationOption::new("fabric_1", 88, 0),
    GenerationOption::new("fabric_2", 0, 2),
];

/// 绫罗布匹 (fabric_5) gen: 绣花荷包 (main) + 绣花荷包Lv2 (level-scaled)
static FABRIC5_GEN: &[GenerationOption] = &[
    GenerationOption::new("pouch_1", 90, 0),
    GenerationOption::new("pouch_2", 0, 2),
];

/// 红色漆盒 Lv1 gen: 茶壶/食篓/织布机 (equal chance)
static RED_BOX_LV1_GEN: &[GenerationOption] = &[
    GenerationOption::new("teapot_1", 1, 0),
    GenerationOption::new("basket_1", 1, 0),
    GenerationOption::new("loom_1", 1, 0),
];
/// 红色漆盒 Lv2 gen: 茶壶Lv2/食篓Lv2/织布机Lv2 (equal chance)
static RED_BOX_LV2_GEN: &[GenerationOption] = &[
    GenerationOption::new("teapot_2", 1, 0),
    GenerationOption::new("basket_2", 1, 0),
    GenerationOption::new("loom_2", 1, 0),
];

/// 绿色漆盒 Lv1 gen: 老母鸡/妆奁/手作盒 (equal chance)
static GREEN_BOX_LV1_GEN: &[GenerationOption] = &[
    GenerationOption::new("poultry_1", 1, 0),
    GenerationOption::new("dresser_1", 1, 0),
    GenerationOption::new("craftBox_1", 1, 0),
];
/// 绿色漆盒 Lv2 gen: 老母鸡Lv2/妆奁Lv2/手作盒Lv2 (equal chance)
static GREEN_BOX_LV2_GEN: &[GenerationOption] = &[
    GenerationOption::new("poultry_2", 1, 0),
    GenerationOption::new("dresser_2", 1, 0),
    GenerationOption::new("craftBox_2", 1, 0),
];

/// All item definitions in the game.

pub fn all_items() -> Vec<ItemDef> {
    let mut items = Vec::new();

    // ── 老母鸡 (Poultry) auto-generator chain (10 levels) ────────────────────
    // Lv1-5: merge only; Lv6-10: auto-generate 鸡蛋 every hour (storage capacity 6)
    let pc = (0.85, 0.65, 0.35); // warm orange/brown
    items.push(ItemDef::child("poultry_1", ChainType::Poultry, 1, "老母鸡", "🐔", Some("poultry_2"), pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("poultry_2", ChainType::Poultry, 2, "老母鸡", "🐔", Some("poultry_3"), pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("poultry_3", ChainType::Poultry, 3, "老母鸡", "🐔", Some("poultry_4"), pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("poultry_4", ChainType::Poultry, 4, "老母鸡", "🐔", Some("poultry_5"), pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("poultry_5", ChainType::Poultry, 5, "老母鸡", "🐔", Some("poultry_6"), pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::auto_generator("poultry_6",  ChainType::Poultry, 6,  "老母鸡", "🐔", POULTRY_GEN, "egg_1", 3600.0, Some("poultry_7"),  pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::auto_generator("poultry_7",  ChainType::Poultry, 7,  "老母鸡", "🐔", POULTRY_GEN, "egg_1", 3600.0, Some("poultry_8"),  pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::auto_generator("poultry_8",  ChainType::Poultry, 8,  "老母鸡", "🐔", POULTRY_GEN, "egg_1", 3600.0, Some("poultry_9"),  pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::auto_generator("poultry_9",  ChainType::Poultry, 9,  "老母鸡", "🐔", POULTRY_GEN, "egg_1", 3600.0, Some("poultry_10"), pc, Some("images/items/item_icon.png")));
    items.push(ItemDef::auto_generator("poultry_10", ChainType::Poultry, 10, "老母鸡", "🐔", POULTRY_GEN, "egg_1", 3600.0, None,             pc, Some("images/items/item_icon.png")));

    // ── 鸡蛋 (Egg) child chain (7 levels) ────────────────────────────────────
    let ec = (0.98, 0.95, 0.75); // light yellow
    items.push(ItemDef::child("egg_1", ChainType::Egg, 1, "鸡蛋",   "🥚", Some("egg_2"), ec, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("egg_2", ChainType::Egg, 2, "荷包蛋", "🍳", Some("egg_3"), ec, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("egg_3", ChainType::Egg, 3, "蛋炒饭", "🍚", Some("egg_4"), ec, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("egg_4", ChainType::Egg, 4, "蛋炒盖饭", "🍛", Some("egg_5"), ec, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("egg_5", ChainType::Egg, 5, "鸡腿盖饭", "🍗", Some("egg_6"), ec, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("egg_6", ChainType::Egg, 6, "飘香烤鸡", "🍖", Some("egg_7"), ec, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("egg_7", ChainType::Egg, 7, "荷叶蒸鸡", "🪺", None,        ec, Some("images/items/item_icon.png")));

    // ── 茶壶 (Teapot) generator chain (11 levels) ─────────────────────────────
    // Lv1-3: merge only; Lv4-11: weighted-generates 凉茶 + rare 酒酝圆子
    let tc = (0.75, 0.55, 0.35); // brown
    items.push(ItemDef::child("teapot_1", ChainType::Teapot, 1, "茶叶", "🫖", Some("teapot_2"), tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("teapot_2", ChainType::Teapot, 2, "茶叶袋", "🫖", Some("teapot_3"), tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("teapot_3", ChainType::Teapot, 3, "茶叶罐", "🫖", Some("teapot_4"), tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_4",  ChainType::Teapot, 4,  "茶壶",   "🫖", TEA_GEN, "coolTea_1", Some("teapot_5"),  tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_5",  ChainType::Teapot, 5,  "雨前茶壶",   "🫖", TEA_GEN, "coolTea_1", Some("teapot_6"),  tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_6",  ChainType::Teapot, 6,  "青烟茶壶",   "🫖", TEA_GEN, "coolTea_1", Some("teapot_7"),  tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_7",  ChainType::Teapot, 7,  "云栖茶壶", "🫖", TEA_GEN, "coolTea_1", Some("teapot_8"),  tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_8",  ChainType::Teapot, 8,  "松涧茶壶", "🫖", TEA_GEN, "coolTea_1", Some("teapot_9"),  tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_9",  ChainType::Teapot, 9,  "围炉燃云", "🫖", TEA_GEN, "coolTea_1", Some("teapot_10"), tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_10", ChainType::Teapot, 10, "围炉煨雪", "🫖", TEA_GEN, "coolTea_1", Some("teapot_11"), tc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("teapot_11", ChainType::Teapot, 11, "围炉煎月", "🫖", TEA_GEN, "coolTea_1", None,             tc, Some("images/items/item_icon.png")));

    // ── 凉茶 (CoolTea) child chain (14 levels) ────────────────────────────────
    let ct = (0.6, 0.85, 0.75); // teal/green
    items.push(ItemDef::child("coolTea_1",  ChainType::CoolTea, 1,  "茶杯",       "🍵", Some("coolTea_2"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_2",  ChainType::CoolTea, 2,  "凉茶",     "🌼", Some("coolTea_3"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_3",  ChainType::CoolTea, 3,  "热茶",     "🍒", Some("coolTea_4"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_4",  ChainType::CoolTea, 4,  "日铸雪芽茶",   "🍐", Some("coolTea_5"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_5",  ChainType::CoolTea, 5,  "方山露芽茶",     "🌿", Some("coolTea_6"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_6",  ChainType::CoolTea, 6,  "双井白芽茶", "🎋", Some("coolTea_7"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_7",  ChainType::CoolTea, 7,  "霍山黄芽茶",   "🌾", Some("coolTea_8"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_8",  ChainType::CoolTea, 8,  "小龙团茶",   "🍃", Some("coolTea_9"),  ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_9",  ChainType::CoolTea, 9,  "密云龙茶",     "💧", Some("coolTea_10"), ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_10", ChainType::CoolTea, 10, "茶百戏",     "🌺", Some("coolTea_11"), ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_11", ChainType::CoolTea, 11, "拂羽掠影", "🧪", Some("coolTea_12"), ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_12", ChainType::CoolTea, 12, "栖云唳月",     "🫙", Some("coolTea_13"), ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_13", ChainType::CoolTea, 13, "衔云啼春",     "🌸", Some("coolTea_14"), ct, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("coolTea_14", ChainType::CoolTea, 14, "喙渡浮光",   "🧉", None,               ct, Some("images/items/item_icon.png")));

    // ── 酒酝圆子 (RiceBall) rare child chain (7 levels) ───────────────────────
    let rbc = (0.92, 0.85, 0.70); // warm cream
    items.push(ItemDef::child("riceBall_1", ChainType::RiceBall, 1, "奶冻", "🍡", Some("riceBall_2"), rbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("riceBall_2", ChainType::RiceBall, 2, "红豆双皮奶", "🍡", Some("riceBall_3"), rbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("riceBall_3", ChainType::RiceBall, 3, "洒酿圆子", "🍡", Some("riceBall_4"), rbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("riceBall_4", ChainType::RiceBall, 4, "百合绿豆汤", "🍡", Some("riceBall_5"), rbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("riceBall_5", ChainType::RiceBall, 5, "芋圆仙草冻", "🍡", Some("riceBall_6"), rbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("riceBall_6", ChainType::RiceBall, 6, "红豆丸子汤", "🍡", Some("riceBall_7"), rbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("riceBall_7", ChainType::RiceBall, 7, "雪梨丸子汤", "🍡", None,          rbc, Some("images/items/item_icon.png")));

    // ── 食篓 (Basket) generator chain (11 levels) ────────────────────────────
    // Lv1-4: merge only; Lv5-11: weighted-generates 面团 + rare 西瓜
    let bsc = (0.75, 0.65, 0.45); // beige
    items.push(ItemDef::child("basket_1", ChainType::Basket, 1, "竹子", "🧺", Some("basket_2"), bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("basket_2", ChainType::Basket, 2, "竹片", "🧺", Some("basket_3"), bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("basket_3", ChainType::Basket, 3, "竹篓", "🧺", Some("basket_4"), bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("basket_4", ChainType::Basket, 4, "竹简食篓", "🧺", Some("basket_5"), bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_5",  ChainType::Basket, 5,  "竹趣食篓", "🧺", BASKET_GEN, "dough_1", Some("basket_6"),  bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_6",  ChainType::Basket, 6,  "竹华食篓", "🧺", BASKET_GEN, "dough_1", Some("basket_7"),  bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_7",  ChainType::Basket, 7,  "竹隐食篓", "🧺", BASKET_GEN, "dough_1", Some("basket_8"),  bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_8",  ChainType::Basket, 8,  "竹食盒", "🧺", BASKET_GEN, "dough_1", Some("basket_9"),  bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_9",  ChainType::Basket, 9,  "街青食盒", "🧺", BASKET_GEN, "dough_1", Some("basket_10"), bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_10", ChainType::Basket, 10, "锁秋食盒", "🧺", BASKET_GEN, "dough_1", Some("basket_11"), bsc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("basket_11", ChainType::Basket, 11, "空筠食盒", "🧺", BASKET_GEN, "dough_1", None,             bsc, Some("images/items/item_icon.png")));

        // ── 面团 (Dough/Pastry) child chain (15 levels) ───────────────────────────
    let dc = (0.95, 0.85, 0.65); // light tan
    items.push(ItemDef::child("dough_1",  ChainType::Dough, 1,  "面粉",     "🫓", Some("dough_2"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_2",  ChainType::Dough, 2,  "面团",     "🍡", Some("dough_3"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_3",  ChainType::Dough, 3,  "条头糕",     "🥐", Some("dough_4"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_4",  ChainType::Dough, 4,  "糖绒饼",   "🎂", Some("dough_5"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_5",  ChainType::Dough, 5,  "双糖绒饼",   "🌸", Some("dough_6"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_6",  ChainType::Dough, 6,  "芙蓉糕",   "🥮", Some("dough_7"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_7",  ChainType::Dough, 7,  "桃渡糕",   "🍮", Some("dough_8"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_8",  ChainType::Dough, 8,  "海棠糕", "🍰", Some("dough_9"),  dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_9",  ChainType::Dough, 9,  "下兔糕", "🟢", Some("dough_10"), dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_10", ChainType::Dough, 10, "柿了糕", "🎑", Some("dough_11"), dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_11", ChainType::Dough, 11, "玉兔摘柿盒",   "🌺", Some("dough_12"), dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_12", ChainType::Dough, 12, "柿柿如意盒",   "🍍", Some("dough_13"), dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_13", ChainType::Dough, 13, "桃渡拈霞盒",   "🌰", Some("dough_14"), dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_14", ChainType::Dough, 14, "芙蓉映红盒",   "🥜", Some("dough_15"), dc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dough_15", ChainType::Dough, 15, "海棠沉绯盒", "🐇", None,           dc, Some("images/items/item_icon.png")));

    // ── 西瓜 (Watermelon) rare child chain (7 levels) ────────────────────────
    let wc = (0.35, 0.75, 0.35); // green
    items.push(ItemDef::child("watermelon_1", ChainType::Watermelon, 1, "小块西瓜", "🍉", Some("watermelon_2"), wc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("watermelon_2", ChainType::Watermelon, 2, "切块西瓜", "🍉", Some("watermelon_3"), wc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("watermelon_3", ChainType::Watermelon, 3, "半块西瓜", "🍉", Some("watermelon_4"), wc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("watermelon_4", ChainType::Watermelon, 4, "西瓜果切", "🍉", Some("watermelon_5"), wc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("watermelon_5", ChainType::Watermelon, 5, "什锦果切", "🍉", Some("watermelon_6"), wc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("watermelon_6", ChainType::Watermelon, 6, "万事如意果盘", "🍉", Some("watermelon_7"), wc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("watermelon_7", ChainType::Watermelon, 7, "十全十美果盘", "🍉", None,             wc, Some("images/items/item_icon.png")));

    // ── 手作盒 (CraftBox) generator chain (11 levels) ─────────────────────────
    // Lv1-4: merge only; Lv5-11: weighted-generates 灯笼
    let cbc = (0.65, 0.45, 0.25); // dark brown
    items.push(ItemDef::child("craftBox_1", ChainType::CraftBox, 1, "毛笔", "📦", Some("craftBox_2"), cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("craftBox_2", ChainType::CraftBox, 2, "直尺毛笔", "📦", Some("craftBox_3"), cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("craftBox_3", ChainType::CraftBox, 3, "手作用具", "📦", Some("craftBox_4"), cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("craftBox_4", ChainType::CraftBox, 4, "初品手作盒", "📦", Some("craftBox_5"), cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_5",  ChainType::CraftBox, 5,  "下品手作盒", "📦", CRAFTBOX_GEN, "lantern_1", Some("craftBox_6"),  cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_6",  ChainType::CraftBox, 6,  "中品手作盒", "📦", CRAFTBOX_GEN, "lantern_1", Some("craftBox_7"),  cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_7",  ChainType::CraftBox, 7,  "上品手作盒", "📦", CRAFTBOX_GEN, "lantern_1", Some("craftBox_8"),  cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_8",  ChainType::CraftBox, 8,  "精研手作盒", "📦", CRAFTBOX_GEN, "lantern_1", Some("craftBox_9"),  cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_9",  ChainType::CraftBox, 9,  "登科手作桌", "📦", CRAFTBOX_GEN, "lantern_1", Some("craftBox_10"), cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_10", ChainType::CraftBox, 10, "翰林手作台", "📦", CRAFTBOX_GEN, "lantern_1", Some("craftBox_11"), cbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("craftBox_11", ChainType::CraftBox, 11, "御制手作桌", "📦", CRAFTBOX_GEN, "lantern_1", None,                cbc, Some("images/items/item_icon.png")));

    // ── 灯笼 (Lantern) child chain (12 levels) ────────────────────────────────
    let lc = (0.95, 0.35, 0.25); // red/orange
    items.push(ItemDef::child("lantern_1",  ChainType::Lantern, 1,  "灯笼图纸", "🕯️", Some("lantern_2"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_2",  ChainType::Lantern, 2,  "订笼骨架",   "🏮", Some("lantern_3"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_3",  ChainType::Lantern, 3,  "圆灯笼",   "🪔", Some("lantern_4"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_4",  ChainType::Lantern, 4,  "椭圆灯笼",   "💡", Some("lantern_5"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_5",  ChainType::Lantern, 5,  "方形灯笼",   "💡", Some("lantern_6"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_6",  ChainType::Lantern, 6,  "花草灯",     "🎑", Some("lantern_7"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_7",  ChainType::Lantern, 7,  "海棠花草灯",     "🎑", Some("lantern_8"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_8",  ChainType::Lantern, 8,  "提篮灯", "✨", Some("lantern_9"),  lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_9",  ChainType::Lantern, 9,  "百花提篮灯", "✨", Some("lantern_10"), lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_10", ChainType::Lantern, 10, "绛纱灯", "🔆", Some("lantern_11"), lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_11", ChainType::Lantern, 11, "珠帘绛纱灯", "🔆", Some("lantern_12"), lc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("lantern_12", ChainType::Lantern, 12, "六角宫灯", "🌟", None,               lc, Some("images/items/item_icon.png")));

    // ── 妆奁 (Dresser) generator chain (11 levels) ────────────────────────────
    // Lv1-4: merge only; Lv5-11: weighted-generates 戒指 + rare 平安扣
    let drc = (0.85, 0.55, 0.75); // pink/purple
    items.push(ItemDef::child("dresser_1", ChainType::Dresser, 1, 空盒", "💄", Some("dresser_2"), drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dresser_2", ChainType::Dresser, 2, "盖盒", "💄", Some("dresser_3"), drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dresser_3", ChainType::Dresser, 3, "木盒", "💄", Some("dresser_4"), drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("dresser_4", ChainType::Dresser, 4, "妆奁", "💄", Some("dresser_5"), drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_5",  ChainType::Dresser, 5,  "黑漆妆奁", "💄", DRESSER_GEN, "ring_1", Some("dresser_6"),  drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_6",  ChainType::Dresser, 6,  "描金妆奁", "💄", DRESSER_GEN, "ring_1", Some("dresser_7"),  drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_7",  ChainType::Dresser, 7,  "雕花妆奁", "💄", DRESSER_GEN, "ring_1", Some("dresser_8"),  drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_8",  ChainType::Dresser, 8,  "藏香妆奁", "💄", DRESSER_GEN, "ring_1", Some("dresser_9"),  drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_9",  ChainType::Dresser, 9,  "拈花妆奁", "💄", DRESSER_GEN, "ring_1", Some("dresser_10"), drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_10", ChainType::Dresser, 10, "叠绛妆奁", "💄", DRESSER_GEN, "ring_1", Some("dresser_11"), drc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("dresser_11", ChainType::Dresser, 11, "浮麝妆奁", "💄", DRESSER_GEN, "ring_1", None,              drc, Some("images/items/item_icon.png")));

    // ── 戒指 (Ring/Jewelry) child chain (11 levels) ───────────────────────────
    let rc = (0.85, 0.75, 0.25); // gold
    items.push(ItemDef::child("ring_1",  ChainType::Ring, 1,  "蓝宝石",   "💍", Some("ring_2"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_2",  ChainType::Ring, 2,  "金戒指", "📿", Some("ring_3"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_3",  ChainType::Ring, 3,  "宝石戒指",   "⭕", Some("ring_4"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_4",  ChainType::Ring, 4,  "金耳坠",   "💚", Some("ring_5"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_5",  ChainType::Ring, 5,  "雕花耳坠",   "💎", Some("ring_6"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_6",  ChainType::Ring, 6,  "单圈金手镯",   "🌟", Some("ring_7"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_7",  ChainType::Ring, 7,  "双圈金手镯", "🔮", Some("ring_8"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_8",  ChainType::Ring, 8,  "宝相花头簪",     "🦚", Some("ring_9"),  rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_9",  ChainType::Ring, 9,  "蝶恋花头簪", "🦋", Some("ring_10"), rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_10", ChainType::Ring, 10, "蝶恋花发冠", "🏺", Some("ring_11"), rc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("ring_11", ChainType::Ring, 11, "凤冠", "☯️", None,            rc, Some("images/items/item_icon.png")));

    // ── 平安扣 (PeaceLock) rare child chain (7 levels) ───────────────────────
    let plc = (0.55, 0.75, 0.85); // jade/teal
    items.push(ItemDef::child("peaceLock_1", ChainType::PeaceLock, 1, "玉石", "🪬", Some("peaceLock_2"), plc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("peaceLock_2", ChainType::PeaceLock, 2, "平安扣", "🪬", Some("peaceLock_3"), plc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("peaceLock_3", ChainType::PeaceLock, 3, "玉叶子", "🪬", Some("peaceLock_4"), plc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("peaceLock_4", ChainType::PeaceLock, 4, "单只玉佩对牌", "🪬", Some("peaceLock_5"), plc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("peaceLock_5", ChainType::PeaceLock, 5, "一对玉佩对牌", "🪬", Some("peaceLock_6"), plc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("peaceLock_6", ChainType::PeaceLock, 6, "翡翠平安锁", "🪬", Some("peaceLock_7"), plc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("peaceLock_7", ChainType::PeaceLock, 7, "墨玉镶珠龙形佩", "🪬", None,            plc, Some("images/items/item_icon.png")));

    // ── 织布机 (Loom) generator chain (11 levels) ─────────────────────────────
    // Lv1-4: merge only; Lv5-11: weighted-generates 布匹
    let lmc = (0.55, 0.65, 0.85); // blue
    items.push(ItemDef::child("loom_1", ChainType::Loom, 1, "棉花", "🪡", Some("loom_2"), lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("loom_2", ChainType::Loom, 2, "棉花团", "🪡", Some("loom_3"), lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("loom_3", ChainType::Loom, 3, "棉线", "🪡", Some("loom_4"), lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("loom_4", ChainType::Loom, 4, "多股棉线", "🪡", Some("loom_5"), lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_5",  ChainType::Loom, 5,  "单锭纺车", "🪡", LOOM_GEN, "fabric_1", Some("loom_6"),  lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_6",  ChainType::Loom, 6,  "多锭纺车", "🪡", LOOM_GEN, "fabric_1", Some("loom_7"),  lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_7",  ChainType::Loom, 7,  "织布机", "🪡", LOOM_GEN, "fabric_1", Some("loom_8"),  lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_8",  ChainType::Loom, 8,  "斜织机", "🪡", LOOM_GEN, "fabric_1", Some("loom_9"),  lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_9",  ChainType::Loom, 9,  "单综织机", "🪡", LOOM_GEN, "fabric_1", Some("loom_10"), lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_10", ChainType::Loom, 10, "多综织机", "🪡", LOOM_GEN, "fabric_1", Some("loom_11"), lmc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("loom_11", ChainType::Loom, 11, "花楼织机", "🪡", LOOM_GEN, "fabric_1", None,            lmc, Some("images/items/item_icon.png")));

    // ── 布匹 (Fabric) child chain (5 levels) ──────────────────────────────────
    // Lv5: max level AND a click-generator for 荷包
    let fc = (0.75, 0.85, 0.95); // light blue
    items.push(ItemDef::child("fabric_1", ChainType::Fabric, 1, "白布匹", "🧵", Some("fabric_2"), fc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("fabric_2", ChainType::Fabric, 2, "染色布匹", "🎨", Some("fabric_3"), fc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("fabric_3", ChainType::Fabric, 3, "绣花布匹", "✨", Some("fabric_4"), fc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("fabric_4", ChainType::Fabric, 4, "三匹绣花布", "🌺", Some("fabric_5"), fc, Some("images/items/item_icon.png")));
    // fabric_5 is max level AND a click-generator that produces 荷包 (no merge available)
    items.push(ItemDef::generator("fabric_5", ChainType::Fabric, 5, "多匹绣花布", "🎀", FABRIC5_GEN, "pouch_1", None, fc, Some("images/items/item_icon.png")));

    // ── 荷包 (Pouch) child chain (10 levels) ──────────────────────────────────
    let poc = (0.65, 0.75, 0.95); // periwinkle
    items.push(ItemDef::child("pouch_1",  ChainType::Pouch, 1,  "绣绷",     "👛", Some("pouch_2"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_2",  ChainType::Pouch, 2,  "香囊",     "👛", Some("pouch_3"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_3",  ChainType::Pouch, 3,  "荷包",     "👜", Some("pouch_4"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_4",  ChainType::Pouch, 4,  "绣花荷包",     "👜", Some("pouch_5"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_5",  ChainType::Pouch, 5,  "绣花手袋",     "🎒", Some("pouch_6"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_6",  ChainType::Pouch, 6,  "描金绣花手袋",     "🎒", Some("pouch_7"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_7",  ChainType::Pouch, 7,  "货郎包",     "💼", Some("pouch_8"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_8",  ChainType::Pouch, 8,  "绣花货郎包",     "💼", Some("pouch_9"),  poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_9",  ChainType::Pouch, 9,  "描金绣花货郎包",     "🏮", Some("pouch_10"), poc, Some("images/items/item_icon.png")));
    items.push(ItemDef::child("pouch_10", ChainType::Pouch, 10, "菡萏云囊挎包", "🌟", None,             poc, Some("images/items/item_icon.png")));

    // ── 红色漆盒 (RedBox) reward gift box (2 levels) ─────────────────────────
    // Lv1 opens into teapot_1/basket_1/loom_1; Lv2 opens into teapot_2/basket_2/loom_2.
    let rdc = (0.90, 0.20, 0.15); // deep red
    items.push(ItemDef::generator("redBox_1", ChainType::RedBox, 1, "红色漆盒", "🧧", RED_BOX_LV1_GEN, "teapot_1", Some("redBox_2"), rdc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("redBox_2", ChainType::RedBox, 2, "红色漆盒", "🧧", RED_BOX_LV2_GEN, "teapot_2", None,             rdc, Some("images/items/item_icon.png")));

    // ── 绿色漆盒 (GreenBox) reward gift box (2 levels) ───────────────────────
    // Lv1 opens into poultry_1/dresser_1/craftBox_1; Lv2 opens into _2 variants.
    let gbc = (0.20, 0.65, 0.30); // deep green
    items.push(ItemDef::generator("greenBox_1", ChainType::GreenBox, 1, "绿色漆盒", "🎁", GREEN_BOX_LV1_GEN, "poultry_1", Some("greenBox_2"), gbc, Some("images/items/item_icon.png")));
    items.push(ItemDef::generator("greenBox_2", ChainType::GreenBox, 2, "绿色漆盒", "🎁", GREEN_BOX_LV2_GEN, "poultry_2", None,               gbc, Some("images/items/item_icon.png")));

    items
}
