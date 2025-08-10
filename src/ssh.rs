use anyhow::{Context, Result};
use ssh2::{Session, Sftp};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

pub fn create_ssh_session(target: &str) -> Result<Session> {
    let mut parts = target.split('@');
    let user = parts.next().context("Missing user")?;
    let host = parts.next().context("Missing host")?;

    let tcp = TcpStream::connect(format!("{host}:22")).context("Failed to connect to SSH")?;
    let mut session = Session::new().context("Failed to create SSH session")?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    if session.authenticated() {
        return Ok(session);
    }

    session.userauth_agent(user).ok();
    if session.authenticated() {
        return Ok(session);
    }

    let password = rpassword::prompt_password(format!("Enter SSH password for {target}:"))?;
    session.userauth_password(user, &password)?;

    if session.authenticated() {
        Ok(session)
    } else {
        Err(anyhow::anyhow!("Authentication failed"))
    }
}

pub trait RemoteExecutor {
    fn exec(&self, cmd: &str) -> Result<String>;
    fn sudo(&self, cmd: &str, password: &str) -> Result<String>;
}

pub trait SecureUpload {
    fn scp_upload(&self, local_path: &Path, remote_path: &Path) -> Result<()>;
    fn upload_file(&self, local_path: &Path, remote_path: &Path) -> Result<()>;
    fn upload_recursive(&self, sftp: &Sftp, local: &Path, remote: &Path) -> Result<()>;
}

impl RemoteExecutor for Session {
    fn exec(&self, cmd: &str) -> Result<String> {
        let mut channel = self.channel_session()?;
        channel.exec(cmd)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }

    fn sudo(&self, cmd: &str, password: &str) -> Result<String> {
        let mut channel = self.channel_session()?;
        channel.request_pty("xterm", None, None)?;

        let sudo_cmd = format!("sudo -S -p '' {cmd}");
        channel.exec(&sudo_cmd)?;

        write!(channel, "{}\n", password)?;
        channel.flush()?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }
}

impl SecureUpload for Session {
    fn scp_upload(&self, local_path: &Path, remote_path: &Path) -> Result<()> {
        let sftp = self.sftp().context("failed to create SFTP session")?;

        if local_path.is_dir() {
            self.upload_recursive(&sftp, local_path, remote_path)
        } else {
            self.upload_file(local_path, remote_path)
        }
    }

    fn upload_file(&self, local_path: &Path, remote_path: &Path) -> anyhow::Result<()> {
        // Read the local file
        let mut local_file = File::open(local_path)?;
        let metadata = local_file.metadata()?;
        let file_size = metadata.len();

        self.exec(&format!("touch {}", remote_path.to_str().unwrap()))?;

        // Start SCP send (mode is usually 0o644 for a regular file)
        let mut remote_file = self.scp_send(remote_path, 0o644, file_size, None)?;

        // Copy contents
        std::io::copy(&mut local_file, &mut remote_file)?;

        Ok(())
    }

    fn upload_recursive(&self, sftp: &Sftp, local: &Path, remote: &Path) -> Result<()> {
        sftp.mkdir(remote, 0o755).ok(); // ignore if already exists
        for entry in fs::read_dir(local).context("reading local dir")? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let local_entry = entry.path();
            let remote_entry = remote.join(entry.file_name());

            if file_type.is_dir() {
                self.upload_recursive(sftp, &local_entry, &remote_entry)?;
            } else if file_type.is_file() {
                self.upload_file(&local_entry, &remote_entry)?;
            }
        }
        Ok(())
    }
}
