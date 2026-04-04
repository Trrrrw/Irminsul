use salvo::{
    Router,
    oapi::{ToSchema, endpoint},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
pub struct ApiRoute {
    pub method: String,
    pub path: String,
}

pub fn router() -> Router {
    Router::with_path("api/routes").get(list_routes)
}

pub fn collect_routes(router: &Router) -> Vec<ApiRoute> {
    let mut routes = Vec::new();
    collect_routes_recursive(router, &[], &[], &mut routes);
    routes.sort();
    routes.dedup();
    routes
}

#[endpoint(
    tags("system"),
    responses((status_code = 200, description = "获取业务接口列表成功", body = Vec<ApiRoute>))
)]
async fn list_routes(res: &mut Response) {
    res.render(Json(super::list_business_api_routes()));
}

fn collect_routes_recursive(
    router: &Router,
    parent_paths: &[String],
    parent_methods: &[String],
    routes: &mut Vec<ApiRoute>,
) {
    let mut current_paths = parent_paths.to_vec();
    let mut current_methods = parent_methods.to_vec();

    for filter in router.filters() {
        let filter_debug = format!("{filter:?}");
        if let Some(path) = filter_debug.strip_prefix("path:") {
            if !path.is_empty() {
                current_paths.push(path.to_string());
            }
        } else if let Some(method) = filter_debug.strip_prefix("method:") {
            current_methods.push(method.to_ascii_uppercase());
        }
    }

    if router.goal.is_some() {
        let path = normalize_path(&current_paths);
        let methods = if current_methods.is_empty() {
            vec![String::from("ANY")]
        } else {
            current_methods.clone()
        };

        for method in methods {
            routes.push(ApiRoute {
                method,
                path: path.clone(),
            });
        }
    }

    for child in router.routers() {
        collect_routes_recursive(child, &current_paths, &current_methods, routes);
    }
}

fn normalize_path(parts: &[String]) -> String {
    let normalized_parts = parts
        .iter()
        .flat_map(|part| part.split('/'))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if normalized_parts.is_empty() {
        String::from("/")
    } else {
        format!("/{}", normalized_parts.join("/"))
    }
}
