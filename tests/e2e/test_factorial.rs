fn main() {
    let mut ans = factorial(100000);
    print_res(ans);
}
fn factorial(n: u64) -> u64 {
    if n == 0 {
        return 1;
    } else {
        let mut fact = factorial(n - 1);
        if fact > 1_000_002 {
            fact = fact % 1_000_003;
        }
        return n * fact;
    }
}
fn print_res(res: u64) {}
