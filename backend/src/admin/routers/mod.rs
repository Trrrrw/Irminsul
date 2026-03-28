use salvo::Router;

pub mod auth;
pub mod invitations;
pub mod users;

pub fn router() -> Router {
    Router::new()
        .push(auth::router())
        .push(invitations::router())
        .push(users::router())
}
