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
struct Pair<T, U> {
    first: T,
    second: U,
}

fn test() -> i32 {
    let t: Te;
    t = Te { a: 7, b: 8 };
    let p: Pair<i32, bool> = Pair { first: 128, second: true };
    let p2 = Pair { first: false, second: true };
    let k = 5;
    let pair = (128, true);
    let unkonwn = (128, true, 1.0);
    let l: [i32; 3] = [1, 2, 3];
    let o = 0..1;

    return 5;
}
