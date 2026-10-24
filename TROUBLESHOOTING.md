# 排障手册：H5（TapTap/X5 内核）真机问题全记录

本手册沉淀自 v1.8.4 → v1.9.23 约 20 个版本、十几轮真机实测（红米 K80 Pro + TapTap 容器 + X5 内核）踩出的全部坑与解法。每一条都有真机日志/截图实证。适用于一切 Rust + macroquad（miniquad wasm）打包 H5 在国产手机容器里运行的项目。

## 一、故障树速查

| 症状 | 根因 | 解法版本 |
|------|------|---------|
| 首次加载慢、点按钮没反应 | 容器对慢加载页面钳制一切定时器，rAF 底座被换成的 timer 也死 | v1.9.20 双加载 + v1.9.21 藏进加载层 |
| 进游戏后点按钮卡 ~5 秒 | 游戏中期（页面 idle 后）容器再次钳制；手势栈内直调 wasm 无效 | v1.9.19 触摸 burst 双层（setTimeout 宏任务） |
| "引擎无响应"大旗误弹 | 弹旗条件只看"触摸≥2 且 4 秒零消费"，健康挂机也中招 | v1.9.15/16 心跳停滞前置判据 + 车夫模式豁免 |
| 菜单→游戏内→重载→回菜单 | 游戏内预防性重载丢进度踢回菜单 | v1.9.22 游戏内禁预防性重载 |
| 画面模糊 | backing=CSS 被 GPU 放大 3 倍 | v1.9.10 固定 3× 超采样（canvas clientWidth getter 沿原型链覆盖） |
| 点击坐标全错 | dpr≠1 乘算与容器不符 | v1.9.9 dpr=1 制（high_dpi=false + cam.rs 读 touches 表） |
| 竖屏提示/画面出屏 | X5 横屏不旋转页面 | orientFix 整页 rotate(90deg) + rotationAware 触摸换算 |
| WebGL 上下文丢失变黑 | 容器回收 GL | contextlost preventDefault + restored 整页重载 |
| 无限重载循环 | 夸克等内核 sessionStorage 刷新即丢，预算标记失效 | v1.9.19 预算 URL 化（query+hash 双保险） |

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
- 默认全关：红十字/黄点/诊断行（c:F:D:S:T:M）/菜单自检面板/errlog/左下坐标面板。
- `?dbg=1` 打开：JS 侧 XX_DBG 显层 + onReady 调 `xx_set_debug(1)`（wasm 导出 → Rust AtomicBool）。
- 真错误（[ERR]/[PANIC]/重载行）无条件强制显示。
- 功能逻辑（点击计数、菜单兜底、8 秒自动进入、心跳/状态上报）不受开关影响——看门狗依赖它们。

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
