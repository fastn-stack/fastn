pub enum Result<C: Continuation + ?Sized> {
    Done(C::Output),
    Stuck(Box<C>, C::NeededInput),
}

pub trait Provider {
    type Input;
    type Output;

    fn provide(&self, input: Self::Input) -> Self::Output;
}

pub trait ProviderWith {
    type Input;
    type Output;
    type Context;

    fn provide(&self, context: &mut Self::Context, input: Self::Input) -> Self::Output;
}

// impl<I, O> Provider for dyn Fn(I) -> O {
//     type Input = I;
//     type Output = O;
//     fn provide(&self, input: Self::Input) -> Self::Output {
//         self(input)
//     }
// }

// impl<I, O> Provider for fn(I) -> O {
//     type Input = I;
//     type Output = O;
//     fn provide(&self, input: Self::Input) -> Self::Output {
//         self(input)
//     }
// }

pub trait Continuation {
    type Output;
    type NeededInput;
    type NeededOutput;
    fn continue_after(self, n: Self::NeededOutput) -> Result<Self>;
}

impl<C: Continuation> Result<C> {
    pub fn consume<P>(mut self, p: P) -> C::Output
    where
        P: Provider<Input = C::NeededInput, Output = C::NeededOutput>,
    {
        loop {
            match self {
                Result::Stuck(ic, input) => {
                    self = ic.continue_after(p.provide(input));
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_fn<F>(mut self, f: F) -> C::Output
    where
        F: Fn(C::NeededInput) -> C::NeededOutput,
    {
        loop {
            match self {
                Result::Stuck(ic, input) => {
                    self = ic.continue_after(f(input));
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_with<P>(mut self, p: P) -> C::Output
    where
        P: ProviderWith<Input = C::NeededInput, Output = C::NeededOutput, Context = C>,
    {
        loop {
            match self {
                Result::Stuck(mut ic, input) => {
                    let o = p.provide(&mut ic, input);
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
        F: Fn(&mut C, C::NeededInput) -> C::NeededOutput,
    {
        loop {
            match self {
                Result::Stuck(mut ic, input) => {
                    let o = f(&mut ic, input);
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub async fn consume_async<Fut>(mut self, f: impl Fn(C::NeededInput) -> Fut) -> C::Output
    where
        Fut: std::future::Future<Output = C::NeededOutput>,
    {
        loop {
            match self {
                Result::Stuck(ic, input) => {
                    self = ic.continue_after(f(input).await);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub async fn consume_with_async<Fut>(
        mut self,
        f: impl Fn(&mut C, C::NeededInput) -> Fut,
    ) -> C::Output
    where
        Fut: std::future::Future<Output = C::NeededOutput>,
    {
        loop {
            match self {
                Result::Stuck(mut ic, input) => {
                    let o = f(&mut ic, input).await;
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }
}
