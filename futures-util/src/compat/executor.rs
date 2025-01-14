
use super::{Compat, Future01CompatExt};
use crate::{
    future::{FutureExt, UnitError},
    try_future::TryFutureExt,
    task::SpawnExt,
};
use futures_01::Future as Future01;
use futures_01::future::{Executor as Executor01, ExecuteError as ExecuteError01};
use futures_core::task::{Spawn as Spawn03, SpawnError as SpawnError03};
use futures_core::future::FutureObj;

/// A future that can run on a futures 0.1
/// [`Executor`](futures_01::future::Executor).
pub type Executor01Future = Compat<UnitError<FutureObj<'static, ()>>>;

/// Extension trait for futures 0.1 [`Executor`](futures_01::future::Executor).
pub trait Executor01CompatExt: Executor01<Executor01Future> +
                               Clone + Send + 'static
{
    /// Converts a futures 0.1 [`Executor`](futures_01::future::Executor) into a
    /// futures 0.3 [`Spawn`](futures_core::task::Spawn).
    ///
    /// ```
    /// #![feature(async_await)]
    /// use futures::task::SpawnExt;
    /// use futures::future::{FutureExt, TryFutureExt};
    /// use futures_util::compat::Executor01CompatExt;
    /// use tokio::executor::DefaultExecutor;
    ///
    /// # let (tx, rx) = futures::channel::oneshot::channel();
    ///
    /// let mut spawner = DefaultExecutor::current().compat();
    /// let future03 = async move {
    ///     println!("Running on the pool");
    ///     spawner.spawn(async {
    ///         println!("Spawned!");
    ///         # tx.send(42).unwrap();
    ///     }).unwrap();
    /// };
    ///
    /// let future01 = future03.unit_error().boxed().compat();
    ///
    /// tokio::run(future01);
    /// # futures::executor::block_on(rx).unwrap();
    /// ```
    fn compat(self) -> Executor01As03<Self>
        where Self: Sized;
}

impl<Ex> Executor01CompatExt for Ex
where Ex: Executor01<Executor01Future> + Clone + Send + 'static
{
    fn compat(self) -> Executor01As03<Self> {
        Executor01As03 {
            executor01: self,
        }
    }
}

/// Converts a futures 0.1 [`Executor`](futures_01::future::Executor) into a
/// futures 0.3 [`Spawn`](futures_core::task::Spawn).
#[derive(Clone)]
pub struct Executor01As03<Ex> {
    executor01: Ex
}

impl<Ex> Spawn03 for Executor01As03<Ex>
where Ex: Executor01<Executor01Future>,
      Ex: Clone + Send + 'static,
{
    fn spawn_obj(
        &mut self,
        future: FutureObj<'static, ()>,
    ) -> Result<(), SpawnError03> {
        let future = future.unit_error().compat();

        self.executor01.execute(future).map_err(|_|
            SpawnError03::shutdown()
        )
    }
}

impl<Sp, Fut> Executor01<Fut> for Compat<Sp>
where
    for<'a> &'a Sp: Spawn03,
    Fut: Future01<Item = (), Error = ()> + Send + 'static,
{
    fn execute(&self, future: Fut) -> Result<(), ExecuteError01<Fut>> {
        (&self.inner).spawn(future.compat().map(|_| ()))
            .expect("unable to spawn future from Compat executor");
        Ok(())
    }
}
