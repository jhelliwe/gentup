// Type definitions for the project go here

#[derive(PartialEq)]
pub enum Upgrade {
    Real,
    Pretend,
    Fetch,
    Kernel,
    KernelPretend,
}

#[derive(PartialEq)]
pub enum PromptType {
    ClearScreen,
    Review,
    PressCR,
}

pub enum CmdVerbose {
    NonInteractive,
    Interactive,
    Quiet,
}

pub type RevDep = Upgrade;
pub type DepClean = Upgrade;
