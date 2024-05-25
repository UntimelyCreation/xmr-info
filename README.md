# xmr-info

A small command line utility to fetch various information related to the Monero cryptocurrency (XMR).


## Features

`xmr-info` currently has the following features:

- **Convert** an amount of XMR to the corresponding amount in another currency.
- **Notify** the user when the [P2Pool](https://p2pool.io/) network has mined a block, through a desktop notification (run as a background service).


## Usage

### Run

```bash
cargo run --release -- <COMMAND>
```

To see a list of available commands and their syntax, use these commands:

```bash
cargo run --release -- --help
cargo run --release -- <COMMAND> --help
```


## License

Distributed under the MIT License.
