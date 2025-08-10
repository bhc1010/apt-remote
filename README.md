# apt-remote

**apt-remote** is a command-line utility for managing **offline APT package installation** over SSH.  
It allows you to download packages and metadata on one machine (with internet access), transfer them to another machine (without internet access), and install or update them there.

---

## ✨ Features

- 📦 **Download & cache** APT packages and metadata for offline use
- 🔄 **Transfer over SSH** with progress indicators
- 🧾 **Checksum verification** to ensure package integrity
- 📋 **Supports**:
  - Installing packages offline
  - Updating APT lists without an internet connection
  - Clearing local caches

---

## 📥 Installation

You can install `apt-remote` from the provided `.deb` file:

```bash
wget https://github.com/<YOUR_USERNAME>/<YOUR_REPO>/releases/latest/download/apt-remote_x.x.x_amd64.deb
sudo dpkg -i apt-remote_x.x.x_amd64.deb
```

> Replace `x.x.x` with the actual version from the [Releases](https://github.com/<YOUR_USERNAME>/<YOUR_REPO>/releases) page.

If you prefer to build from source:

```bash
git clone https://github.com/<YOUR_USERNAME>/<YOUR_REPO>.git
cd <YOUR_REPO>
cargo build --release
sudo install -m 755 target/release/apt-remote /usr/local/bin/apt-remote
```

---

## 🚀 Usage

`apt-remote` is structured around subcommands:

### 1. **Generate a `uri.toml` metadata file**
```bash
apt-remote set <NAME> --packages <pkg1> <pkg2> ...
```

### 2. **Download packages & metadata**
```bash
apt-remote get <NAME>
```

### 3. **Install packages on a remote system**
```bash
apt-remote install <NAME> --target user@remote-host
```

### 4. **Update package lists on a remote system**
```bash
apt-remote update <NAME> --target user@remote-host
```

### 5. **Clear local cache**
```bash
apt-remote clear
```

---

## 📂 Cache Location

By default, all cached files (metadata & `.deb` packages) are stored in:

```
$HOME/.cache/apt-remote/
```

---

## 🔐 SSH Requirements

- Password-based or key-based SSH access to the remote machine
- `sudo` privileges on the remote machine

---

## 🛠 Development

Clone the repository and build in debug mode:

```bash
git clone https://github.com/<YOUR_USERNAME>/<YOUR_REPO>.git
cd <YOUR_REPO>
cargo build
```

Run the CLI:

```bash
./target/debug/apt-remote --help
```

---

## 📜 License

This project is licensed under the **MIT License**.  
See [LICENSE](LICENSE) for details.

---
