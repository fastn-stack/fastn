pub enum ControlFlow {
    Exit,
    WaitForEvent,
    WaitForEventTill(std::time::Instant),
}
