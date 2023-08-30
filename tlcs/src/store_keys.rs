use gears::x::params::ParamsSubspaceKey;
use store::StoreKey;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum TlcsStoreKey {
    Bank,
    Auth,
    Params,
    Timelock,
}

/// WARNING: a key name must not be a prefix of another, there is currently
/// no check in the SDK to prevent this.
impl StoreKey for TlcsStoreKey {
    fn name(&self) -> &'static str {
        match self {
            TlcsStoreKey::Bank => "bank",
            TlcsStoreKey::Auth => "acc",
            TlcsStoreKey::Params => "params",
            TlcsStoreKey::Timelock => "timelock",
        }
    }
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
pub enum TlcsParamsStoreKey {
    Bank,
    Auth,
    BaseApp,
}

/// WARNING: a key name must not be a prefix of another, there is currently
/// no check in the SDK to prevent this.
impl ParamsSubspaceKey for TlcsParamsStoreKey {
    fn name(&self) -> &'static str {
        match self {
            Self::Bank => "bank/",
            Self::Auth => "auth/",
            Self::BaseApp => "baseapp/",
        }
    }
}
