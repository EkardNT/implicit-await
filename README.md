# implicit-await - implicit `Future` awaiting for Rust

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
implicit-await = "0.1"
```

Now you can use `#[implicit_await]`:

```rust
use implicit_await::implicit_await;

#[implicit_await]
async fn foo() {
    // Any functions returning a future (`async fn` or `fn -> impl Future`) will be automatically awaited.
}
```

## Description

Rust's async/await feature uses the postfix `.await` operator to explicitly request suspension points within `async` functions. The discussion around the syntax of the `.await` operator has been long and contentious, driven by the desire to interoperate smoothly with Rust's existing postfix `?` operator (among other concerns).

When I was reading about the various syntax proposals from which `.await` finally emerged the leader, it occurred to me that the problems would be greatly simplified by making the wrapping/unwrapping behavior of `async` functions internally consistent. In standard Rust, `async` functions implicitly wrap their return values in a `Future`, but the *inputs* to `async` functions - the child `Futures` that determine the shape of the compiler-generated state machine - are explicitly unwrapped via `.await`. If instead `async` functions handled both wrapping and unwrapping implicitly, then there would be no need for the `.await` operator, bypassing the syntax concerns about the operator.

This library provides a procedural macro `#[implicit_await]` which transforms `async` functions to implicitly unwrap child `Future`s.

|                                   | Standard Rust                 | #[implicit_await]    |
|-----------------------------------|-------------------------------|----------------------|
| Outputs (function results) are... | implicitly wrapped            | implicitly wrapped   |
| Inputs (child futures) are...     | explicitly unwrapped (.await) | implicitly unwrapped |

## Example

See the [sum](implicit-await/examples/sum.rs) example for a full working demo of the following.

Suppose you have the following three functions, which range from fully synchronous to fully asynchronous.

```rust
type IntResult = Result<u32, ()>;

// Fully synchronous
fn num_sync(num: u32) -> IntResult {
    Ok(num)
}

// Half-and-half
fn num_fut(num: u32) -> impl Future<Output = IntResult> {
    ready(Ok(num))
}

// Fully asynchronous
async fn num_async(num: u32) -> IntResult {
    Ok(num)
}
```

In standard Rust, calls to synchronous and asynchronous functions are distinguished by the need to postfix the `.await` operator.

```rust
async fn sum() -> IntResult {
    // No .await - a synchronous function.
    let one: u32 = num_sync(1)?;
    // .await - asynchronous function.
    let two: u32 = num_fut(2).await?;
    // .await - asynchronous function.
    let three: u32 = num_async(3).await?;
    // Note that the return value is implicitly wrapped.
    Ok(one + two + three)
}
```

With `implicit-await`, synchronous and asynchronous functions behave equivalently due to the implicit awaiting behavior. This allows the `?` operator to work as expected without any additional syntax.

```rust
#[implicit_await]
async fn sum() -> IntResult {
    // Synchronous function.
    let one: u32 = num_sync(1)?;
    // A `fn -> impl Future` is implicitly awaited.
    let two: u32 = num_fut(2)?;
    // An `async fn` is implicitly awaited.
    let three: u32 = num_async(3)?;
    Ok(one + two + three)
}
```

`implicit-await` makes calls to asynchronous functions behave identically to calls to synchronous functions from the point of view of program control flow. The intuitive rule that *functions complete before they return*, which Rust programmers naturally learn from calling synchronous functions, also applies to asynchronous function calls. However, sometimes you have use-cases where you want explicit control over suspension points. The `defer!` macro allows you to *defer* awaits so that you can start multiple asynchronous processes in parallel. A common example of this use case is making multiple network requests at the same time, then waiting for all of them to complete.

```rust
use implicit_await::defer;

#[implicit_await]
async fn sum() -> IntResult {
    // Start three futures in parallel without implicitly awaiting.
    let (one, two, three) = defer!{ (
        num_fut(1),
        num_async(2),
        num_async(3)
    ) };
    // This `join` call is implicitly awaited because the defer! block ended.
    let (one, two) = futures::future::join(one, two);
    // If you have a bare `Future`, you're free to .await it if you want.
    let three = three.await?;
    Ok(one? + two? + three);
}
```

## What's the catch?

In a nutshell: ``error[E0599]: no method named `as_future` found for type `your::type::here` in the current scope``

The catch is that Rust doesn't support negative or mutually exclusive trait bounds. Because of this, I needed to make a choice whether to support `Future` types seamlessly and `!Future` types painfully, or `!Future` types seamlessly and `Future` types painfully. I chose the former. This means that types which do not implement `Future` need to be manually supported.

| You're calling sync functions that return... | Then you need to...                                                                                                                                                                                                                                               |
|----------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Types from `std`                             | Do nothing ("std" is a default feature)                                                                                                                                                                                                                           |
| Types from a third-party crate               |  Check the features list to see whether implicit-await already supports that crate, and if so enable the feature. Otherwise submit an issue requesting support for that crate's types be added.   Rockstars can include a PR to add support along with the issue! |
| Types from your own crate                    |  Use the as_future! macro to add support in your own code. OR submit a PR to implicit-await to support your crate behind a feature flag.

## Features

* `std` (*default-feature*) - Enable support for Rust's standard library types. Disable this feature for `no-std` support.

## Acknowledgements

Apparently futures in Kotlin work similarly to this.