#![cfg_attr(not(feature = "std"), no_std)]

pub mod db;
mod evm;
mod evm_impl;
mod result;
mod journaled_state;

#[cfg(all(feature = "with-serde", not(feature = "serde")))]
compile_error!("`with-serde` feature has been renamed to `serde`.");

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
pub type DummyStateDB = InMemoryDB;

pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use evm::{evm_inner, new, EVM};
pub use result::{ResultAndState, ExecutionResult};
pub use evm_impl::{EVMData, EVMImpl, Transact, CallResult, CreateResult};
pub use journaled_state::{JournalEntry, JournaledState};

extern crate alloc;

// reexport `revm_precompiles`
#[doc(inline)]
pub use revm_precompile as precompile;

// reexport `revm_interpreter`
#[doc(inline)]
pub use revm_interpreter as interpreter;

// reexport `revm_primitives`
#[doc(inline)]
pub use revm_interpreter::primitives;