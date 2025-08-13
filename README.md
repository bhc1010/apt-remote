# Remote apt package managment

This is a command-line utility for **remote package management of offline Debian-based systems**. It enables the creation an image of the desired packages and their dependencies (repository lists) of an offline machine, downloading packages (repository lists) on an online machine, and installing packages (update apt package metadata), all via SSH. Dependency resolution and installation order are handled by _apt-get_ on the offline system.

![](https://raw.githubusercontent.com/bhc1010/apt-remote/refs/heads/main/assets/install-demo.gif)

## Installation

#### Debian/Ubuntu & macOS
The latest debian & macOS release can be installed with this bash script:

```bash
curl -sSL https://raw.githubusercontent.com/bhc1010/apt-remote/main/install.sh | bash
```

#### Windows
Windows users can download the latest `.msi` installer directly from the [GitHub Releases](https://github.com/bhc1010/apt-remote/releases) page and run it manually.


#### Build from source (requires Rust Toolchain):

```bash
git clone https://github.com/benca/apt-remote.git
cd apt-remote
cargo build --release
```

## Usage

`apt-remote` is structured around subcommands:

#### set: **generate a `uri.toml` image file**

```bash
# Specific packages
apt-remote set <NAME> --target user@host --install pkg1 pkg2 ...

# Up-to-date package metadata (like `apt-get update`)
apt-remote set <NAME> --target user@host --update

# Upgradable packages
apt-remote set <NAME> --target user@host --upgrade

# Packages needed to fix broken dependencies
apt-remote set <NAME> --target user@host --fix
```
Only one `uri.toml` file will exist for a given image name. Running set with a different flag will overwrite any existing uri's. The `--install`, `--upgrade`, and `--fix` flags will populate the `uri.toml` with metadata needed to download `.deb` packages, while the `--update` flag will populate it with repository source list metadata.

#### get: download packages/sources from `uri.toml`
```bash
apt-remote get <NAME>
```
When you run `apt-remote get <NAME>`, the packages or source lists described in `uri.toml` will be downloaded to local cache depending on the operating system. On Linux, the `uri.toml` file and any downloaded data are located at `$HOME/.cache/apt-remote/<NAME>`.

#### install: **`dpkg -i` packages on remote target**
```bash
apt-remote install <NAME> --target user@host
```
The install subcommand is intended for when `uri.toml` describes `.deb` packages. When you run `apt-remote install`, all downloaded packages are copied to `user@host:/tmp/apt-remote/<NAME>`, the checksums are verified on the offline system and are installed in the order determined by `apt-get` on the offline system.

#### update: **copy package lists to target and generate package cache**
![](https://raw.githubusercontent.com/bhc1010/apt-remote/refs/heads/main/assets/update-demo.gif)
```bash
apt-remote update <NAME> --target user@host
```
The update subcommand is intended for when `uri.toml` describes repository source lists. When you run `apt-remote update`, all downloaded package metadata is copied to `user@host:/var/lib/apt/lists` and the `pkgcache.bin` and `srcpkgcache.bin` cache files are regenerated. The old list files are moved to `/var/lib/apt/lists.old` but it still **may be recommended to backup these files before updating**

#### clear: **local package cache**
```bash
apt-remote clear
```
When you run `apt-remote clear`, all local cache files are removed.

## SSH Requirements

- Password-based or key-based SSH access to the remote machine
- `sudo` privileges on the remote machine

