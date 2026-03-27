set windows-shell := ["powershell.exe", "-c"]

# 开发环境启动
[unix]
dev:
    cd panel && bun run build
    cargo run
[windows]
dev:
    cd panel; bun run build
    cargo run

# 只运行前端
[unix]
frontend:
    cd panel && bun run dev
[windows]
frontend:
    cd panel; bun run dev

# 只运行后端
backend:
    cargo run

# 构建前端
[unix]
build:
    cd panel && bun run build
    cargo build --release
[windows]
build:
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
