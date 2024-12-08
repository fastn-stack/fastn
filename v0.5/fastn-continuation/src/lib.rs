pub enum Result<C: Continuation + ?Sized> {
    Done(C::Output),
    Stuck(Box<C>, C::Needed),
}

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

pub trait Continuation {
    type Output;
    type Needed;
    type Found;
    fn continue_after(self, found: Self::Found) -> Result<Self>;
}

impl<C: Continuation> Result<C> {
    pub fn consume<P>(mut self, p: P) -> C::Output
    where
        P: Provider<Needed = C::Needed, Found = C::Found>,
    {
        loop {
            match self {
                Result::Stuck(ic, needed) => {
                    self = ic.continue_after(p.provide(needed));
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_fn<F>(mut self, f: F) -> C::Output
    where
        F: Fn(C::Needed) -> C::Found,
    {
        loop {
            match self {
                Result::Stuck(ic, needed) => {
                    self = ic.continue_after(f(needed));
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_with<P>(mut self, p: P) -> C::Output
    where
        P: ProviderWith<Needed = C::Needed, Found = C::Found, Context = C>,
    {
        loop {
            match self {
                Result::Stuck(mut ic, needed) => {
                    let o = p.provide(&mut ic, needed);
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_with_fn<F>(mut self, f: F) -> C::Output
    where
        F: Fn(&mut C, C::Needed) -> C::Found,
    {
        loop {
            match self {
                Result::Stuck(mut ic, needed) => {
                    let o = f(&mut ic, needed);
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    #[cfg(feature = "async_provider")]
    pub async fn consume_async<P>(mut self, p: P) -> C::Output
    where
        P: AsyncProvider<Needed = C::Needed, Found = C::Found>,
    {
        loop {
            match self {
                Result::Stuck(ic, needed) => {
                    self = ic.continue_after(p.provide(needed).await);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub async fn consume_async_fn<Fut>(mut self, f: impl Fn(C::Needed) -> Fut) -> C::Output
    where
        Fut: std::future::Future<Output = C::Found>,
    {
        loop {
            match self {
                Result::Stuck(ic, needed) => {
                    self = ic.continue_after(f(needed).await);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    #[cfg(feature = "async_provider")]
    pub async fn consume_with_async<P>(mut self, p: P) -> C::Output
    where
        P: AsyncProviderWith<Needed = C::Needed, Found = C::Found, Context = C>,
    {
        loop {
            match self {
                Result::Stuck(mut ic, needed) => {
                    let o = p.provide(&mut ic, needed).await;
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub async fn consume_with_async_fn<Fut>(
        mut self,
        f: impl Fn(&mut C, C::Needed) -> Fut,
    ) -> C::Output
    where
        Fut: std::future::Future<Output = C::Found>,
    {
        loop {
            match self {
                Result::Stuck(mut ic, needed) => {
                    let o = f(&mut ic, needed).await;
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }
}
