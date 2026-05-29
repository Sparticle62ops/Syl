use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("[ SYL SDK ] Starting Installation...");

    // 1. Determine LocalAppData path
    let local_app_data = env::var("LOCALAPPDATA").expect("Could not find LOCALAPPDATA environment variable.");
    let install_dir = Path::new(&local_app_data).join("Syl");
    let bin_dir = install_dir.join("bin");
    let lib_dir = install_dir.join("lib");

    // 2. Create directories
    fs::create_dir_all(&bin_dir).expect("Failed to create bin directory.");
    fs::create_dir_all(&lib_dir).expect("Failed to create lib directory.");

    println!("[ SYL SDK ] Installing to: {}", install_dir.display());

    // 3. Find current directory (where setup is running from)
    let current_dir = env::current_dir().expect("Failed to get current directory.");
    
    // 4. Copy syl.exe
    let target_dir = current_dir.join("target").join("debug");
    let syl_exe_src = target_dir.join("syl.exe");
    if syl_exe_src.exists() {
        fs::copy(&syl_exe_src, bin_dir.join("syl.exe")).expect("Failed to copy syl.exe");
        println!("[ SYL SDK ] Copied syl.exe");
    } else {
        println!("[ SYL ERROR ] Could not find syl.exe at {}", syl_exe_src.display());
        std::process::exit(1);
    }

    // 5. Copy raylib.dll (assuming it's in the root or tools folder)
    let raylib_src = current_dir.join("raylib.dll");
    if raylib_src.exists() {
        fs::copy(&raylib_src, bin_dir.join("raylib.dll")).expect("Failed to copy raylib.dll");
        println!("[ SYL SDK ] Copied raylib.dll");
    } else {
        // Look in tools/tcc or similar if not in root.
        let raylib_alt = current_dir.join("tools").join("tcc").join("raylib.dll");
        if raylib_alt.exists() {
            fs::copy(&raylib_alt, bin_dir.join("raylib.dll")).expect("Failed to copy raylib.dll");
            println!("[ SYL SDK ] Copied raylib.dll");
        } else {
            println!("[ SYL WARNING ] Could not find raylib.dll. Visual features may not work.");
        }
    }

    // 6. Copy lib directory
    let lib_src = current_dir.join("lib");
    if lib_src.exists() && lib_src.is_dir() {
        copy_dir_recursive(&lib_src, &lib_dir).expect("Failed to copy lib directory");
        println!("[ SYL SDK ] Copied standard library.");
    } else {
        println!("[ SYL WARNING ] Could not find lib/ directory.");
    }

    // 7. Update User PATH via setx
    println!("[ SYL SDK ] Updating User PATH...");
    update_user_path(&bin_dir);

    println!("[ SYL SDK ] Installation Complete. You may now use 'syl' from any terminal.");
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn update_user_path(bin_dir: &Path) {
    let bin_dir_str = bin_dir.to_str().unwrap();

    // Fetch current user PATH using PowerShell
    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", "[Environment]::GetEnvironmentVariable('Path', 'User')"])
        .output()
        .expect("Failed to execute powershell to read PATH");

    let mut current_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Check if the path is already in the User PATH
    if current_path.contains(bin_dir_str) {
        println!("[ SYL SDK ] Directory {} is already in PATH.", bin_dir_str);
        return;
    }

    // Append to path
    if !current_path.is_empty() && !current_path.ends_with(';') {
        current_path.push(';');
    }
    current_path.push_str(bin_dir_str);

    // Set new PATH using setx
    let setx_status = Command::new("setx")
        .args(&["PATH", &current_path])
        .status()
        .expect("Failed to execute setx");

    if !setx_status.success() {
        println!("[ SYL WARNING ] Failed to update PATH with setx. You may need to manually add {} to your User PATH.", bin_dir_str);
    }
}
