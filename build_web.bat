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
rem keep taptap_h5\index.html in sync with web\index.html (cache-bust params live there)
copy /y web\index.html taptap\taptap_verify\taptap_h5\index.html >nul
rem assert the TapTap trio is complete (index.html + mq_js_bundle.js + xiuxian.wasm)
python -c "import os; root='taptap/taptap_verify/taptap_h5'; need=['index.html','mq_js_bundle.js','xiuxian.wasm']; missing=[n for n in need if not os.path.isfile(os.path.join(root,n))]; assert not missing, 'taptap_h5 missing: '+str(missing)" || exit /b 1
python -c "import zipfile,os; root='taptap/taptap_verify/taptap_h5'; out='taptap/taptap_verify.zip'; z=zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED); [z.write(os.path.join(root,n),'taptap_h5/'+n) for n in sorted(os.listdir(root))]; z.close()"
echo === H5 build done, TapTap ZIP updated ===
