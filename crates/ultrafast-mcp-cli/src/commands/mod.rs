pub mod init;
pub mod generate;
pub mod dev;
pub mod build;
pub mod test;
pub mod validate;
pub mod info;
pub mod server;
pub mod client;
pub mod completions;

// Re-export command argument structs
pub use init::InitArgs;
pub use generate::GenerateArgs;
pub use dev::DevArgs;
pub use build::BuildArgs;
pub use test::TestArgs;
pub use validate::ValidateArgs;
pub use info::InfoArgs;
pub use server::ServerArgs;
pub use client::ClientArgs;
pub use completions::CompletionsArgs;
