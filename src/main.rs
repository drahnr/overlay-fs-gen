use color_eyre::eyre::*;
use color_eyre::Result;
use fs_err as fs;
use libmount::Overlay;
use std::iter;
use std::path::PathBuf;

/// Replace `sccache-dist` procedure of creating a overlay
fn create_overlay() -> Result<()> {
    nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNS)
        .context("Failed to enter a new Linux namespace")?;

    let whoami = dbg!(nix::unistd::Uid::current());
    if !whoami.is_root() {
        // only needed in case of doing a execve in order to preserve caps
        // let uid = nix::unistd::Uid::from_raw(0u32);
        // nix::unistd::setuid(uid)
        //     .context("Failed to setuid after calling unshare to preserve caps")?;
    }

    let lowerdir = PathBuf::from("/tmp/lower");
    let _ = fs::remove_dir_all(&lowerdir);
    fs::create_dir(&lowerdir).context("Failed to create overlay base")?;

    let dest = PathBuf::from("/tmp/ovrly");
    let _ = fs::remove_dir_all(&dest);
    fs::create_dir(&dest).context("Failed to create overlay base")?;

    let work_dir = dest.join("work");
    let upper_dir = dest.join("upper");
    let target_dir = dest.join("target");
    fs::create_dir(&work_dir).context("Failed to create overlay work directory")?;
    fs::create_dir(&upper_dir).context("Failed to create overlay upper directory")?;
    fs::create_dir(&target_dir).context("Failed to create overlay target directory")?;

    // Make sure that all future mount changes are private to this namespace
    // TODO: shouldn't need to add these annotations
    let source: Option<&str> = None;
    let fstype: Option<&str> = None;
    let data: Option<&str> = None;
    // Turn / into a 'slave', so it receives mounts from real root, but doesn't propogate back
    nix::mount::mount(
        source,
        "/",
        fstype,
        nix::mount::MsFlags::MS_REC | nix::mount::MsFlags::MS_PRIVATE,
        data,
    )
    .context("Failed to turn / into a slave")?;

    Overlay::writable(
        iter::once(lowerdir.as_path()),
        upper_dir,
        work_dir,
        &target_dir,
    )
    .mount()?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    create_overlay()
}
