use chrono::Local;
use clap::Parser;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;

///Bulk rename all files in a directory, or all files in command line
#[derive(Parser)]
struct Args {
    ///Ignores folders, rather than renaming all
    #[clap(short = 'I', long = "ignore_folders", default_value = "false")]
    ignore_folders: bool,
    ///Directory to rename their contents
    #[clap(short = 'F', long = "folder", default_value = None)]
    folder: Option<String>,
    ///Format for the new name
    #[clap[short = 'f', long  = "format", default_value = "${rand:16}"]]
    format: String,
    ///All files, directories to rename
    rename_items: Vec<String>,
}

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // println!("Usage: {} [OPTIONS] [file1] [file2] ...", args[0]);
        println!("Usage: knam [OPTIONS] [file1] [file2] ...");
        return;
    }
    let args = Args::parse();

    for file_path in args.rename_items.iter() {
        let path = Path::new(file_path);
        if !path.exists() {
            continue;
        }
        if let Ok(abs_path) = dunce::canonicalize(&path) {
            let is_dir = fs::metadata(&abs_path).unwrap().is_dir();
            if is_dir && args.ignore_folders {
                continue;
            }
            let mut ext = String::new();
            if !is_dir {
                ext = abs_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_string();
                ext = format!(".{}", ext);
            }
            let new_name = format_string(args.format.as_str());
            let mut new_path = abs_path
                .parent()
                .unwrap()
                .join(format!("{}{}", new_name, &ext));
            if new_path.exists() {
                new_path = abs_path.parent().unwrap().join(get_name_iter(
                    abs_path.parent().unwrap(),
                    new_name,
                    ext,
                ));
            }
            if let Err(err) = fs::rename(&abs_path, new_path) {
                println!(
                    "Error renaming file {}: {}",
                    abs_path.to_string_lossy(),
                    err
                );
            }
        }
    }
}

fn format_string(input: &str) -> String {
    let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
    let result = re.replace_all(input, |captures: &regex::Captures| {
        let parts = captures
            .get(1)
            .unwrap()
            .as_str()
            .splitn(2, ':')
            .collect::<Vec<_>>();
        match parts[0] {
            "time" => Local::now().format(parts[1]).to_string(),
            "date" => Local::now().format(parts[1]).to_string(),
            "rand" => thread_rng()
                .sample_iter(&Alphanumeric)
                .take(parts[1].parse::<usize>().unwrap_or(16))
                .map(char::from)
                .collect(),
            _ => captures[0].to_string(),
        }
    });
    result.to_string()
}

fn get_name_iter(path: &Path, name: String, ext: String) -> String {
    let mut idx: i32 = 1;
    let mut lpos: i32 = 0;
    let mut rpos: i32 = 16;
    let parent = dunce::canonicalize(path);
    // Idk the number of existing files
    loop {
        if parent
            .as_ref()
            .unwrap()
            .join(format!("{} ({}){}", name, rpos, ext))
            .exists()
        {
            // Set the range to rpos..(rpos + 32)
            lpos = rpos;
            rpos += 32;
            continue;
        }
        // Decrease range size
        if rpos > 16 {
            if parent
                .as_ref()
                .unwrap()
                .join(format!("{} ({}){}", name, rpos - 16, ext))
                .exists()
            {
                lpos += 16;
            } else {
                rpos -= 16;
            }
        }
        // But here I know where to look for an available index
        for i in lpos..rpos {
            if !parent
                .as_ref()
                .unwrap()
                .join(format!("{} ({}){}", name, i + 1, ext))
                .exists()
            {
                // Break when the file name doesn't exist
                idx = i + 1;
                break;
            }
        }
        break;
    }
    format!("{} ({}){}", name, idx, ext)
}
