# NOVA Installation Guide

## Quick Start (All Platforms)

### Prerequisites
- **Rust 1.75+** (for building from source)
- **Git** (for cloning)
- **C compiler** (for native codegen)

### Option 1: Pre-built Binaries (Recommended)

#### Linux/macOS/WSL
```bash
curl --proto '=https' --tlsv1.2 -sSf https://nova-lang.org/install.sh | sh
```

#### Windows (PowerShell)
```powershell
irm https://nova-lang.org/install.ps1 | iex
```

#### Homebrew (macOS/Linux)
```bash
brew install nova-lang
```

#### Docker
```bash
docker pull nova-lang/nova:latest
docker run -it --rm -v $(pwd):/workspace nova-lang/nova:latest
```

---

### Option 2: Build from Source

#### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update stable
```

#### 2. Install System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get update && sudo apt-get install -y \
    build-essential pkg-config libssl-dev \
    clang llvm lld \
    cmake protobuf-compiler
```

**Fedora/RHEL:**
```bash
sudo dnf install -y \
    gcc gcc-c++ make pkg-config openssl-devel \
    clang llvm lld \
    cmake protobuf-compiler
```

**Arch Linux:**
```bash
sudo pacman -S --needed \
    base-devel pkg-config openssl \
    clang llvm lld \
    cmake protobuf
```

**macOS:**
```bash
brew install llvm pkg-config cmake protobuf
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

**Windows:**
1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) with "C++ build tools"
2. Install [LLVM](https://github.com/llvm/llvm-project/releases) (add to PATH)
3. Install [CMake](https://cmake.org/download/)
3. Use Developer Command Prompt or PowerShell

#### 3. Clone & Build
```bash
git clone https://github.com/samprit-web/NOVA.git
cd NOVA

# Build everything (takes 3-10 minutes first time)
cargo build --workspace --release

# Verify installation
./target/release/nova --version
# Output: nova 1.0.0
```

#### 4. Add to PATH (Optional)
```bash
# Linux/macOS - add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/NOVA/target/release:$PATH"

# Windows PowerShell - run as admin
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$HOME\NOVA\target\release", "User")
```

---

## Verify Installation

```bash
# Check version
nova --version
# nova 1.0.0 (compiler: rustc 1.75.0)

# Run hello world
echo 'fn main() { println!("Hello, NOVA!") }' > hello.nova
nova run hello.nova
# Hello, NOVA!

# Create a project
nova new my-app
cd my-app
nova run
# Hello, NOVA!
```

---

## Platform-Specific Notes

### Linux
- Requires `glibc 2.31+`
- For static linking: `cargo build --release --target x86_64-unknown-linux-musl` (install `musl-tools`)

### macOS
- Supports Intel (x86_64) and Apple Silicon (aarch64)
- Universal binary: `cargo build --release --target universal-apple-darwin`
- Minimum macOS 11.0 (Big Sur)

### Windows
- Requires Windows 10 1903+ (for modern APIs)
- Use `cargo build --release --target x86_64-pc-windows-msvc`
- For GNU toolchain: `--target x86_64-pc-windows-gnu`

### WebAssembly
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build for WASM
nova build --target=wasm32-unknown-unknown --release
# Output: target/wasm32-unknown-unknown/release/app.wasm
```

### Cross-compilation
```bash
# Linux -> Windows
cargo build --release --target x86_64-pc-windows-gnu

# Linux -> macOS (requires osxcross)
cargo build --release --target x86_64-apple-darwin

# macOS -> Linux
cargo build --release --target x86_64-unknown-linux-gnu
```

---

## IDE Setup

### VS Code (Recommended)
```bash
# Install extension
code --install-extension nova-lang.nova

# Or from marketplace: search "NOVA Language"
```

### Other Editors
- **Neovim**: Use `nova-analyzer` LSP with `nvim-lspconfig`
- **Helix**: Built-in NOVA support
- **Zed**: Community extension
- **IntelliJ**: NOVA plugin (coming soon)

---

## Troubleshooting

### Common Issues

**"linker `cc` not found"**
```bash
# Install build tools
# Ubuntu: sudo apt-get install build-essential
# Fedora: sudo dnf install gcc gcc-c++
# macOS: xcode-select --install
```

**"OpenSSL not found"**
```bash
# Ubuntu: sudo apt-get install libssl-dev pkg-config
# Fedora: sudo dnf install openssl-devel pkg-config
# macOS: brew install openssl pkg-config
export OPENSSL_DIR=$(brew --prefix openssl)
```

**"LLVM not found"**
```bash
# Ubuntu: sudo apt-get install llvm-dev libclang-dev
# Fedora: sudo dnf install llvm-devel clang-devel
# macOS: brew install llvm
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

**Slow compilation**
```bash
# Use more RAM for faster builds
export CARGO_BUILD_JOBS=8  # or number of CPU cores
# Or use mold linker (faster)
cargo install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### Getting Help
- **Documentation**: https://docs.nova-lang.org
- **Discord**: https://discord.gg/nova-lang
- **GitHub Issues**: https://github.com/samprit-web/NOVA/issues
- **Forum**: https://forum.nova-lang.org

---

## Next Steps

1. Read the [Language Guide](https://docs.nova-lang.org/guide)
2. Try the [Examples](../examples/)
3. Read the [Standard Library Docs](https://docs.nova-lang.org/std)
4. Join the community!