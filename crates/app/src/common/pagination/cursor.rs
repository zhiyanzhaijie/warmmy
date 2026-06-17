use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPagination<C> {
    pub limit: Option<u32>,
    pub before: Option<C>,
}

impl<C> Default for CursorPagination<C> {
    fn default() -> Self {
        Self {
            limit: None,
            before: None,
        }
    }
}

impl<C> CursorPagination<C> {
    pub const DEFAULT_LIMIT: u32 = 40;
    pub const MAX_LIMIT: u32 = 200;

    pub fn limit(&self) -> usize {
        match self.limit {
            Some(v) if v > 0 => v.min(Self::MAX_LIMIT) as usize,
            _ => Self::DEFAULT_LIMIT as usize,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPage<T, C> {
    pub items: Vec<T>,
    pub next_cursor: Option<C>,
    pub has_more: bool,
    pub total_count: usize,
    pub start_index: usize,
    pub end_index: usize,
}

impl<T, C> Default for CursorPage<T, C> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
            total_count: 0,
            start_index: 0,
            end_index: 0,
        }
    }
}

pub fn paginate_before<T, C, F>(
    items: &[T],
    pagination: &CursorPagination<C>,
    mut cursor_of: F,
) -> CursorPage<T, C>
where
    T: Clone,
    C: Clone + PartialEq,
    F: FnMut(&T) -> C,
{
    let before_index = pagination.before.as_ref().and_then(|before| {
        items
            .iter()
            .position(|item| cursor_of(item) == before.clone())
    });
    let upper_bound = before_index.unwrap_or(items.len());
    let lower_bound = upper_bound.saturating_sub(pagination.limit());

    let page_items = items[lower_bound..upper_bound].to_vec();
    let has_more = lower_bound > 0;
    let next_cursor = if has_more {
        page_items.first().map(&mut cursor_of)
    } else {
        None
    };

    CursorPage {
        items: page_items,
        next_cursor,
        has_more,
        total_count: items.len(),
        start_index: lower_bound,
        end_index: upper_bound,
    }
}
