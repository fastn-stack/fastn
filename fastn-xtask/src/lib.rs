pub mod template;

pub mod core {
    use crate::template;
    pub async fn new_app(app_name: &str) -> fastn_core::Result<()> {
        template::run_template_command(app_name).await
    }
}

