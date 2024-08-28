/// The `utils` module hosts global functions that do not belong to any particular data type.
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// The `trace_init` function initializing logging using the [`tracing`] and [`tracing_subscriber`]
/// crates.
/// Pass the desired log level into the environment when running the app from cargo.
/// E.g. `$RUST_LOG="trace" cargo run` for debugging.
pub fn trace_init() {
    if tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tardy=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .is_ok()
    {};
    tracing::trace!("Loading Tardy...");
}
