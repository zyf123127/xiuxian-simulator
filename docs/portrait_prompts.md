# AI 立绘生成指南（v1.7.2 起·全身像版）

游戏支持用 AI 生成的立绘**覆盖**程序像素立绘：把生成好的 PNG 按下面的文件名放进
`assets/portraits/`，桌面版启动时自动加载（没有文件就回退到内置像素立绘，H5 版暂用像素立绘）。

**v1.7.2 起全部立绘为全身像**：游戏内按图片宽高比等比收进、足底对齐站位，竖版比例越接近 1:2 越好看。

## 文件清单与用途

| 文件名 | 用途 | 出现位置 |
| --- | --- | --- |
| `portrait_0.png` | 凡人档立绘（炼气前） | 现实主界面、角色面板 |
| `portrait_1.png` | 道基档立绘（筑基~金丹） | 同上，境界提升后自动切换 |
| `portrait_2.png` | 紫府档立绘（元婴~合体） | 同上 |
| `portrait_3.png` | 真仙档立绘（渡劫以上） | 同上 |
| `npc_elder.png` | 玄诚长老 | 人物页 |
| `npc_merchant.png` | 胡掌柜 | 人物页 |
| `comp_0.png` ~ `comp_9.png` | 十位女道侣（顺序见下表） | 人物页道侣卡 |

**推荐尺寸 640×1280 或 512×1024（1:2 竖版全身像）**，PNG。游戏内等比缩放绘制。

## 女道侣文件对照

| 文件 | 道侣 | 造型关键词 |
| --- | --- | --- |
| `comp_0.png` | 南宫婉 | 月白鎏银广袖裙·清冷仙子 |
| `comp_1.png` | 紫灵 | 紫纱开衩长裙·神秘妖娆 |
| `comp_2.png` | 陈巧倩 | 鹅黄百褶裙·俏皮灵动 |
| `comp_3.png` | 董萱儿 | 绛红贴身纱衣·妩媚张扬 |
| `comp_4.png` | 慕沛灵 | 水绿轻纱·灵巧可人 |
| `comp_5.png` | 严灵儿 | 玄红劲装·飒爽女剑修 |
| `comp_6.png` | 白瑶怡 | 素白衣袍鎏银缘·冰山医仙 |
| `comp_7.png` | 云芝 | 蓝白华服披帛·端庄师姐 |
| `comp_8.png` | 苏婉 | 藕荷色襦裙·温婉书卷 |
| `comp_9.png` | 月灵 | 银白月华纱裙·超尘出尘 |

## 统一风格规范（让立绘"适合这个游戏"的关键）

画面要贴合游戏现有视觉：**深紫夜空 + 鎏金 UI + 像素仙山夜景 + 古筝**。保持：

1. **构图**：**全身像**——从头到脚完整入画；主角四档为盘膝打坐全身，女道侣与 NPC 为站姿全身；
   人物居中，头顶留白约 5%，足底不留大段空白。
2. **背景**：纯深色（深紫 `#1a1030` ~ 藏蓝 `#141428`）或透明背景，**不要场景**。
3. **女角色服饰要求（吸睛要点）**：
   - 配色大胆明快：正红、紫金、月白鎏银、绛红等高对比主色，忌灰暗；
   - 剪裁显身段：束腰收腰、广袖流云、高开衩裙摆、露肩/锁骨线条等国风仙侠常见设计，
     身姿曼妙、曲线优美；
   - 层次与飘带：披帛、裙摆、丝绦要有飞扬感，饰物精致（鎏金腰链、步摇、耳坠、臂钏）；
   - 整体气质"仙、媚、飒"三选一拉满，克制而不露骨——性感靠剪裁与姿态，不靠裸露。
4. **光线**：柔和轮廓光（rim light），境界/身份越高光越强。
5. **画风**：2D 国风仙侠插画、干净实边；避免厚重 3D 渲染感和照片质感。
6. **系列一致性**：同一角色多档立绘用同一模型、固定风格词；女道侣十人先生成一张定稿
   风格再做其余。

## 提示词（可直接粘贴）

### portrait_0 凡人档（主角·打坐全身）
> 2D国风仙侠游戏立绘，全身像，十六岁农家少年修士盘膝打坐，粗布麻衣，黑色长发简单束起，表情坚毅平静，纯深紫色背景，无场景，柔和边缘光，干净实边插画风格，居中构图，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body portrait, 16-year-old farm boy cultivator seated in meditation, coarse linen robe, black hair in a simple topknot, calm determined expression, plain deep purple background, soft rim light, clean cel-shaded illustration, full figure with feet visible

### portrait_1 道基档（主角·打坐全身）
> 2D国风仙侠游戏立绘，全身像，青年修士盘膝打坐，青色宗门道袍银线绣纹，发簪玉冠，周身若有若无的白色灵气，纯深紫色背景，无场景，柔和轮廓光，干净实边插画风格，全身完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body portrait, young cultivator in teal sect robe with silver embroidery and jade hair crown, seated in meditation, faint white qi aura, plain deep purple background, soft rim light, clean cel-shaded illustration, full figure

