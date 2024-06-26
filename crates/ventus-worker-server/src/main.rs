use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use ventus_proto::{VentusRequest, VentusResponse};
use wasmer::{Instance, IntoBytes, Module};

use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};
use wasmer_wasix::WasiEnv;

// use tikv_jemallocator::Jemalloc;

// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

async fn root(program: axum::Extension<Program2>, body: axum::body::Bytes) -> impl IntoResponse {
    let instant = std::time::Instant::now();

    let engine = wasmer::sys::EngineBuilder::headless();

    let mut store = wasmer::Store::new(engine);

    // let module = unsafe { wasmer::Module::deserialize(&store, program.0 .0.as_ref()) }.unwrap();
    let module = Module::clone(&program.0 .0);

    let mut wasi_env = WasiEnv::builder("ventus-example-function")
        // .stdin(Box::new(stdin_rx))
        // .stdout(Box::new(stdout_tx))
        .finalize(&mut store)
        .unwrap();

    wasi_env.on_exit(&mut store, None);

    axum::response::Response::builder()
        // .status(response.status_code)
        // .body(http_body_util::Full::new(response.body.into_bytes()))
        .status(200)
        .body(http_body_util::Full::new("test".into_bytes()))
        .unwrap()
}

#[derive(Clone)]
struct Program(Arc<Vec<u8>>);

#[derive(Clone)]
struct Program2(Arc<Module>);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let agent = PyroscopeAgent::builder("http://localhost:4040", "myapp-profile")
    //     .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
    //     .build()?;
    // agent.start()?;

    let store = wasmer::Store::default();
    let wasm_path = "target/wasm32-wasi/release/ventus-example-function.wasm";
    let wasm_bytes = tokio::fs::read(wasm_path).await.unwrap();

    let module = wasmer::Module::new(&store, wasm_bytes).unwrap();

    module
        .serialize_to_file(std::path::Path::new("ventus-example-function.so"))
        .unwrap();

    let path = std::path::Path::new("ventus-example-function.so");
    let program = std::fs::read(path)?;

    let app = axum::Router::new()
        .route("/", get(root).post(root))
        .layer(axum::Extension(Program(Arc::new(program))))
        .layer(axum::Extension(Program2(Arc::new(module))));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await.map_err(|e| e.into())
}