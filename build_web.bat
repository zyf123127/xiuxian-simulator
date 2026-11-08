@echo off
rem One-click H5 build + deploy to web\ and taptap\taptap_verify\taptap_h5\
rem (ASCII only: cmd reads batch files in the OEM codepage; UTF-8 Chinese here
rem  gets mis-decoded under cp936 and splits lines. Keep this file ASCII+CRLF.)
set RUSTFLAGS=-C link-arg=--import-undefined
cargo build --release --target wasm32-unknown-unknown || exit /b 1
rem ---- load-speed 2: optional wasm-opt -Oz pass (pinned tool, graceful skip) ----
rem Tool: tools\binaryen\wasm-opt.exe, pinned from binaryen version_133 (see
rem tools\binaryen\VERSION.txt). Directory is gitignored; without the tool this
rem build ships the raw cargo wasm (previous behavior). If the tool exists but
rem FAILS, abort instead of silently shipping a possibly-corrupt artifact.
rem Feature flags = rustc wasm32-unknown-unknown defaults (bulk-memory /
rem sign-ext / mutable-globals / nontrapping-float-to-int); binaryen cannot
rem auto-detect them because cargo strip=true removed the target_features section.
set WASM_RAW=target\wasm32-unknown-unknown\release\xiuxian.wasm
set WASM_OPT_OUT=target\wasm32-unknown-unknown\release\xiuxian.opt.wasm
if not exist tools\binaryen\wasm-opt.exe goto wasmopt_skip
python -c "import os; print('[wasm-opt] in :', os.path.getsize(r'target\wasm32-unknown-unknown\release\xiuxian.wasm'), 'bytes')" || exit /b 1
tools\binaryen\wasm-opt.exe --enable-bulk-memory --enable-sign-ext --enable-mutable-globals --enable-nontrapping-float-to-int -Oz %WASM_RAW% -o %WASM_OPT_OUT% || exit /b 1
python -c "import os; s=os.path.getsize(r'target\wasm32-unknown-unknown\release\xiuxian.opt.wasm'); assert s>0, 'wasm-opt produced empty file'; print('[wasm-opt] out:', s, 'bytes')" || exit /b 1
copy /y %WASM_OPT_OUT% web\xiuxian.wasm >nul
echo [wasm-opt] -Oz applied (binaryen version_133)
goto wasmopt_done
:wasmopt_skip
echo [skip] tools\binaryen\wasm-opt.exe not found, shipping raw cargo wasm
copy /y %WASM_RAW% web\xiuxian.wasm >nul
:wasmopt_done
copy /y web\xiuxian.wasm taptap\taptap_verify\taptap_h5\xiuxian.wasm >nul
rem v1.16.9 portrait-externalize: ship 18 AI portraits as loose files next to the
rem wasm (LazyTex fetches portraits/<name>.png at runtime; -696KB off the wasm
rem data segment). Copy into both the TapTap deploy dir and web\ (local
rem http.server acceptance). Fail hard if the source set is not exactly 18.
xcopy /y /d assets\portraits_wasm\*.png taptap\taptap_verify\taptap_h5\portraits\ >nul || exit /b 1
xcopy /y /d assets\portraits_wasm\*.png web\portraits\ >nul || exit /b 1
python -c "import os; d='assets/portraits_wasm'; pngs=[f for f in os.listdir(d) if f.endswith('.png')]; assert len(pngs)==18, f'portraits_wasm must be exactly 18 pngs, got {len(pngs)}: {pngs}'" || exit /b 1
rem v1.16.10 font-externalize: ship the deflated font as a loose file next to the
rem wasm (index.html prefetches font.ttf.deflate?v=GAME_VER in parallel with the
rem wasm download and hands it to the wasm via the xx_font env plugin; the
rem embedded copy is desktop-only now, so the wasm loses ~378KB). Copy into both
rem the TapTap deploy dir and web\ (local http.server acceptance). Fail hard if
rem the source is missing or implausibly small.
copy /y assets\font.ttf.deflate taptap\taptap_verify\taptap_h5\font.ttf.deflate >nul || exit /b 1
copy /y assets\font.ttf.deflate web\font.ttf.deflate >nul || exit /b 1
python -c "import os; s=os.path.getsize('assets/font.ttf.deflate'); assert s>100000, f'font.ttf.deflate missing or implausibly small: {s}'" || exit /b 1
rem keep taptap_h5\index.html in sync with web\index.html (cache-bust params live there)
rem v1.16.8 load-knife 1: minify index.html at deploy time (source
rem web\index.html stays readable; only the shipped copy is stripped).
rem Aborts the build if any contract token (GAME_VER / miniquad_add_plugin /
rem xx_storage) is lost.
rem v1.16.9 knife B: mq_js_bundle.js is INLINED into the shipped index.html
rem (3rd arg = bundle path) - one less serial request on cold boot.
python tools\minify_index.py web\index.html taptap\taptap_verify\taptap_h5\index.html web\mq_js_bundle.js || exit /b 1
rem assert the TapTap deploy set is complete (v1.16.10: index.html + xiuxian.wasm
rem + font.ttf.deflate + portraits\ 18 pngs (bundle is INLINED into index.html, not a separate entry))
python -c "import os; root='taptap/taptap_verify/taptap_h5'; need=['index.html','xiuxian.wasm','font.ttf.deflate']; missing=[n for n in need if not os.path.isfile(os.path.join(root,n))]; assert not missing, 'taptap_h5 missing: '+str(missing); pngs=[f for f in os.listdir(os.path.join(root,'portraits')) if f.endswith('.png')]; assert len(pngs)==18, f'taptap_h5/portraits must be exactly 18 pngs, got {len(pngs)}'" || exit /b 1
rem v1.16.9: recursive zip walk (portraits/ subdir now ships INSIDE the zip;
rem the old os.listdir would have produced an empty-dir entry and dropped PNGs -
rem verify_taptap_zip.py name-list assertion is the safety net for that)
rem v1.16.9 knife B cleanup: mq_js_bundle.js is no longer a deploy entry (inlined
rem into index.html) - drop any stale copy from previous deploys before zipping
if exist taptap\taptap_verify\taptap_h5\mq_js_bundle.js del taptap\taptap_verify\taptap_h5\mq_js_bundle.js
python -c "import zipfile,os; root='taptap/taptap_verify/taptap_h5'; out='taptap/taptap_verify.zip'; z=zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED); [z.write(os.path.join(dp,f), 'taptap_h5/'+os.path.relpath(os.path.join(dp,f),root).replace(os.sep,'/')) for dp,_,fs in os.walk(root) for f in sorted(fs)]; z.close()"
echo === H5 build done, TapTap ZIP updated ===
