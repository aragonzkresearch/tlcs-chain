# TLCS

This repo contains node software for running a Blockchain which provides a Time Lock Cryptography Service.

NOTE: This is a WIP, e.g. it does not contain the necessary CLI commands and REST endpoints to interact with
the TLCS parts of the chain!

# Getting Started

TLCS uses the [tendermint-abci](https://crates.io/crates/tendermint-abci) crate to communicate with a Tendermint instance which runs as a separate process. This means that to run a full node, Tendermint must be installed and run separately (see instructions below).

## Requirements

**Rust compiler**

The minimum supported Rust version is 1.67.1. Follow the [installation instructions](https://doc.rust-lang.org/book/ch01-01-installation.html).

**Tendermint**

To install Tendermint, clone the [Tendermint repo](https://github.com/tendermint/tendermint) checkout v0.34.21 then follow the [installation instructions](https://github.com/tendermint/tendermint/blob/v0.34.21/docs/introduction/install.md).

**libclang**

This is needed by the rocks db crate, run `sudo apt install libclang-dev build-essential`.

## Running a local chain

1. Clone this repo:

```console
git clone https://github.com/aragonzkresearch/tlcs-chain
cd tlcs-chain
```

1.5 Install TLCS-C binaries

```console
git clone https://github.com/aragonzkresearch/tlcs-c
follow instruction in README
---more instructions to come
```

2. Initialize a new chain:

```console
make init
```

3. Build and start the application:

```console
make run
```

The application will listen for connections on tcp://127.0.0.1:26658.

4. From a different terminal window start Tendermint:

```console
make tendermint-start
```

Tendermint will connect to the application and bind it's RPC server to 127.0.0.1:26657.

The chain (consisting of one node) is now up and running.


## Querying the chain

So far we've been running tlcs indirectly using make commnds and the rust build tool, Cargo. In the next
section we'll install tlcs and use it to query the chain (just like cosmos-sdk chains the tlcs binary serves as a
node and client).

1. Install tlcs:

```console
make install
```

2. Query a balance:

```console
tlcs query bank balances cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
```

Which returns:

```json
{
  "balances": [
    {
      "denom": "uatom",
      "amount": "34"
    }
  ],
  "pagination": null
}
```

The balance of this address was set to 34 in the genesis file.

3. Import the key corresponding to the above address into the tlcs key store:

```console
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | tlcs keys add kevin --recover
echo "all victory hero talent forget twice quote you office vacant sleep kangaroo disorder scorpion humble gorilla coast pudding edge garlic bid dutch excuse magic" | tlcs keys add alice --recover
```

4. Send tokens:

```console
tlcs tx kevin bank send cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut 10uatom --fee 1uatom
```

5. Query the address balance and observe that it has decreased by 11uatom which is the sum of the amount transferred and the fee:

```console
tlcs query bank balances cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
```

Which returns:

```json
{
  "balances": [
    {
      "denom": "uatom",
      "amount": "23"
    }
  ],
  "pagination": null
}

```

6. Submit participant data:

```console
tlcs tx tlcs request_keypair kevin 100 1 1692800327 
```

```console
tlcs tx tlcs contribute kevin 100 1 0
```

```console
tlcs query tlcs contributions_by_round 100
```

