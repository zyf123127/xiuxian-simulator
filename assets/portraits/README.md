# AI 立绘放这里

按以下命名放入本目录，桌面版启动时自动加载并替代内置像素立绘；
缺失即回退。生成提示词与风格规范见 `docs/portrait_prompts.md`。

- 境界四档：`portrait_0.png` ~ `portrait_3.png`（凡人/道基/紫府/真仙）
- NPC：`npc_elder.png`（玄诚长老）、`npc_merchant.png`（胡掌柜）、
  `npc_senior.png`（宁清霜）、`npc_drunk.png`（醉道人）
- 女道侣：`comp_0.png` ~ `comp_9.png`（南宫婉/紫灵/陈巧倩/董萱儿/慕沛灵/严灵儿/白瑶怡/云芝/苏婉/月灵，
  顺序与游戏内 COMPANION_NAMES 一致）

## 重要：新图/换图后必须重跑预处理

```bash
python tools/process_portraits.py
```

该脚本会把图抠背景、裁到人物包围盒并等比缩放到高 1024（原图 1440×2880 约 4MB/张，
处理后约 650KB/张，透明底能自然融入游戏画面）。**H5 版加载的是处理后的内嵌副本**，
所以处理完还要 `build_web.bat` 重新构建；新增槽位图（如 npc_senior）需同时在
`src/art.rs` 的 wasm 内嵌表里各加一行 include_bytes。原始大图在 git 历史（v1.8.0 提交）可随时找回。
