#[macro_export]
macro_rules! agents {
    ( $($agent:expr),* $(,)? ) => {
        vec![
            $(
                std::sync::Arc::new(tokio::sync::Mutex::new(Box::new($agent) as Box<dyn AgentFunctions>))
            ),*
        ]
    };
}
