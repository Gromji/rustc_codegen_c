// allow unused vars

#![allow(unused_variables)]
fn main() {
    let _a = 1;

    let a = test();
    let b = _a;
}


fn test() -> i32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let pair = (128, true);
    let pair = (128, 'c');
    
    let d = a + b + c;

    return d;
}