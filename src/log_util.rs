use std::{env, fmt, str::FromStr};
use tracing::Metadata;
use tracing::{Event, Level, Subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::{Context, Filter};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{
  filter::filter_fn,
  fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext, FormattedFields,
  },
  layer::SubscriberExt,
  util::SubscriberInitExt,
  Layer,
};

pub struct _StdFormatter;

pub struct SqlFormatter;

impl<S, N> FormatEvent<S, N> for _StdFormatter
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self,
    ctx: &FmtContext<'_, S, N>,
    mut writer: format::Writer<'_>,
    event: &Event<'_>,
  ) -> fmt::Result {
    // Format values from the event's's metadata:
    let metadata = event.metadata();
    write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

    // Format all the spans in the event's span context.
    if let Some(scope) = ctx.event_scope() {
      for span in scope.from_root() {
        write!(writer, "{}", span.name())?;

        // `FormattedFields` is a formatted representation of the span's
        // fields, which is stored in its extensions by the `fmt` layer's
        // `new_span` method. The fields will have been formatted
        // by the same field formatter that's provided to the event
        // formatter in the `FmtContext`.
        let ext = span.extensions();
        let fields = &ext
          .get::<FormattedFields<N>>()
          .expect("will never be `None`");

        // Skip formatting the fields if the span had no fields.
        if !fields.is_empty() {
          write!(writer, "{{{}}}", fields)?;
        }
        write!(writer, ": ")?;
      }
    }

    // Write fields on the event
    ctx.field_format().format_fields(writer.by_ref(), event)?;

    let write_result = writeln!(writer);
    return write_result;
  }
}

impl<S, N> FormatEvent<S, N> for SqlFormatter
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self,
    ctx: &FmtContext<'_, S, N>,
    mut writer: format::Writer<'_>,
    event: &Event<'_>,
  ) -> fmt::Result {
    // Format values from the event's's metadata:

    // Format all the spans in the event's span context.
    if let Some(scope) = ctx.event_scope() {
      for span in scope.from_root() {
        write!(writer, "{}", span.name())?;

        // `FormattedFields` is a formatted representation of the span's
        // fields, which is stored in its extensions by the `fmt` layer's
        // `new_span` method. The fields will have been formatted
        // by the same field formatter that's provided to the event
        // formatter in the `FmtContext`.
        let ext = span.extensions();
        let fields = &ext
          .get::<FormattedFields<N>>()
          .expect("will never be `None`");

        // Skip formatting the fields if the span had no fields.
        if !fields.is_empty() {
          write!(writer, "{{{}}}", fields)?;
        }
        write!(writer, ": ")?;
      }
    }

    // Write fields on the event
    ctx.field_format().format_fields(writer.by_ref(), event)?;

    let write_result = writeln!(writer);
    return write_result;
  }
}

pub struct LogGuards {
  _sql_guard: tracing_appender::non_blocking::WorkerGuard,
  _std_guard: tracing_appender::non_blocking::WorkerGuard,
}

struct NoneSqlLevelFilter {
  level: Level
}

impl<S> Filter<S> for NoneSqlLevelFilter {
    #[doc = r" Returns `true` if this layer is interested in a span or event with the"]
    #[doc = r" given [`Metadata`] in the current [`Context`], similarly to"]
    #[doc = r" [`Subscriber::enabled`]."]
    #[doc = r""]
    #[doc = r" If this returns `false`, the span or event will be disabled _for the"]
    #[doc = r" wrapped [`Layer`]_. Unlike [`Layer::enabled`], the span or event will"]
    #[doc = r" still be recorded if any _other_ layers choose to enable it. However,"]
    #[doc = r" the layer [filtered] by this filter will skip recording that span or"]
    #[doc = r" event."]
    #[doc = r""]
    #[doc = r" If all layers indicate that they do not wish to see this span or event,"]
    #[doc = r" it will be disabled."]
    #[doc = r""]
    #[doc = r" [`metadata`]: tracing_core::Metadata"]
    #[doc = r" [`Subscriber::enabled`]: tracing_core::Subscriber::enabled"]
    #[doc = r" [filtered]: crate::filter::Filtered"]
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
      return meta.level() <= &self.level && meta.target() != "sql";
    }
}

pub fn log_init() -> LogGuards {
  let sql_appender = RollingFileAppender::new(Rotation::NEVER, "./", "sql.log");
  let (sql_blocking, sql_guard) = tracing_appender::non_blocking(sql_appender);
  let sql_layer = tracing_subscriber::fmt::layer()
    .event_format(SqlFormatter)
    .with_writer(sql_blocking)
    .with_filter(filter_fn(|metadata| metadata.target() == "sql"));

  let rust_log_env = env::var("RUST_LOG").unwrap_or_else(|_| String::from("INFO"));
  let log_level = Level::from_str(&rust_log_env).unwrap();

  let (std_blocking, std_guard) = tracing_appender::non_blocking(std::io::stdout());
  let std_layer = tracing_subscriber::fmt::layer()
    .with_writer(std_blocking)
    .with_filter(NoneSqlLevelFilter{level: log_level});

  tracing_subscriber::registry()
    .with(sql_layer)
    .with(std_layer)
    .init();

  LogGuards {
    _sql_guard: sql_guard,
    _std_guard: std_guard,
  }
}
