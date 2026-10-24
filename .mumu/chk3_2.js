
        // v1.5.0 H5 本地存档：注册 miniquad 插件，为 wasm 提供 localStorage 存取的 env 导入。
        // 必须在 load() 之前注册（register_plugins 在 wasm 实例化前消费 plugins 列表）。
        // Rust 侧 save.rs 的 wasm 后端按固定键读写，存档内容与桌面版 xiuxian_save.txt 完全同格式；
        // 长度一律 UTF-8 字节数（Rust 侧按字节分配缓冲区，中文字段 3 字节/字）。
        window.miniquad_add_plugin({
            name: "xx_storage",
            version: 1,
            register_plugin: function (importObject) {
                var XX_KEY = "xiuxian_save";
                importObject.env.xx_save_len = function () {
                    try {
                        var v = localStorage.getItem(XX_KEY);
                        return v == null ? -1 : new TextEncoder().encode(v).length;
                    } catch (e) { return -1; }
                };
                importObject.env.xx_save_read = function (ptr, len) {
                    try {
                        var v = localStorage.getItem(XX_KEY) || "";
                        var bytes = new TextEncoder().encode(v);
                        var arr = new Uint8Array(wasm_memory.buffer, ptr, len);
                        arr.set(bytes.subarray(0, len));
                    } catch (e) { console.error("xx_save_read:", e); }
                };
                importObject.env.xx_save_write = function (ptr, len) {
                    try {
                        var bytes = new Uint8Array(wasm_memory.buffer, ptr, len);
                        localStorage.setItem(XX_KEY, new TextDecoder("utf-8").decode(bytes));
                    } catch (e) { console.error("xx_save_write:", e); }
                };
            },
            on_init: function () {}
        });

        // v1.6.0 B 清晰度（移植自 GetFish v7.9.6）：GPU 能力懒探测。vendor/macroquad
        // 图集封顶补丁在 wasm 首次询问 gf_max_texture_size 时——此时 miniquad 已在
        // glcanvas 上建好渲染上下文——直接查询 MAX_TEXTURE_SIZE，图集按
        // min(设备上限) 动态封顶，配合 high_dpi 支撑 dpr>1 设备的原生字形栅格。
        // 必须在 load() 之前注册（与 xx_storage 同理）。
        var XX_GPU_OK = !!(window.WebGLRenderingContext || window.WebGL2RenderingContext);
        var XX_MAX_TEX = 0; // 0 = 未探测
        function xx_probe_max_tex() {
            if (XX_MAX_TEX) return XX_MAX_TEX;
            try {
                var c = document.getElementById("glcanvas");
                var gl = c.getContext("webgl2") || c.getContext("webgl") ||
                         c.getContext("experimental-webgl");
                if (gl) {
                    var v = gl.getParameter(gl.MAX_TEXTURE_SIZE);
                    if (typeof v === "number" && v > 0) XX_MAX_TEX = v;
                }
            } catch (e) { /* 探测失败走默认，图集侧另有 512 下限兜底 */ }
            if (!XX_MAX_TEX) XX_MAX_TEX = 8192;
            return XX_MAX_TEX;
        }
        window.miniquad_add_plugin({
            name: "xx_gpu",
            version: 1,
            register_plugin: function (importObject) {
                importObject.env.gf_max_texture_size = function () { return xx_probe_max_tex() | 0; };
            },
            // v1.8.2 注意：on_init 是无参调用（bundle: e[t].on_init()），拿不到
            // exports；就绪信号改由下方 WebAssembly.instantiate 钩子赋值。
            on_init: function () {}
        });

        // v1.9.10 wasm 陷阱可见化 + 回归断言信号（Rust 侧 main.rs xx_dbg 模块声明
        // 了这两个 env 导入——wasm 实例化会校验导入存在，二者必须成对出现）：
        // xx_panic：Rust panic hook 把 panic 原文写入 errlog（[PANIC] 前缀）并改
        //   document.title——此前 wasm 陷阱在真机上完全静默（F 冻结但 errlog 无记录，
        //   无法区分"容器时钟停摆"与"Rust 陷阱/互斥锁中毒"，十轮真机排查的盲区）。
        // xx_state：场景变化上报（0=Menu 1=Reality 2=Draft 3=Sim …），供 ?tap=
        //   自动回归断言"点按钮 → __xxState 离开 0（进游戏）"。
        window.__xxState = -1;
        window.miniquad_add_plugin({
            name: "xx_dbg",
            version: 1,
            register_plugin: function (importObject) {
                importObject.env.xx_panic = function (ptr, len) {
                    try {
                        var mem =
                            window.wasm_memory && window.wasm_memory.buffer
                                ? window.wasm_memory.buffer
                                : window.wasm_exports && window.wasm_exports.memory
                                  ? window.wasm_exports.memory.buffer
                                  : null;
                        if (mem) {
                            var msg = new TextDecoder("utf-8", { fatal: false }).decode(
                                new Uint8Array(mem, ptr, Math.min(len | 0, 512)),
                            );
                            var el = document.getElementById("errlog");
                            if (el) el.textContent += "[PANIC] " + msg + "\n";
                            try { document.title = "PANIC " + msg.slice(0, 60); } catch (e2) {}
                        }
                    } catch (e) {}
                };
                importObject.env.xx_state = function (code) {
                    window.__xxState = code | 0;
                };
                // v1.9.12 三通道（Rust xx_dbg 模块声明，须成对）：
                // xx_click：Rust 每消费一次点击上报计数（桥看门狗数据源）
                // xx_js_fr：JS 帧时钟心跳年龄 ms（Rust 画进菜单自检面板）
                // xx_js_touches：JS 侧累计触摸组数（同上）
                importObject.env.xx_click = function (n) {
                    window.__xxClicksSeen = n | 0;
                };
                importObject.env.xx_js_fr = function () {
                    return Math.min(999999, Math.round(performance.now() - (window.__xxFrameAt || 0)));
                };
                importObject.env.xx_js_touches = function () {
                    return window.__xxTouches || 0;
                };
                // v1.9.13 循环心跳（Rust 每 30 帧上报帧号）——JS 看门狗据此
                // 识别"循环死亡"（真机实证：首次真触摸杀死 async 主循环，
                // frame() 照常返回、fr 恒新鲜，唯一死证是 Rust 帧号停止前进）
                importObject.env.xx_alive = function (n) {
                    window.__xxAlive = { n: n | 0, at: performance.now() };
                };
            },
            on_init: function () {}
        });

        // v1.8.2 关键修复：加载层"就绪"信号。此前 window.wasm_exports 从未被
        // 任何人赋值——12 秒看门狗必然把加载层变成"加载失败·点击重试"覆盖层
        //（pointer-events:auto + reload），玩家一点就无限重载：手机上表现为
        // "点按钮没反应/再也加载不进去"。这里包装 WebAssembly.instantiate，
        // 实例化完成即把 exports 挂到 window（bundle 的 load 走 compileStreaming
        // + WebAssembly.instantiate 两步，本钩子先于 bundle 的 then 执行）。
        (function () {
            var _inst = WebAssembly.instantiate;
            WebAssembly.instantiate = function () {
                return _inst.apply(this, arguments).then(function (res) {
                    if (res && res.instance && res.instance.exports) {
                        window.wasm_exports = res.instance.exports;
                    }
                    return res;
                });
            };
        })();


        // 打不开诊断：无 WebGL / 无 WebAssembly 时给出可见提示，而不是无限白屏
        function xx_gpu_report() {
            var el = document.getElementById("errlog");
            var msgs = "";
            if (!XX_GPU_OK) {
                msgs += "诊断：此设备的 WebView 未开启 WebGL 硬件加速，游戏画面无法创建。\n" +
                        "请到系统/浏览器设置开启「硬件加速」后重试。\n";
            }
            if (!window.WebAssembly) {
                msgs += "诊断：此设备的 WebView 过旧、不支持 WebAssembly，无法运行游戏。\n" +
                        "请升级系统 WebView 或更换设备。\n";
            }
            if (msgs) {
                el.textContent += msgs;
                var m = document.getElementById("ld_msg");
                if (m) m.textContent = "设备不支持，无法运行（详见页面诊断信息）";
            }
        }
        xx_gpu_report();
    