use ipfsdag::get_contract;
use types::{MetadataRequest, Transaction};
use wasi_common::pipe::WritePipe;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

use crate::appconfig::CONFIG;

pub fn wasi_runner(
  method: String,
  cid: String,
  metadatas: Vec<MetadataRequest>,
  transaction: Transaction,
) -> String {
  let mut result = "".to_string();

  let ipfs_bytes = get_contract(
    cid.clone(),
    CONFIG.get::<String>("IPFS_ADDR").unwrap(),
    0,
  );

  if ipfs_bytes.success {
    let m = serde_json::to_string(&metadatas).unwrap();
    let t = serde_json::to_string(&transaction).unwrap();

    let args = vec![
      m,
      t,
    ];
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

    let stdout = WritePipe::new_in_memory();

    let wasi = WasiCtxBuilder::new()
        // .inherit_stdio()
        .stdout(Box::new(stdout.clone()))
        .args(&args).unwrap()
        .build();

    let mut store = Store::new(&engine, wasi);
    
    let module = Module::from_binary(&engine, &ipfs_bytes.block).unwrap();

    let linking1 = linker.instantiate(&mut store, &module).unwrap();
    let run = linking1.get_typed_func::<(), ()>(&mut store, method.as_str()).unwrap();
    run.call(&mut store, ()).unwrap();

    drop(store);

    let contents: Vec<u8> = stdout.try_into_inner()
        .map_err(|_err| anyhow::Error::msg("sole remaining reference")).unwrap()
        .into_inner();
    
    result = String::from_utf8(contents).expect("Invalid UTF-8");
  }

  result
}