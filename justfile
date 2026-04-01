set windows-shell := ["powershell.exe", "-c"]

admin_dev_origin := "http://127.0.0.1:3401"

[private]
[unix]
_ensure_frontend_deps:
    if [ ! -d "panel/node_modules" ]; then cd panel && bun install; fi

[private]
[windows]
_ensure_frontend_deps:
    if (!(Test-Path "panel/node_modules")) { cd panel; bun install }

[private]
[unix]
_ensure_cargo_watch:
    cargo watch --version >/dev/null 2>&1 || { echo 'cargo-watch 未安装，请先执行: cargo install cargo-watch'; exit 1; }

[private]
[windows]
_ensure_cargo_watch:
    cargo watch --version *> $null; if ($LASTEXITCODE -ne 0) { Write-Error "cargo-watch 未安装，请先执行: cargo install cargo-watch"; exit 1 }

[private]
[unix]
_frontend_dev:
    cd panel && bun run dev

[private]
[windows]
_frontend_dev:
    cd panel; bun run dev

[private]
[unix]
_backend_watch:
    ADMIN_DEV_SERVER_ORIGIN="{{ admin_dev_origin }}" cargo watch -w backend -w shared -w Cargo.toml -w Cargo.lock -x run

[private]
[windows]
_backend_watch:
    $env:ADMIN_DEV_SERVER_ORIGIN="{{ admin_dev_origin }}"; cargo watch -w backend -w shared -w Cargo.toml -w Cargo.lock -x run

# 开发环境启动
[unix]
dev: _ensure_frontend_deps _ensure_cargo_watch
    bash -lc 'set -euo pipefail; just _frontend_dev & frontend_pid=$!; trap "kill $frontend_pid" EXIT INT TERM; just _backend_watch'

[windows]
dev: _ensure_frontend_deps _ensure_cargo_watch
    $frontend = Start-Process -FilePath "just" -ArgumentList "_frontend_dev" -PassThru; try { just _backend_watch } finally { if ($frontend -and -not $frontend.HasExited) { Stop-Process -Id $frontend.Id } }

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

# 安装前端依赖
[unix]
install:
    cd panel && bun install

[windows]
install:
    cd panel; bun install

# 清理构建产物
[unix]
clean:
    cargo clean
    rm -rf panel/dist

[windows]
clean:
    cargo clean
    Remove-Item -Recurse -Force panel\dist -ErrorAction SilentlyContinue
