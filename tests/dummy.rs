// allow unused vars

#![allow(unused_variables)]
fn main() {
    let a = "123";
    test_len(a);
    let _a = 1;
    let k1: f32 = 10.1;
    let k2: f32 = 20.0;
    let k3 = k1 + k2;
    let t1: i128 = 100;
    let t2: i128 = 200;
    let t3: i128 = t1 * t2;
    let b1: bool = false;
    let t4 = t2 - t1;
    let i1 = 123;
    let i2 = 321;
    let i3 = i1 - i2;
    let b1 = t1 < t2;
    let b2 = k1 > k2;
    let b3 = b1 && b2;
    let b4 = b1 || b1 || b2;

    let a = test();

    let a = test_enum(TestEnum::A(1, 2, 3));

    let a = test_plain_enum(PlainEnum::A);

    let a = test_plain_union(TestUnion { a: 1 });

    let a = add_many(a);
    let a = test_dyn(&Pair::new(1, true));
    let a = test_dyn(&Pair::new(12, 12));

    let a = test_closure(a, a);

    if a > 0 {
        let b = 2;
    } else {
        let c = 3;
    }

    let b = _a;
    let x = func();
    let y = test_closure_arr(5, -1);
    let z = test_closure_ref(&y);
    let arr = [b; 1542];
}
fn func() -> [u32; 4] {
    let mut a = [1, 2, 3, 4];
    a[0] = 5;
    return a;
}
struct Te {
    _a: i32,
    _b: i32,
    pair: Pair<i32, bool>,
}
struct Pair<T, U> {
    first: T,
    second: U,
}

fn test_len(a: &str) -> usize {
    a.len()
}

impl<T, V> Pair<T, V> {
    fn new(first: T, second: V) -> Pair<T, V> {
        Pair { first, second }
    }

    #[allow(dead_code)]
    fn get_first(&self) -> &T {
        &self.first
    }

    fn get_second(&self) -> &V {
        &self.second
    }
}

trait TestTrait {
    fn test(&self) -> i32;
}

impl<T> TestTrait for Pair<i32, T> {
    fn test(&self) -> i32 {
        self.first
    }
}

fn test_dyn(a: &dyn TestTrait) -> i32 {
    a.test() + 5
}

enum TestEnum {
    A(i32, i32, i32),
    _B(i32),
    _C,
}

enum PlainEnum {
    A,
    _B,
    _C,
}

union TestUnion {
    a: i32,
    _b: f32,
}

fn test_enum(a: TestEnum) -> i32 {
    match a {
        TestEnum::A(a, b, c) => a + b + c,
        TestEnum::_B(_) => 0,
        TestEnum::_C => 1,
    }
}

fn test_plain_enum(a: PlainEnum) -> i32 {
    match a {
        PlainEnum::A => 0,
        PlainEnum::_B => 1,
        PlainEnum::_C => 2,
    }
}

fn test_plain_union(a: TestUnion) -> i32 {
    unsafe {
        return a.a;
    }
}

fn test() -> i32 {
    let mut t: Te;
    t = Te {
        _a: 7,
        _b: 8,
        pair: Pair {
            first: 128,
            second: true,
        },
    };
    t.pair.first = 10;
    let p: Pair<i32, bool> = Pair::new(128, false);
    let p2 = Pair::new(false, p.get_second());
    let k = 5;
    let pair = (128, t);
    let unkonwn = (128, true, 1.0);
    let l: [i32; 3] = [1, 2, 3];
    let o: std::ops::Range<i32> = 0..1;

    return 5;
}

fn test_closure(val1: i32, val2: i32) -> i32 {
    let a = |x: i32| x + val1 + val2;
    return a(5);
}
fn test_closure_arr(val1: i32, val2: i32) -> [i32; 3] {
    let a = |x: i32| [x + val1 + val2, x + val1, x + val1];
    return a(5);
}
fn test_closure_ref(val: &[i32; 3]) -> &[i32; 3] {
    return val;
}

fn add_many(x: i32) -> i32 {
    let mut b = x;

    while b < b * b {
        b += 3;
    }

    return x;
}
