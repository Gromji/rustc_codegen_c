// allow unused vars

#![allow(unused_variables)]
fn main() {
    let _a = 1;

    let a = test();
    let b = _a;
}
struct Te {
    a: i32,
    b: i32,
}

fn test() -> i32 {
    // This does not yet work, because we have not yet handle struct implementation!
    // let p: Te;
    // p = Te { a: 7, b: 8 };
    let k = 5;
    let pair = (128, true);
    let l: [i32; 3] = [1, 2, 3];

    return k;
}
