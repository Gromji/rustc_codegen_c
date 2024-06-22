fn main() {
    let ans = gcd(8472658423765982476, 12894732948210983240);
    print_res(ans);
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        let k = gcd(a % b, 0);
        gcd(b, k)
    }
}
fn print_res(res: u64) {}
