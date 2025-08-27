# custom contender spammer example

first, run a node:

```sh
anvil -b 1
```

then run the spammer:

```sh
cargo run
```

It will spam your node with 100 txs/sec for 20 seconds, then generate a report and open it in your web browser.

Check out the [source code](./src/main.rs) to see how it works!

