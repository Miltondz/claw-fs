use clap::{Parser, Subcommand};
use serde::Serialize;
use regex::Regex;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;

/// claw-fs: Universal Text & Code Manipulation Tool for AI Agents
#[derive(Parser)]
#[command(name = "claw-fs")]
#[command(about = "A safe, robust CLI tool for AI agents to manipulate text and code.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Append text to the end of a file
    Append {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        content: String,
    },
    /// Insert text at a specific line (1-indexed)
    Insert {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        line: usize,
        #[arg(short, long)]
        content: String,
    },
    /// Replace a specific block of lines (1-indexed, inclusive)
    ReplaceLines {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        start: usize,
        #[arg(short, long)]
        end: usize,
        #[arg(short, long)]
        content: String,
    },
    /// Read a specific chunk of lines (1-indexed, inclusive)
    ReadChunk {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        start: usize,
        #[arg(short, long)]
        end: usize,
    },
    /// Search for text with context
    FindRefs {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value_t = 3)]
        context: usize,
    },
    /// Get an outline of functions/classes (JS/TS/PY)
    Outline {
        #[arg(short, long)]
        file: String,
    },
    /// Undo the last change (restores from .bak)
    Undo {
        #[arg(short, long)]
        file: String,
    },
    /// Get stats of a file
    Stats {
        #[arg(short, long)]
        file: String,
    },
}

#[derive(Serialize)]
struct Output {
    status: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

fn print_json_and_exit(status: &str, message: &str, data: Option<serde_json::Value>) -> ! {
    let out = Output {
        status: status.to_string(),
        message: message.to_string(),
        data,
    };
    println!("{}", serde_json::to_string(&out).unwrap());
    if status == "error" {
        process::exit(1);
    } else {
        process::exit(0);
    }
}

fn create_backup(path: &Path) {
    if path.exists() {
        let mut bak_path = path.to_path_buf();
        bak_path.set_extension("bak");
        if let Err(e) = fs::copy(path, &bak_path) {
            print_json_and_exit("error", &format!("Could not create backup: {}", e), None);
        }
    }
}

fn atomic_write(path: &Path, content: &str) {
    create_backup(path);
    let mut tmp_path = path.to_path_buf();
    tmp_path.set_extension("tmp");
    if let Err(e) = fs::write(&tmp_path, content) {
        print_json_and_exit("error", &format!("Failed to write to temporary file: {}", e), None);
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        print_json_and_exit("error", &format!("Failed to finalize atomic write: {}", e), None);
    }
}

fn read_file(path: &Path) -> String {
    if !path.exists() {
        return "".to_string(); // Or throw error based on logic
    }
    match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            print_json_and_exit("error", &format!("Could not read file {}: {}", path.display(), e), None);
            unreachable!()
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Append { file, content } => {
            let path = Path::new(file);
            let mut text = read_file(path);
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(content);
            if !text.ends_with('\n') {
                text.push('\n');
            }
            atomic_write(path, &text);
            
            let line_cnt = text.lines().count();
            let word_cnt = text.split_whitespace().count();
            
            print_json_and_exit("success", "Contenido añadido correctamente al final del archivo.", Some(serde_json::json!({
                "lines": line_cnt,
                "words": word_cnt
            })));
        }
        
        Commands::Insert { file, line, content } => {
            let path = Path::new(file);
            if !path.exists() {
                print_json_and_exit("error", "El archivo no existe. Usa append para crearlo.", None);
            }
            let text = read_file(path);
            let mut lines: Vec<&str> = text.lines().collect();
            
            let idx = if *line > 0 { *line - 1 } else { 0 };
            if idx > lines.len() {
                print_json_and_exit("error", &format!("Rango inválido. El archivo solo tiene {} líneas.", lines.len()), None);
            }
            
            let mut new_text = String::new();
            for (i, l) in lines.iter().enumerate() {
                if i == idx {
                    new_text.push_str(content);
                    if !content.ends_with('\n') { new_text.push('\n'); }
                }
                new_text.push_str(l);
                new_text.push('\n');
            }
            if idx == lines.len() {
                new_text.push_str(content);
                if !content.ends_with('\n') { new_text.push('\n'); }
            }
            
            atomic_write(path, &new_text);
            print_json_and_exit("success", &format!("Texto insertado en la línea {}.", line), None);
        }

        Commands::ReplaceLines { file, start, end, content } => {
            let path = Path::new(file);
            if !path.exists() {
                print_json_and_exit("error", "El archivo no existe.", None);
            }
            if start > end || *start == 0 {
                print_json_and_exit("error", "El rango de líneas es inválido.", None);
            }
            
            let text = read_file(path);
            let lines: Vec<&str> = text.lines().collect();
            if *end > lines.len() {
                print_json_and_exit("error", &format!("El archivo solo tiene {} líneas. No puedes reemplazar hasta la {}.", lines.len(), end), None);
            }
            
            let start_idx = start - 1;
            let end_idx = end - 1;
            
            let mut new_text = String::new();
            for i in 0..start_idx {
                new_text.push_str(lines[i]);
                new_text.push('\n');
            }
            
            new_text.push_str(content);
            if !content.ends_with('\n') {
                new_text.push('\n');
            }
            
            for i in (end_idx + 1)..lines.len() {
                new_text.push_str(lines[i]);
                new_text.push('\n');
            }
            
            atomic_write(path, &new_text);
            print_json_and_exit("success", &format!("Reemplazadas las líneas {} a {}.", start, end), None);
        }

        Commands::ReadChunk { file, start, end } => {
            let path = Path::new(file);
            if !path.exists() {
                print_json_and_exit("error", "El archivo no existe.", None);
            }
            let text = read_file(path);
            let lines: Vec<&str> = text.lines().collect();
            
            let s_idx = if *start > 0 { start - 1 } else { 0 };
            let mut e_idx = if *end > 0 { end - 1 } else { 0 };
            if e_idx >= lines.len() { e_idx = lines.len() - 1; }
            if s_idx > e_idx || s_idx >= lines.len() {
                print_json_and_exit("error", "Rango inválido solicitado.", None);
            }
            
            let chunk = lines[s_idx..=e_idx].join("\n");
            print_json_and_exit("success", "Lectura exitosa.", Some(serde_json::json!({
                "chunk": chunk,
            })));
        }

        Commands::FindRefs { file, query, context } => {
            let path = Path::new(file);
            if !path.exists() {
                print_json_and_exit("error", "El archivo no existe.", None);
            }
            let text = read_file(path);
            let lines: Vec<&str> = text.lines().collect();
            
            let re = match Regex::new(query) {
                Ok(r) => r,
                Err(e) => print_json_and_exit("error", &format!("Regex inválido: {}", e), None),
            };
            
            let mut matches = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    let start_ctx = i.saturating_sub(*context);
                    let end_ctx = std::cmp::min(i + context + 1, lines.len());
                    
                    let mut chunk = String::new();
                    for j in start_ctx..end_ctx {
                        chunk.push_str(&format!("{}: {}\n", j + 1, lines[j]));
                    }
                    matches.push(serde_json::json!({
                        "line": i + 1,
                        "context_block": chunk
                    }));
                }
            }
            
