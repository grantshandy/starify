mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(server::router().await.into_make_service())
        .await
        .expect("serve HTTP")
}
