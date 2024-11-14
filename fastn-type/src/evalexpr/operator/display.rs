use std::fmt::{Display, Error, Formatter};

use fastn_type::evalexpr::operator::*;

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use fastn_type::evalexpr::operator::Operator::*;
        match self {
            RootNode => Ok(()),
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Neg => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Mod => write!(f, "%"),
            Exp => write!(f, "^"),

            Eq => write!(f, "=="),
            Neq => write!(f, "!="),
            Gt => write!(f, ">"),
            Lt => write!(f, "<"),
            Geq => write!(f, ">="),
            Leq => write!(f, "<="),
            And => write!(f, "&&"),
            Or => write!(f, "||"),
            Not => write!(f, "!"),

            Assign => write!(f, " = "),
            AddAssign => write!(f, " += "),
            SubAssign => write!(f, " -= "),
            MulAssign => write!(f, " *= "),
            DivAssign => write!(f, " /= "),
            ModAssign => write!(f, " %= "),
            ExpAssign => write!(f, " ^= "),
            AndAssign => write!(f, " &&= "),
            OrAssign => write!(f, " ||= "),

            Tuple => write!(f, ", "),
            Chain => write!(f, "; "),

            Const { value } => write!(f, "{}", value),
            VariableIdentifierWrite { identifier } | VariableIdentifierRead { identifier } => {
                write!(f, "{}", identifier)
            }
            FunctionIdentifier { identifier } => write!(f, "{}", identifier),
        }
    }
}
