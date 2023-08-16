use revm::{primitives::{ShanghaiSpec, Bytes, B160, U256, AccountInfo}, interpreter::{Interpreter, CallInputs, Transfer, CallContext, StuckReason, InstructionResult, Gas, CreateInputs, return_ok}, CallResult, CreateResult, DatabaseCommit};

pub struct GameEnvironment<'a> {
    executor: revm::EVMImpl<'a, ShanghaiSpec, revm::InMemoryDB>,
    interpreters: Vec<InterpreterSlot>,
    stuck_state: StuckState,
    pub attacker_account: revm::primitives::B160,
    pub defender_account: revm::primitives::B160,
}

pub enum InterpreterSlot {
    // the return len to return to when this is popped
    Fake{call_inputs: Box<CallInputs>, return_len: usize, return_offset: usize},
    Interpreter(Box<CallInputs>, Box<Interpreter>),
}

#[derive(Debug)]
pub enum StuckState {
    MoveAttacker,
    CallAttacker{call_inputs: Box<CallInputs>, return_len: usize, return_offset: usize},
    PrepareAttackerReturn{call_inputs: Box<CallInputs>, return_len: usize, return_offset: usize},
    CallDefender{call_inputs: Box<CallInputs>, return_len: usize, return_offset: usize},
    SomeoneReturn{result: CallResult, return_len: usize, return_offset: usize},
    Noop,
}

