use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

fn get_root_dir_from_cli_arguments() -> PathBuf {
    let arguments = env::args().collect::<Vec<String>>();

    match arguments.len() {
        1 => {
            eprintln!("Usage: {} <directory>", arguments[0]);
            std::process::exit(1);
        }

        2 => PathBuf::from(&arguments[1]),

        _ => {
            println!("Only `{}` will be used.", arguments[1]);
            PathBuf::from(&arguments[1])
        }
    }
}

fn find_files_with_end_pattern(dir: &Path, end_pattern: &str) -> io::Result<Vec<PathBuf>> {
    let mut matches = Vec::<PathBuf>::new();

    let end_pattern = end_pattern.to_lowercase();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            matches.extend(find_files_with_end_pattern(&path, &end_pattern)?);
        } else if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
            if filename.to_lowercase().ends_with(&end_pattern) {
                matches.push(path.to_path_buf());
            }
        }
    }

    Ok(matches)
}

fn filter_files_with_keyword(files: &[PathBuf], keywords: &[&str]) -> io::Result<Vec<PathBuf>> {
    let mut matching_files = Vec::<PathBuf>::new();

    let keywords = keywords
        .iter()
        .map(|keyword| keyword.to_lowercase())
        .collect::<Vec<String>>();

    for path in files {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    let line = match line {
                        Ok(line) => line.to_lowercase(),

                        Err(error) => {
                            eprintln!("Error reading line in {:?}: {}", path, error);
                            continue;
                        }
                    };

                    if keywords.iter().any(|keyword| line.contains(keyword)) {
                        matching_files.push(path.to_path_buf());
                        break;
                    }
                }
            }

            Err(error) => {
                eprintln!("Error reading file {:?}: {}", path, error);
            }
        }
    }

    Ok(matching_files)
}

fn main() -> io::Result<()> {
    println!("处理中...\n");

    let path = get_root_dir_from_cli_arguments();

    if !path.is_dir() {
        eprintln!(
            "The provided argument `{}` is not a valid directory.",
            path.display()
        );

        std::process::exit(1);
    }

    let msgs = find_files_with_end_pattern(&path, "Msg.java")?;
    let msgs = filter_files_with_keyword(&msgs, &["Double"])?;

    let msg_names = msgs
        .iter()
        .filter_map(|file| file.file_stem())
        .filter_map(|file| file.to_str())
        .collect::<Vec<&str>>();

    let mut logics = find_files_with_end_pattern(&path, "Logic.java")?;
    logics.extend(find_files_with_end_pattern(&path, "LogicBase.java")?);

    // 找出交集。
    let logics = filter_files_with_keyword(&logics, &msg_names)?;
    let logics = logics
        .iter()
        .filter_map(|file| file.file_name())
        .filter_map(|file| file.to_str())
        .collect::<Vec<&str>>();

    let mut output = File::create("output.txt")?;

    for item in logics {
        println!("{}", item);
        writeln!(&mut output, "{}", item)?;
    }

    println!("\n完成！");

    Ok(())
}