### portrait_2 紫府档（主角·打坐全身）
> 2D国风仙侠游戏立绘，全身像，气质出尘的修士悬浮打坐，墨紫仙袍金线云纹，束发金冠，身周一圈淡金色灵环与光点，纯深紫色背景，无场景，紫金轮廓光，干净实边插画风格，全身完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body portrait, ethereal cultivator floating in meditation, dark-violet immortal robe with gold cloud patterns and golden crown, a ring of pale golden spirit light, plain deep purple background, violet-gold rim light, clean cel-shaded illustration, full figure

### portrait_3 真仙档（主角·打坐全身）
> 2D国风仙侠游戏立绘，全身像，仙风道骨的修士悬浮打坐，月白仙袍流金缘边，头戴星冠，头顶金色仙环，周身星辉流转，目光平静如渊，纯深紫色背景，无场景，克制的金色轮廓光，干净实边插画风格，全身完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body portrait, transcendent immortal cultivator floating in meditation, moon-white robe with flowing gold trim, star crown and golden halo, starlight aura, plain deep purple background, restrained golden rim light, clean cel-shaded illustration, full figure

### npc_elder 玄诚长老（站姿全身）
> 2D国风仙侠游戏立绘，全身站姿，白发白须的宗门长老，深青长老袍云鹤纹，手持拂尘，目光慈和有威严，纯深紫色背景，无场景，柔和轮廓光，干净实边插画风格，全身完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, white-haired sect elder with long beard, dark-teal elder robe with cloud-crane patterns, holding a horsetail whisk, kind yet dignified gaze, plain deep purple background, soft rim light, clean cel-shaded illustration

### npc_merchant 胡掌柜（站姿全身）
> 2D国风仙侠游戏立绘，全身站姿，圆滑富态的中年坊市掌柜，绛红掌柜锦袍金扣，手持算盘，笑容可掬眼里精光，纯深紫色背景，无场景，暖色轮廓光，干净实边插画风格，全身完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, plump middle-aged market shopkeeper in crimson brocade robe with gold buttons, holding an abacus, beaming shrewd smile, plain deep purple background, warm rim light, clean cel-shaded illustration

### comp_0 南宫婉
> 2D国风仙侠游戏立绘，全身站姿，清冷仙子，月白色广袖长裙鎏银缘边，束腰显身段，裙裾如流水铺地，青丝如瀑垂腰，鬓边银步摇，肌肤胜雪眉目如画，身姿曼妙气质出尘，披帛轻扬，纯深紫色背景，无场景，冷白轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, ethereal cold fairy, moon-white wide-sleeved gown with silver trim and cinched waist, flowing train like water, waist-length black hair with silver hairpin, porcelain skin, graceful elegant figure, floating shawl, plain deep purple background, cool white rim light, clean cel-shaded illustration, feet visible

### comp_1 紫灵
> 2D国风仙侠游戏立绘，全身站姿，神秘妖娆女修，深紫纱质长裙高开衩露出一侧长腿线条，银线星纹，紫纱遮下半面只露灵动双眸，长腿配鎏金踝链，身段窈窕曲线优美，纯深紫色背景，无场景，紫色轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, mysterious alluring cultivator, deep-violet gauze gown with high side slit revealing one leg with a golden ankle chain, silver star embroidery, lower face veiled in purple gauze showing lively eyes, slender graceful figure, plain deep purple background, violet rim light, clean cel-shaded illustration, feet visible

### comp_2 陈巧倩
> 2D国风仙侠游戏立绘，全身站姿，俏皮少女修士，鹅黄上衣配水绿百褶短裙，纤细腰肢与灵巧小腿，双环髻缀小铃铛，笑容明快，赤足踩一双系带软鞋，身形轻盈灵动，纯深紫色背景，无场景，暖黄轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, playful young cultivator girl, pale-yellow top and jade-green pleated short skirt, slim waist, twin-bun hairstyle with tiny bells, bright cheerful smile, light ribbon sandals, lively nimble figure, plain deep purple background, warm yellow rim light, clean cel-shaded illustration, feet visible

### comp_3 董萱儿
> 2D国风仙侠游戏立绘，全身站姿，妩媚张扬女修，绛红贴身纱衣层叠薄透，束金腰带勾勒腰线，长裙高开衩至大腿，长发松挽金步摇，眼波流转笑意盈盈，锁骨与香肩线条优美，身姿婀娜，纯深紫色背景，无场景，绯红轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, alluring cultivator in layered crimson gauze robes with golden waistband, high-slit long skirt, loosely pinned long hair with golden hairpin, charming knowing smile, elegant collarbone and shoulder lines, graceful bewitching figure, plain deep purple background, crimson rim light, clean cel-shaded illustration, feet visible

