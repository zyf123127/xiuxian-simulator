# 排障手册：H5（TapTap/X5 内核）真机问题全记录

本手册沉淀自 v1.8.4 → v1.9.24 约 20 个版本、十几轮真机实测（红米 K80 Pro + TapTap 容器 + X5 内核）踩出的全部坑与解法。每一条都有真机日志/截图实证。适用于一切 Rust + macroquad（miniquad wasm）打包 H5 在国产手机容器里运行的项目。

## 一、故障树速查

| 症状 | 根因 | 解法版本 |
|------|------|---------|
| 手机发热/卡顿/被系统杀 | 内存常驻过大：16 张立绘启动全量解码 + 地图底图缓存无节制 | v1.9.24 懒加载 + 缓存收紧 + LTO 瘦身 |
| MuMu 里"游戏打不开/看不见"（页面标题对但画面是搜索结果） | 夸克把 am start 的 URL 开进后台标签，前台停留在搜索页；或分辨率是 widescreen 预设时夸克 UI 只占右侧一列 | uiautomator dump 找游戏标签 bounds 点切前台；分辨率用 phone 竖屏预设（见技能） |
| 首次加载慢、点按钮没反应 | 容器对慢加载页面钳制一切定时器，rAF 底座被换成的 timer 也死 | v1.9.20 双加载 + v1.9.21 藏进加载层 |
| 进游戏后点按钮卡 ~5 秒 | 游戏中期（页面 idle 后）容器再次钳制；手势栈内直调 wasm 无效 | v1.9.19 触摸 burst 双层（setTimeout 宏任务） |
| "引擎无响应"大旗误弹 | 弹旗条件只看"触摸≥2 且 4 秒零消费"，健康挂机也中招 | v1.9.15/16 心跳停滞前置判据 + 车夫模式豁免 |
| 菜单→游戏内→重载→回菜单 | 游戏内预防性重载丢进度踢回菜单 | v1.9.22 游戏内禁预防性重载 |
| 画面模糊 | backing=CSS 被 GPU 放大 3 倍 | v1.9.10 固定 3× 超采样（canvas clientWidth getter 沿原型链覆盖） |
| 点击坐标全错 | dpr≠1 乘算与容器不符 | v1.9.9 dpr=1 制（high_dpi=false + cam.rs 读 touches 表） |
| 竖屏提示/画面出屏 | X5 横屏不旋转页面 | orientFix 整页 rotate(90deg) + rotationAware 触摸换算 |
| WebGL 上下文丢失变黑 | 容器回收 GL | contextlost preventDefault + restored 整页重载 |
| 无限重载循环 | 夸克等内核 sessionStorage 刷新即丢，预算标记失效 | v1.9.19 预算 URL 化（query+hash 双保险） |
| **点按钮毫无反应 + errlog 刷 `already borrowed`**（miniquad native/wasm.rs:34 tl_event_handler；v1.15.0 MuMu 夸克 100% 复现） | JS→wasm 导出在 frame() 持有 EVENT_HANDLER RefCell 借用期间重入：帧路径新增 std `Mutex`/`Instant`/`OnceLock`（wasm 侧引入额外 env 导入路径）触发首 panic → 实例死 → touch/mouse_* 全 `unreachable`，消费帧全灭 → 冻结 → 看门狗重载循环 = 玩家体感"按按钮很卡、经常毫无反应" | 帧路径禁新增 std 同步/时间原语；去抖改纯原子位打包 + `macroquad::time::get_time()`（既有 sys_now 导入零新增）。A/B：Mutex 版 100% 恐慌风暴 → 原子版零恐慌（v1.15.0） |
| **X5 系内核启动死**（QQ浏览器/TapTap 容器）：加载层已撤、黑屏、点哪都没反应、无 panic 无 console.error | **真 X5 实测确证（v1.15.0 启动分段标记）：main() 卡死在 `sounds::init()` 的 decodeAudioData await**（st 停 90→91 之间，suspended 态解码永不返回，主循环一帧未跑）；另有 FPBridge Java 异常（下载/编译期）与 wasm 下载停滞 | ① 音频改 `start_coroutine` 懒加载：同步出空 Audio、主循环每帧 `take_ready` 收货补 BGM——X5 上静音进游戏不再卡死；② 劫持 `console.error` 进 errlog（bundle 启动失败链只走 console.error，内核把 onerror 掩码成 "Script error."）；③ 丢弃式 canvas 预探测 WebGL 扩展；④ 启动死看门狗：exports 就绪 15s 无场景码（st∉0-6）→ 预算内自动重载（v1.15.0） |
| **X5 唤醒后仅 ~0.9fps、游戏时间极慢**（车夫 1s 拍恰=0.9fps；触摸 burst 每击 +8 帧） | 容器对"静音页"施加 intensive throttling：rAF/16ms 心跳/定时器全被钳，触摸 burst 与车夫是唯二帧源，dt 每帧钳 50ms → 游戏时间以 ~4.5% 流速 | 近无声音频保活（1s 循环 buffer、头两样本 0.03 幅值、增益 0.004，不可闻但页面处于 audible 态换 intensive-throttling 豁免）；X5 上 `new AudioContext()` 无手势即 running、而 quad-snd 的上下文 resume/解码可能永不完成——保活须自建专用上下文，不可依赖游戏音频（v1.15.0） |
| 弹层（修炼组等）开着时底栏怎么点都没反应 | 底栏 8 钮 `&& !sub_open` 直接吞点击（弹层不遮底栏但锁死全部按钮） | v1.15.0：弹层开着点底栏 = 关弹层并同击执行，一击必有可见反馈 |

