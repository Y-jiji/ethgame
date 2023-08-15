use tch::*;
mod env;

fn main() {
    let d = Device::cuda_if_available();
    let t = Tensor::from_slice(&[3, 1, 4, 1, 5]).to(d);
    let t = t * 2;
    t.print();
}
