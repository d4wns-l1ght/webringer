//! This module handles the site functions
//! Where all the [axum] handlers etc live

use askama::Template;
use axum::response::Html;

pub mod admin;
pub mod join;
pub mod leave;
pub mod login;
pub mod ring;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

pub async fn index() -> Html<String> {
	Html(IndexTemplate {}.render().unwrap())
}
