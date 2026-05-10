# Installation Guide

Hornet can be installed on Linux, macOS, and Windows. Since the compiler is written in Rust and targets LLVM, you will need the Rust toolchain and LLVM development headers.

## 1. Prerequisites

### Linux (General)
You need `build-essential`, `llvm`, and `clang`.

#### Ubuntu / Debian / Mint
```bash
sudo apt update
sudo apt install build-essential llvm-dev libclang-dev clang
```

#### Fedora / RHEL / CentOS
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install llvm-devel clang-devel
```

#### Arch Linux / Manjaro
```bash
sudo pacman -S base-devel llvm clang
```

### macOS
Ensure you have Xcode Command Line Tools and Homebrew installed.
```bash
xcode-select --install
brew install llvm
```

### Windows
1. Install **Visual Studio Build Tools** (Select "Desktop development with C++").
2. Install **LLVM**:
   - Using winget: `winget install LLVM.LLVM`
   - Using Scoop: `scoop install llvm`

---

## 2. Installing the Hornet Toolchain

### Option A: One-Line Installer (Linux & macOS)
Run the following command in your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/roland-yegon/hornet/main/install.sh | bash
```

### Option B: Build from Source (All OS)
1. **Clone the repository**:
   ```bash
   git clone https://github.com/roland-yegon/hornet.git
   cd hornet
   ```
2. **Build with Cargo**:
   ```bash
   cargo build --release
   ```
3. **Install to PATH**:
   - **Linux/macOS**:
     ```bash
     sudo cp target/release/hornet /usr/local/bin/
     ```
   - **Windows (PowerShell Admin)**:
     ```powershell
     Copy-Item .\target\release\hornet.exe -Destination C:\Windows\System32\
     ```
     *(Or add the `target/release` folder to your Environment Variables PATH)*

---

## 3. Post-Installation
Verify that hornet is installed by running:
```bash
hornet --help
```

You should see the Hornet CLI menu.

## 4. Uninstallation
To remove Hornet from your system:
- **Linux/macOS**: `sudo rm /usr/local/bin/hornet`
- **Windows**: Delete `hornet.exe` from the directory where you placed it.
