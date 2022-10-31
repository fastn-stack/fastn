use ftd::evalexpr::Node;
use std::fmt::{Display, Error, Formatter};

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.operator.fmt(f)?;
        for child in self.children() {
            write!(f, " {}", child)?;
        }
        Ok(())
    }
}
