use salvo::Router;

pub mod audit;
pub mod auth;
pub mod content;
pub mod invitations;
pub mod settings;
pub mod users;

pub fn router() -> Router {
    Router::new()
        .push(auth::router())
        .push(audit::router())
        .push(content::router())
        .push(invitations::router())
        .push(settings::router())
        .push(users::router())
}
