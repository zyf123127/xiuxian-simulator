@echo off
rem One-click H5 build + deploy to web\ and taptap\taptap_verify\taptap_h5\
rem (ASCII only: cmd reads batch files in the OEM codepage; UTF-8 Chinese here
rem  gets mis-decoded under cp936 and splits lines. Keep this file ASCII+CRLF.)
set RUSTFLAGS=-C link-arg=--import-undefined
cargo build --release --target wasm32-unknown-unknown || exit /b 1
copy /y target\wasm32-unknown-unknown\release\xiuxian.wasm web\xiuxian.wasm >nul
copy /y target\wasm32-unknown-unknown\release\xiuxian.wasm taptap\taptap_verify\taptap_h5\xiuxian.wasm >nul
rem assert the TapTap trio is complete (index.html + mq_js_bundle.js + xiuxian.wasm)
python -c "import os; root='taptap/taptap_verify/taptap_h5'; need=['index.html','mq_js_bundle.js','xiuxian.wasm']; missing=[n for n in need if not os.path.isfile(os.path.join(root,n))]; assert not missing, 'taptap_h5 missing: '+str(missing)" || exit /b 1
python -c "import zipfile,os; root='taptap/taptap_verify/taptap_h5'; out='taptap/taptap_verify.zip'; z=zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED); [z.write(os.path.join(root,n),'taptap_h5/'+n) for n in sorted(os.listdir(root))]; z.close()"
echo === H5 build done, TapTap ZIP updated ===
