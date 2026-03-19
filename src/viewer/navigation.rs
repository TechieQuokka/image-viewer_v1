/// Navigate prev/next within a source of `total` images.
/// Wraps around: prev from 0 → total-1, next from total-1 → 0.
pub fn prev_index(current: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    if current == 0 {
        total - 1
    } else {
        current - 1
    }
}

pub fn next_index(current: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    (current + 1) % total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_prev() {
        assert_eq!(prev_index(0, 5), 4);
        assert_eq!(prev_index(3, 5), 2);
    }

    #[test]
    fn wraps_next() {
        assert_eq!(next_index(4, 5), 0);
        assert_eq!(next_index(2, 5), 3);
    }

    #[test]
    fn empty_source() {
        assert_eq!(prev_index(0, 0), 0);
        assert_eq!(next_index(0, 0), 0);
    }
}
