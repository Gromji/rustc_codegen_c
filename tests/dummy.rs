// allow unused vars

#![allow(unused_variables)]
fn main() {
    let _a = 1;

    let a = test();

    let a = add_many(a);
    
    if a > 0 {
        let b = 2;
    } else {
        let c = 3;
    }

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


fn add_many(x: i32) -> i32 {
    let mut b = x;
    
    while b < b*b {
        b += 3;
    }

    return x;
}