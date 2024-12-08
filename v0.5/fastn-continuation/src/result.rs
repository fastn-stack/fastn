pub enum Result<C: fastn_continuation::Continuation + ?Sized> {
    Init(Box<C>),
    Stuck(Box<C>, C::Needed),
    Done(C::Output),
}

impl<C: fastn_continuation::Continuation + Default> Default for Result<C> {
    fn default() -> Self {
        Result::Init(Box::default())
    }
}

impl<C: fastn_continuation::Continuation> Result<C>
where
    C::Found: Default,
{
    pub fn consume<P>(mut self, p: P) -> C::Output
    where
        P: fastn_continuation::Provider<Needed = C::Needed, Found = C::Found>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(ic, needed) => {
                    self = ic.continue_after(p.provide(needed));
                }
                fastn_continuation::Result::Done(c) => {
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
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(ic, needed) => {
                    self = ic.continue_after(f(needed));
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_with<P>(mut self, p: P) -> C::Output
    where
        P: fastn_continuation::ProviderWith<Needed = C::Needed, Found = C::Found, Context = C>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(mut ic, needed) => {
                    let o = p.provide(&mut ic, needed);
                    self = ic.continue_after(o);
                }
                fastn_continuation::Result::Done(c) => {
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
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(mut ic, needed) => {
                    let o = f(&mut ic, needed);
                    self = ic.continue_after(o);
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    #[cfg(feature = "async_provider")]
    pub async fn consume_async<P>(mut self, p: P) -> C::Output
    where
        P: fastn_continuation::AsyncProvider<Needed = C::Needed, Found = C::Found>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(ic, needed) => {
                    self = ic.continue_after(p.provide(needed).await);
                }
                fastn_continuation::Result::Done(c) => {
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
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(ic, needed) => {
                    self = ic.continue_after(f(needed).await);
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    #[cfg(feature = "async_provider")]
    pub async fn consume_with_async<P>(mut self, p: P) -> C::Output
    where
        P: fastn_continuation::AsyncProviderWith<Needed = C::Needed, Found = C::Found, Context = C>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(mut ic, needed) => {
                    let o = p.provide(&mut ic, needed).await;
                    self = ic.continue_after(o);
                }
                fastn_continuation::Result::Done(c) => {
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
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
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

    pub fn mut_consume<P>(mut self, mut p: P) -> C::Output
    where
        P: fastn_continuation::MutProvider<Needed = C::Needed, Found = C::Found>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(ic, needed) => {
                    self = ic.continue_after(p.provide(needed));
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn mut_consume_with<P>(mut self, mut p: P) -> C::Output
    where
        P: fastn_continuation::MutProviderWith<Needed = C::Needed, Found = C::Found, Context = C>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(mut ic, needed) => {
                    let o = p.provide(&mut ic, needed);
                    self = ic.continue_after(o);
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    #[cfg(feature = "async_provider")]
    pub async fn mut_consume_async<P>(mut self, mut p: P) -> C::Output
    where
        P: fastn_continuation::AsyncMutProvider<Needed = C::Needed, Found = C::Found>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(ic, needed) => {
                    self = ic.continue_after(p.provide(needed).await);
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    #[cfg(feature = "async_provider")]
    pub async fn mut_consume_with_async<P>(mut self, p: P) -> C::Output
    where
        P: fastn_continuation::AsyncProviderWith<Needed = C::Needed, Found = C::Found, Context = C>,
    {
        loop {
            match self {
                fastn_continuation::Result::Init(ic) => {
                    self = ic.continue_after(Default::default());
                }
                fastn_continuation::Result::Stuck(mut ic, needed) => {
                    let o = p.provide(&mut ic, needed).await;
                    self = ic.continue_after(o);
                }
                fastn_continuation::Result::Done(c) => {
                    return c;
                }
            }
        }
    }
}
