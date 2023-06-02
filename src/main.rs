extern crate wasm_pack;

#[cfg(feature = "perf")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "perf")]
    let _profiler = dhat::Profiler::new_heap();

    wasm_pack::main(std::env::args_os())
}
