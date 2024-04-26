
use axum::{routing::{get, post}, Router};

mod test_main;
mod test_aes;

#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/", get(root))
    // .route("/users", post(create_user))
    ;
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
  "Hello World!"
}