            if matches.is_empty() {
                print_json_and_exit("error", "No se encontraron resultados para la búsqueda solicitada.", None);
            } else {
                print_json_and_exit("success", &format!("Se encontraron {} coincidencias.", matches.len()), Some(serde_json::json!({ "matches": matches })));
            }
        }

        Commands::Outline { file } => {
            let path = Path::new(file);
            if !path.exists() {
                print_json_and_exit("error", "El archivo no existe.", None);
            }
            let text = read_file(path);
            let lines: Vec<&str> = text.lines().collect();
            
            // Simple generic regex for classes, functions, const/let/var that look like functions
            let re = Regex::new(r"^(?P<indent>\s*)(?:export\s+|async\s+|public\s+|private\s+|protected\s+)?(?:class|function|def|struct|impl|interface)\s+(?P<name>\w+)").unwrap();
            let re_var = Regex::new(r"^(?P<indent>\s*)(?:export\s+)?(?:const|let|var)\s+(?P<name>\w+)\s*=\s*(?:async\s*)?(?:function|\(.*?\)\s*=>)").unwrap();
            
            let mut outline = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                if let Some(cap) = re.captures(line) {
                    outline.push(format!("{}: {}", i + 1, line.trim()));
                } else if let Some(cap) = re_var.captures(line) {
                    outline.push(format!("{}: {}", i + 1, line.trim()));
                }
            }
            
            print_json_and_exit("success", "Outline extraído con éxito.", Some(serde_json::json!({
                "outline": outline
            })));
        }

        Commands::Undo { file } => {
            let path = Path::new(file);
            let mut bak_path = path.to_path_buf();
            bak_path.set_extension("bak");
            
            if !bak_path.exists() {
                print_json_and_exit("error", "No hay un archivo de backup (.bak) para deshacer la última acción.", None);
            }
            
            if let Err(e) = fs::rename(&bak_path, path) {
                print_json_and_exit("error", &format!("No se pudo deshacer: {}", e), None);
            }
            
            print_json_and_exit("success", "Último cambio deshecho correctamente. El archivo volvió a su estado anterior.", None);
        }

        Commands::Stats { file } => {
            let path = Path::new(file);
            if !path.exists() {
                print_json_and_exit("error", "El archivo no existe.", None);
            }
            let text = read_file(path);
            let metadata = fs::metadata(path).unwrap();
            let size = metadata.len();
            
            let lines = text.lines().count();
            let words = text.split_whitespace().count();
            
            print_json_and_exit("success", "Estadísticas del archivo.", Some(serde_json::json!({
                "lines": lines,
                "words": words,
                "size_bytes": size,
            })));
        }
    }
}
