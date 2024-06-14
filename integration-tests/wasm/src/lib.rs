#[ft_sdk::processor]
fn test_route() -> ft_sdk::processor::Result {
    ft_sdk::processor::json(serde_json::json!({
        "ok": true,
    }))
}

#[ft_sdk::processor]
fn misc() -> ft_sdk::processor::Result {
    ft_sdk::processor::json(serde_json::json!({
        "ok": true,
    }))
}
