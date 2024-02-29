// Type definitions for the project go here

use std::error::Error;

#[derive(PartialEq)]
pub enum Upgrade {
    Pretend,
    Real,
    RealExcludeKernels,
    RealIncludeKernels,
}
#[derive(PartialEq)]
pub enum PromptType {
    Review,
    PressCR,
}
pub enum CmdVerbose {
    NonInteractive,
    Interactive,
    Quiet,
}
#[derive(Debug)]
pub struct GentupArgs {
    pub cleanup: bool,
    pub force: bool,
    pub separate: bool,
    pub optional: bool,
}
pub type RevDep = Upgrade;
pub type DepClean = Upgrade;
pub type ShellOutResult = (Result<String, Box<dyn Error>>, i32);
