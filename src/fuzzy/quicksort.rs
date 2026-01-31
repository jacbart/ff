use std::cmp::Ordering;

/// Quicksort implementation for sorting items
pub fn quicksort<T, F>(items: &mut [T], compare: &F)
where
    F: Fn(&T, &T) -> Ordering,
{
    if items.len() <= 1 {
        return;
    }

    let pivot_index = partition(items, compare);
    quicksort(&mut items[..pivot_index], compare);
    quicksort(&mut items[pivot_index + 1..], compare);
}

fn partition<T, F>(items: &mut [T], compare: &F) -> usize
where
    F: Fn(&T, &T) -> Ordering,
{
    let len = items.len();
    let pivot_index = len - 1;
    let mut store_index = 0;

    for i in 0..len - 1 {
        if compare(&items[i], &items[pivot_index]) == Ordering::Less {
            items.swap(i, store_index);
            store_index += 1;
        }
    }

    items.swap(pivot_index, store_index);
    store_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quicksort_empty() {
        let mut items: Vec<i32> = vec![];
        quicksort(&mut items, &|a, b| a.cmp(b));
        assert_eq!(items, vec![]);
    }

    #[test]
    fn test_quicksort_single() {
        let mut items = vec![42];
        quicksort(&mut items, &|a, b| a.cmp(b));
        assert_eq!(items, vec![42]);
    }

    #[test]
    fn test_quicksort_sorted() {
        let mut items = vec![1, 2, 3, 4, 5];
        quicksort(&mut items, &|a, b| a.cmp(b));
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_quicksort_reverse() {
        let mut items = vec![5, 4, 3, 2, 1];
        quicksort(&mut items, &|a, b| a.cmp(b));
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_quicksort_duplicates() {
        let mut items = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        quicksort(&mut items, &|a, b| a.cmp(b));
        assert_eq!(items, vec![1, 1, 2, 3, 3, 4, 5, 5, 5, 6, 9]);
    }
}
