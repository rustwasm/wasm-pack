use error::Error;
use parity_wasm::elements::{self, Serialize};
use wasm_snip;

#[derive(Clone, Debug, StructOpt)]
pub struct SnipOpitons {
    input: String,
    #[structopt(long = "output", short = "o")]
    pub(crate) output: Option<String>,
    functions: Vec<String>,
    #[structopt(long = "pattern", short = "p")]
    patterns: Vec<String>,
    #[structopt(long = "snip_rust_fmt_code")]
    snip_rust_fmt_code: bool,
    #[structopt(long = "snip_rust_panicking_code")]
    snip_rust_panicking_code: bool,
}

impl Into<wasm_snip::Options> for SnipOpitons {
    fn into(self) -> wasm_snip::Options {
        wasm_snip::Options {
            input: ::std::path::PathBuf::from(self.input),
            functions: self.functions,
            patterns: self.patterns,
            snip_rust_fmt_code: self.snip_rust_fmt_code,
            snip_rust_panicking_code: self.snip_rust_panicking_code,
        }
    }
}

pub(crate) fn snip(opts: SnipOpitons) -> Result<(), Error> {
    let module = wasm_snip::snip(opts.clone().into())?;

    if let Some(output) = opts.output {
        elements::serialize_to_file(output, module)?;
    } else {
        let stdout = ::std::io::stdout();
        let mut stdout = stdout.lock();
        module.serialize(&mut stdout)?;
    }
    Ok(())
}
