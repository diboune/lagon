use anyhow::Result;
use dashmap::DashMap;
use lagon_runtime_utils::Deployment;
use lagon_serverless::serverless::start;
use lagon_serverless_downloader::FakeDownloader;
use lagon_serverless_pubsub::FakePubSub;
use serial_test::serial;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod utils;

#[tokio::test]
#[serial]
async fn html_assets() -> Result<()> {
    let client = utils::setup();
    let deployments = Arc::new(DashMap::new());
    deployments.insert(
        "127.0.0.1:4000".into(),
        Arc::new(Deployment {
            id: "assets".into(),
            function_id: "function_id".into(),
            function_name: "function_name".into(),
            domains: HashSet::new(),
            assets: HashSet::from(["hello.html".into(), "world/index.html".into()]),
            environment_variables: HashMap::new(),
            memory: 128,
            tick_timeout: 1000,
            total_timeout: 1000,
            is_production: true,
            cron: None,
        }),
    );
    let serverless = start(
        deployments,
        "127.0.0.1:4000".parse().unwrap(),
        Arc::new(FakeDownloader),
        FakePubSub::default(),
        client,
        // Arc::new(Mutex::new(Cronjob::new().await)),
    )
    .await?;
    tokio::spawn(serverless);

    let response = reqwest::get("http://127.0.0.1:4000").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "Dynamic asset: /");

    let response = reqwest::get("http://127.0.0.1:4000/hello").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "hello asset!\n");

    let response = reqwest::get("http://127.0.0.1:4000/world").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "world asset!\n");

    let response = reqwest::get("http://127.0.0.1:4000/other").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "Dynamic asset: /other");

    // Assets don't care about query
    let response = reqwest::get("http://127.0.0.1:4000/hello?test=yo").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "hello asset!\n");

    Ok(())
}

#[tokio::test]
#[serial]
async fn assets_nested() -> Result<()> {
    let client = utils::setup();
    let deployments = Arc::new(DashMap::new());
    deployments.insert(
        "127.0.0.1:4000".into(),
        Arc::new(Deployment {
            id: "assets".into(),
            function_id: "function_id".into(),
            function_name: "function_name".into(),
            domains: HashSet::new(),
            assets: HashSet::from(["index.css".into(), "static/app.js".into()]),
            environment_variables: HashMap::new(),
            memory: 128,
            tick_timeout: 1000,
            total_timeout: 1000,
            is_production: true,
            cron: None,
        }),
    );
    let serverless = start(
        deployments,
        "127.0.0.1:4000".parse().unwrap(),
        Arc::new(FakeDownloader),
        FakePubSub::default(),
        client,
        // Arc::new(Mutex::new(Cronjob::new().await)),
    )
    .await?;
    tokio::spawn(serverless);

    let response = reqwest::get("http://127.0.0.1:4000/index.css").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.text().await?,
        "body {
    display: flex;
}
"
    );

    let response = reqwest::get("http://127.0.0.1:4000/static/app.js").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await?, "console.log('yo');\n");

    Ok(())
}

#[tokio::test]
#[serial]
async fn set_content_type() -> Result<()> {
    let client = utils::setup();
    let deployments = Arc::new(DashMap::new());
    deployments.insert(
        "127.0.0.1:4000".into(),
        Arc::new(Deployment {
            id: "assets".into(),
            function_id: "function_id".into(),
            function_name: "function_name".into(),
            domains: HashSet::new(),
            assets: HashSet::from([
                "hello.html".into(),
                "index.css".into(),
                "static/app.js".into(),
            ]),
            environment_variables: HashMap::new(),
            memory: 128,
            tick_timeout: 1000,
            total_timeout: 1000,
            is_production: true,
            cron: None,
        }),
    );
    let serverless = start(
        deployments,
        "127.0.0.1:4000".parse().unwrap(),
        Arc::new(FakeDownloader),
        FakePubSub::default(),
        client,
        // Arc::new(Mutex::new(Cronjob::new().await)),
    )
    .await?;
    tokio::spawn(serverless);

    // TODO: set default content type?
    // let response = reqwest::get("http://127.0.0.1:4000").await?;
    // assert_eq!(response.status(), 200);
    // assert_eq!(response.headers()["content-type"], "");

    let response = reqwest::get("http://127.0.0.1:4000/hello").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/html");

    let response = reqwest::get("http://127.0.0.1:4000/index.css").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/css");

    let response = reqwest::get("http://127.0.0.1:4000/static/app.js").await?;
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "application/javascript");

    Ok(())
}
