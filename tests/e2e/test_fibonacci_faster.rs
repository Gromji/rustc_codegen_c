fn main() {
    let mut arr = [0; 10000];
    let ans = faster_fibonnaci(92, &mut arr);
}
fn faster_fibonnaci(n: u64, arr: &mut [u64; 10000]) -> u64 {
    if n == 0 {
        return 0;
    } else if n == 1 {
        return 1;
    } else {
        if arr[(n - 1) as usize] == 0 {
            arr[(n - 1) as usize] = faster_fibonnaci(n - 1, arr);
        }
        if arr[(n - 2) as usize] == 0 {
            arr[(n - 2) as usize] = faster_fibonnaci(n - 2, arr);
        }
        arr[(n - 1) as usize] + arr[(n - 2) as usize]
    }
}
