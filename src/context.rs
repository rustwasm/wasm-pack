use Cli;
use command::Command;
use progressbar::ProgressOutput;
use manifest::CargoManifest;

// FIXUP: Cannot derive 'Debug' trait because 'ProgressOutput' does not derive.

struct Context {
    cmd: Command,
    log_level: u8,
    manifest: Option<CargoManifest>,
    pbar: ProgressOutput,
}

impl Context {
    fn new(args: Cli) -> Context {
        Context {
            cmd: args.cmd,
            log_level: args.verbosity,
            manifest: None,
            pbar: ProgressOutput::new(),
        }
    }

    fn run(&self) {
        unimplemented!();
    }
}