### comp_4 慕沛灵
> 2D国风仙侠游戏立绘，全身站姿，灵巧可人女修，水绿色轻纱襦裙束腰，裙摆缀小银铃，单侧发髻系青丝带，腕上玉镯，神情温婉乖巧，身段纤细轻盈，纯深紫色背景，无场景，淡青轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, graceful adorable cultivator in water-green lightweight gauze ruqun with cinched waist, tiny silver bells on the hem, side bun with teal ribbon, jade bracelet, soft demure expression, slender delicate figure, plain deep purple background, pale cyan rim light, clean cel-shaded illustration, feet visible

### comp_5 严灵儿
> 2D国风仙侠游戏立绘，全身站姿，飒爽女剑修，玄色束袖劲装配正红腰封，长靴利落，背负长剑，高马尾红发带飞扬，身姿挺拔长腿利落，目光锐利英气逼人，纯深紫色背景，无场景，银白轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, valiant swordswoman in dark fitted martial outfit with crimson waist sash and long boots, long sword on her back, high ponytail with flowing red ribbon, tall striking figure with sharp heroic gaze, plain deep purple background, silver rim light, clean cel-shaded illustration, feet visible

### comp_6 白瑶怡
> 2D国风仙侠游戏立绘，全身站姿，冰山医仙，素白衣袍鎏银缘边束腰，广袖间隐有药香雾气，领口微敞露出优美锁骨线条，神情疏离淡然，身姿清瘦高挑，纯深紫色背景，无场景，月白轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, aloof healer immortal, plain white physician robe with silver trim and cinched waist, faint medicinal mist around wide sleeves, elegant collarbone line, detached serene expression, tall slender figure, plain deep purple background, moonlight-white rim light, clean cel-shaded illustration, feet visible

### comp_7 云芝
> 2D国风仙侠游戏立绘，全身站姿，端庄宗门师姐，蓝白相间华服层叠云纹缘边，束金玉腰带，裙摆曳地，披帛绕臂飞扬，发髻高挽玉簪步摇，举止大方沉静，身姿挺拔丰满有致，纯深紫色背景，无场景，天青轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, dignified senior sister in layered blue-and-white ornate robe with cloud patterns and golden-jade waistband, floor-length skirt with flowing shawl around arms, high bun with jade pin, calm composed manner, tall graceful figure, plain deep purple background, azure rim light, clean cel-shaded illustration, feet visible

### comp_8 苏婉
> 2D国风仙侠游戏立绘，全身站姿，温婉书卷女修，藕荷色襦裙束腰，裙上暗绣墨梅，外罩淡青披帛垂落及地，怀抱一卷古籍，眉眼温柔含笑，身段柔美纤细，纯深紫色背景，无场景，柔和暖白轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, gentle scholarly cultivator in lotus-pink ruqun with cinched waist and subtle ink-plum embroidery, pale-teal shawl trailing to the ground, holding an ancient book scroll, soft warm smiling eyes, delicate graceful figure, plain deep purple background, soft warm-white rim light, clean cel-shaded illustration, feet visible

### comp_9 月灵
> 2D国风仙侠游戏立绘，全身站姿，月华仙子，银白纱裙层叠如月光倾泻，束银链腰饰，裙裾与丝绦向上飘浮，额间一点月印记，周身萦绕淡淡月辉，身姿超尘出尘曼妙轻盈，纯深紫色背景，无场景，银辉轮廓光，干净实边插画风格，足底完整入画，640x1280
>
> **英文**：2D Chinese xianxia game full-body standing portrait, moon fairy in layered silver-white gauze dress cascading like moonlight, silver-chain waist ornament, hem and ribbons floating upward, a small moon mark on her forehead, faint lunar glow, ethereal graceful figure, plain deep purple background, silver glow rim light, clean cel-shaded illustration, feet visible

### 通用负向提示词（negative prompt）
> 多人, 半身像, 特写, 文字, 水印, 签名, 复杂背景, 场景, 相框, 过曝, 3D渲染, 照片质感, 崩坏的手, 多余的肢体
>
> **英文**：multiple people, bust shot, close-up, text, watermark, signature, busy background, scenery, frame, overexposed, 3D render, photorealistic, bad hands, extra limbs

## 使用步骤

1. 用你顺手的工具（即梦 / Midjourney / SDXL 等）按上面提示词生成，**选择竖版 1:2 出图**；
   风格定稿后锁参数生成整套。
2. 按文件清单重命名，放进 `assets/portraits/`（目录不存在就新建）。
3. 重新 `cargo build --release`（桌面版即生效）；H5 版当前固定使用像素立绘。
4. 效果不满意可随时删文件回退像素立绘，游戏行为不受影响。

> 版权说明：AI 生成内容的权利与使用限制以你所用生成服务的用户协议为准；发布渠道（TapTap 等）如要求标注 AI 素材，请在游戏介绍中如实标注。
