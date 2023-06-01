#[derive(Debug)]
pub struct Elem {
    pub start: u32,
    pub fns: Vec<String>,
}

impl Elem {
    pub fn to_wat(&self) -> String {
        use itertools::Itertools;

        format!(
            "(elem (i32.const {}) {})",
            self.start,
            self.fns.iter().map(|v| format!("${}", v)).join(" ")
        )
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(
            super::Elem {
                start: 10,
                fns: vec!["f1".to_string(), "foo".to_string()]
            }
            .to_wat(),
            "(elem (i32.const 10) $f1 $foo)"
        );
    }
}
