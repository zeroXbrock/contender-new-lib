use contender_core::{
    Contender, ContenderCtx, RunOpts,
    generator::{FunctionCallDefinition, types::SpamRequest},
    spammer::{NilCallback, TimedSpammer},
};
use contender_testfile::TestConfig;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    contender_core::util::init_core_tracing(None);

    // setup test scenario
    let config = TestConfig::new().with_spam(vec![SpamRequest::new_tx(
        &FunctionCallDefinition::new("{_sender}") // send tx to self
            .with_from_pool("spammers"),
    )]);

    // setup contender w/ default settings, may be overridden
    let ctx = ContenderCtx::builder_simple(config, "http://localhost:8545").build();
    let mut contender = Contender::new(ctx);

    // run spammer
    contender
        .spam(
            TimedSpammer::new(Duration::from_secs(1)),
            NilCallback.into(),
            RunOpts::new().txs_per_period(100).periods(20),
        )
        .await?;

    Ok(())
}
