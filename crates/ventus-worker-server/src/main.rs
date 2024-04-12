use opentelemetry_sdk::trace::Sampler;
use std::{path::Path, sync::Arc};
use tracing::span;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use no_debug::NoDebug;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::Level;
use ventus_proto::{VentusRequest, VentusResponse};
use wasmer::{Instance, IntoBytes, Module};

use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};
use wasmer_wasix::{PluggableRuntime, WasiEnv};

use std::cell::Cell;

thread_local! {
    pub static runtime: Arc<PluggableRuntime> = Arc::new(wasmer_wasix::runtime::PluggableRuntime::new(Arc::new(
        wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager::default(),
    )));
}

#[tracing::instrument]
async fn root(
    program: axum::Extension<Program2>,
    // runtime: axum::Extension<Arc<PluggableRuntime>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let instant = std::time::Instant::now();

    let span = span!(Level::INFO, "Store Creation");
    let span2 = span.enter();
    let engine = wasmer::sys::EngineBuilder::headless();
    let mut store = wasmer::Store::new(engine);
    drop(span2);
    drop(span);

    let (mut stdin_tx, stdin_rx) = wasmer_wasix::Pipe::channel();
    let (stdout_tx, mut stdout_rx) = wasmer_wasix::Pipe::channel();

    let request = ventus_proto::VentusRequest {
        body: body.into(),
        method: "GET".to_string(),
        uri: "/".to_string(),
        headers: Default::default(),
    };

    let span = span!(Level::INFO, "Write Request to stdin");
    let span2 = span.enter();
    stdin_tx
        .write(rmp_serde::to_vec(&request).unwrap().as_slice())
        .await
        .unwrap();
    drop(span2);
    drop(span);

    let span = span!(Level::INFO, "WasiEnv Creation");
    let span2 = span.enter();
    let mut wasi_env = runtime.with(|runtime2| {
        WasiEnv::builder("ventus-example-function")
            .stdin(Box::new(stdin_rx))
            .stdout(Box::new(stdout_tx))
            .runtime(Arc::<PluggableRuntime>::clone(runtime2))
            .finalize(&mut store)
            .unwrap()
    });
    drop(span2);
    drop(span);

    let span = span!(Level::INFO, "Import object");
    let span2 = span.enter();
    let module = program.0 .0.as_ref();
    let import_object = wasi_env.import_object(&mut store, &module).unwrap();
    drop(span2);
    drop(span);

    let span = span!(Level::INFO, "Initiate instance");
    let span2 = span.enter();
    // let (mut store, program, import_object, instance) = tokio::task::spawn_blocking(move || {
    let module = program.0 .0.as_ref();
    let instance = Instance::new(&mut store, &module, &import_object).unwrap();
    //     (store, program, import_object, instance)
    // })
    // .await
    // .unwrap();
    drop(span2);
    drop(span);

    let span = span!(Level::INFO, "WasiEnv Initialize");
    let span2 = span.enter();
    if let Err(e) = wasi_env.initialize(&mut store, instance.clone()) {
        eprintln!("Error initializing WASI: {:?}", e);
    }
    drop(span2);
    drop(span);

    let span = span!(Level::INFO, "Call _start");
    let span2 = span.enter();
    // let mut store = tokio::task::spawn_blocking(move || {
    let span3 = span!(Level::INFO, "Get _start");
    let span4 = span.enter();
    let start = instance.exports.get_function("_start").unwrap();
    drop(span4);
    drop(span3);

    // let mut store = store;
    // start.call(&mut store, &[]).unwrap();
    // // store
    // // })
    // // .await
    // // .unwrap();
    // drop(span2);
    // drop(span);

    // let span = span!(Level::INFO, "Read Response from stdout");
    // let span2 = span.enter();
    // wasi_env.on_exit(&mut store, None);

    // let mut buf = Vec::<u8>::new();
    // stdout_rx.read_to_end(&mut buf).await.unwrap();
    // drop(span2);
    // drop(span);

    // let span = span!(Level::INFO, "Deserialize Response");
    // let span2 = span.enter();
    // let response: VentusResponse = rmp_serde::from_slice(&buf).unwrap();
    // drop(span2);
    // drop(span);

    let span = span!(Level::INFO, "Response");
    let span2 = span.enter();
    let response = axum::response::Response::builder()
        // .status(response.status_code)
        .status(200)
        // .body(http_body_util::Full::new(response.body.into_bytes()))
        .body(http_body_util::Full::new("test".into_bytes()))
        .unwrap();
    drop(span2);
    drop(span);

    response
}

#[derive(Clone)]
struct Program(Arc<Vec<u8>>);

#[derive(Clone, Debug)]
struct Program2(Arc<NoDebug<Module>>);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // use opentelemetry::trace::TracerProvider;
    // use tracing_subscriber::prelude::*;
    // let tracer = opentelemetry_otlp::new_pipeline()
    //     .tracing()
    //     .with_trace_config(opentelemetry_sdk::trace::Config::default().with_sampler(
    //         Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(0.001))),
    //     ))
    //     .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    //     .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // tracing_subscriber::Registry::default()
    //     .with(telemetry)
    //     .init();

    // let agent = PyroscopeAgent::builder("http://localhost:4040", "myapp-profile")
    //     .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
    //     .build()?;
    // agent.start()?;

    let engine = wasmer_compiler_llvm::LLVM::new();
    let store = wasmer::Store::new(engine);
    let wasm_path =
        "/home/southball/projects/ventus/target/wasm32-wasi/release/ventus-example-function.wasm";
    let wasm_bytes = tokio::fs::read(wasm_path).await.unwrap();

    let module = wasmer::Module::new(&store, wasm_bytes).unwrap();

    module
        .serialize_to_file(std::path::Path::new("ventus-example-function.so"))
        .unwrap();

    let path = std::path::Path::new("ventus-example-function.so");
    let program = std::fs::read(path)?;
    let module = unsafe { wasmer::Module::deserialize(&store, program.as_slice()) }.unwrap();

    // let runtime = wasmer_wasix::runtime::PluggableRuntime::new(Arc::new(
    //     wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager::default(),
    // ));

    let app = axum::Router::new()
        .route("/", get(root).post(root))
        .layer(axum::Extension(Program(Arc::new(program))))
        .layer(axum::Extension(Program2(Arc::new(module.into()))));
    // .layer(axum::Extension(Arc::new(runtime)));

    // read port from env
    let port: i32 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    axum::serve(listener, app).await.map_err(|e| e.into())
}
