use std::{fmt::Debug, str::FromStr};

use clap::Parser;
use tracing::info;

/// A server for hosting a webring!
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
	/// Address to bind to
	#[arg(short, long, value_name = "ADDR", default_value = "0.0.0.0")]
	pub address: String,

	/// Port to listen on
	#[arg(short, long, value_name = "PORT", default_value_t = 10983)]
	pub port: u16,
}

pub(super) fn read_env_var<T>(key: &str, default: T) -> T
where
	T: FromStr + Debug,
{
	dotenvy::var(key)
		.ok()
		.and_then(|s| s.parse().ok())
		.unwrap_or_else(|| {
			info!("Using default {:?} value of {:?}", key, default);
			default
		})
}
