const ONE_PRIME: u128 = 12321321361;
const TWO_PRIME: u128 = 5534132137;
const COUNT: usize = 20000;
fn main() {
    let big_prime = ONE_PRIME * TWO_PRIME;
    let mut arr: [u128; COUNT] = [751; COUNT];
    let mut i: usize = 1;
    let end = COUNT;
    let mut res = 0;
    while i <= end {
        arr = mult_array(i as u128, &mut arr, big_prime);
        i = i + 1;
    }
    i = 0;
    while i < end {
        res = (res + arr[i]) % big_prime;
        i = i + 1;
    }
    print_res((res % 18446744073709551615 as u128) as u64);
}
fn mult_array(seed: u128, arr: &[u128; COUNT], big_prime: u128) -> [u128; COUNT] {
    let k = ((seed * 751) % 97) + 1;
    let mut new_arr = [751; COUNT];
    let mut i: usize = 0;
    while i < COUNT {
        new_arr[i] = (arr[i] * k) % big_prime;
        i = i + 1;
    }
    new_arr
}
fn print_res(res: u64) {}
