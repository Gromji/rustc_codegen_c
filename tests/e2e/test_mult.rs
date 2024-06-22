fn main() {
    let base = 5534132137;
    let mut k: u128 = base * 243523463;
    while k % 5534341 != 0 {
        k = (k * k) % 5534132132324357;
    }
    print_res(k);
}
fn print_res(res: u128) {}
