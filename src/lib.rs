use std::process::{Command, Output};
use camino::Utf8PathBuf;

pub mod contexts;
pub mod includes;

#[derive(Debug)]
pub enum Error {
    StdIo(std::io::Error),
    MissingExecutable(Utf8PathBuf),
    MissingBash(Utf8PathBuf),
    MissingConfig(Utf8PathBuf),
    MissingRom(Utf8PathBuf),
    MissingMovie(Utf8PathBuf),
    MissingLua(Utf8PathBuf),
    IncompatibleOSVersion,
    AbsolutePathFailed,
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::StdIo(value)
    }
}

/// Behavior used to run an emulator.
pub trait EmulatorContext: Sized {
    /// Returns the name of the base command to be executed.
    /// 
    /// Typically this is the executable file of the emulator (e.g. `BizHawk.exe` or `fceux.exe`).
    /// 
    /// If the executable is a linux binary in the working-directory, be sure to use `./cmd_name` rather than just `cmd_name`.
    fn cmd_name(&self) -> String;
    
    /// Returns a list of arguments to be passed to [`Command::args`].
    fn args(&self) -> Vec<String>;
    
    /// Returns a list of environment variables to be passed to [`Command::envs`].
    fn env(&self) -> Vec<(String, String)>;
    
    /// Returns the path to the working directory intended for the command's child process.
    /// 
    /// Refer to [`Command::current_dir`] for more details.
    fn working_dir(&self) -> Utf8PathBuf;
    
    /// Perform any file copying or final checks to ensure context is ready for running.
    /// 
    /// Returns an error if preparation failed.
    fn prepare(&mut self) -> Result<(), Error>;
    
    /// Creates and executes a [`Command`] and returns the output result.
    /// 
    /// Default trait implementation simply calls [`run`].
    fn run(self) -> Result<Output, Error> {
        run(self)
    }
}

/// Prepares and executes an emulator based on the provided context.
/// 
/// Returns any errors encountered while preparing (context-dependent) and any IO errors caused by running the command.
pub fn run<C: EmulatorContext>(mut ctx: C) -> Result<Output, Error> {
    ctx.prepare()?;
    
    command(ctx).output().map_err(|err| err.into())
}

/// Buildes a [`Command`] using data pulled from an [`EmulatorContext`].
pub fn command<C: EmulatorContext>(ctx: C) -> Command {
    let mut cmd = Command::new(ctx.cmd_name());
    cmd.args(ctx.args())
        .envs(ctx.env())
        .current_dir(ctx.working_dir());
    
    cmd
}