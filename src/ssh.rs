//! # SSH Utilities for apt-remote
//!
//! This module provides helper functions and traits for establishing SSH
//! connections, executing commands on remote hosts, and uploading files
//! or directories securely. It abstracts away low-level details of
//! the `ssh2` crate to simplify common SSH and SFTP workflows.

use anyhow::{Context, Result};
use ssh2::{Session, Sftp};
use std::{
    fs::{self, File},
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

/// Establish an SSH session with the given target in the form `user@host`.
///
/// This function:
/// 1. Connects to the host via TCP on port 22.
/// 2. Attempts to authenticate via SSH agent.
/// 3. Falls back to password authentication if necessary.
///
/// # Arguments
/// * `target` - The SSH target in `user@host` format.
///
/// # Returns
/// A fully authenticated [`ssh2::Session`] ready for use.
///
/// # Errors
/// Returns an error if:
/// - The `target` string is malformed.
/// - TCP connection fails.
/// - SSH handshake fails.
/// - Authentication fails.
///
/// # Examples
/// ```no_run
/// let session = create_ssh_session("user@example.com")?;
/// ```
pub fn create_ssh_session(target: &str) -> Result<Session> {
    // Split `user@host` into username and hostname parts
    let mut parts = target.split('@');
    let user = parts.next().context("Missing user")?;
    let host = parts.next().context("Missing host")?;

    // Connect to the SSH server on port 22
    let tcp = TcpStream::connect(format!("{host}:22")).context("Failed to connect to SSH")?;

    // Create a new SSH session and attach the TCP stream
    let mut session = Session::new().context("Failed to create SSH session")?;
    session.set_tcp_stream(tcp);

    // Perform the SSH handshake
    session.handshake()?;

    // If already authenticated (unlikely at this point), return early
    if session.authenticated() {
        return Ok(session);
    }

    // Attempt to authenticate using the SSH agent
    session.userauth_agent(user).ok();
    if session.authenticated() {
        return Ok(session);
    }

    // Prompt for password if agent authentication failed
    let password = rpassword::prompt_password(format!("Enter SSH password for {target}:"))?;
    session.userauth_password(user, &password)?;

    // Final authentication check
    if session.authenticated() {
        Ok(session)
    } else {
        Err(anyhow::anyhow!("Authentication failed"))
    }
}

/// A trait for executing commands on a remote SSH session.
pub trait RemoteExecutor {
    /// Execute a shell command on the remote host.
    ///
    /// # Arguments
    /// * `cmd` - The command string to run.
    ///
    /// # Returns
    /// The captured stdout and stderr from the remote command.
    fn exec(&self, cmd: &str) -> Result<String>;

    /// Execute a command with `sudo` privileges on the remote host.
    ///
    /// # Arguments
    /// * `cmd` - The command string to run with `sudo`.
    /// * `password` - The sudo password for the remote user.
    ///
    /// # Returns
    /// The captured stdout and stderr from the remote command.
    fn sudo(&self, cmd: &str, password: &str) -> Result<String>;
}

/// A trait for securely uploading files and directories to a remote SSH host.
pub trait SecureUpload {
    /// Upload a file or directory to the remote host.
    ///
    /// If `local_path` is a directory, uploads recursively.
    fn scp_upload(&self, local_path: &Path, remote_path: &Path) -> Result<()>;

    /// Upload a single file to the remote host using SCP.
    fn upload_file(&self, local_path: &Path, remote_path: &Path) -> Result<()>;

    /// Recursively upload a directory to the remote host using SFTP.
    fn upload_recursive(&self, sftp: &Sftp, local: &Path, remote: &Path) -> Result<()>;
}


impl RemoteExecutor for Session {
fn exec(&self, cmd: &str) -> Result<String> {
        // Create a new SSH channel for the command
        let mut channel = self.channel_session()?;
        // Execute the command on the remote host
        channel.exec(cmd)?;
        // Capture the command output
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        // Wait for the command to finish
        channel.wait_close()?;
        Ok(output)
    }

    fn sudo(&self, cmd: &str, password: &str) -> Result<String> {
        // Create a new SSH channel with a pseudo-terminal (required for sudo)
        let mut channel = self.channel_session()?;
        channel.request_pty("xterm", None, None)?;

        // Format the sudo command to suppress password prompt text
        let sudo_cmd = format!("sudo -S -p '' {cmd}");
        channel.exec(&sudo_cmd)?;

        // Send the password to sudo
        write!(channel, "{}\n", password)?;
        channel.flush()?;

        // Capture the sudo command output
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }
}

impl SecureUpload for Session {
    fn scp_upload(&self, local_path: &Path, remote_path: &Path) -> Result<()> {
        // Start an SFTP session
        let sftp = self.sftp().context("failed to create SFTP session")?;

        // Upload either a directory (recursive) or a single file
        if local_path.is_dir() {
            self.upload_recursive(&sftp, local_path, remote_path)
        } else {
            self.upload_file(local_path, remote_path)
        }
    }

    fn upload_file(&self, local_path: &Path, remote_path: &Path) -> anyhow::Result<()> {
        // Open the local file for reading
        let mut local_file = File::open(local_path)?;
        let metadata = local_file.metadata()?;
        let file_size = metadata.len();

        // Ensure the remote file exists before SCP (touch creates it)
        self.exec(&format!("touch {}", remote_path.to_str().unwrap()))?;

        // Open remote file for writing via SCP
        let mut remote_file = self.scp_send(remote_path, 0o644, file_size, None)?;

        // Copy the local file's contents to the remote file
        std::io::copy(&mut local_file, &mut remote_file)?;

        Ok(())
    }

    fn upload_recursive(&self, sftp: &Sftp, local: &Path, remote: &Path) -> Result<()> {
        // Create the remote directory if it doesn't exist
        sftp.mkdir(remote, 0o755).ok(); // ignore "already exists" errors

        // Iterate through the local directory entries
        for entry in fs::read_dir(local).context("reading local dir")? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let local_entry = entry.path();
            let remote_entry = remote.join(entry.file_name());

            if file_type.is_dir() {
                // Recursively upload subdirectories
                self.upload_recursive(sftp, &local_entry, &remote_entry)?;
            } else if file_type.is_file() {
                // Upload files
                self.upload_file(&local_entry, &remote_entry)?;
            }
        }
        Ok(())
    }
}
