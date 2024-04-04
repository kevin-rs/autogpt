#[cfg(feature = "mail")]
use autogpt::agents::mailer::MailerGPT;
use tracing::debug;
use tracing_subscriber::{filter, fmt, prelude::*, reload};
use autogpt::traits::functions::Functions;
use autogpt::common::utils::Scope;
use autogpt::common::utils::Tasks;

#[tokio::test]
#[cfg(feature = "mail")]
async fn test_mailer_gpt() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Expertise at summarizing emails";
    let request = "Summarize the content of the 5 recent email messages";
    let position = "Mailer";

    let mut mailer = MailerGPT::new(objective, position).await;
    let mut tasks = Tasks {
        description: request.into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    let _ = mailer.execute(&mut tasks, true, 3).await;
}
