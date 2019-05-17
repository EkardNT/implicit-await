use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use implicit_await_macro::as_future_internal;

pub trait FutureAsFuture {
    fn as_future(self) -> Self;
}

impl<T: Future> FutureAsFuture for T {
    fn as_future(self) -> Self {
        self
    }
}

pub trait NonFutureAsFuture : Sized {
    fn as_future(self) -> Ready<Self>;
}

// Clone of future-rs' Ready, to avoid needing to take a dependency on them.
pub struct Ready<T>(Option<T>);

impl<T> Unpin for Ready<T> {}

impl<T> Future for Ready<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0.take().unwrap())
    }
}

pub fn ready<T>(t: T) -> Ready<T> {
    Ready(Some(t))
}

#[cfg(feature = "std")]
impl<T, E> NonFutureAsFuture for Result<T, E> {
    fn as_future(self) -> Ready<Self> {
        ready(self)
    }
}

#[cfg(feature = "std")]
as_future_internal!{
    String,
    &str,
    (),
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    usize,
    isize,
    std::io::BufReader<T>,
    std::option::Option<T>}