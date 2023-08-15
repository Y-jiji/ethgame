use crate::{primitives::{Log, State, Bytes, EVMError, Eval, Halt}, evm_impl::{CallResult, CreateResult}};
use alloc::vec::Vec;

pub type EVMResult<DBError> = core::result::Result<ResultAndState, EVMError<DBError>>;

#[derive(Debug)]
pub struct ResultAndState {
    /// Status of execution
    pub result: ExecutionResult,
    /// State that got updated
    pub state: State,
}

#[derive(Debug)]
pub enum Output {
    Call(CallResult),
    Create(CreateResult),
    Stuck(Box<crate::interpreter::Interpreter>),
}

impl Output {
    /// Returns the output data of the execution output.
    pub fn into_data(self) -> Bytes {
        match self {
            Output::Call(data) => data.return_value,
            Output::Create(data) => data.return_value,
            Output::Stuck(_) => unreachable!("you cannot call into bytes for something stuck"),
        }
    }

    /// Returns the output data of the execution output.
    pub fn data(&self) -> &Bytes {
        match self {
            Output::Call(data) => &data.return_value,
            Output::Create(data) => &data.return_value,
            Output::Stuck(_) => unreachable!("you cannot find any data for something stuck"),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult {
    /// Returned successfully
    Success {
        reason: Eval,
        gas_used: u64,
        gas_refunded: u64,
        logs: Vec<Log>,
        output: Output,
    },
    /// Reverted by `REVERT` opcode that doesn't spend all gas.
    Revert { gas_used: u64, output: Bytes },
    /// Reverted for various reasons and spend all gas.
    Halt {
        reason: Halt,
        /// Halting will spend all the gas, and will be equal to gas_limit.
        gas_used: u64,
    },
    /// Stuck
    Stuck { interpreter: Box<crate::interpreter::Interpreter> }
}

impl ExecutionResult {
    /// Returns if transaction execution is successful.
    /// 1 indicates success, 0 indicates revert.
    /// https://eips.ethereum.org/EIPS/eip-658
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Return logs, if execution is not successful, function will return empty vec.
    pub fn logs(&self) -> Vec<Log> {
        match self {
            Self::Success { logs, .. } => logs.clone(),
            _ => Vec::new(),
        }
    }

    /// Returns the output data of the execution.
    ///
    /// Returns `None` if the execution was halted.
    pub fn output(&self) -> Option<&Bytes> {
        match self {
            Self::Success { output, .. } => Some(output.data()),
            Self::Revert { output, .. } => Some(output),
            _ => None,
        }
    }

    /// Consumes the type and returns the output data of the execution.
    ///
    /// Returns `None` if the execution was halted.
    pub fn into_output(self) -> Option<Bytes> {
        match self {
            Self::Success { output, .. } => Some(output.into_data()),
            Self::Revert { output, .. } => Some(output),
            _ => None,
        }
    }

    /// Consumes the type and returns logs, if execution is not successful, function will return empty vec.
    pub fn into_logs(self) -> Vec<Log> {
        match self {
            Self::Success { logs, .. } => logs,
            _ => Vec::new(),
        }
    }

    pub fn gas_used(&self) -> u64 {
        let (Self::Success { gas_used, .. }
        | Self::Revert { gas_used, .. }
        | Self::Halt { gas_used, .. }) = self else { panic!() };

        *gas_used
    }
}