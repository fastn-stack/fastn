// borrowed from https://github.com/QnnOkabayashi/tracing-forest/ (license: MIT)

pub fn write_immediate<S>(
    event: &fastn_observer::Event,
    _current: Option<&tracing_subscriber::registry::SpanRef<S>>,
) -> std::io::Result<()>
where
    S: for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    dbg!(event);
    Ok(())
}

type IndentVec = smallvec::SmallVec<[Indent; 32]>;

use ansi_term::Color;

/// Format logs for pretty printing.
///
/// # Examples
///
/// An arbitrarily complex example:
/// ```log
/// INFO     try_from_entry_ro [ 324µs | 8.47% / 100.00% ]
/// INFO     ┝━ server::internal_search [ 296µs | 19.02% / 91.53% ]
/// INFO     │  ┝━ ｉ [filter.info]: Some filter info...
/// INFO     │  ┝━ server::search [ 226µs | 10.11% / 70.01% ]
/// INFO     │  │  ┝━ be::search [ 181µs | 6.94% / 55.85% ]
/// INFO     │  │  │  ┕━ be::search -> filter2idl [ 158µs | 19.65% / 48.91% ]
/// INFO     │  │  │     ┝━ be::idl_arc_sqlite::get_idl [ 20.4µs | 6.30% ]
/// INFO     │  │  │     │  ┕━ ｉ [filter.info]: Some filter info...
/// INFO     │  │  │     ┕━ be::idl_arc_sqlite::get_idl [ 74.3µs | 22.96% ]
/// ERROR    │  │  │        ┝━ 🚨 [admin.error]: On no, an admin error occurred :(
/// DEBUG    │  │  │        ┝━ 🐛 [debug]: An untagged debug log
/// INFO     │  │  │        ┕━ ｉ [admin.info]: there's been a big mistake | alive: false | status: "very sad"
/// INFO     │  │  ┕━ be::idl_arc_sqlite::get_identry [ 13.1µs | 4.04% ]
/// ERROR    │  │     ┝━ 🔐 [security.critical]: A security critical log
/// INFO     │  │     ┕━ 🔓 [security.access]: A security access log
/// INFO     │  ┕━ server::search<filter_resolve> [ 8.08µs | 2.50% ]
/// WARN     │     ┕━ 🚧 [filter.warn]: Some filter warning
/// TRACE    ┕━ 📍 [trace]: Finished!
/// ```
#[derive(Debug)]
pub struct Pretty;

impl Pretty {
    pub fn fmt(&self, tree: &fastn_observer::Tree) -> Result<String, std::fmt::Error> {
        let mut writer = String::with_capacity(256);

        Pretty::format_tree(tree, None, &mut IndentVec::new(), &mut writer)?;

        Ok(writer)
    }
}

impl Pretty {
    fn format_tree(
        tree: &fastn_observer::Tree,
        duration_root: Option<f64>,
        indent: &mut IndentVec,
        writer: &mut String,
    ) -> std::fmt::Result {
        match tree {
            fastn_observer::Tree::Event(event) => {
                Pretty::format_shared(&event.shared, writer)?;
                Pretty::format_indent(indent, writer)?;
                Pretty::format_event(event, writer)
            }
            fastn_observer::Tree::Span(span) => {
                Pretty::format_shared(&span.shared, writer)?;
                Pretty::format_indent(indent, writer)?;
                Pretty::format_span(span, duration_root, indent, writer)
            }
        }
    }

    fn format_shared(shared: &fastn_observer::Shared, writer: &mut String) -> std::fmt::Result {
        use std::fmt::Write;

        write!(writer, "{:<8} ", ColorLevel(shared.level))
    }

    fn format_indent(indent: &[Indent], writer: &mut String) -> std::fmt::Result {
        use std::fmt::Write;

        for indent in indent {
            writer.write_str(indent.repr())?;
        }
        Ok(())
    }

    fn format_event(event: &fastn_observer::Event, writer: &mut String) -> std::fmt::Result {
        use std::fmt::Write;

        // write!(writer, "{} [{}]: ", tag.icon(), tag)?;

        if let Some(ref message) = event.message {
            writer.write_str(message)?;
        }

        for field in event.shared.fields.iter() {
            write!(
                writer,
                " | {} {}: {}",
                fastn_observer::DurationDisplay(event.shared.on.as_nanos() as f64),
                field.key(),
                field.value()
            )?;
        }

        writeln!(writer)
    }

    fn format_span(
        span: &fastn_observer::Span,
        duration_root: Option<f64>,
        indent: &mut IndentVec,
        writer: &mut String,
    ) -> std::fmt::Result {
        use std::fmt::Write;

        let total_duration = span.duration.as_nanos() as f64;
        let root_duration = duration_root.unwrap_or(total_duration);

        write!(
            writer,
            "{} {} [ {} ] ",
            fastn_observer::DurationDisplay(span.shared.on.as_nanos() as f64),
            span.name,
            fastn_observer::DurationDisplay(total_duration)
        )?;

        for (n, field) in span.shared.fields.iter().enumerate() {
            write!(
                writer,
                "{} {}: {}",
                if n == 0 { "" } else { " |" },
                field.key(),
                field.value()
            )?;
        }
        writeln!(writer)?;

        if let Some((last, remaining)) = span.nodes.split_last() {
            match indent.last_mut() {
                Some(edge @ Indent::Turn) => *edge = Indent::Null,
                Some(edge @ Indent::Fork) => *edge = Indent::Line,
                _ => {}
            }

            indent.push(Indent::Fork);

            for tree in remaining {
                if let Some(edge) = indent.last_mut() {
                    *edge = Indent::Fork;
                }
                Pretty::format_tree(tree, Some(root_duration), indent, writer)?;
            }

            if let Some(edge) = indent.last_mut() {
                *edge = Indent::Turn;
            }
            Pretty::format_tree(last, Some(root_duration), indent, writer)?;

            indent.pop();
        }

        Ok(())
    }
}

enum Indent {
    Null,
    Line,
    Fork,
    Turn,
}

impl Indent {
    fn repr(&self) -> &'static str {
        match self {
            Self::Null => "   ",
            Self::Line => "│  ",
            Self::Fork => "┝━ ",
            Self::Turn => "┕━ ",
        }
    }
}

// From tracing-tree
struct ColorLevel(tracing::Level);

impl std::fmt::Display for ColorLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self.0 {
            tracing::Level::TRACE => Color::Purple,
            tracing::Level::DEBUG => Color::Blue,
            tracing::Level::INFO => Color::Green,
            tracing::Level::WARN => Color::RGB(252, 234, 160), // orange
            tracing::Level::ERROR => Color::Red,
        };
        let style = color.bold();
        write!(f, "{}", style.prefix())?;
        f.pad(self.0.as_str())?;
        write!(f, "{}", style.suffix())
    }
}
