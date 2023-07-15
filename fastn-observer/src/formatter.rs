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
