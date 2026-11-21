// web_mime_retry_check.js —— v1.16.35 单2 加载链自验件纳库（复审 low 顺手收编）。
// compileStreaming MIME 兜底 retry 两态行为级单测（Node 沙箱）：从 web/index.html
// 源态提取包装 IIFE，对 window/WebAssembly/fetch 打桩执行——六用例（状态A 复用原
// body 零重拉/状态B __xxWasmUrl 兜底重拉/状态B' res.url/直通零副作用/src 拒绝终局/
// 字节坏不无益重拉）。加载链改动后重跑：node tools/web_mime_retry_check.js
"use strict";
const fs = require("fs");
const path = require("path");
const html = fs.readFileSync(
  path.join(__dirname, "..", "web", "index.html"), "utf8");
const blocks = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]);
const blk = blocks.find((b) => b.includes("_cs"));
if (!blk) throw new Error("compileStreaming 包装块未找到");

function fresh(csImpl, compileImpl, fetchImpl, xxWasmUrl) {
  const W = { compileStreaming: csImpl, compile: compileImpl };
  const win = { WebAssembly: W };
  if (xxWasmUrl !== undefined) win.__xxWasmUrl = xxWasmUrl;
  const fetchSpy = { n: 0, urls: [] };
  const fetchFn = function (u) {
    fetchSpy.n++;
    fetchSpy.urls.push(String(u));
    return fetchImpl(u);
  };
  new Function("window", "WebAssembly", "fetch", blk)(win, W, fetchFn);
  if (W.compileStreaming === csImpl) throw new Error("包装未安装");
  return { win, W, fetchSpy };
}

const MIME_ERR = new TypeError("invalid MIME type (simulated)");
const compileOK = async (buf) => ({ tag: "module", size: buf.byteLength });
const A_BYTES = new Uint8Array([0, 0x61, 0x73, 0x6d, 1, 0, 0, 0, 0xaa]);
const B_BYTES = new Uint8Array([0, 0x61, 0x73, 0x6d, 1, 0, 0, 0, 0xbb]);

async function main() {
  // 状态A：MIME 拒绝 + 原 body 可读 → 复用原 body 编译，零重拉（主收益）
  {
    const { W, fetchSpy } = fresh(
      async () => {
        throw MIME_ERR;
      },
      compileOK,
      async (u) => {
        throw new Error("状态A不应重拉: " + u);
      },
      "xiuxian.wasm?v=1.16.34"
    );
    const res = { url: "", arrayBuffer: async () => A_BYTES };
    const mod = await W.compileStreaming(Promise.resolve(res));
    if (fetchSpy.n !== 0) throw new Error("状态A发生了重拉 " + fetchSpy.urls);
    if (mod.size !== A_BYTES.byteLength) throw new Error("状态A编译字节不符");
    console.log("状态A PASS：复用原 body 编译，fetch 0 次，编译字节=原 body");
  }

  // 状态B：MIME 拒绝 + 原 body 锁定 → 按 __xxWasmUrl 兜底重拉（tee 路径构造 Response 无 url）
  {
    const { W, fetchSpy } = fresh(
      async () => {
        throw MIME_ERR;
      },
      compileOK,
      async (u) => ({ url: u, arrayBuffer: async () => B_BYTES }),
      "xiuxian.wasm?v=1.16.34"
    );
    const res = {
      url: "",
      arrayBuffer: () => Promise.reject(new TypeError("body stream already used")),
    };
    const mod = await W.compileStreaming(Promise.resolve(res));
    if (fetchSpy.n !== 1 || fetchSpy.urls[0] !== "xiuxian.wasm?v=1.16.34")
      throw new Error("状态B重拉地址不符: " + fetchSpy.urls);
    if (mod.size !== B_BYTES.byteLength) throw new Error("状态B编译字节不符");
    console.log("状态B PASS：body 锁定后按 __xxWasmUrl 兜底重拉，编译字节=重拉 body");
  }

  // 状态B'：res.url 在位（无 tee 老内核直通路径）→ 按 res.url 重拉
  {
    const { W, fetchSpy } = fresh(
      async () => {
        throw MIME_ERR;
      },
      compileOK,
      async (u) => ({ url: u, arrayBuffer: async () => B_BYTES }),
      ""
    );
    const res = {
      url: "xiuxian.wasm?v=1.16.34",
      arrayBuffer: () => Promise.reject(new TypeError("locked")),
    };
    await W.compileStreaming(Promise.resolve(res));
    if (fetchSpy.urls[0] !== "xiuxian.wasm?v=1.16.34")
      throw new Error("res.url 重拉不符: " + fetchSpy.urls);
    console.log("状态B' PASS：res.url 在位时按 res.url 重拉");
  }

  // 直通：compileStreaming 成功 → 原样返回，零副作用
  {
    const MOD = { tag: "direct" };
    const { W, fetchSpy } = fresh(
      async () => MOD,
      compileOK,
      async () => {
        throw new Error("直通路径不应 fetch");
      },
      undefined
    );
    const out = await W.compileStreaming(
      Promise.resolve({ url: "", arrayBuffer: async () => A_BYTES })
    );
    if (out !== MOD) throw new Error("直通返回值被改动");
    if (fetchSpy.n !== 0) throw new Error("直通路径发生 fetch");
    console.log("直通 PASS：成功原样返回，零副作用");
  }

  // src 网络拒绝 → 终局拒绝无死循环（不重拉）
  {
    const { W, fetchSpy } = fresh(
      async (src) => {
        await src;
        throw MIME_ERR;
      },
      compileOK,
      async () => {
        throw new Error("src 拒绝后不应重拉");
      },
      ""
    );
    let threw = null;
    try {
      await W.compileStreaming(Promise.reject(new Error("network dead")));
    } catch (e) {
      threw = e;
    }
    if (!threw) throw new Error("src 拒绝未终局");
    if (fetchSpy.n !== 0) throw new Error("src 拒绝后发生了重拉");
    console.log("src拒绝 PASS：终局拒绝无死循环，fetch 0 次");
  }

  // 复用 body 编译失败（字节坏）→ 终局拒绝，不做无益重拉
  {
    const { W, fetchSpy } = fresh(
      async () => {
        throw MIME_ERR;
      },
      async () => {
        throw new Error("compile failed: bad bytes");
      },
      async () => {
        throw new Error("字节坏不应重拉");
      },
      ""
    );
    let threw = null;
    try {
      await W.compileStreaming(
        Promise.resolve({ url: "", arrayBuffer: async () => A_BYTES })
      );
    } catch (e) {
      threw = e;
    }
    if (!threw || !/bad bytes/.test(String(threw)))
      throw new Error("编译失败未终局: " + threw);
    if (fetchSpy.n !== 0) throw new Error("编译失败后发生了无益重拉");
    console.log("字节坏 PASS：编译失败终局拒绝，无无益重拉");
  }

  console.log("compileStreaming 兜底 retry 行为级单测：6/6 PASS");
}
main()
  .then(() => process.exit(0))
  .catch((e) => {
    console.error("FAIL:", (e && e.message) || e);
    process.exit(1);
  });
