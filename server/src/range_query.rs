use std::ops::{Bound, RangeBounds};

use axum::{extract::{rejection::QueryRejection, Query}, http::Uri};
use serde::Deserialize;

/// A [`serde::Deserialize`] type for [`core::ops::Range<usize>`] and family.
///
/// This type does not allow for a starting with a [`Bound::Excluded`], or
/// ending with a [`Bound::Included`].
///
/// # Security
/// This type uses [`usize`], which is usually either 32 or 64 bytes long,
/// depending on the target archetecture. Using this type may expose if the
/// server is running on a 32-bit or 64-bit archetecture.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct RangeQuery {
    from: Option<usize>,
    to: Option<usize>,
}

/// Gives a `(Bound, Bound)` tuple equivalent to the given [`RangeBounds`].
fn to_bound_tuple(value: &impl RangeBounds<usize>) -> (Bound<usize>, Bound<usize>) {
    (value.start_bound().cloned(), value.end_bound().cloned())
}

impl RangeQuery {
    /// Gives an equivalent `(Bound, Bound)` tuple
    pub fn to_tuple(&self) -> (Bound<usize>, Bound<usize>) {
        to_bound_tuple(self)
    }

    /// Parses a Range Query from a given [`Uri`].
    /// 
    /// # Implementation Notes
    /// Uses Axum's [`axum::extract::Query::try_from_uri`] to parse.
    pub fn try_parse_from_uri(uri: &Uri) -> Result<RangeQuery, QueryRejection> {
        match Query::<RangeQuery>::try_from_uri(uri) {
            Ok(Query(parsed_query)) => Ok(parsed_query),
            Err(err) => Err(err),
        }
    }
}

impl RangeBounds<usize> for RangeQuery {
    fn start_bound(&self) -> Bound<&usize> {
        match &self.from {
            Some(idx) => Bound::Included(idx),
            None => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&usize> {
        match &self.to {
            Some(idx) => Bound::Excluded(idx),
            None => Bound::Unbounded,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Assert that the start and end bounds are equivalent for both `a` and `b`.
    ///
    /// This is a helper function for brevity.
    fn assert_range_equivalent<A, B>(a: &A, b: &B)
    where
        A: RangeBounds<usize>,
        B: RangeBounds<usize>,
    {
        assert_eq!(super::to_bound_tuple(a), super::to_bound_tuple(b));
    }

    #[test]
    fn testinternal_range_equivalent_equal() {
        assert_range_equivalent(&(2..5), &(Bound::Included(2), Bound::Excluded(5)));
    }

    #[test]
    #[should_panic]
    fn testinternal_range_equivalent_non_equal() {
        // included ending
        assert_range_equivalent(&(2..5), &(Bound::Included(2), Bound::Included(5)));
    }

    /// Parse a URI into a potential [`RangeQuery`].
    ///
    /// Helper function for brevity. Uses the parser being used.
    fn parse_uri_range(uri: &'static str) -> Result<RangeQuery, QueryRejection> {
        let parsed_uri = Uri::from_static(uri);
        RangeQuery::try_parse_from_uri(&parsed_uri)
    }

    #[test]
    fn testinternal_parse_range() {
        let simple_range = parse_uri_range("/?from=2&to=5").unwrap();
        assert_eq!(simple_range.from, Some(2));
        assert_eq!(simple_range.to, Some(5));
        // and for something that shouldn't parse
        assert!(parse_uri_range("/?from=false&to=notANumber").is_err());
    }

    #[test]
    fn range_query_to_range_simple() {
        assert_range_equivalent(&parse_uri_range("/?from=2&to=5").unwrap(), &(2..5));
    }

    #[test]
    fn range_query_to_range_variants() {
        assert_range_equivalent(&parse_uri_range("/?from=2&to=5").unwrap(), &(2..5));
        assert_range_equivalent(&parse_uri_range("/?from=2").unwrap(), &(2..));
        assert_range_equivalent(&parse_uri_range("/?to=5").unwrap(), &(..5));
        assert_range_equivalent(&parse_uri_range("/").unwrap(), &(..));
    }
}
