use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wasmtime::*;
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;
use wasmtime_wasi::{DirPerms, FilePerms, I32Exit, WasiCtxBuilder};
use tracing::{debug, error};

/// Maximum output buffer size (10 MB).
const OUTPUT_BUFFER_CAPACITY: usize = 10 * 1024 * 1024;

/// Manages the WebAssembly runtime and container state.
pub struct WasmContainerRunner {
    engine: Engine,
    module: Module,
}

impl WasmContainerRunner {
    /// Initialize the engine and compile the WASM module.
    pub fn new(wasm_path: &Path) -> Result<Self> {
        let mut config = Config::new();
        config.async_support(false);
        config.cranelift_opt_level(OptLevel::Speed);

        let engine = Engine::new(&config)?;
        debug!("Compiling WASM container module from {:?}", wasm_path);
        let module = Module::from_file(&engine, wasm_path)?;

        Ok(Self { engine, module })
    }

    /// Execute a command in the container.
    /// Maps the working directory to /workspace.
    pub fn run_command(
        &self,
        command: &str,
        working_dir: &Path,
        additional_mounts: &[(std::path::PathBuf, String)],
        env_vars: &std::collections::HashMap<String, String>,
        state_key: &str,
    ) -> Result<(String, String, i32)> {
        // Prepare pipes for output capturing
        let stdout_pipe = MemoryOutputPipe::new(OUTPUT_BUFFER_CAPACITY);
        let stderr_pipe = MemoryOutputPipe::new(OUTPUT_BUFFER_CAPACITY);
        let stdout_capture = stdout_pipe.clone();
        let stderr_capture = stderr_pipe.clone();

        // Build a wrapper command to capture state
        // We run the user command, then a sentinel, then pwd and env
        let wrapped_command = format!(
            "{} && echo '__STATE_SENTINEL__' && pwd && env",
            command
        );

        // Build WASI context using the P1 (preview1) API for core modules
        let mut builder = WasiCtxBuilder::new();
        builder.stdout(stdout_capture.clone());
        builder.stderr(stderr_capture.clone());

        // Map the workspace to /mnt/wasi0 (c2w's default mount point)
        builder.preopened_dir(
            working_dir,
            "/mnt/wasi0",
            DirPerms::all(),
            FilePerms::all(),
        )?;

        // Map additional directories (e.g. credentials)
        for (host_path, guest_path) in additional_mounts {
            if host_path.exists() {
                builder.preopened_dir(
                    host_path,
                    guest_path,
                    DirPerms::all(),
                    FilePerms::all(),
                )?;
            }
        }

        // Create a temporary script file in the workspace to avoid sh -c argument issues
        let script_name = format!("c2w_cmd_{}.sh", uuid::Uuid::new_v4());
        let script_path = working_dir.join(&script_name);
        std::fs::write(&script_path, &wrapped_command)?;
        
        // Pass the script path as the argument
        builder.arg("sh");
        builder.arg(format!("/mnt/wasi0/{}", script_name));
        
        // Add environment variables
        for (k, v) in env_vars {
            builder.env(k, v);
        }
        
        // Set essential c2w env vars
        builder.env("C2W_NET", "1");
        builder.env("C2W_ENTRYPOINT", "/bin/sh");

        // Enable networking
        builder.inherit_network();

        // Build the WASIp1 context
        let wasi_ctx: WasiP1Ctx = builder.build_p1();
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&self.engine);
        p1::add_to_linker_sync(&mut linker, |ctx| ctx)?;

        let mut store = Store::new(&self.engine, wasi_ctx);
        let instance = linker.instantiate(&mut store, &self.module)?;
        let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;

        debug!("Starting WASM container command execution");

        let result = start.call(&mut store, ());

        // Cleanup the temporary script from the host
        let _ = std::fs::remove_file(&script_path);

        // Handle exit code
        let exit_code = match result {
            Ok(_) => 0,
            Err(e) => {
                if let Some(exit) = e.downcast_ref::<I32Exit>() {
                    exit.0
                } else {
                    error!("WASM execution error: {:?}", e);
                    1 // Treat as failure
                }
            }
        };

        // Extract captured output
        let stdout_bytes = stdout_capture.contents();
        let stderr_bytes = stderr_capture.contents();
        let stdout_str = String::from_utf8_lossy(&stdout_bytes).to_string();
        let stderr_str = String::from_utf8_lossy(&stderr_bytes).to_string();

        // Parse state from stdout
        let mut final_stdout = stdout_str;
        if let Some(pos) = final_stdout.rfind("__STATE_SENTINEL__") {
            let state_part = &final_stdout[pos + "__STATE_SENTINEL__".len()..];
            let state_part_owned = state_part.to_string();
            final_stdout = final_stdout[..pos].trim().to_string();

            // The state part contains pwd and env
            let mut lines = state_part_owned.lines().filter(|l| !l.is_empty());
            if let Some(pwd) = lines.next() {
                debug!("Captured container CWD: {}", pwd);
                // Update session shell state
                let state_arc = crate::session_shell_state(state_key);
                let mut state = state_arc.lock();
                state.cwd = Some(PathBuf::from(pwd));

                // Parse env vars
                for line in lines {
                    if let Some((k, v)) = line.split_once('=') {
                        state.env_vars.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }

        Ok((final_stdout, stderr_str, exit_code))
    }
}

/// Global cache for WasmContainerRunner to ensure "warm" starts.
pub static CONTAINER_RUNNER_CACHE: once_cell::sync::Lazy<dashmap::DashMap<PathBuf, Arc<WasmContainerRunner>>> =
    once_cell::sync::Lazy::new(dashmap::DashMap::new);

pub fn get_or_create_runner(wasm_path: &Path) -> Result<Arc<WasmContainerRunner>> {
    let path = wasm_path.to_path_buf();
    if let Some(runner) = CONTAINER_RUNNER_CACHE.get(&path) {
        return Ok(runner.clone());
    }

    let runner = Arc::new(WasmContainerRunner::new(wasm_path)?);
    CONTAINER_RUNNER_CACHE.insert(path, runner.clone());
    Ok(runner)
}
