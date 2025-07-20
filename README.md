# SafeBackup Rust Implementation

A secure file backup utility implemented in Rust with memory safety guarantees.

## Features

- **Secure file operations**: Backup, restore, and delete files with proper validation
- **Memory safety**: No buffer overflows or memory corruption vulnerabilities
- **Atomic operations**: Temp files + rename for crash safety
- **Input validation**: Strict filename whitelisting
- **Comprehensive logging**: Timestamped audit trail

## Installation

1. Install Rust (if needed):
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2. Clone repository
   ```sh
   git clone https://github.com/sundusmubeen/safe_backup_rust
   cd safe_backup_rust
3. Build release version
   ```sh
   cargo build --release