pub enum ControlFlow {
    Exit,
    Wait,
    WaitUntil(std::time::Instant),
}
