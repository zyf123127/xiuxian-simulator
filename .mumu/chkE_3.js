
        var errlog = document.getElementById("errlog");
        // 加载阶段标记：写在加载层的消息节点上，真机"打不开"时能看出卡在哪一步。
        // v1.9.1：所有阶段提示统一走 ld_msg 节点（此前直接覆盖 #loading 的
        // textContent 会连进度条一起抹掉），并叠加"已等待 N 秒"——手机上大包
        // 首次下载+编译要 10~40 秒，让玩家确认页面没死、不再乱点黑屏。
        var xxLoading = document.getElementById("loading");
        var xxMsg = document.getElementById("ld_msg");
        var xxStageMsg = "加载中…";
        var xxLoadStart = performance.now();
        function xx_paint_stage() {
            if (xxMsg && xxLoading && !xxLoading.classList.contains("hidden")) {
                var s = Math.round((performance.now() - xxLoadStart) / 1000);
                xxMsg.textContent =
                    "凡人修仙 v" + GAME_VER + " · " + xxStageMsg + "（已等待 " + s + " 秒）";
            }
        }
        function xx_stage(msg) {
            xxStageMsg = msg;
            xx_paint_stage();
        }
        setInterval(xx_paint_stage, 1000);
        // 诊断区盖上版本号：真机反馈时一眼确认跑的是哪版（防"上传了但设备吃旧缓存"）
        errlog.textContent = "v" + GAME_VER + (errlog.textContent ? "\n" + errlog.textContent : "");
        // v1.9.17 日志自动淡化 + 关键行唤醒（实现见 CSS .xx-dim/.xx-alert）
        (function () {
            var el = errlog;
            var armed = false;
            setInterval(function () {
                if (armed) return;
                if (performance.now() > 30000 && window.wasm_exports) {
                    armed = true;
                    el.classList.add("xx-dim");
                }
            }, 2000);
            // 新日志若是关键行（错误/panic/重载）恢复全显，普通诊断不吵人
            var orig = el.textContent;
            setInterval(function () {
                if (el.textContent !== orig) {
                    orig = el.textContent;
                    if (/\[ERR\]|\[PANIC\]|重载|WARN/.test(el.textContent.slice(-200))) {
                        el.classList.remove("xx-dim");
                        el.classList.add("xx-alert");
                    }
                }
            }, 500);
        })();
        // v1.9.13 死因穿越：上次会话若被自愈看门狗重载，这里把死因快照打出来
        // （一张截图即可读到"死于第几帧/触达几次/消费几次"），随后清除。
        try {
            var __post = sessionStorage.getItem("xxpost");
            if (__post) {
                sessionStorage.removeItem("xxpost");
                var __pj = JSON.parse(__post);
                errlog.textContent += "[上次] " + (__pj.reason === "manual" ? "手动" : "自动") +
                    "重载：循环停于第" + __pj.frame + "帧 触达" + __pj.touches +
                    " 已收" + __pj.clicks + " st" + __pj.st + "\n";
            }
        } catch (e) {}

        // v1.9.7：canvas 清零式重同步（resyncBacking）已整体移除——真机 X5 上
        // canvas.width=0 会杀死 WebGL 帧回调（v1.9.6 真机 F:28 冻帧实锤：启动后
        // ~2 秒恰好是就绪钩子的 +2s resync 时刻，之后 F 永不增长），本地 Chromium
        // 却无恙（历轮本地测试全部测不出）。screen_size 同步由下方 xxSyncScreen
        // （直调 wasm_exports.resize）独立承担，不再需要任何 canvas 手术。

        // v1.9.4 直通同步：不再绕 bundle 的 resize()（其"值未变不通知"的短路在
        // 真机上被证实仍会漏），直接调 wasm_exports.resize(backing)——miniquad
        // 的 resize 导入就是 wasm 侧 screen_size 的唯一写入者，直调即强同步。
        // 触摸处理器在每次映射前也会调用：保证输入瞬间 screen_size == canvas。
        var __xxSentW = -1, __xxSentH = -1;
        window.xxSyncScreen = function () {
            if (!window.wasm_exports) return false;
            var cv = document.getElementById("glcanvas");
            var w = cv.width | 0, h = cv.height | 0;
            if (w === __xxSentW && h === __xxSentH) return false;
            __xxSentW = w;
            __xxSentH = h;
            try { window.wasm_exports.resize(w, h); } catch (e) { /* 实例化竞态 */ }
            return true;
        };

        // v1.9.4 左下角调试面板：真机一张截图即可拿到全部坐标系关键数字
        // （视口/dpr/backing/CSS/已发送 screen/fit/最近一次触摸的映射）。
        var __xxDbg = document.createElement("div");
        __xxDbg.style.cssText =
            "position:fixed;left:2px;bottom:2px;z-index:98;color:#7f7;" +
            "background:rgba(0,0,0,.55);font:10px/1.35 Consolas,monospace;" +
            "white-space:pre;pointer-events:none;padding:2px;";
        document.body.appendChild(__xxDbg);
        var __xxLastTouch = null;
        window.xxPaintDbg = function () {
            if (!window.wasm_exports) return;
            var cv = document.getElementById("glcanvas");
            var rect = cv.getBoundingClientRect();
            var env = window.importObject && window.importObject.env ? window.importObject.env : null;
            var lines = [
                "v" + GAME_VER + " ss" + XX_SS,
                "inner " + window.innerWidth + "x" + window.innerHeight +
                    " dpr " + (window.devicePixelRatio || 1).toFixed(2) +
                    " hdp=" + (window.high_dpi ? 1 : 0) +
                    " env=" + (env ? env.dpi_scale().toFixed(2) : "?"),
                "cv " + cv.width + "x" + cv.height +
                    " css " + Math.round(rect.width) + "x" + Math.round(rect.height) +
                    " sent " + __xxSentW + "x" + __xxSentH,
                "fit " + Math.min(cv.width / 1280, cv.height / 720).toFixed(3) +
                    // fr：距上次成功 wasm 帧的毫秒数（>200 即帧停摆，混沌仿真/真机判读用）
                    // t/k：JS 触摸组数 / Rust 已消费点击数（t 涨 k 不涨 = 输入桥断开）
                    " fr" + Math.round(performance.now() - (window.__xxFrameAt || 0)) +
                    "ms st" + window.__xxState +
                    " t" + (window.__xxTouches || 0) + " k" + (window.__xxClicksSeen || 0)
            ];
            if (__xxLastTouch) {
                lines.push("t c(" + __xxLastTouch[0] + "," + __xxLastTouch[1] + ")->m(" +
                    __xxLastTouch[2] + "," + __xxLastTouch[3] + ")");
            }
            __xxDbg.textContent = lines.join("\n");
        };
        setInterval(window.xxPaintDbg, 1000);
        window.onerror = function (msg, src, line, col, err) {
            errlog.textContent += "ERR: " + msg + " @" + (src || "") + ":" + line + "\n" +
                (err && err.stack ? err.stack : "") + "\n";
        };
        // Promise 链上的失败（如 wasm 实例化被拒）不经过 window.onerror，单独兜底
        window.addEventListener("unhandledrejection", function (ev) {
            var r = ev && ev.reason;
            errlog.textContent += "ERR(promise): " + (r && (r.message || r) || "unknown") + "\n" +
                (r && r.stack ? r.stack + "\n" : "");
        });
        var isFile = location.protocol === "file:";
        if (isFile) {
            // 浏览器禁止 file:// 加载 WASM，直接给出提示而不是一片空白
            errlog.textContent = "无法以 file:// 方式运行游戏（浏览器禁止 file:// 加载 WASM）。\n\n" +
                "【手动】先启动本地服务器：\n" +
                "  cd E:\\Rust_Game\\Daoism_Game\\web\n" +
                "  python -m http.server 8124\n\n" +
                "然后浏览器打开：http://localhost:8124/";
            document.getElementById("loading").textContent = "请通过 http://localhost:8124/ 访问";
        } else {
            // 点击/触摸时聚焦 canvas，保证进入游戏后键盘输入立即可用
            var glcanvas = document.getElementById("glcanvas");
            ["pointerdown", "touchstart", "click"].forEach(function (ev) {
                glcanvas.addEventListener(ev, function () { glcanvas.focus(); }, { passive: true });
            });
            // WebGL 上下文丢失处理：miniquad 不注册任何 handler，手机 WebView
            // 切后台被系统回收 GL 上下文后恢复时纹理全空 → 整个 UI 变黑。
            // contextlost 必须 preventDefault 才允许恢复；restored 后 miniquad 不会
            // 重建任何纹理，唯一可靠做法是整页重载（存档已在关键操作时落盘，进度不丢）
            glcanvas.addEventListener("webglcontextlost", function (e) {
                e.preventDefault();
                errlog.textContent += "WARN: WebGL 上下文丢失（切后台/显存回收），等待恢复…\n";
            }, false);
            glcanvas.addEventListener("webglcontextrestored", function () {
                location.reload();
            }, false);
            // v1.8.4 关键修复：把帧循环的 requestAnimationFrame 底座换成
            // setTimeout——TapTap 容器会把页面标记为不可见从而挂起 rAF，
            // 主循环停摆后画面定格、所有点击无人消费（红米 K80 Pro 实测
            // "点按钮没反应"的最终根因）。setTimeout 前台 ~16ms ≈ 60fps，
            // 且不受可见性挂起影响。单帧源、零逻辑改动。
            (function () {
                // v1.8.8 帧驱动 v2：多时钟共存 + 全局帧门闩 + 停摆自愈。
                // 背景：TapTap 容器对不可见页面会挂起 rAF、钳制定时器、静默禁用
                // blob Worker——任何单一时钟都可能停摆（实测：启动 53fps 后定格）。
                // 现在所有时钟同时启动，帧率由全局门闩统一限制（~60fps）：
                // 任一时钟被挂起，其余自动补位；全部停摆超过 800ms 时重建
                // MessageChannel 通道自愈。输入处理见 pointer/touch/mouse 接管段。
                var el = document.getElementById("errlog");
                var cb = null;
                var frames = 0;
                var last = 0;
                var lastFrameAt = 0;
                var rebuilt = 0;
                window.requestAnimationFrame = function (f) {
                    cb = f;
                    return 0;
                };
                window.cancelAnimationFrame = function () {};
                var tick = function (now) {
                    if (window.__xxClockGate) return; // ?chaos=1 仿真：人为冻结全部时钟
                    if (now - last < 14) return; // 全局帧门闩：~60fps 上限，多时钟共存不叠加
                    last = now;
                    lastFrameAt = now;
                    frames += 1;
                    var f = cb;
                    // v1.9.7 cb 自愈：帧回调丢失（真机 canvas 手术/GL 异常后 bundle
                    // 不再重挂 animation）时从 bundle 顶层函数复活，时钟继续出帧。
                    // 必须等 wasm 就绪——过早调 animation 会在 bundle 赋值
                    // wasm_exports 之前抛 TypeError（本地实测启动噪音）。
                    if (!f && window.animation && window.wasm_exports) {
                        f = window.animation;
                        cb = f;
                    }
                    // v1.9.10 最后生路：bundle 顶层 animation 也不可用（加载残缺/
                    // 极端容器）时直调 wasm frame 导出——绕开一切 JS 中间层，只要
                    // wasm 实例活着帧就不断。
                    if (!f && window.wasm_exports) {
                        f = window.wasm_exports.frame;
                        cb = f;
                    }
                    if (!f) return;
                    try {
                        f(now);
                        // v1.9.7 只在真正执行了 wasm 帧后才刷新帧时刻——回调死亡时
                        // tick 空转不再冒充"刚出过帧"，触摸帧泵保持有效。
                        window.__xxFrameAt = now;
                        // v1.9.16 __xxTickAt：仅 tick（真 rAF/时钟链）出帧时刷新。
                        // 车夫让路判据专用——触摸同步帧也刷 __xxFrameAt（v1.9.15
                        // 真机实锤：频繁触摸会把车夫的 200ms 让路窗口无限重置，
                        // 车夫从不补帧，帧率只剩触摸驱动 ~2fps，画面一卡一卡）。
                        window.__xxTickAt = now;
                    } catch (err) {
                        if (frames < 100 || frames % 120 === 0) { // 限流防刷屏
                            el.textContent += "[ERR] frame " + String(err).slice(0, 120);
                        }
                        cb = null;
                    }
                };
                // 时钟 1：MessageChannel 自循环（hidden 不节流）
                var chan = null;
                function buildChan() {
                    try {
                        var ch = new MessageChannel();
                        ch.port1.onmessage = function () {
                            tick(performance.now());
                            // 4ms 心跳投递：port 消息不受可见性节流，帧率由 tick 门闩限制
                            if (chan === ch) {
                                setTimeout(function () {
                                    if (chan === ch) ch.port2.postMessage(0);
                                }, 4);
                            }
                        };
                        ch.port2.postMessage(0);
                        chan = ch;
                        return true;
                    } catch (e) {
                        return false;
                    }
                }
                // 时钟 2：Worker 定时器（worker 端不受可见性节流）
                var worker = null;
                function buildWorker() {
                    try {
                        var wcode = "setInterval(function(){postMessage(0)},16);";
                        worker = new Worker(
                            URL.createObjectURL(new Blob([wcode], { type: "application/javascript" })),
                        );
                        worker.onmessage = function () { tick(performance.now()); };
                        return true;
                    } catch (e) {
                        return false;
                    }
                }
                // 时钟 3：setTimeout 循环（兜底，hidden 被钳 1fps 但绝不冻结）
                setInterval(function () { tick(performance.now()); }, 16);
                buildChan();
                buildWorker();
                // 停摆看门狗：>800ms 无帧 → 重建 MessageChannel 并重发 rAF 循环
                setInterval(function () {
                    if (!window.wasm_exports) return;
                    var since = performance.now() - lastFrameAt;
                    if (since > 800 && rebuilt < 6) {
                        rebuilt += 1;
                        if (chan) {
                            chan.port1.onmessage = null;
                            chan = null;
                        }
                        buildChan();
                        if (el && rebuilt === 1) {
                            el.textContent += "[诊断] 帧驱动停摆，已自动重建时钟";
                        }
                    }
                }, 500);
            })();
            xx_stage("编译游戏引擎…");
            try {
                load("xiuxian.wasm?v=" + GAME_VER);
            } catch (e) {
                errlog.textContent += "load() threw: " + (e && e.message ? e.message : e) + "\n";
            }
            // 加载分阶段提示：游戏包含内嵌立绘、体积较大，手机上首次下载+编译
            // 需要 10~40 秒——中途给状态文字，避免玩家以为卡死
            var stageTimers = [
                [12, "游戏包较大，首次加载约需 10~40 秒，请稍候…"],
                [30, "仍在编译游戏引擎（低端设备会慢一些），请继续等待…"],
            ];
            stageTimers.forEach(function (st) {
                setTimeout(function () {
                    if (!window.wasm_exports) xx_stage(st[1]);
                }, st[0] * 1000);
            });
            // 加载看门狗：45 秒仍未就绪（无 WebGL/无 WASM/网络失败都会卡在
            // 白屏加载层）才把加载层变成可点的重试按钮。
            // v1.8.2 前此值为 12 秒且 window.wasm_exports 恒为 undefined，
            // 手机上必触发"加载失败→点击→reload"无限循环（玩家实测反馈）。
            setTimeout(function () {
                if (!window.wasm_exports && xxLoading &&
                    !xxLoading.classList.contains("hidden")) {
                    xx_stage("加载失败或超时，点击此处重试");
                    xxLoading.style.pointerEvents = "auto";
                    xxLoading.onclick = function () { location.reload(); };
                }
            }, 45000);
        }
        // 首帧检测：wasm 导出就绪（on_init 里 window.wasm_exports 被赋值）即隐藏
        // 加载覆盖层。用 setInterval 而非 rAF：页面不可见时 rAF 会被挂起。
        // v1.8.2：删除旧的"8 秒无条件隐藏"兜底——它会在 12MB 包还在编译时
        // 提前撤掉加载层，玩家面对黑屏点击（点按钮没反应的第一诱因）。
        (function pollFirstFrame() {
            var loading = document.getElementById("loading");
            var synced = false;
            function onReady() {
                if (synced) return;
                // v1.9.21 双加载藏进加载层（用户定案体验流：加载界面→开始选项
                // →进入游戏→流畅，重载界面绝不可见）：wasm 就绪时若本次加载
                // 耗时 >1.5s（慢加载=首开高钳制风险）且 URL 无 r2 标记——
                // **不隐藏加载条、立即重载**，重载全程被加载界面盖住；r2 页面
                // 缓存热秒开，加载条一闪即过。快加载直接进入（无白闪无重载）。
                if (performance.now() - xxLoadStart > 1500 && !/[?&]r2=1/.test(location.search)) {
                    try { document.getElementById("ld_msg").textContent =
                        "凡人修仙 v" + GAME_VER + " · 正在进入（已等待 " +
                        Math.round((performance.now() - xxLoadStart) / 1000) + " 秒）"; } catch (e) {}
                    location.href = location.href.split("#")[0] +
                        (location.search ? "&" : "?") + "r2=1";
                    return; // 加载层保持显示，导航即将发生
                }
                synced = true;
                loading.classList.add("hidden");
                // v1.9.7：只做 screen_size 直通同步。canvas 清零式 resyncBacking
                // 已删除——它在 X5 上杀死帧回调（v1.9.6 真机 F:28 冻帧实锤）。
                window.xxSyncScreen();
                setTimeout(window.xxSyncScreen, 2000);
            }
            var timer = setInterval(function () {
                if (window.wasm_exports) onReady();
            }, 250);
        })();
        // v1.8.2 指针自愈（两条）：
        // 1) dpr/backing 失步自愈——miniquad 只在 resize 事件重算 backing，若
        //    devicePixelRatio 在实例化前后发生变化（浏览器缩放/分屏/机型切换），
        //    backing 与指针缩放系数脱钩，所有点击错位。检测到失步就合成一次
        //    resize（bundle 的 window.onresize 会重算双方），免重载自愈。
        // 2) 竖屏旋转点击映射——orientFix 把整页 rotate(90deg) 后，视觉坐标系
        //    与页面坐标系相差一个旋转，miniquad 的线性换算全部错位（竖持手机
        //    点按钮没反应的第二诱因）。在 canvas 捕获阶段拦截鼠标/触摸事件，
        //    旋转时先 stopImmediatePropagation 再派发"换算后的同型事件"
        //    （clientX/clientY 对调并翻转），bundle 原处理逻辑零改动。
        (function pointerSetup() {
            var canvas = document.getElementById("glcanvas");
            // v1.8.3 指针映射彻底接管：不再依赖 miniquad 基于 devicePixelRatio 的
            // 线性换算（TapTap 容器缩放/顶部横幅占位/dpr 漂移都会让它失准，手机
            // 实测点按钮没反应），改用 getBoundingClientRect 实测比例直接驱动
            // wasm 导出。竖屏旋转（orientFix 的 90° 页面旋转）用归一坐标旋转逆
            // 映射：(nx,ny) → (ny, 1-nx)。
            var rotationAware = function (e) {
                var r = canvas.getBoundingClientRect();
                if (r.width < 1 || r.height < 1) return null;
                var nx = (e.clientX - r.left) / r.width;
                var ny = (e.clientY - r.top) / r.height;
                if (!(nx >= 0 && nx <= 1 && ny >= 0 && ny <= 1)) return null;
                var portrait = window.innerHeight > window.innerWidth;
                if (!portrait) return { x: nx * canvas.width, y: ny * canvas.height };
                return { x: ny * canvas.width, y: (1 - nx) * canvas.height };
            };
            var btn = function (b) { return b === 1 ? 2 : b === 2 ? 1 : b | 0; };
            var takeOver = function () {
                if (!window.wasm_exports || canvas.__ptr_taken) return;
                canvas.__ptr_taken = true;
                var E = window.wasm_exports;
                // v1.9.10 导出调用可见化：wasm 陷阱（panic/互斥锁中毒）会让后续每次
                // 导出调用抛 RuntimeError——旧代码要么让异常裸奔（onerror 记录淹没在
                // 堆栈里）、要么静默吞掉，真机十轮排查都拿不到失败原文。现在首例
                // 异常按导出名记入 errlog（[ERR] mouse_move: unreachable …），之后
                // 吞掉继续跑：单点坏死不再阻断同组后续调用与触摸帧泵。
                var __xE = {};
                var xxSafe = function (name) {
                    var fn = E[name];
                    return function () {
                        try {
                            fn.apply(null, arguments);
                        } catch (err) {
                            if (!__xE[name]) {
                                __xE[name] = 1;
                                var el2 = document.getElementById("errlog");
                                if (el2)
                                    el2.textContent +=
                                        "[ERR] " + name + ": " +
                                        String(err && err.message ? err.message : err).slice(0, 90) + "\n";
                            }
                        }
                    };
                };
                var S_move = xxSafe("mouse_move");
                var S_down = xxSafe("mouse_down");
                var S_up = xxSafe("mouse_up");
                var S_touch = xxSafe("touch");
                var S_frame = xxSafe("frame");
                canvas.onmousemove = function (e) {
                    var p = rotationAware(e);
                    if (p) S_move(p.x | 0, p.y | 0);
                };
                canvas.onmousedown = function (e) {
                    var p = rotationAware(e);
                    if (p) S_down(p.x | 0, p.y | 0, btn(e.button));
                };
                canvas.onmouseup = function (e) {
                    var p = rotationAware(e);
                    if (p) S_up(p.x | 0, p.y | 0, btn(e.button));
                };
                // 触摸：容器内核用 addEventListener（无法覆盖属性），在 window
                // 捕获阶段拦下并直接调 wasm_exports.touch，坐标实测换算。
                // v1.9.3 触控加固：真机容器（TapTap/X5）可能把触摸派发给 body
                // 或注入的包装层而非 canvas——旧代码 e.target!==canvas 直接丢弃，
                // 表现为"手机上按钮全部失效、桌面鼠标路径正常"。改为 window 捕获
                // （先于 document 触发，防页面级拦截）+ 只按坐标过滤：触点落在
                // 画布区域内一律接管；区域外（如诊断日志）放行不 preventDefault。
                var el = document.getElementById("errlog");
                var seenKinds = {};
                var mark = function (kind) {
                    if (!seenKinds[kind]) {
                        seenKinds[kind] = true;
                        if (el) el.textContent += "[诊断] 已收到输入事件（" + kind + "）\n";
                    }
                };
                var TOUCH = { touchstart: 10, touchmove: 11, touchend: 12, touchcancel: 13 };
                Object.keys(TOUCH).forEach(function (type) {
                    window.addEventListener(
                        type,
                        function (e) {
                            var pts = [];
                            for (var i = 0; i < e.changedTouches.length; i++) {
                                var p = rotationAware(e.changedTouches[i]);
                                if (p) pts.push([e.changedTouches[i], p]);
                            }
                            if (!pts.length) return; // 触点不在画布内：放行页面默认行为
                            e.preventDefault();
                            e.stopImmediatePropagation();
                            mark("touch");
                            window.__xxTouches = (window.__xxTouches || 0) + 1;
                            window.xxSyncScreen(); // 触摸瞬间强同步 screen_size == canvas
                            for (var j = 0; j < pts.length; j++) {
                                var t = pts[j][0];
                                var p2 = pts[j][1];
                                __xxLastTouch = [Math.round(t.clientX), Math.round(t.clientY), p2.x | 0, p2.y | 0];
                                window.xxPaintDbg();
                                // v1.9.8 恢复 E.touch：cam::mouse()（wasm）改为优先读
                                // macroquad 的 touches 表（原样存储、零 dpi 参与），
                                // 必须把触摸位置喂进去；E.mouse_* 保留作点击判定。
                                S_touch(TOUCH[type], t.identifier, p2.x, p2.y);
                                // 同步鼠标状态：游戏按钮判定走鼠标通路
                                S_move(p2.x | 0, p2.y | 0);
                                if (type === "touchstart") S_down(p2.x | 0, p2.y | 0, 0);
                                if (type === "touchend" || type === "touchcancel") S_up(p2.x | 0, p2.y | 0, 0);
                            }
                            // v1.9.19 触摸同步 burst（中期钳制点击卡 5 秒的根治）：
                            // 真机视频定案：容器钳制不止首开——游戏中期（页面 idle
                            // 一段时间后）再次钳制，"重载后必好"只在首开成立；
                            // 点坊市卡 5 秒 = 同步 3 发推不动死透的 future、只能
                            // 等看门狗 3 轮冲击后重载。burst 12 发做当拍唤醒。
                            // v1.9.19 MuMu 实测修正：真实安卓 WebView 的手势回调
                            // 栈内直调 frame() 推不动 future（本地 Chromium 有效、
                            // 真机/模拟器无效——环境差异），必须 setTimeout(0)
                            // 脱离手势栈再 burst（宏任务在钳制态可跑：v1.9.14
                            // 真机 400ms 唤醒链实证）。两层都发：同步层兜本地，
                            // 宏任务层保真机。tick 活着时（<60ms）零开销跳过。
                            if (performance.now() - (window.__xxFrameAt || 0) > 60 && window.wasm_exports) {
                                try {
                                    for (var fi = 0; fi < 12; fi++) window.wasm_exports.frame();
                                    window.__xxFrameAt = performance.now();
                                } catch (err) {}
                                setTimeout(function () {
                                    try {
                                        for (var fj = 0; fj < 12; fj++) window.wasm_exports.frame();
                                        window.__xxFrameAt = performance.now();
                                        window.__xxRevivedAt = performance.now(); // burst 即复活：大旗静默
                                        window.__xxWedged = 1; // 低帧率实锤，开启评估
                                    } catch (e2) {}
                                }, 0);
                            }
                            // v1.9.17 wedge 低帧率评估：触摸是 wedge 态唯一可靠
                            // 事件源，评估挂在这里最可靠（interval 可能同样被钳）
                            if (window.__xxWedgeCheck) window.__xxWedgeCheck();
                            // v1.9.6 帧泵（降级为冗余路径）：同步帧已被上面抢先，
                            // 正常不触发；万一同步帧抛异常，这里还有一次机会。
                            // ?wedge=1 仿真禁用（真机 timer 死透时此泵同样不工作，
                            // 保证快速唤醒链在仿真里与真机同路径触发）。
                            if (performance.now() - (window.__xxFrameAt || 0) > 120 && !window.__xxSimWedge) {
                                setTimeout(function () {
                                    window.__xxFrameAt = performance.now();
                                    S_frame();
                                }, 0);
                            }
                            // v1.9.14 触摸驱动快速唤醒：v1.9.13 真机实证首触杀死
                            // 主循环、且连续 frame() 叩击可唤醒（第 4 轮 +30 帧）。
                            // 不再等 2.5s 看门狗——touchend 后 400ms 检查帧号是否
                            // 前进（对比 touchstart 时刻），未前进立即进入连续叩击，
                            // 唤醒后把用户这一击原坐标重放（第一击直接生效）。
                            // v1.9.15 触摸同步帧已当拍消费点击（k 判据会先 return），
                            // 此钩子降级为同步帧异常时的兜底；车夫接管后彻底让路。
                            if (type === "touchstart" && window.__xxAlive && !window.__xxWaking && !window.__xxCoach) {
                                var n0 = window.__xxAlive.n;
                                var k0 = window.__xxClicksSeen || 0;
                                var lt = __xxLastTouch;
                                setTimeout(function () {
                                    var a = window.__xxAlive;
                                    if (!a || window.__xxWaking) return;
                                    if ((window.__xxClicksSeen || 0) !== k0) return; // 已消费，循环活着
                                    if (a.n > n0) return; // 帧号在涨，循环活着
                                    window.__xxWakeReplay(lt);
                                }, 400);
                            }
                        },
                        true,
                    );
                });
                // pointer 兜底：个别内核只派发 pointer 事件不发 compat mouse
                var POINTER = { pointerdown: 0, pointermove: -1, pointerup: 0 };
                Object.keys(POINTER).forEach(function (type) {
                    canvas.addEventListener(type, function (e) {
                        if (e.pointerType === "touch") return; // 触摸走上面通道
                        mark("pointer");
                        var p = rotationAware(e);
                        if (!p) return;
                        S_move(p.x | 0, p.y | 0);
                        if (type === "pointermove") return;
                        if (type === "pointerdown") S_down(p.x | 0, p.y | 0, 0);
                        else S_up(p.x | 0, p.y | 0, 0);
                    });
                });
                // 兼容 mouse 首达诊断
                canvas.onmousemove = (function (orig) {
                    return function (e) {
                        mark("mouse");
                        var p = rotationAware(e);
                        if (p) S_move(p.x | 0, p.y | 0);
                    };
                })();
                canvas.onmousedown = function (e) {
                    var p = rotationAware(e);
                    if (p) S_down(p.x | 0, p.y | 0, btn(e.button));
                };
                canvas.onmouseup = function (e) {
                    var p = rotationAware(e);
                    if (p) S_up(p.x | 0, p.y | 0, btn(e.button));
                };
            };
            setInterval(takeOver, 500);
            // v1.9.13 自愈看门狗（上接 v1.9.12 大旗实测数据：真机首次真触摸
            // 杀死 async 主循环——F 冻在点击当帧（19/100/101 三次实测）、frame()
            // 照常返回、fr 恒新鲜、无任何 [PANIC]/[ERR]；用户实证重载后必好：
            // 缓存热秒开、循环存活）。两级处置：
            //   ①循环心跳看门狗（主）：Rust 每 30 帧上报 xx_alive；>2.5s 无心跳
            //     且 wasm 已就绪 = 循环死亡 → 立即静默自动重载（每会话一次），
            //     死因快照存 sessionStorage 穿越到重载后的页面（errlog 首行可读）。
            //   ②触摸桥看门狗（备）：心跳预算用尽或恢复后再次断桥时弹大旗 +
            //     立即重载按钮（用户已实证手动重载有效）。
            (function xxWatch() {
                var lastSeen = -1, starved = 0, banner = null;
                function xxPostmortem(reason) {
                    try {
                        sessionStorage.setItem("xxpost", JSON.stringify({
                            v: window.GAME_VER, reason: reason,
                            frame: window.__xxAlive ? window.__xxAlive.n : -1,
                            touches: window.__xxTouches || 0,
                            clicks: window.__xxClicksSeen || 0,
                            st: window.__xxState
                        }));
                    } catch (e) {}
                }
                // v1.9.17 重载预算：URL 参数为主（?xxrl=1）+ sessionStorage 为辅。
                // 实测夸克（MuMu）的 sessionStorage 在 location.reload() 后丢失
                // ——预算标记一丢，wedge 页重载后又把自己冻结→循环死→又重载，
                // 无限重载循环（用户看到"很卡进不去"的就是这个循环）。URL 参数
                // 跨重载绝不丢失，任何内核都可靠。
                // 挂 window：xxSim（另一 IIFE）的 wedge 分支也要查预算。
                // v1.9.19 预算 60 秒化：真机视频定案"重载后必好"只在首开成立
                // ——游戏中期（页面 idle 后）容器会再次钳制，"每会话一次"预算
                // 导致第二次卡顿只能靠大旗兜底。改为 60 秒窗口（URL 时间戳
                // xxrlt 为主 + sessionStorage 兜底）：既允许周期性换血，又防
                // 循环重载。
                window.xxRlUsed = function () {
                    // 三层：URL 参数 / URL hash / sessionStorage（夸克实测会清
                    // sessionStorage、个别内核规范化 query——hash 最稳）
                    var m = location.search.match(/[?&]xxrlt=(\d+)/) || location.hash.match(/xxrlt=(\d+)/);
                    if (m && Date.now() - parseInt(m[1], 10) < 60000) return true;
                    try {
                        var t = sessionStorage.getItem("xxrlAt");
                        if (t && Date.now() - parseInt(t, 10) < 60000) return true;
                    } catch (e) {}
                    return false;
                };
                function xxReload(reason) {
                    xxPostmortem(reason);
                    try { sessionStorage.setItem("xxrlAt", String(Date.now())); } catch (e) {}
                    var t = Date.now();
                    var href = location.href.replace(/[?&]xxrlt=\d+/, "").replace(/#.*$/, "");
                    href += (href.indexOf("?") >= 0 ? "&" : "?") + "xxrlt=" + t + "#r" + t;
                    location.href = href;
                }
                // v1.9.20 首开双加载 → v1.9.21 移入加载层隐藏逻辑（pollFirstFrame
                // 的 onReady 内决策）：v1.9.20 的独立 interval 与加载层隐藏存在
                // 竞争（用户可能先看到菜单闪现再重载白屏）；v1.9.21 保证重载
                // 全程被加载条盖住，用户体验流=加载界面→开始选项→进入游戏→流畅。
                // 此处仅保留 r2 页面标记（真机截图确认双加载已发生）。
                if (/[?&]r2=1/.test(location.search)) {
                    var iv3 = setInterval(function () {
                        if (!window.wasm_exports) return;
                        clearInterval(iv3);
                        try { errlog.textContent += "[诊断] r2 双加载就绪\n"; } catch (e) {}
                    }, 200);
                }
                // v1.9.18 首开卡顿根治 → v1.9.19 滚动采样版：真机实锤"首开被
                // 容器钳制、重载后缓存热秒开循环存活"。旧版固定窗口（2s 基准
                // +3s 增量）有盲区——钳制若落在窗口关闭后才发生（本地复现实锤）
                // 增量混入正常帧而漏检。改为每秒采样帧号、盯最近 2 秒增量
                // （<25 帧 = <12.5fps，慢设备 20fps 不误触发），持续盯 35 秒
                // 冷启期，任一 2 秒窗口低帧即热身重载。timer 若也被钳则本
                // 检测静默（不误重载），兜底交给触摸 burst+评估链。
                // v1.9.22 游戏内即停：进游戏后（st>0）冷启检测终止——真机实锤
                // "菜单→游戏内→重载→回菜单"的断流体验来自游戏内预防性重载
                // （丢当前进度踢回菜单）；游戏内低帧率由触摸 burst 当拍自救，
                // 预防性重载只允许发生在菜单阶段（无进度可丢）。
                (function () {
                    var samples = [];
                    var iv = setInterval(function () {
                        if (!window.wasm_exports || !window.__xxAlive) return;
                        if (xxRlUsed()) { clearInterval(iv); return; }
                        if ((window.__xxState || 0) > 0) { clearInterval(iv); return; } // 已进游戏：不打扰
                        samples.push(window.__xxAlive.n);
                        if (samples.length > 6) samples.shift();
                        if (samples.length >= 3) {
                            var inc2 = samples[samples.length - 1] - samples[samples.length - 3];
                            if (inc2 < 25) {
                                clearInterval(iv);
                                try { errlog.textContent += "[诊断] 首开卡顿（2 秒 +" + inc2 + " 帧），热身重载换满帧\n"; } catch (e) {}
                                xxReload("coldstart");
                            }
                        }
                    }, 1000);
                    setTimeout(function () { clearInterval(iv); }, 35000);
                })();
                // v1.9.17 wedge 低帧率自动重载：MuMu+夸克模拟器实证游戏代码在
                // 正常浏览器一击即进+满帧零噪音——"很卡"的唯一来源是 X5 容器
                // 钳死一切定时器（v1.9.16 真机日志实锤：车夫 setInterval(1000)
                // 也被钳，stale>6000 仍触发），wedge 态唯一可靠事件源=触摸。
                // 既然用户三次实测"重载后必好"（缓存热秒开+循环存活），就把
                // 重载从兜底升级为主路径：快速唤醒成功（=wedge 实锤）后，8 秒
                // 窗口平均帧率 <8fps → 自动重载换满帧。重载预算每会话一次
                // （xxrl），用完后车夫+触摸同步帧继续保底可玩。
                window.__xxFpsWin = null;
                window.__xxWedgeCheck = function () {
                    // gate 不豁免：chaos 仿真下触摸同步帧让点击被消费、快速唤醒
                    // 不触发、__xxWedged 不置位，本函数自然静默（标记即开关）
                    if (!window.__xxWedged) return;
                    // v1.9.22 游戏内不评估：预防性重载会丢当前进度踢回菜单（真机
                    // 实锤"菜单→游戏内→重载→回菜单"断流）；游戏内低帧率由触摸
                    // burst 当拍自救，真死由主看门狗 xxReload("loop") 处理
                    if ((window.__xxState || 0) > 0) return;
                    var now = performance.now();
                    var a = window.__xxAlive;
                    if (!a) return;
                    if (!window.__xxFpsWin) { window.__xxFpsWin = { t: now, n: a.n }; return; }
                    var w = window.__xxFpsWin;
                    if (now - w.t < 8000) return; // 8 秒评估窗口
                    var fps = ((a.n - w.n) * 1000) / (now - w.t);
                    window.__xxFpsWin = { t: now, n: a.n };
                    try { errlog.textContent += "[诊断] wedge 评估：" + fps.toFixed(1) + "fps\n"; } catch (e0) {}
                    if (fps >= 15) return; // <15fps 体验即"卡"，重载换满帧
                    if (xxRlUsed()) return; // 重载预算已用：车夫保底，不再折腾
                    try { errlog.textContent += "[诊断] wedge 态帧率仅 " + fps.toFixed(1) + "fps，自动重载恢复满帧\n"; } catch (e3) {}
                    xxReload("wedge-lowfps");
                };
                // ① 循环心跳：首见 xx_alive 后武装；>2.5s 停跳即死亡。
                // 死亡处置三步（复现→定位→治疗，全部在设备现场执行）：
                //   step1 复活冲击：连调 wasm frame 导出——若 async 主循环
                //        只是"调度断了"（future 还活着），帧号立即恢复，零感知
                //        复活（无重载无闪烁）；无效则 future 已 wedged。
                //   step2 重载自愈：死因快照存 sessionStorage → location.reload()
                //        （用户实证重载后必好：缓存热、循环存活）。
                //   step3 大旗兜底：重载预算用尽后仍死 → 弹旗 + 手动重载按钮。
                // v1.9.14 快速唤醒（xxWakeReplay）：触摸后 400ms 帧号未动即启动
                // 连续叩击（不等 2.5s 看门狗），唤醒后按 __xxLastTouch 原坐标重放
                // 完整点击（touch+mouse_down+frame+mouse_up）——真机"首击杀死循环"
                // 的场景下用户第一击直接生效，无需再点。唤醒期间置 __xxWaking
                // 防重入；复活成功后 4 秒内大旗静默（v1.9.13 真机误弹实锤修复）。
                window.__xxRevivedAt = 0;
                // v1.9.15 帧车夫：wedge 态的持续供血。真机实锤 wedge 后三时钟
                // 全被容器钳死（tick ~1fps），rAF 底座名存实亡，但 ~1s 级
                // setInterval 照常跑（主看门狗日志实证）、触摸事件照常派发。
                // 唤醒成功不再"打一轮就撤"（v1.9.14 实测撤了又死、每次点击
                // 都要再等 400ms 重唤醒）——车夫以 1s 拍持续补帧：每拍若 tick
                // 没在出帧（__xxFrameAt >200ms 未刷新）则直调 8 发 frame()，
                // 维持 ~8fps 底血；tick 恢复时逐拍让路，永不双驱。触摸路径
                // 的即时性由"触摸同步帧"负责，车夫只保无输入时的画面推进。
                window.__xxCoach = 0;
                window.__xxCoachStart = function (why) {
                    if (window.__xxCoach || !window.wasm_exports) return;
                    // ?wedge=1 仿真：车夫 interval 模拟"同被容器钳死"——标志置位
                    // （对齐真机"接管日志打了、interval 死了"的状态）但循环不跑
                    if (window.__xxSimWedge) {
                        window.__xxCoach = 1;
                        try { errlog.textContent += "[SIM] 车夫启动被仿真拒绝（interval 被钳）\n"; } catch (e) {}
                        return;
                    }
                    window.__xxCoach = 1;
                    try { errlog.textContent += "[诊断] 帧车夫接管（" + why + "）：1s 补帧维持\n"; } catch (e) {}
                    setInterval(function () {
                        // v1.9.16 三处加固（v1.9.15 真机"一卡一卡"+大旗误弹的教训）：
                        // ①让路判据改用 __xxTickAt——只有 tick 真出帧才让路，触摸
                        //   同步帧不再屏蔽车夫（旧判据 __xxFrameAt 被触摸反复刷新，
                        //   频繁点击时车夫永远在让路，帧率只剩触摸驱动 ~2fps）；
                        // ②每拍 8→24 发：wedge 态维持 ~24fps 底血（文字游戏流畅线）；
                        // ③gate 豁免删除：车夫是供血机制不是仿真对象，冻结仿真下
                        //   也应补帧（chaos 回归断言的是点击被消费与解冻恢复，兼容）。
                        if (performance.now() - (window.__xxTickAt || 0) < 300) return; // tick 活着，让路
                        for (var i = 0; i < 24; i++) {
                            try { window.wasm_exports.frame(); } catch (e) { break; }
                        }
                        try { window.__xxFrameAt = performance.now(); } catch (e) {}
                    }, 1000);
                };
                window.__xxWakeReplay = function (lt) {
                    if (window.__xxWaking || !window.wasm_exports) return;
                    window.__xxWaking = true;
                    var E = window.wasm_exports;
                    var n0 = window.__xxAlive ? window.__xxAlive.n : -1;
                    var bursts = 0;
                    var log1 = false;
                    (function burst() {
                        for (var i = 0; i < 12; i++) {
                            try { E.frame(); } catch (e) { break; }
                        }
                        bursts += 1;
                        var a = window.__xxAlive;
                        if (a && a.n > n0) {
                            // 唤醒成功：重放用户这一击（touch + 鼠标按下 → 出帧消费 → 抬起）
                            window.__xxRevivedAt = performance.now();
                            try { errlog.textContent += "[诊断] 快速唤醒成功（第" + bursts + "轮，+" + (a.n - n0) + " 帧），重放点击\n"; } catch (e) {}
                            try {
                                if (lt) {
                                    E.touch(10, 999001, lt[2], lt[3]);
                                    E.mouse_move(lt[2] | 0, lt[3] | 0);
                                    E.mouse_down(lt[2] | 0, lt[3] | 0, 0);
                                    E.frame(); // 消费按下
                                    E.touch(12, 999001, lt[2], lt[3]);
                                    E.mouse_up(lt[2] | 0, lt[3] | 0, 0);
                                }
                            } catch (e2) {}
                            window.__xxWaking = false;
                            window.__xxWedged = 1; // v1.9.17：唤醒成功=wedge 实锤，开启低帧率重载评估
                            window.__xxCoachStart("快速唤醒后"); // 不再"唤醒即撤"，持续供血
                            return;
                        }
                        if (bursts < 12) {
                            setTimeout(burst, 220); // 连续叩击，每轮 12 发
                        } else {
                            // 12 轮（约 2.6s）未醒——交回看门狗（重载兜底）
                            try { errlog.textContent += "[诊断] 快速唤醒失败（12 轮），交由看门狗\n"; } catch (e) {}
                            window.__xxWaking = false;
                        }
                    })();
                };
                var reviveTries = 0;
                setInterval(function () {
                    if (window.__xxWaking) return; // 快速唤醒进行中，让路
                    if (window.__xxWedgeCheck) window.__xxWedgeCheck(); // v1.9.17 interval 活着时也评估
                    if (!window.wasm_exports || !window.__xxAlive) return;
                    // 车夫模式下上报间隔最坏 ~4s（1s 拍被钳 + 15 帧粒度），放宽死线防误杀
                    var staleLimit = window.__xxCoach ? 6000 : 2500;
                    var stale = performance.now() - window.__xxAlive.at;
                    if (stale > staleLimit) {
                        // step1：复活冲击（直接驱动 frame 导出 5 次）
                        var before = window.__xxAlive.n;
                        for (var i = 0; i < 5; i++) {
                            try { window.wasm_exports.frame(); } catch (e) { break; }
                        }
                        if (window.__xxAlive.at > performance.now() - 1200 && window.__xxAlive.n > before) {
                            window.__xxRevivedAt = performance.now();
                            try { errlog.textContent += "[诊断] 循环复活成功（冲击 +" + (window.__xxAlive.n - before) + " 帧）\n"; } catch (e) {}
                            reviveTries = 0; // 复活成功重置计数，允许下次再救
                            window.__xxCoachStart("看门狗复活后"); // 冲击有效即接管，持续供血
                            return;
                        }
                        reviveTries += 1;
                        if (reviveTries <= 3) {
                            try { errlog.textContent += "[诊断] 复活冲击无效（第" + reviveTries + "次）\n"; } catch (e) {}
                            return; // 等下一轮再试（共 3 轮）
                        }
                        try { errlog.textContent += "[诊断] 循环心跳停止于第 " + window.__xxAlive.n + " 帧（future wedged），自动重载\n"; } catch (e) {}
                        try {
                            if (!xxRlUsed()) { xxReload("loop"); return; }
                        } catch (e2) {}
                        // 自动重载预算已用：仍无心跳 → 弹大旗（按钮走 ② 的样式）
                        if (!banner) xxShowBanner();
                    }
                }, 1000);
                // ② 触摸桥：点了但引擎零消费（心跳看门狗通常先一步处理）
                function xxShowBanner() {
                    if (banner) return;
                    // v1.9.14 复活静默期：唤醒/复活成功后 4 秒内不弹旗（v1.9.13
                    // 真机实锤：复活成功后触摸桥 starved 判定仍弹旗造成误报）
                    if (window.__xxRevivedAt && performance.now() - window.__xxRevivedAt < 4000) return;
                    // v1.9.16 车夫供血中不弹旗：车夫模式下帧号在推进（心跳上报
                    // 只是 8 帧粒度、上报间隔可能拖长），画面活着，弹"引擎无响应"
                    // 纯属误导（v1.9.15 真机又弹了一次，用户被误导点重载）
                    if (window.__xxCoach) return;
                    // v1.9.22 删除"wedge 弹旗前直接重载"分支：游戏内预防性重载
                    // 会丢当前进度踢回菜单（真机实锤断流体验）；走到这里的都是
                    // 真死态，由主看门狗 xxReload("loop") 预算内自动处理，大旗
                    // 仅作视觉提示兜底。
                    banner = document.createElement("div");
                    banner.style.cssText =
                        "position:fixed;left:8%;right:8%;top:36%;z-index:200;" +
                        "background:rgba(120,10,10,.94);color:#fff;border:2px solid #f66;" +
                        "border-radius:10px;padding:14px 18px;font:15px/1.5 sans-serif;" +
                        "text-align:center;pointer-events:auto;";
                    banner.innerHTML =
                        "⚠ 游戏引擎已无响应（循环停于第 " + (window.__xxAlive ? window.__xxAlive.n : "?") +
                        " 帧，已点 " + (window.__xxTouches || 0) + " 次）<br>" +
                        "点下方按钮重载即可进入（已验证有效）。";
                    var btn = document.createElement("button");
                    btn.textContent = "立即重载游戏";
                    btn.style.cssText =
                        "margin-top:8px;padding:8px 22px;font:16px sans-serif;" +
                        "background:#f66;color:#fff;border:0;border-radius:6px;";
                    btn.onclick = function () { xxReload("manual"); };
                    banner.appendChild(btn);
                    document.body.appendChild(banner);
                    try { errlog.textContent += "[诊断] 引擎无响应，已弹重载大旗\n"; } catch (e) {}
                }
                setInterval(function () {
                    if (banner) return; // 大旗只由故障弹，不重复
                    if (!window.wasm_exports) return;
                    var seen = window.__xxClicksSeen || 0;
                    var tch = window.__xxTouches || 0;
                    if (seen !== lastSeen) { lastSeen = seen; starved = 0; return; }
                    if (tch < 2) return;
                    // v1.9.15 修误弹：心跳还在（Rust 循环活着、帧号在涨）就绝不
                    // 弹旗——旧条件在"点过一次后挂机 4s"的健康页面也会弹（一次
                    // tap = start+end 计 2 次触摸，之后无新点击即 starved≥2）。
                    // 真机 v1.9.13/14 的"引擎无响应"大旗有一部分就是它误报的。
                    // v1.9.16 死线 2.5s→5s：心跳 8 帧粒度在低帧率下上报间隔
                    // 可达 3-4s，5s 死线只拦真正的心跳停滞。
                    if (window.__xxAlive && performance.now() - window.__xxAlive.at < 5000) { starved = 0; return; }
                    starved += 1;
                    if (starved < 2) return; // 连续 2 轮（≥4s）零消费才弹旗
                    xxShowBanner();
                }, 2000);
                // 消费恢复即撤旗（瞬时抖动自愈）
                setInterval(function () {
                    if (banner && (window.__xxClicksSeen || 0) > 0 &&
                        window.__xxAlive && performance.now() - window.__xxAlive.at < 2500) {
                        banner.remove && banner.remove();
                        banner = null; starved = 0;
                    }
                }, 1000);
            })();
            // v1.9.7 差异检测：backing ≠ CSS×dpr（容器缩放/dpr 漂移）时直接把
            // backing 写正并直通同步 wasm——不清零画布（X5 上清零会杀死帧回调，
            // v1.9.6 真机冻帧实锤）、不派发 resize 事件。
            var heals = 0;
            setInterval(function () {
                if (!window.wasm_exports) return;
                var dpr = window.high_dpi ? window.devicePixelRatio || 1 : 1;
                var w = Math.floor(canvas.clientWidth * dpr);
                var h = Math.floor(canvas.clientHeight * dpr);
                if ((canvas.width !== w || canvas.height !== h) && heals < 30) {
                    heals += 1;
                    canvas.width = w;
                    canvas.height = h;
                    window.xxSyncScreen();
                }
            }, 1500);
            // 竖持提示：横版游戏竖持时世界两侧出屏、按钮不可见——给出引导而不是
            // 让玩家以为"点了没反应"
            var rot = document.createElement("div");
            rot.id = "rot_hint";
            rot.textContent = "请横持设备游玩";
            rot.style.cssText =
                "position:fixed;left:0;top:0;width:100%;height:100%;display:none;" +
                "align-items:center;justify-content:center;background:#141024;color:#d8c48a;" +
                "font:24px sans-serif;z-index:50;pointer-events:none;";
            document.body.appendChild(rot);
            function rotHint() {
                // ?landscape=1：模拟器验证用——竖窗里靠 orientFix 旋转视野游玩，
                // 不弹横持引导（真机 TapTap 物理横屏不受影响）
                var skip = false;
                try { skip = /[?&]landscape=1/.test(location.search); } catch (e) {}
                rot.style.display =
                    window.innerHeight > window.innerWidth && window.wasm_exports && !skip ? "flex" : "none";
            }
            window.addEventListener("resize", rotHint, true);
            setInterval(rotHint, 800);
        })();
        // 手机竖持时强制横屏视野：整页旋转 90°（TapTap/X5 内核横屏不旋转页面的标准解法）。
        // 缩放交由 Rust 侧信箱相机（src/cam.rs）等比适配，此处原样 JS 零缩放。
        function orientFix() {
            var b = document.body;
            if (window.innerHeight > window.innerWidth) {
                b.style.transformOrigin = "left top";
                b.style.transform = "rotate(90deg) translate(0px, -100%)";
                b.style.width = window.innerHeight + "px";
                b.style.height = window.innerWidth + "px";
            } else {
                b.style.transform = "";
                b.style.width = "100%";
                b.style.height = "100%";
            }
            var c = document.getElementById("glcanvas");
            if (c) {
                c.style.width = "100%";
                c.style.height = "100%";
            }
        }
        window.addEventListener("resize", orientFix);
        window.addEventListener("orientationchange", function () { setTimeout(orientFix, 120); });
        window.addEventListener("load", orientFix);

        // ------------------------------------------------------------------
        // v1.9.10 TapTap 容器仿真回归（?chaos=1 / ?tap=1 才激活，生产零影响）。
        // 背景：本地 dpr=1 的 Chromium 与 X5 真机的差异已两次躲过全部本地测试
        // （v1.9.5 时钟冻结、v1.9.8 dpr 乘 3.25）。此工具把两类真机毒环境在本地
        // 复现成可回归的确定性场景：
        //   chaos：每 6s 把三种帧时钟全部冻结 2.5s（等价 X5 可见性钳制/时钟停摆），
        //          验证触摸帧泵 + tick 直调兜底能让游戏在冻结结束后照常进。
        //   tap：每 3.5s 在「开始新的人生」按钮中心派发真实 TouchEvent（Touch
        //        构造器 + 冒泡，与真机同路径），并断言 __xxState 离开 0（进游戏）。
        // 世界→CSS 换算与 cam.rs view_cam 互逆（v1.9.9 本地注入 315→(469,547)
        // 实测校准过）。
        // ------------------------------------------------------------------
            (function xxSim() {
            var q;
            try { q = new URLSearchParams(location.search); } catch (e) { return; }
            if (!q.get("chaos") && !q.get("tap") && !q.get("flow") && !q.get("wedge")) return;
            var el = document.getElementById("errlog");
            var log = function (s) { if (el) el.textContent += s + "\n"; };
            // 世界坐标 -> 页面 CSS 坐标（与 cam.rs view_cam 互逆；v1.9.9 本地注入
            // 315→(469,547) 实测校准）。canvas 满窗时 rect.left/top≈0。
            var w2c = function (wx, wy) {
                var c = document.getElementById("glcanvas");
                var r = c.getBoundingClientRect();
                var fit = Math.min(r.width / 1280, r.height / 720);
                var dw = r.width / fit, dh = r.height / fit;
                return [r.left + (wx - (640 - dw / 2)) * fit, r.top + (wy - (360 - dh / 2)) * fit];
            };
            // 真机同路径派发触摸；compat=true 时再补 X5 兼容鼠标事件
            // （mousedown→mouseup→click，v1.9.10 真机 [诊断](mouse) 标记证实 X5 会发）
            var fireTap = function (wx, wy, compat, cb) {
                var c = document.getElementById("glcanvas");
                var xy = w2c(wx, wy);
                var id = Date.now() % 1000000;
                try {
                    var t = new Touch({ identifier: id, target: c, clientX: xy[0], clientY: xy[1] });
                    c.dispatchEvent(new TouchEvent("touchstart", { cancelable: true, bubbles: true, changedTouches: [t], touches: [t] }));
                    setTimeout(function () {
                        c.dispatchEvent(new TouchEvent("touchend", { cancelable: true, bubbles: true, changedTouches: [t], touches: [] }));
                        if (compat) {
                            setTimeout(function () {
                                var ev = { clientX: xy[0], clientY: xy[1], button: 0, bubbles: true, cancelable: true };
                                try {
                                    c.dispatchEvent(new MouseEvent("mousedown", ev));
                                    c.dispatchEvent(new MouseEvent("mouseup", ev));
                                    c.dispatchEvent(new MouseEvent("click", ev));
                                } catch (e3) {}
                            }, 120);
                        }
                        if (cb) cb(xy);
                    }, 90);
                } catch (e2) {
                    log("[SIM] 事件派发失败: " + e2.message);
                }
            };
            if (q.get("chaos")) {
                window.__xxClockGate = false;
                setInterval(function () {
                    if (!window.wasm_exports) return;
                    window.__xxClockGate = true;
                    log("[SIM] 冻结全部帧时钟 2.5s（模拟 X5 钳制）…");
                    setTimeout(function () {
                        window.__xxClockGate = false;
                        log("[SIM] 解冻。fr=" + Math.round(performance.now() - (window.__xxFrameAt || 0)) + "ms");
                    }, 2500);
                }, 6000);
            }
            // v1.9.17 wedge 仿真 → v1.9.19 周期性钳制：真机视频定案容器会反复
            // 钳制（首开+游戏中期 idle 后），仿真在 60 秒重载预算解锁后再次
            // 冻结——完整模拟"冻结→触摸 burst 当拍自救→评估重载→60 秒新鲜
            // 期→再冻结"周期（每 5 秒轮询预算状态）。
            if (q.get("wedge")) {
                setInterval(function () {
                    if (!window.wasm_exports || window.__xxSimWedge) return;
                    if (window.xxRlUsed && window.xxRlUsed()) return; // 预算内=重载后新鲜期
                    window.__xxClockGate = true; // 三时钟死
                    window.__xxSimWedge = 1; // 车夫禁启
                    log("[SIM] wedge：冻结三时钟+车夫禁启（模拟 X5 钳制）");
                }, 5000);
            }
            if (q.get("tap")) {
                setInterval(function () {
                    if (!window.wasm_exports || window.__xxClockGate) return;
                    if (window.__xxState !== 0 && window.__xxState !== -1) return; // 已进游戏，停止
                    var s0 = window.__xxState;
                    fireTap(470, 549, false, function (xy) {
                        setTimeout(function () {
                            log("[SIM] tap @css(" + Math.round(xy[0]) + "," + Math.round(xy[1]) + ") st " +
                                s0 + "→" + window.__xxState +
                                (window.__xxState > 0 ? " ✓进游戏" : " ✗未进"));
                        }, 800);
                    });
                }, 3500);
            }
            // v1.9.10 真机"进去过一次、重进页面后点开始无效"——入口自动回归：
            // 菜单点「开始新的人生」→ Scene::Reality（begin_mortal 直达现实场景，
            // Draft 是轮回镜选天赋屏、不在首命入口链上）。每步带 X5 兼容鼠标事件，
            // early=1 模拟真机"一就绪就点"（用户实测在 F≈100 即点）。
            if (q.get("flow")) {
                var early = q.get("flow") === "early";
                var fired = false, tries = 0;
                var compat = q.get("compat") !== "0"; // 默认带兼容鼠标事件
                var iv = setInterval(function () {
                    if (!window.wasm_exports) return;
                    tries++;
                    var st = window.__xxState;
                    if (st === 1) {
                        clearInterval(iv);
                        log("[FLOW] ✓✓ 进入 Reality（进游戏成功）compat=" + compat +
                            (early ? " 早期时序" : "") + " 第" + tries + "次轮询确认");
                    } else if (st > 1) {
                        clearInterval(iv);
                        log("[FLOW] ？意外场景 st" + st);
                    } else if (tries === 3 && !fired) {
                        fired = true;
                        log("[FLOW] ①点「开始新的人生」(compat=" + compat + (early ? " 早期时序" : "") + ")");
                        fireTap(470, 549, compat);
                    } else if (tries > 200) {
                        clearInterval(iv);
                        log("[FLOW] ✗ 超时：点击后未进游戏，末态 st" + st);
                    }
                }, 300);
            }
        })();

        // v1.9.1 关键修复：删除 macroquad 模板遗留的"load 事件隐藏加载层"。
        // window.load 只等文档资源就绪，不等 wasm（fetch 不阻塞 load）——加载层
        // 在页面打开约 1 秒内就被 display:none 撤掉，之后手机上 wasm 下载+编译的
        // 10~40 秒里玩家面对纯黑画布：分阶段提示与 45s 重试看门狗全部被废，
        // 点哪都没反应（TapTap 真机"无法进入游戏、点击没反应"的最终根因）。
        // 加载层的隐藏只由 pollFirstFrame 负责：window.wasm_exports 就绪才撤。

    