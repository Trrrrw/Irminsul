use salvo::{Router, prelude::StaticDir, proxy::Proxy};

#[derive(Debug, Clone, PartialEq, Eq)]
enum AdminAssetsMode {
    Static,
    Proxy { upstream: String },
}

fn proxy_route_patterns() -> [&'static str; 3] {
    ["admin", "admin/{**rest}", "_bun/{**rest}"]
}

fn bun_runtime_proxy_path(rest: &str) -> String {
    format!("/_bun/{}", rest.trim_start_matches('/'))
}

fn resolve_admin_assets_mode() -> AdminAssetsMode {
    if cfg!(debug_assertions)
        && let Ok(upstream) = std::env::var("ADMIN_DEV_SERVER_ORIGIN")
    {
        let upstream = upstream.trim().trim_end_matches('/').to_string();

        if !upstream.is_empty() {
            return AdminAssetsMode::Proxy { upstream };
        }
    }

    AdminAssetsMode::Static
}

pub fn static_router() -> Router {
    match resolve_admin_assets_mode() {
        AdminAssetsMode::Static => Router::with_path("admin/{**path}").get(
            StaticDir::new("panel/dist")
                .defaults("index.html")
                .fallback("index.html")
                .auto_list(false),
        ),
        AdminAssetsMode::Proxy { upstream } => {
            let [admin_root, admin_rest, bun_runtime] = proxy_route_patterns();

            Router::new()
                .push(Router::with_path(admin_root).goal(Proxy::use_hyper_client(upstream.clone())))
                .push(Router::with_path(admin_rest).goal(Proxy::use_hyper_client(upstream.clone())))
                .push(
                    Router::with_path(bun_runtime).goal(
                        Proxy::use_hyper_client(upstream).url_path_getter(|req, _| {
                            req.params().tail().map(bun_runtime_proxy_path)
                        }),
                    ),
                )
        }
    }
}
