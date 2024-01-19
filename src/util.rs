/// Rearranges elements in the slice into two parts such that `pred` returns
/// `true` for each element in the first part and `false` for each element in
/// the second. Returns the two parts.
pub fn partition_by<T, F>(slice: &mut [T], pred: F) -> (&mut [T], &mut [T])
where
    F: Fn(&T) -> bool,
{
    if slice.len() == 0 {
        return slice.split_at_mut(0);
    }

    let mut left = 0;
    let mut right = slice.len() - 1;

    while left != right {
        while left < right && pred(&slice[left]) {
            left += 1;
        }
        while right > left && !pred(&slice[right]) {
            right -= 1;
        }
        slice.swap(left, right);
    }

    slice.split_at_mut(if pred(&slice[left]) { left + 1 } else { left })
}
