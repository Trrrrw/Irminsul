use salvo::{Listener, Server, conn::TcpListener};
use std::path::PathBuf;

mod admin;
mod audit;
mod catalog;
mod config;
mod db;
mod error;
mod models;
mod mongo;
mod routers;

#[tokio::main]
async fn main() {
    // 初始化日志子系统
    tracing_subscriber::fmt::init();

    // 初始化本地配置、数据库与 Mongo 连接
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data");
    db::init(&data_dir).await;

    // 绑定服务器监听地址，默认使用 7040 端口。
    let bind_addr =
        std::env::var("IRMINSUL_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7040".to_string());
    let acceptor = TcpListener::new(bind_addr.clone())
        .try_bind()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "failed to bind server on {bind_addr}. 请确认端口未被占用，或通过 IRMINSUL_BIND_ADDR 指定其他地址: {error}"
            )
        });
    tracing::info!("Irminsul admin server listening on {bind_addr}");

    let router = routers::root();

    // 开始服务请求
    Server::new(acceptor).serve(router).await;
}
