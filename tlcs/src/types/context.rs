use database::{PrefixDB, DB};
use tendermint_informal::{abci::Event, block::Header};

use crate::store::{KVStore, MultiStore, Store};

pub struct TxContext<'a, T: DB> {
    pub multi_store: &'a mut MultiStore<T>,
    height: u64,
    pub events: Vec<Event>,
    header: Header,
    _tx_bytes: Vec<u8>,
}

impl<'a, T: DB> TxContext<'a, T> {
    pub fn new(
        multi_store: &'a mut MultiStore<T>,
        height: u64,
        header: Header,
        tx_bytes: Vec<u8>,
    ) -> Self {
        TxContext {
            multi_store,
            height,
            events: vec![],
            header,
            _tx_bytes: tx_bytes,
        }
    }

    pub fn get_header(&self) -> &Header {
        &self.header
    }

    pub fn as_any<'b>(&'b mut self) -> Context<'b, 'a, T> {
        Context::TxContext(self)
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: Store) -> &mut KVStore<PrefixDB<T>> {
        return self.multi_store.get_mutable_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}

pub struct InitContext<'a, T: DB> {
    pub multi_store: &'a mut MultiStore<T>,
    height: u64,
    pub events: Vec<Event>,
    pub chain_id: String,
}

impl<'a, T: DB> InitContext<'a, T> {
    pub fn new(multi_store: &'a mut MultiStore<T>, height: u64, chain_id: String) -> Self {
        InitContext {
            multi_store,
            height,
            events: vec![],
            chain_id,
        }
    }

    pub fn as_any<'b>(&'b mut self) -> Context<'b, 'a, T> {
        Context::InitContext(self)
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: Store) -> &mut KVStore<PrefixDB<T>> {
        return self.multi_store.get_mutable_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}

/// This is used when a method can be used in either a tx or init context
pub enum Context<'a, 'b, T: DB> {
    TxContext(&'a mut TxContext<'b, T>),
    InitContext(&'a mut InitContext<'b, T>),
}

impl<'a, 'b, T: DB> Context<'a, 'b, T> {
    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        match self {
            Context::TxContext(ctx) => return ctx.get_kv_store(store_key),
            Context::InitContext(ctx) => return ctx.multi_store.get_kv_store(store_key),
        }
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: Store) -> &mut KVStore<PrefixDB<T>> {
        match self {
            Context::TxContext(ctx) => return ctx.get_mutable_kv_store(store_key),
            Context::InitContext(ctx) => return ctx.multi_store.get_mutable_kv_store(store_key),
        }
    }

    pub fn get_height(&self) -> u64 {
        match self {
            Context::TxContext(ctx) => ctx.height,
            Context::InitContext(ctx) => ctx.height,
        }
    }

    pub fn get_chain_id(&self) -> &str {
        match self {
            Context::TxContext(ctx) => ctx.header.chain_id.as_str(),
            Context::InitContext(ctx) => &ctx.chain_id,
        }
    }

    pub fn push_event(&mut self, event: Event) {
        match self {
            Context::TxContext(ctx) => ctx.push_event(event),
            Context::InitContext(ctx) => ctx.events.push(event),
        };
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        match self {
            Context::TxContext(ctx) => ctx.append_events(events),
            Context::InitContext(ctx) => ctx.events.append(&mut events),
        }
    }
}

/// A Context which holds an immutable reference to a MultiStore
pub struct QueryContext<'a, T: DB> {
    pub multi_store: &'a MultiStore<T>,
    _height: u64,
}

impl<'a, T: DB> QueryContext<'a, T> {
    pub fn new(multi_store: &'a MultiStore<T>, height: u64) -> Self {
        QueryContext {
            multi_store,
            _height: height,
        }
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    pub fn _get_height(&self) -> u64 {
        self._height
    }
}

// type Context struct {
// 	ctx           context.Context
// 	ms            MultiStore
// 	header        tmproto.Header
// 	headerHash    tmbytes.HexBytes
// 	chainID       string
// 	txBytes       []byte
// 	logger        log.Logger
// 	voteInfo      []abci.VoteInfo
// 	gasMeter      GasMeter
// 	blockGasMeter GasMeter
// 	checkTx       bool
// 	recheckTx     bool // if recheckTx == true, then checkTx must also be true
// 	minGasPrice   DecCoins
// 	consParams    *abci.ConsensusParams
// 	eventManager  *EventManager
// }
