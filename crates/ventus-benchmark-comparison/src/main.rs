use axum::{response::IntoResponse, routing::get};

async fn root(body: axum::body::Bytes) -> impl IntoResponse {
    body
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new().route("/", get(root).post(root));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap()
}
