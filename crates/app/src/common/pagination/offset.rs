use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageMeta {
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
    pub current_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub meta: PageMeta,
}

impl<T> Page<T> {
    pub fn map<U, F>(self, mut f: F) -> Page<U>
    where
        F: FnMut(T) -> U,
    {
        Page {
            items: self.items.into_iter().map(&mut f).collect(),
            meta: self.meta,
        }
    }
}

impl PageMeta {
    pub fn new(total: u64, limit: u32, offset: u32) -> Self {
        let total_pages = if total == 0 {
            0
        } else {
            let pages = total.div_ceil(limit as u64);
            pages.min(u32::MAX as u64) as u32
        };
        let current_page = if total == 0 { 0 } else { offset / limit + 1 };

        Self {
            total,
            limit,
            offset,
            current_page,
            total_pages,
        }
    }
}

impl Pagination {
    pub const DEFAULT_LIMIT: u32 = 50;
    pub const MAX_LIMIT: u32 = 500;

    pub fn limit(self) -> u32 {
        match self.limit {
            Some(v) if v > 0 => v.min(Self::MAX_LIMIT),
            _ => Self::DEFAULT_LIMIT,
        }
    }

    pub fn offset(self) -> u32 {
        self.offset.unwrap_or(0)
    }

    pub fn meta(self, total: u64) -> PageMeta {
        PageMeta::new(total, self.limit(), self.offset())
    }

    pub fn to_page<T>(self, items: Vec<T>, total: u64) -> Page<T> {
        Page {
            items,
            meta: self.meta(total),
        }
    }
}
