# AI 立绘生成指南（v1.7.0 起）

游戏支持用 AI 生成的立绘**覆盖**程序像素立绘：把生成好的 PNG 按下面的文件名放进
`assets/portraits/`，桌面版启动时自动加载（没有文件就回退到内置像素立绘，H5 版暂用像素立绘）。

## 文件清单与用途

| 文件名 | 用途 | 出现位置 |
| --- | --- | --- |
| `portrait_0.png` | 凡人档立绘（炼气前） | 现实主界面、角色面板 |
| `portrait_1.png` | 道基档立绘（筑基~金丹） | 同上，境界提升后自动切换 |
| `portrait_2.png` | 紫府档立绘（元婴~合体） | 同上 |
| `portrait_3.png` | 真仙档立绘（渡劫以上） | 同上 |
| `npc_elder.png` | 玄诚长老 | 人物页、宗门界面 |
| `npc_merchant.png` | 胡掌柜 | 人物页、坊市界面 |
| `comp_0.png` ~ `comp_9.png` | 十位女道侣（顺序见下表） | 人物页道侣卡 |

## 女道侣文件对照

| 文件 | 道侣 | 气质关键词 |
| --- | --- | --- |
| `comp_0.png` | 南宫婉 | 清冷白衣仙子 |
| `comp_1.png` | 紫灵 | 神秘紫衣面纱 |
| `comp_2.png` | 陈巧倩 | 俏皮鹅黄少女 |
| `comp_3.png` | 董萱儿 | 妩媚红衣 |
| `comp_4.png` | 慕沛灵 | 灵巧浅绿侍女 |
| `comp_5.png` | 严灵儿 | 英气女剑修 |
| `comp_6.png` | 白瑶怡 | 冰山医仙 |
| `comp_7.png` | 云芝 | 端庄宗门师姐 |
| `comp_8.png` | 苏婉 | 温婉书卷气 |
| `comp_9.png` | 月灵 | 月华银纱仙子 |

尺寸建议 **512×640（4:5）**，PNG。游戏内会等比缩放绘制，不需要多余留白。

## 统一风格规范（让立绘"适合这个游戏"的关键）

画面要贴合游戏现有视觉：**深紫夜空 + 鎏金 UI + 像素仙山夜景 + 古筝**。生成时保持：

1. **构图**：单人半身像，居中，正面或 3/4 侧，头部占上 1/3；坐姿打坐优先（呼应"轮回镜前静修"主视觉）。
2. **背景**：纯深色（深紫 `#1a1030` ~ 藏蓝 `#141428`）或透明背景，**不要场景**——立绘要叠在游戏面板上。
3. **色调**：主体服饰用青/白/墨色，点缀**鎏金**纹样（与 UI 金框呼应）；高境界档允许紫金灵光。
4. **光线**：柔和轮廓光（rim light），境界越高光越强（凡人几乎无光 → 真仙周身仙环）。
5. **画风**：2D 国风仙侠插画、干净实边、接近游戏立绘卡风格；避免厚重 3D 渲染感和照片质感。
6. **系列一致性**：四档立绘是"同一个主角在成长"——用同一模型、固定风格描述词、相同服装体系（只升级材质与光效），先生成一张定稿风格再做其余三张。

## 提示词（可直接粘贴）

### portrait_0 凡人档
> 2D国风仙侠游戏立绘，单人半身像，十六岁农家少年修士，粗布麻衣，黑色长发简单束起，盘膝打坐，表情坚毅平静，纯深紫色背景，无场景，柔和边缘光，干净实边插画风格，居中构图，512x640
>
> **英文**：2D Chinese xianxia game character portrait, single half-body, 16-year-old farm boy cultivator, coarse linen robe, black hair in a simple topknot, seated in meditation, calm determined expression, plain deep purple background, no scenery, soft rim light, clean cel-shaded illustration, centered composition

