use tracing::{subscriber::set_global_default, Subscriber};
use tracing_subscriber::EnvFilter;

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::init_tests;

pub fn get_subscriber(env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .finish()
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("failed to set subscriber");
}
