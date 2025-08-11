# apt-remote

**apt-remote** is a command-line utility for managing **APT update and package installation over SSH to an offline machine**. It allows you to create a uri image of the desired packages and their dependencies (repository lists) of an offline machine, download packages (repository lists) on an online machine, and install packages (update apt package metadata) all via SSH. Dependency resolution and installation order are handled by _apt-get_ on the offline machine.

---

## ✨ Features

- 📦 **Download & cache** APT packages and metadata
- 🔄 **Transfer over SSH** to an offline machine
- 🧾 **Checksum verification** to ensure package integrity
- 📋 **Supports**:
  - Installing packages offline
  - Updating APT lists and package cache
  - Clearing local image cache

---

## 📥 Installation

You can install `apt-remote` from pre-built binaries:

```bash
curl -sSL https://raw.githubusercontent.com/benca/apt-remote/main/scripts/install.sh | bash
```

or build from source (requires Rust Toolchain):

```bash
git clone https://github.com/benca/apt-remote.git
cd apt-remote
cargo build --release
```

---

## 🚀 Usage

`apt-remote` is structured around subcommands:

### set: **Generate a `uri.toml` image file**

```bash
apt-remote set <NAME> --target user@host --install <pkg1> <pkg2> ...
```
```bash
apt-remote set <NAME> --target user@host --update
```
```bash
apt-remote set <NAME> --target user@host --upgrade 
```
```bash
apt-remote set <NAME> --target user@host --fix
```

### get: **Download packages/sources from `uri.toml`**
```bash
apt-remote get <NAME>
```

### install: **`scp` and `dpkg -i` packages on remote target**
```bash
apt-remote install <NAME> --target user@host
```

### update: **`scp` package lists to target and generate package cache**
```bash
apt-remote update <NAME> --target user@host
```

### clear: **Clear local package cache**
```bash
apt-remote clear
```


By default, all cached files (metadata & `.deb` packages) are stored in:

```
$HOME/.cache/apt-remote/<NAME>
```

---

## 🔐 SSH Requirements

- Password-based or key-based SSH access to the remote machine
- `sudo` privileges on the remote machine

