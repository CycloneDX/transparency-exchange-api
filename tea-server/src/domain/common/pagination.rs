use serde::{Deserialize, Serialize};

/// SA-14: Pagination parameters for list endpoints.
///
/// Clients pass `?limit=50&offset=0` query params.
/// - `limit`: max records per page. Capped at 200. Default 50.
/// - `offset`: zero-based record offset. Default 0.
/// - Response includes `total` so clients can compute page count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

const MAX_PAGE_SIZE: usize = 200;

fn default_limit() -> usize {
    50
}

impl PaginationParams {
    /// Clamp limit to MAX_PAGE_SIZE so clients cannot request unbounded pages.
    pub fn clamped_limit(&self) -> usize {
        self.limit.min(MAX_PAGE_SIZE)
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            offset: 0,
        }
    }
}

/// Paginated response wrapper returned by all list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl<T> Page<T> {
    pub fn new(all: Vec<T>, params: &PaginationParams) -> Page<T> {
        let limit = params.clamped_limit();
        let offset = params.offset;
        let total = all.len();
        let items = all.into_iter().skip(offset).take(limit).collect();
        Page {
            items,
            total,
            limit,
            offset,
        }
    }
}
