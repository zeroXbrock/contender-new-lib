use contender_core::{
    Contender, ContenderCtx, RunOpts,
    alloy::node_bindings::WEI_IN_ETHER,
    generator::{
        FunctionCallDefinition, RandSeed,
        agent_pools::{AgentPools, AgentSpec},
        types::SpamRequest,
    },
    spammer::{LogCallback, NilCallback, TimedSpammer},
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

    run_simple(&config).await?;
    run_with_db_and_report(&config).await?;

    Ok(())
}

/// Run a simple contender spammer with no callback, no DB, and no report.
///
/// Reports require a working DB, as do contract deployments, but if you don't need those,
/// this is a very quick and convenient way to start a minimalistic spammer.
async fn run_simple(config: &TestConfig) -> Result<(), Box<dyn std::error::Error>> {
    // setup contender w/ default settings, may be overridden
    let ctx = ContenderCtx::builder_simple(config.to_owned(), "http://localhost:8545").build();
    let mut contender = Contender::new(ctx);

    // run spammer
    contender
        .spam(
            TimedSpammer::new(Duration::from_secs(1)),
            NilCallback.into(),
            RunOpts::new().txs_per_period(100).periods(4),
        )
        .await?;

    Ok(())
}

/// Run a contender spammer with a working DB (saved at `./myContender.db`), enabling contract deployments,
/// data persistence across spam runs, and reports.
///
/// This function generates a report when it's finished spamming, and saves it under `./reports/reports/`.
async fn run_with_db_and_report(config: &TestConfig) -> Result<(), Box<dyn std::error::Error>> {
    // make a seed to generate accounts; this may be saved to file and reused to generate the same accounts
    let seeder = RandSeed::new();
    // generate only 2 spam accounts (default is 10)
    let agents = config.build_agent_store(&seeder, AgentSpec::default().spam_accounts(2));

    // build contender context w/ modified agents & seeder
    let ctx = contender_sqlite::ctx_builder_filedb(
        config.to_owned(),
        "myContender.db",
        "http://localhost:8545",
    )?
    // alternatively, use an in-memory DB if you don't need to persist data longer than your program's lifetime:
    // let ctx = contender_sqlite::ctx_builder_memdb(config.to_owned(), "http://localhost:8545")
    .seeder(seeder)
    .agent_store(agents)
    .funding(WEI_IN_ETHER) // send 1 ETH
    .build();

    // build a TestScenario so we can use its rpc client & db
    let scenario = ctx.build_scenario().await?;

    // create a Contender instance, consuming ctx & locking in the scenario
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
                .periods(4)
                .name("SimpleSample"),
        )
        .await?;

    report(None, 0, &*scenario.db, "./reports").await?;

    Ok(())
}
