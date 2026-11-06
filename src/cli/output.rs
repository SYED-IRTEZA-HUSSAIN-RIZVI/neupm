use std::path::Path;

/// Print standard header banner for CLI operations.
pub fn print_banner() {
    println!("NEUPM - Package Manager");
}

/// Print resolving metadata message.
pub fn print_resolving() {
    println!("Resolving package metadata...\n");
}

/// Print resolved package information.
pub fn print_package_info(name: &str, version: &str) {
    println!("Package: {name}");
    println!("Version: {version}");
}

/// Print download progress messages.
pub fn print_downloading() {
    println!("\nDownloading package...");
}

pub fn print_download_completed(bytes: usize) {
    println!("Download completed: {bytes} bytes");
}

/// Print integrity verification messages.
pub fn print_verifying_integrity() {
    println!("\nVerifying SHA-512 integrity...");
}

pub fn print_integrity_passed() {
    println!("SHA-512 integrity verification passed.");
}

/// Print extracting message.
pub fn print_extracting() {
    println!("\nExtracting package...");
}

/// Print installation success summary box.
pub fn print_install_success(package: &str, version: &str, location: &Path) {
    println!("\n----------------------------------------");
    println!("Package installed successfully!");
    println!("----------------------------------------");
    println!("Name:     {package}");
    println!("Version:  {version}");
    println!("Location: {}", location.display());
    println!("Integrity: Verified");
    println!("----------------------------------------");
}
