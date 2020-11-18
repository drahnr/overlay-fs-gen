use color_eyre::Result;
use color_eyre::eyre::*;
use fs_err as fs;
use std::iter;
use std::path::PathBuf;
use libmount::Overlay;

fn create_overlay() -> Result<()> {
    nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNS)
        .context("Failed to enter a new Linux namespace")?;

    // XXX only necessary for excve
    // let whoami = nix::unistd::Uid::current();
    // if !whoami.is_root() {
    //     let uid = nix::unistd::Uid::from_raw(0u32);
    //     nix::unistd::setuid(uid)
    //         .context("Failed to setuid after calling unshare to preserve caps")?;
    // }

    let dest = PathBuf::from("/tmp/ovrly");
    let work_dir = dest.join("work");
    let upper_dir = dest.join("upper");
    let target_dir = dest.join("target");
    fs::create_dir_all(&work_dir).context("Failed to create overlay work directory")?;
    fs::create_dir(&upper_dir)
        .context("Failed to create overlay upper directory")?;
    fs::create_dir(&target_dir)
        .context("Failed to create overlay target directory")?;

    
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
        iter::once(dest.as_path()),
        upper_dir,
        work_dir,
        &target_dir,
        // This error is unfortunately not `Send+Sync`
    )
    .mount()
    .map_err(|e| eyre!("Failed to mount overlay FS: {}", e.to_string()))?;
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    
    create_overlay()
}
