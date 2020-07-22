//! Bunch of utility functions

/// Pre-condition: vec is an array of pairs sorted on the first elem of each pair
/// then on the second
pub fn binary_search_pair(vec: &[[u64; 2]], pair: [u64; 2]) -> bool {
    vec.binary_search(&pair).is_ok()
}

/// Pre-condition: vec is sorted on the first elem of each pair
pub fn first(vec: &[[u64; 2]], x: u64, low: usize, high: usize, n: usize, key_pos: usize) -> usize {
    if high >= low {
        let mid = low + (high - low) / 2;

        if mid == low && mid == high {
            if vec[mid][key_pos] != x {
                return n;
            }
        }
        if (mid == 0 || x > vec[mid - 1][key_pos]) && vec[mid][key_pos] == x {
            return mid;
        } else if x > vec[mid][key_pos] {
            return first(vec, x, mid + 1, high, n, key_pos);
        } else {
            return first(vec, x, low, mid, n, key_pos);
        }
    }
    return n;
}
