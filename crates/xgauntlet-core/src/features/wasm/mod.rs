//! Embedded Wasmtime runtime host with Zero Ambient Authority.
//!
//! Executes WebAssembly policy engine components in-memory without access to
//! filesystem, network, clock, environment variables, or randomness.

use thiserror::Error;
use wasmtime::{Engine, Instance, Memory, Module, Store, TypedFunc};

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("Wasm engine initialization failed: {0}")]
    Engine(String),

    #[error("Wasm module compilation failed: {0}")]
    Module(String),

    #[error("Wasm instantiation failed: {0}")]
    Instantiation(String),

    #[error("Wasm export not found: {0}")]
    ExportNotFound(String),

    #[error("Wasm memory error: {0}")]
    MemoryError(String),

    #[error("Wasm execution error: {0}")]
    Execution(String),

    #[error("Invalid UTF-8 output from Wasm: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Zero-ambient-authority WebAssembly runtime host using embedded Wasmtime.
pub struct WasmRuntimeHost {
    _engine: Engine,
    _module: Module,
    store: Store<()>,
    instance: Instance,
    memory: Memory,
    alloc_fn: TypedFunc<u32, u32>,
    dealloc_fn: TypedFunc<(u32, u32), ()>,
    eval_fn: TypedFunc<(u32, u32, u32, u32), u64>,
}

impl WasmRuntimeHost {
    /// Creates and instantiates a new Wasm runtime host from bytecode.
    ///
    /// The runtime is initialized in-memory with pure functional state:
    /// no ambient authority, no WASI capabilities, no host system access.
    pub fn new(wasm_bytes: &[u8]) -> Result<Self, WasmError> {
        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        let engine = Engine::new(&config).map_err(|e| WasmError::Engine(e.to_string()))?;
        let module =
            Module::new(&engine, wasm_bytes).map_err(|e| WasmError::Module(e.to_string()))?;

        // Pure functional store: zero host state and zero ambient authority
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| WasmError::ExportNotFound("memory".into()))?;

        let alloc_fn = instance
            .get_typed_func::<u32, u32>(&mut store, "alloc")
            .map_err(|e| WasmError::ExportNotFound(format!("alloc: {e}")))?;

        let dealloc_fn = instance
            .get_typed_func::<(u32, u32), ()>(&mut store, "dealloc")
            .map_err(|e| WasmError::ExportNotFound(format!("dealloc: {e}")))?;

        let eval_fn = instance
            .get_typed_func::<(u32, u32, u32, u32), u64>(&mut store, "evaluate_json_wasm")
            .map_err(|e| WasmError::ExportNotFound(format!("evaluate_json_wasm: {e}")))?;

        Ok(Self {
            _engine: engine,
            _module: module,
            store,
            instance,
            memory,
            alloc_fn,
            dealloc_fn,
            eval_fn,
        })
    }

    /// Evaluates a capability request and enforcement context passed as raw JSON strings.
    pub fn evaluate_json(&mut self, req_json: &str, ctx_json: &str) -> Result<String, WasmError> {
        let req_bytes = req_json.as_bytes();
        let ctx_bytes = ctx_json.as_bytes();

        let req_len = req_bytes.len() as u32;
        let ctx_len = ctx_bytes.len() as u32;

        // Allocate memory inside WebAssembly linear memory
        let req_ptr = self
            .alloc_fn
            .call(&mut self.store, req_len)
            .map_err(|e| WasmError::Execution(format!("alloc req failed: {e}")))?;

        let ctx_ptr = self
            .alloc_fn
            .call(&mut self.store, ctx_len)
            .map_err(|e| WasmError::Execution(format!("alloc ctx failed: {e}")))?;

        // Write input bytes into linear memory
        self.memory
            .write(&mut self.store, req_ptr as usize, req_bytes)
            .map_err(|e| WasmError::MemoryError(format!("write req failed: {e}")))?;

        self.memory
            .write(&mut self.store, ctx_ptr as usize, ctx_bytes)
            .map_err(|e| WasmError::MemoryError(format!("write ctx failed: {e}")))?;

        // Execute deterministic policy evaluation in Wasm
        let packed_result = self
            .eval_fn
            .call(&mut self.store, (req_ptr, req_len, ctx_ptr, ctx_len))
            .map_err(|e| WasmError::Execution(format!("evaluate_json_wasm failed: {e}")))?;

        // Free request and context buffers in WebAssembly
        let _ = self.dealloc_fn.call(&mut self.store, (req_ptr, req_len));
        let _ = self.dealloc_fn.call(&mut self.store, (ctx_ptr, ctx_len));

        // Unpack result length and pointer: (len << 32) | ptr
        let out_len = (packed_result >> 32) as usize;
        let out_ptr = (packed_result & 0xFFFF_FFFF) as usize;

        if out_len == 0 || out_ptr == 0 {
            return Err(WasmError::MemoryError(
                "Null or empty result returned from wasm evaluator".into(),
            ));
        }

        // Read output bytes from linear memory
        let mut out_buf = vec![0u8; out_len];
        self.memory
            .read(&mut self.store, out_ptr, &mut out_buf)
            .map_err(|e| WasmError::MemoryError(format!("read result failed: {e}")))?;

        // Free result buffer in WebAssembly
        let _ = self
            .dealloc_fn
            .call(&mut self.store, (out_ptr as u32, out_len as u32));

        String::from_utf8(out_buf).map_err(WasmError::from)
    }

    /// Invokes the exported get-policy-version function from WebAssembly.
    pub fn get_policy_version(&mut self) -> Result<String, WasmError> {
        let get_ptr_fn = self
            .instance
            .get_typed_func::<(), u32>(&mut self.store, "get_policy_version_ptr")
            .map_err(|e| WasmError::ExportNotFound(format!("get_policy_version_ptr: {e}")))?;

        let get_len_fn = self
            .instance
            .get_typed_func::<(), u32>(&mut self.store, "get_policy_version_len")
            .map_err(|e| WasmError::ExportNotFound(format!("get_policy_version_len: {e}")))?;

        let ptr = get_ptr_fn
            .call(&mut self.store, ())
            .map_err(|e| WasmError::Execution(e.to_string()))? as usize;
        let len = get_len_fn
            .call(&mut self.store, ())
            .map_err(|e| WasmError::Execution(e.to_string()))? as usize;

        let mut buf = vec![0u8; len];
        self.memory
            .read(&mut self.store, ptr, &mut buf)
            .map_err(|e| WasmError::MemoryError(e.to_string()))?;

        String::from_utf8(buf).map_err(WasmError::from)
    }
}
