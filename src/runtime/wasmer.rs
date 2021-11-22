use std::cell::Cell;
use wasmer::{imports, wat2wasm, Instance, MemoryView, Module, Store};

fn wasmer_bench(
  state: &mut [u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let wasm_bytes = include_bytes!("./testdata/01_wasm/01_wasm.wasm");

  let store = Store::default();

  let module = Module::new(&store, wasm_bytes)?;
  let import_object = imports! {};

  let instance = Instance::new(&module, &import_object)?;

  let handle = instance
    .exports
    .get_function("handle")?
    .native::<(u32, u32, u32, u32), u32>()?;

  let alloc = instance
    .exports
    .get_function("_alloc")?
    .native::<u32, u32>()?;
  let get_len = instance
    .exports
    .get_function("get_len")?
    .native::<(), u32>()?;
  let ptr = alloc.call(state.len() as u32)?;

  let memory = instance.exports.get_memory("memory")?;
  let mut raw_mem = unsafe { memory.data_unchecked_mut() };
  raw_mem[ptr as usize..ptr as usize + state.len()].swap_with_slice(state);

  let result_ptr =
    handle.call(ptr, state.len() as u32, ptr, state.len() as u32)? as usize;

  let view: MemoryView<u8> = memory.view();
  let result_len = get_len.call()? as usize;

  let result = view[result_ptr..result_ptr + result_len]
    .iter()
    .map(Cell::get)
    .collect();

  Ok(result)
}

pub async fn bench() {
  use crate::runtime::wasm;
  use deno_core::serde_json::json;
  use std::time::{Duration, Instant};
  let mut state = json!({
      "counter": 0,
  });
  let mut state_bytes = deno_core::serde_json::to_vec(&state).unwrap();

  let iters = 1;

  {
    let now = Instant::now();

    for i in 0..iters {
      wasmer_bench(&mut state_bytes).unwrap();
    }

    println!("Wasmer {} ms", now.elapsed().as_millis());
  }
  let mut state = json!({
      "counter": 0,
  });
  let mut state_bytes = deno_core::serde_json::to_vec(&state).unwrap();
  {
    let now = Instant::now();

    for i in 0..iters {
      let mut rt = wasm::WasmRuntime::new(include_bytes!(
        "./testdata/01_wasm/01_wasm.wasm"
      ))
      .await
      .unwrap();

      rt.call(&mut state_bytes).await.unwrap();
    }
    println!("V8 {} ms", now.elapsed().as_millis());
  }
}
#[cfg(test)]
mod tests {
  use crate::runtime::wasmer::bench;
  #[tokio::test]
  async fn wasmer_bench_test() {
    bench().await;
  }
}