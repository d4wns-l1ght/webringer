use axum::{http::{self, header::CONTENT_TYPE, StatusCode}, response::{self, IntoResponse}};

#[derive(rust_embed::Embed)]
#[folder = "static/"]
struct Static;

struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
	T: Into<String>,
{
	fn into_response(self) -> response::Response {
		let path: String = self.0.into();

		match Static::get(path.as_str()) {
			Some(content) => {
				let mime = mime_guess::from_path(path).first_or_octet_stream();
				(
					[(CONTENT_TYPE, mime.as_ref())],
					content.data,
				)
					.into_response()
			}
			None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
		}
	}
}

pub(super) async fn static_handler(uri: http::Uri) -> impl IntoResponse {
	let mut path = uri.path().trim_start_matches('/').to_string();

	if path.starts_with("static/") {
		path = path.replace("static/", "");
	}

	StaticFile(path)
}

pub(super) async fn not_found() -> impl IntoResponse {
	static_handler(http::Uri::from_static("/static/404.html")).await
}
