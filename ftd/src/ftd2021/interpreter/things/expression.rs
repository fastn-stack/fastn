#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Boolean {
    // if: $caption is not null
    IsNotNull {
        value: ftd::PropertyValue,
    },
    // if: $caption is null
    IsNull {
        value: ftd::PropertyValue,
    },
    // if: $list is not empty
    IsNotEmpty {
        value: ftd::PropertyValue,
    },
    // if: $list is empty
    IsEmpty {
        value: ftd::PropertyValue,
    },
    // if: $caption == hello | if: $foo
    Equal {
        left: ftd::PropertyValue,
        right: ftd::PropertyValue,
    },
    // if: $caption != hello
    NotEqual {
        left: ftd::PropertyValue,
        right: ftd::PropertyValue,
    },
    // if: not $show_something
    Not {
        of: Box<Boolean>,
    },
    // if: false
    Literal {
        value: bool,
    },
    // if: $array is empty
    ListIsEmpty {
        value: ftd::PropertyValue,
    },
}
