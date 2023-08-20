use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logger() {
    // Env
    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();

    // Init
    tracing_subscriber::registry()
        .with(fmt::layer().without_time().with_target(false))
        .with(env_filter)
        .init();
}
