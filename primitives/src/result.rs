use ruint::aliases::U256;
use std::fmt;

/// When transaction return successfully without halts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Eval {
    Stop,
    Return,
    SelfDestruct,
}

/// Indicates that the EVM has experienced an exceptional halt. This causes execution to
/// immediately end with all gas being consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Halt {
    OutOfGas(OutOfGasError),
    OpcodeNotFound,
    InvalidFEOpcode,
    InvalidJump,
    NotActivated,
    StackUnderflow,
    StackOverflow,
    OutOfOffset,
    CreateCollision,
    PrecompileError,
    NonceOverflow,
    /// Create init code size exceeds limit (runtime).
    CreateContractSizeLimit,
    /// Error on created contract that begins with EF
    CreateContractStartingWithEF,
    /// EIP-3860: Limit and meter initcode. Initcode size limit exceeded.
    CreateInitcodeSizeLimit,

    /* Internal Halts that can be only found inside Inspector */
    OverflowPayment,
    StateChangeDuringStaticCall,
    CallNotAllowedInsideStatic,
    OutOfFund,
    CallTooDeep,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OutOfGasError {
    // Basic OOG error
    BasicOutOfGas,
    // Tried to expand past REVM limit
    MemoryLimit,
    // Basic OOG error from memory expansion
    Memory,
    // Precompile threw OOG error
    Precompile,
    // When performing something that takes a U256 and casts down to a u64, if its too large this would fire
    // i.e. in `as_usize_or_fail`
    InvalidOperand,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EVMError<DBError> {
    Transaction(InvalidTransaction),
    /// REVM specific and related to environment.
    PrevrandaoNotSet,
    Database(DBError),
}

#[cfg(feature = "std")]
impl<DBError> std::error::Error for EVMError<DBError> where Self: fmt::Debug + fmt::Display {}

impl<DBError> fmt::Display for EVMError<DBError>
where
    DBError: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EVMError::Transaction(v) => write!(f, "Transaction error: {:?}", v),
            EVMError::PrevrandaoNotSet => f.write_str("Prevrandao not set"),
            EVMError::Database(v) => write!(f, "Database error: {}", v),
        }
    }
}

impl<DBError> From<InvalidTransaction> for EVMError<DBError> {
    fn from(invalid: InvalidTransaction) -> Self {
        EVMError::Transaction(invalid)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidTransaction {
    GasMaxFeeGreaterThanPriorityFee,
    GasPriceLessThanBasefee,
    CallerGasLimitMoreThanBlock,
    CallGasCostMoreThanGasLimit,
    /// EIP-3607 Reject transactions from senders with deployed code
    RejectCallerWithCode,
    /// Transaction account does not have enough amount of ether to cover transferred value and gas_limit*gas_price.
    LackOfFundForMaxFee {
        fee: u64,
        balance: U256,
    },
    /// Overflow payment in transaction.
    OverflowPaymentInTransaction,
    /// Nonce overflows in transaction.
    NonceOverflowInTransaction,
    NonceTooHigh {
        tx: u64,
        state: u64,
    },
    NonceTooLow {
        tx: u64,
        state: u64,
    },
    /// EIP-3860: Limit and meter initcode
    CreateInitcodeSizeLimit,
    InvalidChainId,
    /// Access list is not supported is not supported
    /// for blocks before Berlin hardfork.
    AccessListNotSupported,
}
