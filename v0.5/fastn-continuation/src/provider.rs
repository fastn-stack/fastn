pub trait Provider {
    type Needed;
    type Found;

    fn provide(&self, needed: Self::Needed) -> Self::Found;
}

#[cfg(feature = "async_provider")]
#[async_trait::async_trait]
pub trait AsyncProvider {
    type Needed;
    type Found;

    async fn provide(&self, needed: Self::Needed) -> Self::Found;
}

#[cfg(feature = "async_provider")]
#[async_trait::async_trait]
pub trait AsyncProviderWith {
    type Needed;
    type Found;
    type Context;

    async fn provide(&self, context: &mut Self::Context, needed: Self::Needed) -> Self::Found;
}

pub trait ProviderWith {
    type Needed;
    type Found;
    type Context;

    fn provide(&self, context: &mut Self::Context, needed: Self::Needed) -> Self::Found;
}
