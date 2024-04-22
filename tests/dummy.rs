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
    
    let d = a + b + c;

    return d;
}