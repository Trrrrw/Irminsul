# --- 编译前端 ---
FROM oven/bun:alpine AS frontend-builder
WORKDIR /app/panel
COPY panel/package.json panel/bun.lock ./
RUN bun install
COPY panel/ .
RUN bun run build

# --- 编译后端 ---
FROM rust:alpine AS rust-builder
WORKDIR /app
COPY Cargo.toml ./
COPY backend ./backend/
COPY plugins ./plugins/
COPY shared ./shared/
RUN cargo build --release --workspace

# --- 运行时 ---
FROM alpine:latest AS runtime
LABEL authors="Trrrrw"
WORKDIR /app
COPY --from=frontend-builder /app/panel/dist /app/panel/dist
COPY --from=rust-builder /app/target/release/Irminsul /app/Irminsul
RUN mkdir -p /app/plugins
COPY --from=rust-builder /app/target/release/plugin-* /app/plugins/
RUN chmod +x /app/Irminsul /app/plugins/plugin-*
CMD ["./Irminsul"]
