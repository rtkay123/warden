use warden_stack::{Monitoring, tracing::TracingBuilder};

fn main() {
    let config = Monitoring {
        log_level: "info".to_string(),
    };
    let _tracing = TracingBuilder::default().build(&config);

    tracing::info!("hello from tracing");
}
