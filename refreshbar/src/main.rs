use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut found = false;
    for prc in procfs::process::all_processes()? {
        if let Ok(stat) = prc?.stat() {
            if stat.comm == "ministatus" {
                found = true;
                signal::kill(Pid::from_raw(stat.pid), Signal::SIGUSR1)?;
                break;
            }
        }
    }
    if !found {
        eprintln!("ministatus not running!");
        std::process::exit(1);
    }
    Ok(())
}
