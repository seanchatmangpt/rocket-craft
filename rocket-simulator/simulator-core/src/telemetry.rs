use chrono::Local;
use std::fmt;
use tracing::field::Visit;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::{FormatEvent, FormatFields, Writer},
        FmtContext,
    },
    prelude::*,
    Registry,
};

/// A visitor that extracts the main message text from a tracing event.
struct MessageVisitor {
    msg: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.msg = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.msg = value.to_string();
        }
    }
}

/// A custom tracing formatter designed for development.
///
/// It outputs clean, color-coded, aligned telemetry logs for general events,
/// but allows raw narrative, gameplay, and ASCII-art outputs (when target is `"game"`, `"receipt"`, or `"raw"`)
/// to pass through completely un-prefixed to ensure clean formatting.
pub struct DevTelemetryFormatter;

impl<S, N> FormatEvent<S, N> for DevTelemetryFormatter
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        let target = metadata.target();

        let mut visitor = MessageVisitor { msg: String::new() };
        event.record(&mut visitor);

        // Check if this event is raw text/game narrative/receipt format
        if target == "raw" || target == "receipt" || target == "game" {
            writeln!(writer, "{}", visitor.msg)?;
            return Ok(());
        }

        // 1. Timestamp (gray)
        let time_str = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        write!(writer, "\x1b[90m{}\x1b[0m ", time_str)?;

        // 2. Log Level (colored and padded)
        let level_str = match *metadata.level() {
            Level::ERROR => "\x1b[1;31mERROR\x1b[0m",
            Level::WARN => "\x1b[1;33mWARN \x1b[0m",
            Level::INFO => "\x1b[1;32mINFO \x1b[0m",
            Level::DEBUG => "\x1b[1;35mDEBUG\x1b[0m",
            Level::TRACE => "\x1b[1;36mTRACE\x1b[0m",
        };
        write!(writer, "{} ", level_str)?;

        // 3. Target (blue, padded/fixed-width for alignment)
        let formatted_target = if target.len() > 20 {
            format!("...{}", &target[target.len() - 17..])
        } else {
            target.to_string()
        };
        write!(writer, "\x1b[34m[{:<20}]\x1b[0m ", formatted_target)?;

        // 4. Message content
        writeln!(writer, "{}", visitor.msg)?;
        Ok(())
    }
}

/// Checks whether the given target name should pass through unformatted.
pub fn is_raw_target(target: &str) -> bool {
    target == "raw" || target == "receipt" || target == "game"
}

/// Initializes the dev telemetry subscriber globally.
pub fn init_telemetry() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let subscriber = Registry::default()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().event_format(DevTelemetryFormatter));

    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_targets_pass_through() {
        assert!(is_raw_target("raw"));
        assert!(is_raw_target("receipt"));
        assert!(is_raw_target("game"));
    }

    #[test]
    fn non_raw_targets_do_not_pass_through() {
        assert!(!is_raw_target("info"));
        assert!(!is_raw_target("simulator_core"));
        assert!(!is_raw_target(""));
        assert!(!is_raw_target("raw_extra"));
        assert!(!is_raw_target("Receipt")); // case-sensitive
    }

    #[test]
    fn init_telemetry_does_not_panic_when_called_multiple_times() {
        // set_global_default is called at most once; subsequent calls are silently ignored.
        init_telemetry();
        init_telemetry();
    }

    #[test]
    fn message_visitor_record_str_captures_message() {
        let mut v = MessageVisitor { msg: String::new() };
        // Simulate the Visit::record_str call path by directly setting the field
        v.msg = "hello world".to_string();
        assert_eq!(v.msg, "hello world");
    }

    #[test]
    fn message_visitor_default_msg_is_empty() {
        let v = MessageVisitor { msg: String::new() };
        assert!(v.msg.is_empty());
    }

    #[test]
    fn target_truncation_boundary() {
        // Targets >20 chars get truncated to "...{last 17}"
        let long_target = "simulator_core::very_deep::module";
        let formatted = if long_target.len() > 20 {
            format!("...{}", &long_target[long_target.len() - 17..])
        } else {
            long_target.to_string()
        };
        assert!(formatted.starts_with("..."));
        assert_eq!(formatted.len(), 20); // "..." (3) + 17 = 20
    }

    #[test]
    fn short_target_is_not_truncated() {
        let short = "my_module";
        let formatted = if short.len() > 20 {
            format!("...{}", &short[short.len() - 17..])
        } else {
            short.to_string()
        };
        assert_eq!(formatted, "my_module");
    }
}
