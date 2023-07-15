// borrowed from https://github.com/QnnOkabayashi/tracing-forest/ (license: MIT)

pub struct DurationDisplay(pub(crate) f64);

// Taken from chrono
impl std::fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut t = self.0;
        for unit in ["ns", "µs", "ms", "s"] {
            if t < 10.0 {
                return write!(f, "{:.2}{}", t, unit);
            } else if t < 100.0 {
                return write!(f, "{:.1}{}", t, unit);
            } else if t < 1000.0 {
                return write!(f, "{:.0}{}", t, unit);
            }
            t /= 1000.0;
        }
        write!(f, "{:.0}s", t * 1000.0)
    }
}
