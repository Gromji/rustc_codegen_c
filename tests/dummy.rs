// allow unused vars

#![allow(unused_variables)]
fn main() {
    let _a = 1;

    let a = test();
    let b = _a;
}

fn test() -> i32 {
    let a = 1 + 2;
    let b = 2;
    let c = 3;
    let pair = (128, true);
    let l: [i32; 3] = [1, 2, 3];

    let d = a + b + c;

    return d;
}
