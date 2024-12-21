use once_cell::sync::Lazy;

use super::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("debug".into());
    init_subscriber(subscriber);
});

pub async fn init_tests() {
    Lazy::force(&TRACING);
}
