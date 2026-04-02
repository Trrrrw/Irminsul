use salvo::{Listener, Server, conn::TcpListener};

mod admin;
mod audit;
mod content;
mod db;
mod error;
mod models;
mod routers;
mod vector;

#[tokio::main]
async fn main() {
    // 初始化数据库连接
    db::init("./data").await;

    // 初始化日志子系统
    tracing_subscriber::fmt::init();

    // 绑定服务器到端口7040
    let acceptor = TcpListener::new("0.0.0.0:7040").bind().await;

    let router = routers::root();
    println!("{router:?}");

    // 开始服务请求
    Server::new(acceptor).serve(router).await;
}