## 二、核心机制（防退化必读）

### 1. 帧驱动：不信任任何单一时钟
- rAF 被劫持为"存回调"（v1.8.4），实际驱动 = MessageChannel 4ms 心跳 + Worker 16ms + setTimeout 16ms 三时钟 → tick() 14ms 门闩统一。
- 全部时钟都可能被容器钳死（intensive throttling：页面不可见/闲置时 chained timer 钳到分钟级）。
- **唯一可靠事件源是触摸**：触摸事件 + 触摸后短暂豁免期内的 setTimeout 能跑。

### 2. 死透态（wedge）与自救链
- 指纹：`frame()` 照常返回（JS 外壳活、fr 恒新鲜）但 Rust 帧号冻结、无任何 panic。
- 自救分层：
  1. **触摸 burst 双层**：手势回调内同步 12 发（桌面有效）+ setTimeout(0) 宏任务 12 发（真机有效——**手势栈内直调 wasm 在真实安卓 WebView 推不动 future，本地 Chromium 测不出**）。
  2. **快速唤醒**：touchstart 后 400ms 检查点击未消费且帧号未涨 → burst 连击 + 重放该次点击。
  3. **帧车夫**：唤醒成功后 1s 拍 setInterval 补帧 24 发（interval 本身也可能被钳，能跑就是增益）。
  4. **冷启滚动检测**（仅菜单阶段）：每秒采样帧号，最近 2 秒 <25 帧即热身重载。
  5. **wedge 评估**（仅菜单阶段）：8 秒窗口 <15fps → 重载换满帧。
- **重载预算 60 秒化**：xxrlt 时间戳写 URL query + hash + sessionStorage 三层（sessionStorage 会丢）；**游戏内（场景>菜单）禁一切预防性重载**（丢进度），真死由主看门狗 2.5s/6s 死线 + 3 轮冲击无效后处理。

### 3. 首开双加载（藏在加载层里）
- 铁律：首开慢加载被钳制、重载后缓存热秒开循环存活——三次真机实证。
- 实现：加载层隐藏逻辑（onReady）内决策——wasm 就绪耗时 >1.5s 且 URL 无 r2=1 → **保持加载条、立即重载**；快加载直接进。用户体验流 = 加载界面→菜单→游戏→流畅，重载不可见。

### 4. 调试开关（v1.9.23）
### 5. 内存占用四板斧（v1.9.24）
- **立绘懒加载（art.rs LazyTex）**：16 张 AI 覆盖立绘解码后约 7MB RGBA，原先 build_art 启动全量解码常驻 GPU；现改首用才解码（缺图 None 也缓存，防每帧重查）。Texture2D 是 Arc 句柄，get() 克隆零拷贝。
- **地图底图缓存 8→3**：每张 1264×480×4≈2.4MB，8 张上限即 ~19MB 常驻；同屏只见一张，3 张覆盖"当前+刚离开"。
- **wasm 体积 2.29→2.03MB（-11.5%）**：`lto="fat"` + `codegen-units=1` + `strip=true` + `panic="abort"`。手机上下载/解析/编译的峰值内存同步下降。
- **errlog 封顶**：textContent += 是 O(n) 整串拷贝，长时间游玩无限累积；超 8KB 截到最近 6KB。
- **实测结论**：wasm 线性内存（Rust 堆）在游戏内完全平稳（18.9MB 恒定）；PNG 解码是瞬时分配后释放，不叠加峰值。真正的节省在 GPU 纹理常驻（黑盒测不到，结构上可证：没创建就不占）。
- **PSS 黑盒对比不可行**：夸克主进程自身随时间涨数百 MB（空白页也涨到 731MB），游戏增量被完全淹没。要测游戏内存用本地 Chromium `wasm_exports.memory.buffer.byteLength`。
- 默认全关：红十字/黄点/诊断行（c:F:D:S:T:M）/菜单自检面板/errlog/左下坐标面板。
- `?dbg=1` 打开：JS 侧 XX_DBG 显层 + onReady 调 `xx_set_debug(1)`（wasm 导出 → Rust AtomicBool）。
- 真错误（[ERR]/[PANIC]/重载行）无条件强制显示。
- 功能逻辑（点击计数、菜单兜底、8 秒自动进入、心跳/状态上报）不受开关影响——看门狗依赖它们。

