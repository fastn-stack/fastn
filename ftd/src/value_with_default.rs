#[derive(PartialEq, Debug, Clone, serde::Deserialize)]
pub enum ValueWithDefault<T> {
    Default(T),
    Found(T),
}

impl<T> Default for ValueWithDefault<T>
where
    T: Default,
{
    fn default() -> Self {
        ValueWithDefault::Default(Default::default())
    }
}

impl<T> serde::ser::Serialize for ValueWithDefault<T>
where
    T: serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            ValueWithDefault::Found(v) => serde::ser::Serialize::serialize(v, serializer),
            ValueWithDefault::Default(v) => serde::ser::Serialize::serialize(v, serializer),
        }
    }
}

impl<T> ValueWithDefault<T> {
    pub fn inner(&self) -> &T {
        match self {
            Self::Default(t) => t,
            Self::Found(t) => t,
        }
    }

    pub fn found(v: T) -> Self {
        Self::Found(v)
    }

    pub fn default(v: T) -> Self {
        Self::Default(v)
    }
}
