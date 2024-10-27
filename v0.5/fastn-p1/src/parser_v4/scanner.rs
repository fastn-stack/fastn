#[derive(Default)]
pub struct Scanner {
    // source: String,
    pub tokens: Vec<char>,
    pub size: usize,
    index: usize,
    ticks: std::cell::RefCell<usize>,
    pub output: fastn_p1::ParseOutput,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        let tokens: Vec<_> = source.chars().collect();
        Scanner {
            size: tokens.len(),
            tokens,
            ..Default::default()
        }
    }
}
