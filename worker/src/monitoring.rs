use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::{filter::Directive, fmt::format::Pretty};
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_web::{performance_layer, MakeConsoleWriter};

#[worker::event(start)]
fn start() {
  console_error_panic_hook::set_once();

  // directives for debug builds
  #[cfg(debug_assertions)]
  let default_directive = Directive::from(LevelFilter::TRACE);

  #[cfg(debug_assertions)]
  let filter_directives = if let Ok(filter) = std::env::var("RUST_LOG") {
    filter
  } else {
    "worker=trace".to_string()
  };

  // directives for release builds
  #[cfg(not(debug_assertions))]
  let default_directive = Directive::from(LevelFilter::INFO);

  #[cfg(not(debug_assertions))]
  let filter_directives = if let Ok(filter) = std::env::var("RUST_LOG") {
    filter
  } else {
    "worker=trace".to_string()
  };

  let filter = EnvFilter::builder()
    .with_default_directive(default_directive)
    .parse_lossy(filter_directives);

  let fmt_layer = tracing_subscriber::fmt::layer()
    .json()
    .with_ansi(false) // Only partially supported across JavaScript runtimes
    .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
    .with_writer(MakeConsoleWriter) // write events to the console
    .with_filter(filter);
  let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
  tracing_subscriber::registry().with(fmt_layer).with(perf_layer).init();
}
