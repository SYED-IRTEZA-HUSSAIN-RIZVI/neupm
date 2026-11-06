/// Parallel dependency resolution graph explorer.
#[derive(Debug, Default)]
pub struct ParallelResolver {
    pub max_concurrency: usize,
}

impl ParallelResolver {
    pub fn new(max_concurrency: usize) -> Self {
        Self { max_concurrency }
    }
}
