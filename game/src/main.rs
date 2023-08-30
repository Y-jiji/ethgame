use tch::*;
use ethers::prelude::BaseContract;
use revm::primitives::bytes::BufMut;
mod env;

#[derive(Debug)]
pub struct Attacker {
    state: Vec<tch::Tensor>,
    w1: tch::Tensor,
    b1: tch::Tensor,
    w2: tch::Tensor,
    b2: tch::Tensor,
    w3: tch::Tensor,
    b3: tch::Tensor,
    w4: tch::Tensor,
    b4: tch::Tensor,
    abi: BaseContract,
    functions: Vec<String>,
    inputs_bytes: Vec<usize>,
    value_list: Vec<usize>,
}

impl Attacker {
    pub fn new(h: usize, d: Device, abi: BaseContract, functions: Vec<String>, inputs_bytes: Vec<usize>, value_list: Vec<usize>) -> Self {
        let h = h as i64;
        let opt = (Kind::Float, d);
        Self {
            state: vec![tch::Tensor::zeros([h], opt)],
            w1: tch::Tensor::randn([h, h], opt),
            b1: tch::Tensor::randn([h], opt),
            w2: tch::Tensor::randn([h, 2], opt),
            b2: tch::Tensor::randn([2], opt),
            w3: tch::Tensor::randn([h, 256], opt),
            b3: tch::Tensor::randn([256], opt),
            w4: tch::Tensor::randn([h, value_list.len() as i64], opt),
            b4: tch::Tensor::randn([value_list.len() as i64], opt),
            value_list,
            functions, abi, inputs_bytes,
        }
    }
    fn get_one_byte(&mut self) -> (u8, tch::Tensor) {
        let x0 = self.state.last().unwrap();
        let ps = (x0.matmul(&self.w3) + &self.b3).softmax(-1, None);
        let mut size = ps.size(); *size.last_mut().unwrap() = 1;
        let p = tch::Tensor::rand(size, (ps.kind(), ps.device()));
        let which = ((ps.cumsum(-1, None) - p).ge(0.0) * 1).sum_dim_intlist(-1, true, None) - 1;
        let which = which.f_int64_value(&[0]).unwrap();
        let x1 = x0.matmul(&self.w1) + &self.b1;
        self.state.push(x1);
        (which as u8, ps.slice(-1, which, which+1, 1))
    }
    fn get_value(&mut self) -> (usize, tch::Tensor) {
        let x0 = self.state.last().unwrap();
        let ps = (x0.matmul(&self.w4) + &self.b4).softmax(-1, None);
        let mut size = ps.size(); *size.last_mut().unwrap() = 1;
        let p = tch::Tensor::rand(size, (ps.kind(), ps.device()));
        let which = ((ps.cumsum(-1, None) - p).ge(0.0) * 1).sum_dim_intlist(-1, true, None) - 1;
        let which = which.f_int64_value(&[0]).unwrap();
        let x1 = x0.matmul(&self.w1) + &self.b1;
        self.state.push(x1);
        (self.value_list[which as usize], ps.slice(-1, which, which+1, 1))
    }
    // return bytes and a tensor representing probability
    pub fn fill_data(&mut self) -> (revm::primitives::Bytes, usize, tch::Tensor) {
        let x0 = self.state.last().unwrap();
        let kind = x0.kind();
        let device = x0.device();
        // select a function
        let ps = (x0.matmul(&self.w2) + &self.b2).softmax(-1, None);
        let mut size = ps.size(); *size.last_mut().unwrap() = 1;
        let p = tch::Tensor::rand(size, (kind, device));
        let which = ((ps.cumsum(-1, None) - p).ge(0.0) * 1).sum_dim_intlist(-1, true, None) - 1;
        let which = which.f_int64_value(&[0]).unwrap();
        let selection_p = ps.slice(-1, which, which+1, 1).squeeze();
        // get the number of required bytes
        let byte_len = self.inputs_bytes[which as usize];
        // get inputs
        let call_inputs = (0..byte_len).map(|_| self.get_one_byte()).fold((vec![], tch::Tensor::ones(1, (kind, device))), 
            |(mut a0, mut a1), (b, c)| {
                a0.push(b);
                a1 *= c;
                (a0, a1)
            }
        );
        // fill bytes and form the output
        let signature = self.abi.abi().function(&self.functions[which as usize]).unwrap().short_signature();
        let mut input_bytes = vec![];
        input_bytes.put_slice(&signature);
        input_bytes.put_slice(&call_inputs.0);
        let value = self.get_value();
        (revm::primitives::Bytes::from(input_bytes), value.0, call_inputs.1 * selection_p * value.1)
    }
}

fn main() {
    let mut json = std::fs::File::open("/home/y-jiji/eth-game/tmp/SillyBank.abi").unwrap();
    let abi: BaseContract = ethers::abi::Abi::load(&mut json).unwrap().into();
    let mut attacker = Attacker::new(32, Device::Cuda(0), abi, vec!["deposit".to_string(), "withdraw".to_string()], vec![0, 0], vec![0, 1000]);
    let data = attacker.fill_data();
    println!("{:?}", data);
}