# Installing maeh

`maeh` is distributed as GitHub Release binaries. The repo is public, so no GitHub token is required. The installer detects the current platform, downloads the matching release asset plus its checksum, verifies the checksum, and installs the binary as `maeh`.

## Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash
```

This installs to `~/.local/bin/maeh`.

## Release asset install

The release workflow publishes `install.sh` as a release asset. After the next tagged release that includes the installer, this URL also works:

```bash
curl -fsSL https://github.com/ahnsv/maeh/releases/latest/download/install.sh | bash
```

## Custom install directory

```bash
curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash -s -- --dir /usr/local/bin
```

Equivalent environment variable:

```bash
MAEH_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash
```

## Pin a version

```bash
curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash -s -- --version v0.1.0
```

Equivalent environment variable:

```bash
MAEH_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash
```

## Supported platforms

Current release assets:

- `maeh-linux-x86_64`
- `maeh-macos-arm64`

Unsupported platforms fail before downloading.

## Verification

```bash
maeh --help
maeh doctor
```

If `maeh` installs successfully but your shell cannot find it, add the install directory to `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```
