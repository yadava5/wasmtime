// Fuel consumed by instantiating a module whose only work is a data segment,
// with copy-on-write memory init on and off, compiled in memory or loaded
// from a .cwasm file (gh-14331).
use wasmtime::*;

const WAT: &str = r#"(module (memory (export "memory") 1) (data (i32.const 0) "hello"))"#;
const BUDGET: u64 = 1_000_000;

fn consumed(cow: bool, from_file: bool) -> Result<u64> {
    let mut config = Config::new();
    config.consume_fuel(true).memory_init_cow(cow);
    let engine = Engine::new(&config)?;
    let module = if from_file {
        let bytes = Module::new(&engine, WAT)?.serialize()?;
        let path = std::env::temp_dir().join(format!("fuel-probe-{cow}.cwasm"));
        std::fs::write(&path, bytes)?;
        unsafe { Module::deserialize_file(&engine, &path)? }
    } else {
        Module::new(&engine, WAT)?
    };
    let mut store = Store::new(&engine, ());
    store.set_fuel(BUDGET)?;
    let instance = Instance::new(&mut store, &module, &[])?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    assert_eq!(&memory.data(&store)[..5], b"hello");
    Ok(BUDGET - store.get_fuel()?)
}

fn main() -> Result<()> {
    println!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    for from_file in [false, true] {
        for cow in [true, false] {
            let source = if from_file { "deserialize_file" } else { "Module::new" };
            println!("{source:16} memory_init_cow={cow:5} fuel={}", consumed(cow, from_file)?);
        }
    }
    Ok(())
}