impl<'a> GameEnvironment<'a> {
    pub fn new(
        env: &'a mut revm::primitives::Env,
        db: &'a mut revm::InMemoryDB,
        attacker_account: revm::primitives::B160,
        attacker_balance: revm::primitives::U256,
        contract_deployment_code: Bytes,
    ) -> Self {
        use revm::EVMImpl;
        use revm::primitives::*;
        env.tx.caller = B160::zero();
        env.tx.transact_to = TransactTo::Call(B160::zero());
        env.tx.data = Bytes::default();
        env.tx.value = U256::ZERO;
        let mut this = GameEnvironment {
            stuck_state: StuckState::MoveAttacker,
            attacker_account,
            defender_account: B160::zero(),
            executor: EVMImpl::new(db, env, revm::precompile::Precompiles::new(revm::precompile::SpecId::BERLIN).clone()),
            interpreters: vec![]
        };
        this.executor.data.db.insert_account_info(B160::zero(), AccountInfo{
            balance: U256::MAX / U256::from(2), nonce: 1,
            code_hash: revm::primitives::KECCAK_EMPTY, code: None,
        });
        let (create_result, _interpreter) = this.executor.create(&CreateInputs{
            caller: B160::zero(),
            scheme: revm::primitives::CreateScheme::Create,
            init_code: contract_deployment_code.clone(),
            value: U256::ZERO,
            gas_limit: 1000000,
        }, None);
        this.defender_account = create_result.created_address.unwrap();
        let code = Bytes::default();
        this.executor.data.db.insert_account_info(this.attacker_account, AccountInfo { balance: attacker_balance, nonce: 1, code_hash: revm::primitives::keccak256(&code), code: None });
        return this;
    }
    fn pop_return(&mut self) {
        let StuckState::SomeoneReturn { result, return_len, return_offset } = 
            std::mem::replace(&mut self.stuck_state, StuckState::Noop) else { panic!() };
        let interpreter = self.interpreters.pop();
        match interpreter {
            None => {}
            Some(InterpreterSlot::Fake { call_inputs, return_len, return_offset }) => {
                self.stuck_state = StuckState::PrepareAttackerReturn { call_inputs, return_len, return_offset };
            }
            Some(InterpreterSlot::Interpreter(inputs, mut interpreter)) => {
                interpreter.stuck_reason = StuckReason::CallReturn(result.result, result.gas, result.return_value, return_len, return_offset);
                self.executor.call(&inputs, Some(interpreter));
            }
        }
    }
    fn attacker_move(&mut self, data: revm::primitives::Bytes, value: revm::primitives::U256, gas_limit: u64) {
        use revm::interpreter::CallScheme;
        self.executor.data.env.tx.data = data.clone();
        let call_inputs = Box::new(CallInputs {
            contract: self.defender_account,
            transfer: Transfer { source: self.attacker_account, target: self.defender_account, value },
            input: data,
            gas_limit,
            context: CallContext {
                caller: self.attacker_account,
                address: self.defender_account,
                code_address: self.defender_account,
                apparent_value: value,
                scheme: CallScheme::CallCode,
            },
            is_static: false
        });
        self.stuck_state = StuckState::CallDefender { call_inputs, return_len: 0, return_offset: 0 }
    }
    fn attacker_prepare_return(&mut self, call_result: CallResult) {
        let StuckState::PrepareAttackerReturn { return_len, return_offset, .. } = std::mem::replace(&mut self.stuck_state, StuckState::Noop) 
            else { panic!() };
        self.stuck_state = StuckState::SomeoneReturn { result: call_result, return_len, return_offset }
    }
    fn attacker_call(&mut self, backcall_inputs: Option<Box<CallInputs>>) {
        let StuckState::CallAttacker { call_inputs, return_len, return_offset } = 
            std::mem::replace(&mut self.stuck_state, StuckState::Noop) else { panic!() };
        if let Some(backcall_inputs) = backcall_inputs {
            self.interpreters.push(InterpreterSlot::Fake{ call_inputs, return_len, return_offset });
            self.stuck_state = StuckState::CallDefender { call_inputs: backcall_inputs, return_len: 0, return_offset: 0 };
        } else {
            self.stuck_state = StuckState::PrepareAttackerReturn { call_inputs, return_len, return_offset };
        }
    }
    fn defender_call(&mut self, pass: bool) {
        let StuckState::CallDefender { call_inputs, return_len, return_offset } = 
            std::mem::replace(&mut self.stuck_state, StuckState::Noop) else { panic!() };
        if !pass {
            self.stuck_state = StuckState::SomeoneReturn{ result: CallResult {
                result: InstructionResult::Revert, 
                gas: Gas::new(0),
                return_value: Bytes::default(),
            }, return_len, return_offset };
            return;
        }
        let (call_result, maybe_interpreter) = self.executor.call(&call_inputs, None);
        if !matches!(call_result.result, InstructionResult::Stuck) {
            self.stuck_state = StuckState::SomeoneReturn { result: call_result, return_len, return_offset };
            return;
        }
        let interpreter = maybe_interpreter.unwrap();
        match &interpreter.stuck_reason {
            StuckReason::Call(call_inputs, return_len, return_offset) => {
                let _call_inputs = call_inputs.clone();
                let return_len = *return_len;
                let return_offset = *return_offset;
                if _call_inputs.contract == self.attacker_account {
                    self.stuck_state = StuckState::CallAttacker { call_inputs: _call_inputs, return_len, return_offset };
                } else {
                    self.stuck_state = StuckState::CallDefender { call_inputs: _call_inputs, return_len, return_offset };
                }
                self.interpreters.push(InterpreterSlot::Interpreter(call_inputs.clone(), interpreter));
            },
            _ => unimplemented!()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use revm::{primitives::{Env, hex, hex_literal::*}, InMemoryDB};
    use ethers::prelude::BaseContract;

    #[test]
    fn deploy_silly_bank() {
        let mut env = Env::default();
        let mut db = InMemoryDB::default();
        let data = hex!("608060405234801561001057600080fd5b5061046a806100206000396000f3fe6080604052600436106100345760003560e01c806327e235e3146100395780633ccfd60b14610076578063d0e30db01461008d575b600080fd5b34801561004557600080fd5b50610060600480360381019061005b91906102ad565b610097565b60405161006d91906102f3565b60405180910390f35b34801561008257600080fd5b5061008b6100af565b005b6100956101f3565b005b60006020528060005260406000206000915090505481565b60008060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050600081116100ff57600080fd5b60003373ffffffffffffffffffffffffffffffffffffffff16826040516101259061033f565b60006040518083038185875af1925050503d8060008114610162576040519150601f19603f3d011682016040523d82523d6000602084013e610167565b606091505b50509050806101ab576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016101a2906103b1565b60405180910390fd5b60008060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055505050565b346000803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008282546102419190610400565b92505081905550565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061027a8261024f565b9050919050565b61028a8161026f565b811461029557600080fd5b50565b6000813590506102a781610281565b92915050565b6000602082840312156102c3576102c261024a565b5b60006102d184828501610298565b91505092915050565b6000819050919050565b6102ed816102da565b82525050565b600060208201905061030860008301846102e4565b92915050565b600081905092915050565b50565b600061032960008361030e565b915061033482610319565b600082019050919050565b600061034a8261031c565b9150819050919050565b600082825260208201905092915050565b7f4661696c656420746f2073656e64204574686572000000000000000000000000600082015250565b600061039b601483610354565b91506103a682610365565b602082019050919050565b600060208201905081810360008301526103ca8161038e565b9050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600061040b826102da565b9150610416836102da565b925082820190508082111561042e5761042d6103d1565b5b9291505056fea264697066735822122037ddd9486011c735edb384438937c3db22ec58f4325d00c7fad440b0007e1d7f64736f6c63430008120033").to_vec();
        let mut game = GameEnvironment::new(&mut env, &mut db, B160::random(), U256::from(10000), data.into());
        let mut json = std::fs::File::open("/home/y-jiji/eth-game/tmp/SillyBank.abi").unwrap();
        let abi: BaseContract = ethers::abi::Abi::load(&mut json).unwrap().into();
        let input = hex::decode(hex::encode(abi.encode("deposit", ()).unwrap())).unwrap();
        println!("input: {}", hex::encode(&input));
        game.attacker_move(input.into(), U256::from(100), 1000000);
        game.defender_call(true);
        println!("====== players ======");
        println!("Attacker: {:?}", game.attacker_account);
        println!("Defender: {:?}", game.defender_account);
        println!("====== stuck state ======");
        println!("{:#?}", game.stuck_state);
        println!("====== journaled state ======");
        println!("{:#?}", game.executor.data.journaled_state);
        println!("====== contracts ======");
        println!("{:#?}", db.contracts);
        println!("====== accounts ======");
        println!("{:#?}", db.accounts);
    }
}