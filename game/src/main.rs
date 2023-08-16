use tch::*;
mod env;

pub struct Attacker {
    hidden_state: Vec<Tensor>,
    module: Box<dyn nn::Module>,
    output: Box<dyn nn::Module>,
}

impl Attacker {
    // an rnn like structure, should change to something fancy in the future
    fn new(d: Device, p: nn::Path) -> Self {
        let hidden_state = vec![Tensor::rand([128, ], (Kind::Double, d))];
        let module = Box::new(nn::seq()
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu()))
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu())));
        let output = Box::new(nn::seq()
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu()))
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu())));
        Self { hidden_state, module, output }
    }
}

pub struct Defender {
    hidden_state: Vec<Tensor>,
    module: Box<dyn nn::Module>,
    output: Box<dyn nn::Module>,
}

impl Defender {
    // an rnn like structure, should change to something fancy in the future
    fn new(d: Device, p: nn::Path) -> Self {
        let hidden_state = vec![Tensor::rand([128, ], (Kind::Double, d))];
        let module = Box::new(nn::seq()
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu()))
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu())));
        let output = Box::new(nn::seq()
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu()))
            .add(nn::linear(p.clone(), 128, 128, nn::LinearConfig::default()))
            .add(nn::func(|xs| xs.relu())));
        Self { hidden_state, module, output }
    }
}

fn main() {
    let d = Device::cuda_if_available();
    let t = Tensor::from_slice(&[3, 1, 4, 1, 5]).to(d);
    let t = t * 2;
    t.print();
}
