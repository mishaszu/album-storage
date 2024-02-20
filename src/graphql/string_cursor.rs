use async_graphql::{
    connection::{self, Connection, CursorType, Edge, EmptyFields},
    Result, SimpleObject,
};
use std::convert::Infallible;

use super::CursorParams;

pub enum StringCursorError {
    Invalid,
}

impl std::fmt::Display for StringCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid cursor")
    }
}

pub trait Identifiable {
    fn get_id(&self) -> String;
}

pub struct StringCursor(String);

impl StringCursor {
    const fn new(uid: String) -> Self {
        Self(uid)
    }

    fn encode(&self) -> String {
        self.0.to_string()
    }

    fn decode(s: &str) -> Result<Self, StringCursorError> {
        Ok(Self::new(s.to_owned()))
    }
}

impl CursorType for StringCursor {
    type Error = StringCursorError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        StringCursor::decode(s)
    }

    fn encode_cursor(&self) -> String {
        self.encode()
    }
}

/// Additional fields to attach to the connection
#[derive(SimpleObject)]
pub struct StringConnectionFields {
    /// Total result set count
    total_count: usize,
}

/// Relay connection result
pub type ConnectionResult<T> =
    Result<Connection<StringCursor, T, StringConnectionFields, EmptyFields>>;

/// Creates a new Relay-compliant connection. Iterator must implement `ExactSizeIterator` to
/// determine page position in the total result set.
pub async fn query<
    T: async_graphql::OutputType + Identifiable + Clone,
    I: ExactSizeIterator<Item = T> + Clone,
>(
    iter: I,
    p: CursorParams,
    default_page_size: usize,
) -> ConnectionResult<T> {
    connection::query::<_, _, StringCursor, _, _, StringConnectionFields, _, _, _, Infallible>(
        p.after,
        p.before,
        p.first,
        p.last,
        |after, before, first, last| async move {
            let iter_len = iter.len();

            let (start, end) = {
                let (after, before) = {
                    let mut index_after = None;
                    let mut index_before = None;

                    let mut index = 0;

                    for item in iter.clone() {
                        let cursor_after = after.as_ref().map(|c| c.0.clone());
                        if let Some(after) = cursor_after {
                            if after == item.get_id() {
                                index_after = Some(index + 1);
                            }
                        }

                        let cursor_after = before.as_ref().map(|c| c.0.clone());
                        if let Some(before) = cursor_after {
                            if before == item.get_id() {
                                index_before = Some(index);
                            }
                        }

                        index += 1;
                    }
                    (index_after.unwrap_or(0), index_before.unwrap_or(iter_len))
                };

                // Calculate start/end based on the provided first/last. Note that async-graphql disallows
                // providing both (returning an error), so we can safely assume we have, at most, one of
                // first or last.
                match (first, last) {
                    // First
                    (Some(first), _) => (after, (after.saturating_add(first)).min(before)),
                    // Last
                    (_, Some(last)) => ((before.saturating_sub(last)).max(after), before),
                    // Default page size
                    _ => (after, default_page_size.min(before)),
                }
            };

            let mut connection = Connection::with_additional_fields(
                start > 0,
                end < iter_len,
                StringConnectionFields {
                    total_count: iter_len,
                },
            );
            connection.edges.extend(
                (start..end)
                    .zip(iter.skip(start))
                    .map(|(_index, node)| Edge::new(StringCursor::new(node.get_id()), node)),
            );
            Ok(connection)
        },
    )
    .await
}
