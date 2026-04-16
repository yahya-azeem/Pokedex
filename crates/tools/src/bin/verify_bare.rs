use wasmtime::*;
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::{WasiCtxBuilder};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wasm_path = PathBuf::from(".pokedex/images/alpine-amd64.wasm");
    println!("Loading WASM from {:?}", wasm_path);

    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, &wasm_path)?;

    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdout();
    builder.inherit_stderr();
    
    // c2w command line: [argv0] [args...]
    builder.arg("c2w");
    builder.arg("busybox");
    builder.arg("uname");
    builder.arg("-a");

    let wasi_ctx: WasiP1Ctx = builder.build_p1();
    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    p1::add_to_linker_async(&mut linker, |ctx| ctx)?;

    let mut store = Store::new(&engine, wasi_ctx);
    let instance = linker.instantiate_async(&mut store, &module).await?;
    let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;

    println!("Starting execution...");
    let result = start.call_async(&mut store, ()).await;

    match result {
        Ok(_) => println!("Execution finished successfully."),
        Err(e) => println!("Execution failed: {:?}", e),
    }

    Ok(())
}
