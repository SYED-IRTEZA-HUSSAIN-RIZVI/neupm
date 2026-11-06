/// Chunked / concurrent tarball download scheduler.
#[derive(Debug, Default)]
pub struct ParallelDownloadManager {
    pub max_active_downloads: usize,
}

impl ParallelDownloadManager {
    pub fn new(max_active_downloads: usize) -> Self {
        Self {
            max_active_downloads,
        }
    }
}
