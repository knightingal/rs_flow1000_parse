use std::{env, fmt, str::FromStr};
use tracing::{Event, Level, Subscriber, level_filters::LevelFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{Layer, filter::filter_fn, fmt::{
    FmtContext, FormattedFields, format::{self, FormatEvent, FormatFields},
}, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::registry::LookupSpan;

pub struct MyFormatter;

pub struct SqlFormatter;


impl<S, N> FormatEvent<S, N> for MyFormatter
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

        writeln!(writer)
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

        writeln!(writer)
    }
}

pub fn log_init() {

  let rust_log_env = env::var("RUST_LOG")
    .unwrap_or_else(|_| String::from("INFO"));
  let sql_appender = RollingFileAppender::new(Rotation::NEVER, "./", "sql.log");
  let (sql_blocking, _guard) = tracing_appender::non_blocking(sql_appender);
  let sql_layer = tracing_subscriber::fmt::layer()
    .event_format(SqlFormatter)
    .with_writer(sql_blocking)
    .with_filter(filter_fn(|metadata| {
      metadata.target() == "sql"
    }));

  let (std_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
  let std_layer = tracing_subscriber::fmt::layer()
    .with_writer(std_blocking)
    .with_filter(
      LevelFilter::from_level(
        Level::from_str(&rust_log_env).unwrap()
      )
    );

  tracing_subscriber::registry()
    .with(sql_layer)
    .with(std_layer)
    .init();
}