use tokio::process::Command;

pub mod address;

fn namadaw() -> Command {
    Command::new("namadaw")
}
