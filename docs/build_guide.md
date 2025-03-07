# Building Rusty Smart Stitch

This guide will walk you through building Rusty Smart Stitch from source.

## Prerequisites

1. **Rust Toolchain**:
   ```bash
   # Install Rust from https://rustup.rs/
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   # Update to latest
   rustup update
   ```

2. **System Dependencies**:
   
   Windows:
   ```
   - Visual Studio Build Tools with C++ Desktop development
   - Windows 10 SDK
   ```

   Linux:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install build-essential pkg-config libwebp-dev libjpeg-dev

   # Fedora
   sudo dnf install gcc-c++ webp-devel libjpeg-turbo-devel
   ```

   macOS:
   ```bash
   brew install webp jpeg
   ```

## Project Structure

```
rusty-smart-stitch/
├── src/
│   ├── lib.rs           # Core image processing logic 
│   ├── main.rs          # GUI and entry point
│   ├── main_tab.rs      # Main interface tab
│   ├── advanced_tab.rs  # Advanced settings tab
│   ├── waifu2x_tab.rs   # Waifu2x integration tab
│   ├── about_tab.rs     # About section
│   ├── profile.rs       # Settings profiles management
│   ├── checkupd.rs      # Update system
│   ├── process_handler.rs # Processing logic
│   ├── folder_handler.rs # File management
│   ├── style.rs         # UI styling
│   └── slicelogic/      # Image slicing logic
│       ├── processor.rs  # Main processing logic
│       ├── image_merger.rs # Image merging logic
│       ├── image_saver.rs  # Image saving logic
│       ├── slice_location.rs # Slice location detection
│       └── psd_sup.rs    # PSD support functions
├── assets/              # Icons and resources
├── docs/               # Documentation
└── build.rs           # Build configuration
```

## Dependencies

The project uses several key dependencies (from `Cargo.toml`):

### Core Dependencies
- **GUI Framework**:
  - eframe = "0.29.1"
  - egui = "0.29.1"
  - egui_extras = "0.29.1"

- **Image Processing**:
  - image = "0.25.5"
  - webp = "0.3.0"
  - mozjpeg = "0.10.12"
  - psd = "0.3.5"  # Ensure this is up-to-date with the latest handling changes

- **File Handling**:
  - native-dialog = "0.7.0"
  - walkdir = "2.5.0"
  - tempfile = "3.14.0"
  - dirs = "5.0.0"

- **Utilities**:
  - rayon = "1.10.0" (parallelization)
  - anyhow = "1.0.93" (error handling)
  - serde = { version = "1.0", features = ["derive"] }
  - serde_json = "1.0.133"

- **Update System**:
  - tokio = { version = "1.41.1", features = ["full"] }
  - reqwest = { version = "0.12.9", features = ["json", "stream"] }
  - semver = "1.0.23"
  - self_update = "0.41.0"

### Platform-Specific Dependencies
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3.9", features = ["winbase"] }

[build-dependencies]
winres = "0.1.12"  # Windows resource handling
```

## Building from Source

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/kevinmartz/Rusty-Smart-Stitch.git
   cd Rusty-Smart-Stitch
   ```

2. **Build for Your Platform**:

   Windows (Default):
   ```bash
   cargo build --release
   ```

   Linux:
   ```bash
   cargo build --release
   cargo build --target x86_64-unknown-linux-gnu --release // if you are building on windows check .cargo/config.toml
   ```

   macOS:
   ```bash
   cargo build --release
   ```

   The binary will be at:
   - Windows: `target/release/rusty-smart-stitch.exe`
   - Linux: `target/x86_64-unknown-linux-gnu/release/rusty-smart-stitch`
   - macOS: `target/aarch64-apple-darwin/release/rusty-smart-stitch`

## Release Optimization

The project uses optimized release settings in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"         # Link-time optimization
debug = false       # No debug info
codegen-units = 1   # Better optimization
strip = "symbols"   # Remove symbols
panic = "abort"     # Smaller binary
```

## Common Issues

### Windows Build Issues
1. **Linker Errors**:
   - Install Visual Studio Build Tools
   - Select "Desktop development with C++"
   - Install Windows 10 SDK

2. **WebP/JPEG Errors**:
   - Check Visual Studio installation
   - Ensure Windows 10 SDK is installed

### Linux Build Issues
1. **Missing Libraries**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libwebp-dev libjpeg-dev

   # Fedora
   sudo dnf install gcc-c++ webp-devel libjpeg-turbo-devel
   ```

2. **Permission Issues**:
   ```bash
   sudo chown -R $USER:$USER ~/.cargo
   ```

### macOS Build Issues
1. **Missing Libraries**:
   ```bash
   brew install webp jpeg
   ```

2. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```

## Development Setup

Recommended VS Code extensions:
- Rust Analyzer (rust-lang.rust-analyzer)
- CodeLLDB (vadimcn.vscode-lldb)
- Even Better TOML (tamasfe.even-better-toml)

## Running Tests

```bash
cargo test
```

## Additional Resources

- [Project Issues](https://github.com/kevinmartz/Rusty-Smart-Stitch/issues)
- [egui Documentation](https://docs.rs/egui/0.29.1/egui/)
- [Rust Book](https://doc.rust-lang.org/book/)

## Note on slicelogic Module Changes

Please be aware that the `slicelogic` module has undergone significant updates, particularly in how PSD files are handled. Ensure that you check the module for any new dependencies or changes that may affect the build process. 