### portrait_1 道基档
> 2D国风仙侠游戏立绘，单人半身像，青年修士，青色宗门道袍银线绣纹，发簪玉冠，盘膝打坐，周身若有若无的白色灵气，纯深紫色背景，无场景，柔和轮廓光，干净实边插画风格，与同一主角凡人档为同一人，512x640
>
> **英文**：2D Chinese xianxia game portrait, young cultivator in teal sect robe with silver embroidery, jade hair crown, meditating, faint white qi aura, plain deep purple background, soft rim light, clean cel-shaded illustration, same character as the mortal tier

### portrait_2 紫府档
> 2D国风仙侠游戏立绘，单人半身像，气质出尘的修士，墨紫仙袍金线云纹，束发金冠，悬浮打坐，身周一圈淡金色灵环与光点，纯深紫色背景，无场景，紫金轮廓光，干净实边插画风格，同一主角，512x640
>
> **英文**：2D Chinese xianxia game portrait, ethereal cultivator in dark-violet immortal robe with gold cloud patterns, floating in meditation, a ring of pale golden spirit light and motes, plain deep purple background, violet-gold rim light, clean cel-shaded illustration, same character

### portrait_3 真仙档
> 2D国风仙侠游戏立绘，单人半身像，仙风道骨的修士，月白仙袍流金缘边，头戴星冠，头顶金色仙环，周身星辉与仙光流转，目光平静如渊，纯深紫色背景，无场景，强烈但克制的金色轮廓光，干净实边插画风格，同一主角，512x640
>
> **英文**：2D Chinese xianxia game portrait, transcendent immortal cultivator, moon-white robe with flowing gold trim, star crown, golden halo, starlight aura, calm abyssal gaze, plain deep purple background, restrained golden rim light, clean cel-shaded illustration, same character

### npc_elder 玄诚长老
> 2D国风仙侠游戏立绘，单人半身像，白发白须的宗门长老，深青长老袍云鹤纹，手持拂尘，目光慈和有威严，纯深紫色背景，无场景，柔和轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, white-haired sect elder with long beard, dark-teal elder robe with cloud-crane patterns, holding a horsetail whisk, kind yet dignified gaze, plain deep purple background, soft rim light, clean cel-shaded illustration

### npc_merchant 胡掌柜
> 2D国风仙侠游戏立绘，单人半身像，圆滑富态的中年坊市掌柜，绛红掌柜锦袍金扣，手拨算盘，笑容可掬眼里精光，纯深紫色背景，无场景，暖色轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, plump middle-aged market shopkeeper, crimson brocade robe with gold buttons, hand on an abacus, beaming shrewd smile, plain deep purple background, warm rim light, clean cel-shaded illustration

### comp_0 南宫婉
> 2D国风仙侠游戏立绘，单人半身像，清冷仙子，素白广袖仙裙，青丝如瀑垂腰，眉目如画气质出尘，微微侧首，纯深紫色背景，无场景，冷白轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, ethereal cold fairy in plain white wide-sleeved immortal dress, waist-length black hair, exquisite serene features, plain deep purple background, cool white rim light, clean cel-shaded illustration

### comp_1 紫灵
> 2D国风仙侠游戏立绘，单人半身像，神秘女修，紫纱遮下半面，紫衣绣银纹，只露出的双眸灵动狡黠，纯深紫色背景，无场景，紫色轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, mysterious cultivator in violet robe with silver embroidery, lower face veiled by purple gauze, lively cunning eyes, plain deep purple background, violet rim light, clean cel-shaded illustration

### comp_2 陈巧倩
> 2D国风仙侠游戏立绘，单人半身像，俏皮少女修士，鹅黄衫子水绿裙，梳双环髻缀小铃铛，笑容明快，纯深紫色背景，无场景，暖黄轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, playful young cultivator girl in pale-yellow top and jade-green skirt, twin-bun hairstyle with tiny bells, bright cheerful smile, plain deep purple background, warm yellow rim light, clean cel-shaded illustration

