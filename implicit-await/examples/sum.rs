#![feature(async_await)]

use implicit_await::{defer, implicit_await};

use std::future::Future;
use futures::future::{join, ready, FutureExt};
use futures::executor::ThreadPool;

fn main() {
    ThreadPool::new().unwrap().run(sum().inspect(|sum| match sum {
        Ok(sum) => println!("The sum is {}", sum),
        Err(_) => println!("Summation failed")
    })).unwrap();
}

type IntResult = Result<u32, ()>;

#[implicit_await]
async fn sum() -> IntResult {
    // Call a synchronous function.
    let one: u32 = num_sync(1)?;
    // Call a synchronous function that returns an impl Future.
    let two: u32 = num_fut(2)?;
    // Call an asynchronous function.
    let three: u32 = num_async(3)?;
    // Sometimes you want to perform async actions in parallel. To allow that,
    // async function calls inside of a defer! {} block are not implicitly awaited.
    let (four, five, six) = defer! {(
        num_fut(4),
        num_async(5),
        num_async(6)
    )};
    // The join function is implicitly awaited.
    let (four, five) = join(four, five);
    // If you do end up with a Future value that was not implicitly awaited,
    // you can just use the .await operator as normal.
    let six = six.await?;
    Ok(one + two + three + four? + five? + six)
}

async fn num_async(num: u32) -> IntResult {
    Ok(num)
}

fn num_fut(num: u32) -> impl Future<Output = IntResult> {
    ready(Ok(num))
}

fn num_sync(num: u32) -> IntResult {
    Ok(num)
}