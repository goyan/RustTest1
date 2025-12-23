# 💾 Disk Capacity Dashboard

A modern, futuristic Rust application that displays real-time disk capacity information in a beautiful dashboard interface.

## Features

- 🚀 **Real-time monitoring** - Automatically refreshes disk information
- 🎨 **Modern UI** - Clean, futuristic design with dark theme
- ⚡ **Fast & Efficient** - Built with Rust for optimal performance
- 📊 **Visual Progress Bars** - Easy-to-read capacity indicators
- 🖥️ **Multi-Disk Support** - Shows all connected drives
- 🎯 **Color-coded Alerts** - Visual warnings for high disk usage

## Installation

### Step 1: Install Rust

**For Windows (Recommended):**

1. Download and run the Rust installer from: https://rustup.rs/
2. Or use PowerShell:
   ```powershell
   winget install Rustlang.Rustup
   ```
3. Restart your terminal after installation

**Verify installation:**
```powershell
rustc --version
cargo --version
```

### Step 2: Build and Run

1. Navigate to the project directory:
   ```powershell
   cd RustTest1
   ```

2. Build the project:
   ```powershell
   cargo build --release
   ```

3. Run the application:
   ```powershell
   cargo run --release
   ```

Or simply run:
```powershell
cargo run
```

## Usage

The dashboard will automatically:
- Detect all connected disks (HDD, SSD, etc.)
- Display disk name, type, and mount point
- Show total, used, and available space
- Update information every second
- Color-code disks based on usage:
  - 🟢 Green: < 75% full
  - 🟠 Orange: 75-90% full
  - 🔴 Red: > 90% full

## Project Structure

```
RustTest1/
├── Cargo.toml      # Project dependencies and metadata
├── src/
│   └── main.rs     # Main application code
└── README.md       # This file
```

## Dependencies

- **eframe** - Application framework for egui
- **egui** - Immediate mode GUI library
- **sysinfo** - System information gathering
- **tokio** - Async runtime (for future enhancements)

## Troubleshooting

### Rust not found
- Make sure Rust is installed and added to PATH
- Restart your terminal after installation
- Run `rustup update` to ensure latest version

### Build errors
- Run `cargo clean` and try again
- Ensure you have the latest Rust toolchain: `rustup update stable`

### GUI not displaying
- Make sure you're running on Windows (native, not WSL)
- Check that graphics drivers are up to date

## Future Enhancements

- Custom refresh intervals
- Disk I/O statistics
- Historical usage graphs
- Disk health monitoring
- Export data to CSV/JSON

## License

This project is open source and available for personal use.

