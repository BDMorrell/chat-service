use core::ops::{Bound, Range, RangeBounds, RangeTo};

/// Return a type-normalized range if it is valid and in the given bound.
///
/// Please see [`std::slice::try_range`], which is currently in the
/// experimental std api`slice_range` (see it's
/// [tracking issue](https://github.com/rust-lang/rust/issues/76393)).
///
/// There aren't too many ways to implement this, but
pub fn to_bounded_range(
    range: impl RangeBounds<usize>,
    bound: RangeTo<usize>,
) -> Option<Range<usize>> {
    let start: usize = match range.start_bound() {
        Bound::Included(&val) => val,
        Bound::Excluded(&val) => val.checked_add(1)?,
        Bound::Unbounded => 0,
    };
    let end: usize = match range.end_bound() {
        Bound::Included(&val) => val.checked_add(1)?,
        Bound::Excluded(&val) => val,
        Bound::Unbounded => bound.end,
    };

    if start > end || start >= bound.end || end > bound.end {
        None
    } else {
        Some(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_normal_usage() {
        assert_eq!(Some(2..6), to_bounded_range(2..6, ..10));
        assert_eq!(Some(5..9), to_bounded_range(5..=8, ..10));
        assert_eq!(Some(12..17), to_bounded_range(12.., ..17));
        assert_eq!(Some(0..5), to_bounded_range(..5, ..28));
        assert_eq!(Some(0..6), to_bounded_range(..=5, ..28));
        assert_eq!(Some(0..12), to_bounded_range(.., ..12));
        assert_eq!(Some(0..2), to_bounded_range(..=1, ..2));
        assert_eq!(Some(0..2), to_bounded_range(.., ..2));
    }

    #[test]
    fn bounded_single_elements() {
        assert_eq!(Some(0..1), to_bounded_range(0..=0, ..5)); // inclusive: n..=n
        assert_eq!(Some(1..2), to_bounded_range(1..=1, ..5)); // exclusive: n..(n+1)
        assert_eq!(Some(0..1), to_bounded_range(0..1, ..5));
        assert_eq!(Some(1..2), to_bounded_range(1..2, ..5));
    }

    #[test]
    fn bounded_valid_empty_ranges() {
        assert_eq!(Some(0..0), to_bounded_range(0..0, ..16));
        assert_eq!(Some(3..3), to_bounded_range(3..3, ..16));
        assert_eq!(Some(0..0), to_bounded_range(..0, ..2));
    }

    #[test]
    fn bounds_though_a_bit_strange() {
        assert_eq!(Some(0..1), to_bounded_range(..1, ..2));
        assert_eq!(Some(1..2), to_bounded_range(1.., ..2));
    }

    #[test]
    fn bounded_too_high() {
        assert_eq!(None, to_bounded_range(3..17, ..3));
    }

    #[test]
    fn bounded_arguably_too_high_but_here_is_a_divergence_from_std() {
        assert_eq!(None, to_bounded_range(5.., ..5));
    }

    #[test]
    fn end_of_range_too_high() {
        assert_eq!(None, to_bounded_range(0..1024, ..2));
        assert_eq!(None, to_bounded_range(0..=1024, ..2));
    }

    #[test]
    fn upper_range_max_fail_without_panic() {
        assert_eq!(None, to_bounded_range(7..=usize::MAX, ..usize::MAX));
        assert_eq!(None, to_bounded_range(..=usize::MAX, ..usize::MAX));
        assert_eq!(
            Some(0..usize::MAX),
            to_bounded_range(..usize::MAX, ..usize::MAX)
        );
    }

    #[test]
    fn backwards_bounds() {
        assert_eq!(None, to_bounded_range(5..4, ..usize::MAX));
    }
    #[test]
    fn backwards_but_inclusive() {
        assert_eq!(Some(5..5), to_bounded_range(5..=4, ..usize::MAX));
    }
}
