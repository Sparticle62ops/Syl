use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size},
};

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
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    let mut in_editor = false;

    enable_raw_mode().unwrap();
    let mut out = stdout();
    execute!(out, Hide).unwrap();

    loop {
        let (cols, rows) = size().unwrap();
        let mid = cols / 3;

        queue!(out, Clear(ClearType::All)).unwrap();

        // Draw left pane (Explorer)
        for i in 0..rows {
            queue!(out, MoveTo(mid, i), Print("│")).unwrap();
        }

        queue!(out, MoveTo(0, 0), SetForegroundColor(Color::Cyan), Print("  SYL EXPLORER  "), ResetColor).unwrap();
        for (i, file) in files.iter().enumerate() {
            if i >= (rows - 2) as usize { break; }
            queue!(out, MoveTo(2, (i + 2) as u16)).unwrap();
            let name = file.file_name().unwrap().to_string_lossy();
            if !in_editor && i == selected_file_idx {
                queue!(out, SetBackgroundColor(Color::DarkGrey), Print(format!("> {}", name)), ResetColor).unwrap();
            } else {
                queue!(out, Print(format!("  {}", name))).unwrap();
            }
        }

        // Draw right pane (Editor)
        queue!(out, MoveTo(mid + 2, 0), SetForegroundColor(Color::Cyan), Print("  SYL EDITOR  (Ctrl+S to Save, Ctrl+R to Run, Esc to Explorer)"), ResetColor).unwrap();
        for (i, line) in editor_content.iter().enumerate() {
            if i >= (rows - 2) as usize { break; }
            queue!(out, MoveTo(mid + 2, (i + 2) as u16), Print(line)).unwrap();
        }

        if in_editor {
            queue!(out, MoveTo(mid + 2 + cursor_x as u16, cursor_y as u16 + 2), Show).unwrap();
        } else {
            queue!(out, Hide).unwrap();
        }

        out.flush().unwrap();

        if let Event::Key(KeyEvent { code, modifiers, .. }) = read().unwrap() {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                break;
            }

            if in_editor {
                match code {
                    KeyCode::Esc => {
                        in_editor = false;
                        execute!(out, Hide).unwrap();
                    }
                    KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(ref path) = current_file {
                            fs::write(path, editor_content.join("\n")).unwrap();
                        }
                    }
                    KeyCode::Char('r') if modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(ref path) = current_file {
                            fs::write(path, editor_content.join("\n")).unwrap();
                            disable_raw_mode().unwrap();
                            execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0), Show).unwrap();
                            
                            println!("--- RUNNING {} ---", path.display());
                            let status = Command::new(std::env::current_exe().unwrap())
                                .arg("run")
                                .arg(path)
                                .status();
                            
                            println!("\nPress any key to return to IDE...");
                            enable_raw_mode().unwrap();
                            read().unwrap();
                        }
                    }
                    KeyCode::Left => if cursor_x > 0 { cursor_x -= 1; },
                    KeyCode::Right => {
                        if cursor_y < editor_content.len() && cursor_x < editor_content[cursor_y].len() {
                            cursor_x += 1;
                        }
                    },
                    KeyCode::Up => if cursor_y > 0 {
                        cursor_y -= 1;
                        if cursor_x > editor_content[cursor_y].len() {
                            cursor_x = editor_content[cursor_y].len();
                        }
                    },
                    KeyCode::Down => if cursor_y < editor_content.len() - 1 {
                        cursor_y += 1;
                        if cursor_x > editor_content[cursor_y].len() {
                            cursor_x = editor_content[cursor_y].len();
                        }
                    },
                    KeyCode::Backspace => {
                        if cursor_x > 0 {
                            editor_content[cursor_y].remove(cursor_x - 1);
                            cursor_x -= 1;
                        } else if cursor_y > 0 {
                            let curr = editor_content.remove(cursor_y);
                            cursor_y -= 1;
                            cursor_x = editor_content[cursor_y].len();
                            editor_content[cursor_y].push_str(&curr);
                        }
                    }
                    KeyCode::Enter => {
                        let remainder = editor_content[cursor_y].split_off(cursor_x);
                        cursor_y += 1;
                        cursor_x = 0;
                        editor_content.insert(cursor_y, remainder);
                    }
                    KeyCode::Char(c) => {
                        editor_content[cursor_y].insert(cursor_x, c);
                        cursor_x += 1;
                    }
                    _ => {}
                }
            } else {
                match code {
                    KeyCode::Up => if selected_file_idx > 0 { selected_file_idx -= 1; },
                    KeyCode::Down => if selected_file_idx < files.len().saturating_sub(1) { selected_file_idx += 1; },
                    KeyCode::Enter => {
                        if !files.is_empty() {
                            current_file = Some(files[selected_file_idx].clone());
                            let content = fs::read_to_string(current_file.as_ref().unwrap()).unwrap_or_default();
                            editor_content = content.lines().map(|s| s.to_string()).collect();
                            if editor_content.is_empty() { editor_content.push(String::new()); }
                            in_editor = true;
                            cursor_x = 0;
                            cursor_y = 0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(out, Show).unwrap();
    disable_raw_mode().unwrap();
}
