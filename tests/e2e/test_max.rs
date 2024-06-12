fn max() -> usize {
    let mut i = 0;

    let mut arr = [0usize; 1000000];

    // Initialize with values 1-100000 with while loop
    while (i < 1000000) {
        arr[i] = i + 1;
        i = i + 1;
    }

    let mut max = 0usize;
    i = 0;

    // Find max value in the array
    while (i < 1000000) {
        if (max < arr[i]) {
            max = arr[i];
        }
        i = i + 1;
    }

    return max;
}

fn main() {
    max();
}
