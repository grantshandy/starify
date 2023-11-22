use axum::routing::get;


pub async fn router() -> axum::Router {
	axum::Router::new()
		.route("/", get(root))
}

async fn root() -> &'static str {
	"Hello, World!"
}
