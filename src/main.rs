use color_eyre::eyre::*;
use color_eyre::Result;
use fs_err as fs;
use libmount::Overlay;
use std::iter;

/// Replace `sccache-dist` procedure of creating a overlay
fn create_overlay() -> Result<()> {
    let uid = nix::unistd::Uid::current();
    let gid = nix::unistd::Gid::current();

    let tmp = tempdir::TempDir::new("overlay-fs-gen")?;
    let dest = tempdir::TempDir::new("overlay-fs-gen")?;

    nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWUSER)
        .context("Failed to enter a new user namespace")?;

    nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNS)
        .context("Failed to enter a new Linux namespace")?;

    // Equivalent of `unshare --map-current-user`
    std::fs::write("/proc/self/uid_map", format!("{} {} 1", uid, uid))?;
    std::fs::write("/proc/self/setgroups", "deny")?;
    std::fs::write("/proc/self/gid_map", format!("{} {} 1", gid, gid))?;

    let lowerdir = tmp.path().join("lower");
    fs::create_dir(&lowerdir).context("Failed to create overlay base")?;

    let work_dir = dest.path().join("work");
    let upper_dir = dest.path().join("upper");
    let target_dir = dest.path().join("target");
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
