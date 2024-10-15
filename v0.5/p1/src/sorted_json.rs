pub fn to_json(v: &serde_json::Value) -> String {
    let s = String::new();
    to_json_(v, s, "")
}

fn escape_json_string(out: &mut String, s: &str) {
    out.push('\"');

    let bytes = s.as_bytes();

    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            out.push_str(&s[start..i]);
        }

        let char_escape = CharEscape::from_escape_table(escape, byte);
        out.push_str(&write_char_escape(char_escape));

        start = i + 1;
    }

    if start != bytes.len() {
        out.push_str(&s[start..]);
    }

    out.push('\"');
}

const TAB: &str = "  ";

fn to_json_(v: &serde_json::Value, mut out: String, prefix: &str) -> String {
    // pretty printer for json that prints the dict keys in sorted order
    let prefix2 = format!("{}{}", prefix, TAB);
    match v {
        serde_json::Value::String(s) => escape_json_string(&mut out, s),
        serde_json::Value::Null => out.push_str("null"),
        serde_json::Value::Bool(b) => {
            if *b {
                out.push_str("true")
            } else {
                out.push_str("false")
            }
        }
        serde_json::Value::Number(n) => out.push_str(&format!("{}", n)),
        serde_json::Value::Array(a) => {
            let len = a.len();
            if len == 0 {
                out.push_str("[]");
            } else {
                out.push_str("[\n");
                for (idx, item) in itertools::enumerate(a.iter()) {
                    out.push_str(&prefix2);
                    out = to_json_(item, out, &prefix2);
                    if idx < len - 1 {
                        out.push_str(",\n");
                    }
                }
                out.push('\n');
                out.push_str(prefix);
                out.push(']');
            }
        }
        serde_json::Value::Object(m) => {
            let len = m.len();
            if len == 0 {
                out.push_str("{}");
            } else {
                out.push_str("{\n");
                for (idx, k) in itertools::enumerate(itertools::sorted(m.keys())) {
                    let v = m.get(k).unwrap();
                    out.push_str(&prefix2);
                    escape_json_string(&mut out, k);
                    out.push_str(": ");
                    out = to_json_(v, out, &prefix2);
                    if idx < len - 1 {
                        out.push_str(",\n");
                    }
                }
                out.push('\n');
                out.push_str(prefix);
                out.push('}');
            }
        }
    }
    out
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u'; // \x00...\x1F except the ones above

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[rustfmt::skip]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, BB, TT, NN,  U, FF, RR,  U,  U, // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
    0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];

/// Represents a character escape code in a type-safe manner.
pub enum CharEscape {
    /// An escaped quote `"`
    Quote,
    /// An escaped reverse solidus `\`
    ReverseSolidus,
    // /// An escaped solidus `/`
    // Solidus,
    /// An escaped backspace character (usually escaped as `\b`)
    Backspace,
    /// An escaped form feed character (usually escaped as `\f`)
    FormFeed,
    /// An escaped line feed character (usually escaped as `\n`)
    LineFeed,
    /// An escaped carriage return character (usually escaped as `\r`)
    CarriageReturn,
    /// An escaped tab character (usually escaped as `\t`)
    Tab,
    /// An escaped ASCII plane control character (usually escaped as
    /// `\u00XX` where `XX` are two hex characters)
    AsciiControl(u8),
}

impl CharEscape {
    #[inline]
    fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
        match escape {
            self::BB => CharEscape::Backspace,
            self::TT => CharEscape::Tab,
            self::NN => CharEscape::LineFeed,
            self::FF => CharEscape::FormFeed,
            self::RR => CharEscape::CarriageReturn,
            self::QU => CharEscape::Quote,
            self::BS => CharEscape::ReverseSolidus,
            self::U => CharEscape::AsciiControl(byte),
            _ => unreachable!(),
        }
    }
}

#[inline]
fn write_char_escape(char_escape: CharEscape) -> String {
    use self::CharEscape::*;

    let mut out: Vec<u8> = vec![];
    match char_escape {
        Quote => out.extend(b"\\\""),
        ReverseSolidus => out.extend(b"\\\\"),
        // Solidus => out.extend(b"\\/"),
        Backspace => out.extend(b"\\b"),
        FormFeed => out.extend(b"\\f"),
        LineFeed => out.extend(b"\\n"),
        CarriageReturn => out.extend(b"\\r"),
        Tab => out.extend(b"\\t"),
        AsciiControl(byte) => {
            static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
            let bytes = &[
                b'\\',
                b'u',
                b'0',
                b'0',
                HEX_DIGITS[(byte >> 4) as usize],
                HEX_DIGITS[(byte & 0xF) as usize],
            ];
            out.extend(bytes);
        }
    };
    String::from_utf8(out).unwrap()
}

#[cfg(test)]
mod tests {
    use serde_json::{value::Number, Value};

    #[track_caller]
    fn a(v: Value, out: &'static str) {
        assert_eq!(super::to_json(&v), out);
    }

    #[test]
    fn to_json() {
        a(serde_json::json! {null}, "null");
        a(serde_json::json! {1}, "1");
        a(Value::Number(Number::from_f64(1.0).unwrap()), "1.0");
        a(
            Value::Number(Number::from_f64(-1.0002300e2).unwrap()),
            "-100.023",
        );
        a(serde_json::json! {"foo"}, "\"foo\"");
        a(
            serde_json::json! {r#"hello "world""#},
            r#""hello \"world\"""#,
        );
        a(
            serde_json::json! {[1, 2]},
            "[
  1,
  2
]",
        );
        a(
            serde_json::json! {[1, 2, []]},
            "[
  1,
  2,
  []
]",
        );
        a(
            serde_json::json! {[1, 2, [1, 2, [1, 2]]]},
            "[
  1,
  2,
  [
    1,
    2,
    [
      1,
      2
    ]
  ]
]",
        );
        a(
            serde_json::json! {{"yo": 1, "lo": 2, "no": {}}},
            "{
  \"lo\": 2,
  \"no\": {},
  \"yo\": 1
}",
        );
        a(
            serde_json::json! {{"yo": 1, "lo": 2, "baz": {"one": 1, "do": 2, "tres": {"x": "x", "y": "y"}}}},
            "{
  \"baz\": {
    \"do\": 2,
    \"one\": 1,
    \"tres\": {
      \"x\": \"x\",
      \"y\": \"y\"
    }
  },
  \"lo\": 2,
  \"yo\": 1
}",
        );
        a(
            serde_json::json! {{"yo": 1, "lo": 2, "baz": {"one": 1, "do": 2, "tres": ["x", "x", "y", "y"]}}},
            "{
  \"baz\": {
    \"do\": 2,
    \"one\": 1,
    \"tres\": [
      \"x\",
      \"x\",
      \"y\",
      \"y\"
    ]
  },
  \"lo\": 2,
  \"yo\": 1
}",
        );
    }
}
