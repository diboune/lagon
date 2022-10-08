use std::{collections::HashMap, sync::Once};

use lagon_runtime::{
    http::{Method, Request, RunResult},
    isolate::{Isolate, IsolateOptions},
    runtime::{Runtime, RuntimeOptions},
};

fn setup() {
    static START: Once = Once::new();

    START.call_once(|| {
        Runtime::new(RuntimeOptions::default());
    });
}

#[tokio::test(flavor = "multi_thread")]
async fn handler_reject() {
    setup();
    let mut isolate = Isolate::new(IsolateOptions::new(
        "export function handler() {
    throw new Error('Rejected');
}"
        .into(),
    ));

    assert_eq!(
        isolate.run(Request::default()).await.0,
        RunResult::Error("Uncaught Error: Rejected".into()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn compilation_error() {
    setup();
    let mut isolate = Isolate::new(IsolateOptions::new(
        "export function handler() {
    this syntax is invalid
}"
        .into(),
    ));

    assert_eq!(
        isolate.run(Request::default()).await.0,
        RunResult::Error("SyntaxError: Unexpected identifier 'syntax'".into()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn timeout_reached() {
    setup();
    let mut isolate = Isolate::new(IsolateOptions::new(
        "export function handler() {
    while(true) {}
    return new Response('Should not be reached');
}"
        .into(),
    ));

    assert_eq!(
        isolate.run(Request::default()).await.0,
        RunResult::Timeout(),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_reached() {
    setup();
    let mut isolate = Isolate::new(
        IsolateOptions::new(
            "export function handler() {
    const storage = [];
    const twoMegabytes = 1024 * 1024 * 2;
    while (true) {
        const array = new Uint8Array(twoMegabytes);
        for (let ii = 0; ii < twoMegabytes; ii += 4096) {
        array[ii] = 1; // we have to put something in the array to flush to real memory
        }
        storage.push(array);
    }
    return new Response('Should not be reached');
}"
            .into(),
        )
        // Increase timeout for CI
        .with_timeout(1000),
    );

    assert_eq!(
        isolate
            .run(Request {
                body: "".into(),
                headers: HashMap::new(),
                method: Method::GET,
                url: "".into(),
            })
            .await
            .0,
        RunResult::MemoryLimit(),
    );
}