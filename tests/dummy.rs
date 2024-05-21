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
struct Te {
    a: i32,
    b: i32,
    pair: Pair<i32, bool>,
}
struct Pair<T, U> {
    first: T,
    second: U,
}

fn test() -> i32 {
    let t: Te;
    t = Te { a: 7, b: 8, pair: Pair { first: 128, second: true } };
    let p: Pair<i32, bool> = Pair { first: 128, second: true };
    let p2 = Pair { first: false, second: true };
    let k = 5;
    let pair = (128, t);
    let unkonwn = (128, true, 1.0);
    let l: [i32; 3] = [1, 2, 3];
    let o = 0..1;

    return 5;
}

fn add_many(x: i32) -> i32 {
    let mut b = x;

    while b < b * b {
        b += 3;
    }

    return x;
}
