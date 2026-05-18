use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
extern "C" {
    fn _getch() -> std::os::raw::c_int;
}

#[cfg(windows)]
fn read_char() -> char {
    unsafe { _getch() as u8 as char }
}

#[cfg(not(windows))]
fn read_char() -> char {
    // Basic fallback for non-windows
    let mut buf = [0; 1];
    std::io::Read::read(&mut std::io::stdin(), &mut buf).unwrap();
    buf[0] as char
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn move_cursor(x: u16, y: u16) {
    print!("\x1B[{};{}H", y + 1, x + 1);
}

pub fn start_ide() {
    let mut files: Vec<PathBuf> = fs::read_dir(".")
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "syl"))
        .map(|e| e.path())
        .collect();

    let mut selected_file_idx = 0;
    let mut current_file: Option<PathBuf> = None;
    let mut editor_content: Vec<String> = vec![String::new()];
    let mut output_content: Vec<String> = vec![String::new()];
    let mut in_editor = false;

    loop {
        let cols = 80;
        let rows = 24;
        let mid = cols / 3;
        let editor_h = (rows * 2) / 3;

        clear_screen();

        // Draw left pane (Explorer)
        for i in 0..rows {
            move_cursor(mid, i);
            print!("│");
        }

        // Draw horizontal divider for output pane
        for i in mid+1..cols {
            move_cursor(i, editor_h);
            print!("─");
        }
        move_cursor(mid, editor_h);
        print!("├");

        move_cursor(0, 0);
        print!("\x1b[1;36m  SYL EXPLORER  \x1b[0m");
        for (i, file) in files.iter().enumerate() {
            if i >= (rows - 2) as usize { break; }
            move_cursor(2, (i + 2) as u16);
            let name = file.file_name().unwrap().to_string_lossy();
            if !in_editor && i == selected_file_idx {
                print!("\x1b[40m> {}\x1b[0m", name);
            } else {
                print!("  {}", name);
            }
        }

        // Draw right pane (Editor)
        move_cursor(mid + 2, 0);
        print!("\x1b[1;36m  SYL EDITOR  (ESC=Explorer, S=Save, R=Run, Q=Quit)\x1b[0m");
        for (i, line) in editor_content.iter().enumerate() {
            if i >= (editor_h - 2) as usize { break; }
            move_cursor(mid + 2, (i + 2) as u16);
            print!("{}", line);
        }

        // Draw Output Pane
        move_cursor(mid + 2, editor_h + 1);
        print!("\x1b[1;35m  OUTPUT CONSOLE  \x1b[0m");
        let output_start_row = editor_h + 2;
        let max_output_lines = (rows - output_start_row).saturating_sub(1);
        
        let start_idx = output_content.len().saturating_sub(max_output_lines as usize);
        for (i, line) in output_content.iter().skip(start_idx).enumerate() {
            move_cursor(mid + 2, output_start_row + i as u16);
            print!("{}", line);
        }

        stdout().flush().unwrap();

        let c = read_char();
        if c == 'q' || c == 'Q' {
            break;
        }

        if in_editor {
            match c {
                '\x1b' => { // ESC
                    in_editor = false;
                }
                's' | 'S' => {
                    if let Some(ref path) = current_file {
                        fs::write(path, editor_content.join("\n")).unwrap();
                    }
                }
                'r' | 'R' => {
                    if let Some(ref path) = current_file {
                        fs::write(path, editor_content.join("\n")).unwrap();
                        output_content.clear();
                        output_content.push(format!("--- COMPILING {} ---", path.display()));
                        
                        let output = Command::new(std::env::current_exe().unwrap())
                            .arg("run")
                            .arg(path)
                            .output();
                            
                        if let Ok(out) = output {
                            let stdout_str = String::from_utf8_lossy(&out.stdout);
                            let stderr_str = String::from_utf8_lossy(&out.stderr);
                            for line in stdout_str.lines() { output_content.push(line.to_string()); }
                            for line in stderr_str.lines() { output_content.push(line.to_string()); }
                            output_content.push("--- PROCESS TERMINATED ---".to_string());
                        }
                    }
                }
                '\r' | '\n' => {
                    editor_content.push(String::new());
                }
                '\x08' => { // Backspace
                    if let Some(last) = editor_content.last_mut() {
                        last.pop();
                    }
                }
                ch => {
                    if let Some(last) = editor_content.last_mut() {
                        last.push(ch);
                    }
                }
            }
        } else {
            match c {
                'w' | 'W' => if selected_file_idx > 0 { selected_file_idx -= 1; },
                's' | 'S' => if selected_file_idx < files.len().saturating_sub(1) { selected_file_idx += 1; },
                '\r' | '\n' | 'e' | 'E' => {
                    if !files.is_empty() {
                        current_file = Some(files[selected_file_idx].clone());
                        let content = fs::read_to_string(current_file.as_ref().unwrap()).unwrap_or_default();
                        editor_content = content.lines().map(|s| s.to_string()).collect();
                        if editor_content.is_empty() { editor_content.push(String::new()); }
                        in_editor = true;
                    }
                }
                _ => {}
            }
        }
    }
    
    clear_screen();
}
