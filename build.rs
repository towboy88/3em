#[path = "src/runtime/extensions.rs"]
mod extensions;
use deno_core::{JsRuntime, RuntimeOptions};
use std::path::{Path, PathBuf};
use std::{env, fs};

// From https://github.com/denoland/deno/blob/fdf890a68d3d54d40c766fd78faeccb20bd2e2c6/runtime/build.rs#L37-L41
fn create_snapshot(snapshot_path: &Path, files: Vec<PathBuf>) {
  let mut snapshot_runtime = JsRuntime::new(RuntimeOptions {
    extensions: extensions::get_extensions(),
    will_snapshot: true,
    ..Default::default()
  });

  let display_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
  for file in files {
    println!("cargo:rerun-if-changed={}", file.display());
    let display_path = file.strip_prefix(display_root).unwrap();
    let display_path_str = display_path.display().to_string();
    snapshot_runtime
      .execute_script(
        &("three_em:".to_string() + &display_path_str.replace('\\', "/")),
        &std::fs::read_to_string(&file).unwrap(),
      )
      .unwrap();
  }

  let snapshot = snapshot_runtime.snapshot();
  let snapshot_slice: &[u8] = &*snapshot;
  println!("Snapshot size: {}", snapshot_slice.len());
  std::fs::write(&snapshot_path, snapshot_slice).unwrap();
  println!("Snapshot written to: {} ", snapshot_path.display());
}

fn main() {
  let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
  let runtime_snapshot_path = o.join("CLI_SNAPSHOT.bin");
  create_snapshot(&runtime_snapshot_path, Vec::new());
}