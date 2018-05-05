use command::{init, pack, publish, Command};
use error::Error;
use manifest::CargoManifest;
use progressbar::ProgressOutput;
use Cli;

// FIXUP: Cannot derive 'Debug' trait because 'ProgressOutput' does not derive.

pub struct Context {
    manifest: Option<CargoManifest>,
    pbar: ProgressOutput,
    verbosity: u8,
}

impl Context {
    pub fn new(verbosity: u8) -> Context {
        Context {
            manifest: None,
            pbar: ProgressOutput::new(),
            verbosity,
        }
    }

    pub fn run(&mut self, cmd: Command) -> Result<(), Error> {
        let status = match cmd {
            Command::Init { path, scope } => init(path, scope),
            Command::Pack { path } => pack(path),
            Command::Publish { path } => publish(path),
        };

        match status {
            Ok(_) => {}
            Err(ref e) => {
                self.pbar.error(e.error_type());
            }
        }

        // Make sure we always clear the progress bar before we abort the program otherwise
        // stderr and stdout output get eaten up and nothing will work. If this part fails
        // to work and clear the progress bars then you're really having a bad day with your tools.
        self.pbar.done()?;

        // Return the actual status of the program to the main function
        status
    }

    pub fn manifest(&mut self) -> &CargoManifest {
        unimplemented!();
    }

    fn read_manifest(path: &str) -> Result<CargoManifest, Error> {
        unimplemented!();
    }
}
