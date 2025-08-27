use contender_core::{
    Contender, ContenderCtx, RunOpts,
    alloy::node_bindings::WEI_IN_ETHER,
    generator::{
        FunctionCallDefinition, RandSeed,
        agent_pools::{AgentPools, AgentSpec},
        types::SpamRequest,
    },
    spammer::{LogCallback, TimedSpammer},
};
use contender_report::command::report;
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
    let db = contender_sqlite::SqliteDb::from_file("myContender.db")?;
    let seeder = RandSeed::new();
    let agents = config.build_agent_store(&seeder, AgentSpec::default().spam_accounts(2));
    let ctx = ContenderCtx::builder(config, db, seeder, "http://localhost:8545")
        .agent_store(agents)
        .funding(WEI_IN_ETHER) // send 1 ETH
        .build();
    let scenario = ctx.build_scenario().await?;

    let mut contender = Contender::new(ctx);

    // LogCallback saves tx data to DB
    let callback = LogCallback::new(scenario.rpc_client);

    // run spammer
    contender
        .spam(
            TimedSpammer::new(Duration::from_secs(1)),
            callback.into(),
            RunOpts::new()
                .txs_per_period(100)
                .periods(2)
                .name("SimpleSample"),
        )
        .await?;

    report(None, 0, &*scenario.db, "./reports").await?;

    Ok(())
}
