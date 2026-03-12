use anyhow::Result;

use crate::config::VmConfig;
use crate::vm::ssh::SshSession;

/// Run `cmd` inside the remote workspace directory and stream output to stdout/stderr.
pub fn exec_on_vm(vm: &VmConfig, cmd: &str) -> Result<()> {
    let ssh = SshSession::new(&vm.user, &vm.host);
    let output = ssh.exec_in_path(&vm.path, cmd)?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        anyhow::bail!(
            "Remote command exited with status: {}",
            output.status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

/// Open an interactive SSH shell on the VM, replacing the current process's
/// stdin/stdout/stderr with the remote tty.
pub fn open_shell(vm: &VmConfig) -> Result<()> {
    let ssh = SshSession::new(&vm.user, &vm.host);
    ssh.interactive_shell()
}
