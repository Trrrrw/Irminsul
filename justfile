set windows-shell := ["powershell.exe", "-c"]

[private]
[unix]
_ensure_frontend_deps:
    if [ ! -d "panel/node_modules" ]; then cd panel && bun install; fi

[private]
[windows]
_ensure_frontend_deps:
    if (!(Test-Path "panel/node_modules")) { cd panel; bun install }

# 开发环境启动
[unix]
dev: _ensure_frontend_deps
    cd panel && bun run build
    cargo run
[windows]
dev: _ensure_frontend_deps
    cd panel; bun run build
    cargo run

# 只运行前端
[unix]
frontend: _ensure_frontend_deps
    cd panel && bun run dev
[windows]
frontend: _ensure_frontend_deps
    cd panel; bun run dev

# 只运行后端
backend:
    cargo run

# 构建前端
[unix]
build: _ensure_frontend_deps
    cd panel && bun run build
    cargo build --release
[windows]
build: _ensure_frontend_deps
    cd panel; bun run build
    cargo build --release

# 清理构建产物
[unix]
clean:
    cargo clean
    rm -rf panel/dist
[windows]
clean:
    cargo clean
    Remove-Item -Recurse -Force panel\dist -ErrorAction SilentlyContinue