### comp_3 董萱儿
> 2D国风仙侠游戏立绘，单人半身像，妩媚女修，绛红纱衣金线滚边，长发松挽金步摇，眼波流转笑意盈盈，气质张扬而不失克制，纯深紫色背景，无场景，绯红轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, alluring cultivator in crimson gauze robe with gold trim, loosely pinned long hair with a golden hairpin, charming knowing smile, plain deep purple background, crimson rim light, clean cel-shaded illustration

### comp_4 慕沛灵
> 2D国风仙侠游戏立绘，单人半身像，灵巧女修，浅绿襦裙素雅干净，单侧发髻系青丝带，神情温婉乖巧，纯深紫色背景，无场景，淡青轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, graceful gentle cultivator in light-green ruqun, side bun tied with a teal ribbon, soft demure expression, plain deep purple background, pale cyan rim light, clean cel-shaded illustration

### comp_5 严灵儿
> 2D国风仙侠游戏立绘，单人半身像，英气女剑修，玄色劲装束袖，背负长剑，高马尾红发带，目光锐利飒爽，纯深紫色背景，无场景，银白轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, valiant swordswoman in dark fitted martial outfit, long sword on her back, high ponytail with red ribbon, sharp dashing gaze, plain deep purple background, silver rim light, clean cel-shaded illustration

### comp_6 白瑶怡
> 2D国风仙侠游戏立绘，单人半身像，清冷医仙，素白医袍银线绣药草纹，广袖间隐有药香雾气，神情疏离淡然，纯深紫色背景，无场景，月白轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, aloof healer immortal in plain white physician robe with silver herb embroidery, faint medicinal mist around sleeves, detached serene expression, plain deep purple background, moonlight-white rim light, clean cel-shaded illustration

### comp_7 云芝
> 2D国风仙侠游戏立绘，单人半身像，端庄女修，蓝白相间宗门道袍云纹缘边，发髻整洁玉簪固定，举止大方沉静，纯深紫色背景，无场景，天青轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, dignified cultivator in blue-and-white sect robe with cloud patterns, neat hair bun fixed with a jade pin, calm composed manner, plain deep purple background, azure rim light, clean cel-shaded illustration

### comp_8 苏婉
> 2D国风仙侠游戏立绘，单人半身像，温婉女修，月白襦裙外罩淡青披帛，怀抱一卷古籍，眉眼温柔含笑，书卷气十足，纯深紫色背景，无场景，柔和暖白轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, gentle scholarly cultivator in moon-white ruqun with pale-teal shawl, holding an ancient book scroll, soft warm smiling eyes, plain deep purple background, soft warm-white rim light, clean cel-shaded illustration

### comp_9 月灵
> 2D国风仙侠游戏立绘，单人半身像，月华仙子，银白纱衣层叠如月色流光，额间一点月印记，周身萦绕淡淡月辉，神情清冷悠远，纯深紫色背景，无场景，银辉轮廓光，干净实边插画风格，512x640
>
> **英文**：2D Chinese xianxia game portrait, moon fairy in layered silver-white gauze dress shimmering like moonlight, a small moon mark on her forehead, faint lunar glow around her, serene distant expression, plain deep purple background, silver glow rim light, clean cel-shaded illustration

### 通用负向提示词（negative prompt）

> 多人, 文字, 水印, 签名, 复杂背景, 场景, 相框, 过曝, 3D渲染, 照片质感, 崩坏的手
>
> **英文**：multiple people, text, watermark, signature, busy background, scenery, frame, overexposed, 3D render, photorealistic, bad hands

## 使用步骤

1. 用你顺手的工具（即梦 / Midjourney / SDXL 等）按上面提示词生成，风格定稿后锁参数生成整套。
2. 按文件清单重命名，放进 `assets/portraits/`（目录不存在就新建）。
3. 重新 `cargo build --release`（桌面版即生效）；H5 版当前固定使用像素立绘。
4. 效果不满意可随时删文件回退像素立绘，游戏行为不受影响。

> 版权说明：AI 生成内容的权利与使用限制以你所用生成服务的用户协议为准；发布渠道（TapTap 等）如要求标注 AI 素材，请在游戏介绍中如实标注。