### 6. 帧率卡顿三板斧（v1.10.0 定位 → v1.11.0 修复 → v1.13.0 收尾，用户确认不卡）
> 完整测量证据与十戒清单见 `docs/性能卡顿分析与解决-v1.9.24-v1.13.0.md`（同款经验已沉淀进 macroquad-bug-hunter / rust-macroquad-game 技能）。
- **字体图集 churn（头号根因）**："点按钮卡一下"= 面板首帧大批新 (字符,字号) 字形首见栅格化→图集 dirty→每帧整图集上传（实测坊市首开 79 次/1.3GB/728ms longtask，二开 0 次）。四招：字号离散档、事件帧 prewarm_text、字号 12→6 档收敛、菜单期分帧预热。
- **GPU 常驻两大头**：图集钳 2048（-96MB）、SS 默认 2（dpr>2 保 3 防模糊回归，-37MB/填充率 -56%）。
- **每帧零分配纪律**：绘制路径禁 clone 大结构/重建派生文本（借用化+惰性缓存——**必须验证缓存命中**，label↔desc 错位比对教训）；过绘预烘焙；烘焙/同步 IO 挪出绘制路径。
- **粒子无罪**："怀疑主角特效"的直觉错在特效的文字（横幅大字动态字号），不在几何粒子（固定池零分配）。
- **加载期卡死**（计秒先走后停、页面零变化）：X5 钳制下 onReady 不可达——v1.13.0 加载停滞看门狗（心跳停摆 ≥20s 或无 wasm 120s → 自动重载，wd≤2 防循环）。

## 三、调试插件（xx_dbg / xx_storage）
- `miniquad_add_plugin` 注册 env 导入，Rust `extern "C"` 成对声明：xx_panic（panic 可见化）、xx_state（场景码）、xx_click、xx_alive（心跳）、xx_js_fr/xx_js_touches（JS 视角反读）。
- Rust→JS 单向即可覆盖全部诊断需求；JS→Rust 用 wasm 导出（xx_set_debug、xx_storage 存档）。
- 构建：`RUSTFLAGS=-C link-arg=--import-undefined`（wasm 导出未显式声明也保留）。

## 四、本地仿真回归（xxSim，URL 参数，生产零影响）
- `?chaos=1`：周期冻结全部帧时钟（容器钳制模拟）。
- `?wedge=1`：周期性死透态全链仿真（时钟死+车夫禁启+同步帧禁用；预算解锁后再冻结，模拟中期再钳制）。
- `?tap=1` / `?flow=1`：自动点击「开始新的人生」断言进游戏（flow=early 模拟一就绪就点）。
- `?ss=N`：超采样倍率；`?compat=0`：关兼容鼠标事件；`?landscape=1`：竖窗旋转视野（模拟器用）。
- 本地测不出 ≠ 真机没问题：**手势栈内 wasm 调用、sessionStorage 持久性、timer 钳制**三者只在真实安卓 WebView 复现。

## 五、MuMu 模拟器验证（详见技能 mumu-emulator-verify）
- 用夸克浏览器（自带 Chromium 缺 WebAssembly，游戏永远"正在加载 WASM"）。
- `cmd window set-ignore-orientation-request true` 破浏览器竖屏锁。
- adb reverse tcp:8125 + 黑盒验证（screencap 读图判读 + input tap + 面板 fr 值=帧率）。

## 六、发版 checklist
1. 改代码后：`python tools/make_font.py`（新文案字符进字体子集）→ 双构建（wasm --import-undefined + 桌面）→ `build_web.bat` 流程（三目录同步 + zip）。
2. `python tools/verify_taptap_zip.py`（版本号三处一致、恰 3 条目、wasm 含 xx_storage）。
3. 本地回归：flow / chaos / 快加载跳过 / 冒烟 `GF_DEBUG=1 timeout 6 exe`（exit=124 即通过）。
4. MuMu 剧本：打开→加载→菜单→进游戏（截屏序列对照体验流）；wedge 页全链自愈。
5. commit + tag + push + gh release（附 exe + taptap_verify.zip）。
