use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use ssh2::{Session, Sftp};

impl Session {
    pub fn scp_upload(&self, local_path: &Path, remote_path: &Path) -> Result<()> {
        let sftp = self.sftp().context("failed to create SFTP session")?;

        if local_path.is_dir() {
            self.upload_recursive(&sftp, local_path, remote_path)
        } else {
            self.upload_file(&sftp, local_path, remote_path)
        }
    }

    pub fn scp_download(&self, remote_path: &Path, local_path: &Path) -> Result<()> {
        let sftp = self.sftp().context("failed to create SFTP session")?;

        let stat = sftp.stat(remote_path).context("stat failed")?;
        if stat.is_dir() {
            self.download_recursive(&sftp, remote_path, local_path)
        } else {
            self.download_file(&sftp, remote_path, local_path)
        }
    }

    fn upload_file(&self, sftp: &Sftp, local: &Path, remote: &Path) -> Result<()> {
        let mut local_file = File::open(local)
            .with_context(|| format!("opening local file {:?}", local))?;
        let mut remote_file = sftp.create(remote)
            .with_context(|| format!("creating remote file {:?}", remote))?;
        std::io::copy(&mut local_file, &mut remote_file)
            .with_context(|| format!("uploading file {:?}", local))?;
        Ok(())
    }

    fn download_file(&self, sftp: &Sftp, remote: &Path, local: &Path) -> Result<()> {
        let mut remote_file = sftp.open(remote)
            .with_context(|| format!("opening remote file {:?}", remote))?;
        let mut local_file = File::create(local)
            .with_context(|| format!("creating local file {:?}", local))?;
        std::io::copy(&mut remote_file, &mut local_file)
            .with_context(|| format!("downloading file {:?}", remote))?;
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
                self.upload_file(sftp, &local_entry, &remote_entry)?;
            }
        }
        Ok(())
    }

    fn download_recursive(&self, sftp: &Sftp, remote: &Path, local: &Path) -> Result<()> {
        fs::create_dir_all(local)
            .with_context(|| format!("creating local dir {:?}", local))?;
        let mut dir = sftp.opendir(remote)
            .with_context(|| format!("opening remote dir {:?}", remote))?;

        while let Some(entry) = dir.read()? {
            let filename = match entry.filename() {
                Some(name) if name != "." && name != ".." => name,
                _ => continue,
            };

            let remote_entry = remote.join(&filename);
            let local_entry = local.join(&filename);
            let stat = sftp.stat(&remote_entry)?;

            if stat.is_dir() {
                self.download_recursive(sftp, &remote_entry, &local_entry)?;
            } else {
                self.download_file(sftp, &remote_entry, &local_entry)?;
            }
        }

        Ok(())
    }
}
