fn sort() {
    let mut i = 0usize;
    let mut arr = [0usize; 1000];

    while i < 1000 {
        arr[i] = 1000 - i;
        i = i + 1;
    }

    let mut i = 0;
    let mut j = 0;
    let mut temp = 0;

    while i < 1000 {
        j = i + 1;
        while j < 1000 {
            if arr[i] > arr[j] {
                temp = arr[i];
                arr[i] = arr[j];
                arr[j] = temp;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}

fn main() {
    sort();
}
