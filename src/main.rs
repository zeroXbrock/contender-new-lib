use alloy_node_bindings::WEI_IN_ETHER;
use contender_core::{
    CancellationToken, Contender, ContenderCtx, RunOpts,
    alloy::{
        network::AnyNetwork,
        providers::{DynProvider, ProviderBuilder},
    },
    generator::{
        FunctionCallDefinition, RandSeed,
        agent_pools::{AgentPools, AgentSpec},
        types::SpamRequest,
    },
    spammer::{LogCallback, TimedSpammer},
};
use contender_report::command::report;
use contender_testfile::TestConfig;
use std::{sync::Arc, time::Duration};

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

    let provider = DynProvider::new(
        ProviderBuilder::new()
            .network::<AnyNetwork>()
            .connect_http(ctx.rpc_url.to_owned()),
    );
    let mut contender = Contender::new(ctx);

    // allows us to cancel result collection whenever we call `cancel_token.cancel()`
    let cancel_token: CancellationToken = Default::default();

    // LogCallback saves tx data to DB
    let callback = LogCallback::new(Arc::new(provider), None, false, cancel_token);

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
