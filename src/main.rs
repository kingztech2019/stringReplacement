#[derive(Debug)]
struct Arguments {
    target: String,
    replacement: String,
    filename: String,
    output: String,
    case_insensitive: bool,
    interactive: bool,
    preview: bool,
    log_file: Option<String>,
}

use text_colorizer::*;
use std::{env, fs};
use regex::Regex;
use std::io::{self, Write};

fn print_usage() {
    eprintln!(
        "{} - Change occurrences of one string into another\n",
        "QuickReplace".green().bold()
    );
    eprintln!("{}:", "Usage".blue().bold());
    eprintln!("  quickreplace <target> <replacement> <INPUT> <OUTPUT> [options]");
    eprintln!("\n{}:", "Options".blue().bold());
    eprintln!("  --case-insensitive      Perform case-insensitive replacement");
    eprintln!("  --interactive           Review each replacement interactively");
    eprintln!("  --preview               Show a preview of the changes before saving");
    eprintln!("  --log-file <FILE>       Save a log of changes to the specified file");
    eprintln!();
}

fn parse_args() -> Arguments {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut case_insensitive = false;
    let mut interactive = false;
    let mut preview = false;
    let mut log_file = None;

    // Parse options
    if let Some(index) = args.iter().position(|x| x == "--case-insensitive") {
        case_insensitive = true;
        args.remove(index);
    }
    if let Some(index) = args.iter().position(|x| x == "--interactive") {
        interactive = true;
        args.remove(index);
    }
    if let Some(index) = args.iter().position(|x| x == "--preview") {
        preview = true;
        args.remove(index);
    }
    if let Some(index) = args.iter().position(|x| x == "--log-file") {
        if index + 1 < args.len() {
            log_file = Some(args[index + 1].clone());
            args.drain(index..=index + 1);
        } else {
            eprintln!("{} Missing file name after --log-file\n", "Error:".red().bold());
            print_usage();
            std::process::exit(1);
        }
    }

    if args.len() != 4 {
        print_usage();
        eprintln!(
            "{} Wrong number of arguments: expected 4, got {}.\n",
            "Error:".red().bold(),
            args.len()
        );
        std::process::exit(1);
    }

    Arguments {
        target: args[0].clone(),
        replacement: args[1].clone(),
        filename: args[2].clone(),
        output: args[3].clone(),
        case_insensitive,
        interactive,
        preview,
        log_file,
    }
}

fn replace(
    target: &str,
    replacement: &str,
    text: &str,
    case_insensitive: bool,
) -> Result<(String, Vec<(usize, String)>), regex::Error> {
    let regex = if case_insensitive {
        Regex::new(&format!("(?i){}", target))?
    } else {
        Regex::new(target)?
    };

    let mut log = Vec::new();
    let replaced = regex.replace_all(text, |caps: &regex::Captures| {
        let match_text = caps.get(0).unwrap().as_str().to_string();
        log.push((caps.get(0).unwrap().start(), match_text.clone()));
        replacement.to_string()
    });

    Ok((replaced.to_string(), log))
}

fn main() {
    let args = parse_args();

    let data = match fs::read_to_string(&args.filename) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{} Failed to read from file '{}': {:?}\n",
                "Error:".red().bold(),
                args.filename,
                e
            );
            std::process::exit(1);
        }
    };

    let (replaced_data, log) = match replace(&args.target, &args.replacement, &data, args.case_insensitive) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{} Failed to replace text: {:?}\n",
                "Error:".red().bold(),
                e
            );
            std::process::exit(1);
        }
    };

    // Preview Mode
    if args.preview {
        println!(
            "{}\n{}\n{}",
            "=====================".blue(),
            "Preview of Changes:".yellow().bold(),
            "=====================".blue()
        );
        println!(
            "{}\n\n{} {} changes found.\n",
            replaced_data.lines().take(10).collect::<Vec<_>>().join("\n"),
            "Preview Info:".blue().bold(),
            log.len()
        );
        return;
    }

    // Interactive Mode
    let mut final_data = String::new();
    if args.interactive {
        println!(
            "{}\n{}\n{}",
            "=====================".blue(),
            "Interactive Replacement:".yellow().bold(),
            "=====================".blue()
        );
        for line in replaced_data.lines() {
            println!("{}\nReplace this line? [y/N]:", line.green());
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() == "y" {
                final_data.push_str(line);
            }
            final_data.push('\n');
        }
    } else {
        final_data = replaced_data.clone();
    }

    // Save changes
    match fs::write(&args.output, &final_data) {
        Ok(_) => println!(
            "{} Changes saved to '{}'\n",
            "Success:".green().bold(),
            args.output
        ),
        Err(e) => {
            eprintln!(
                "{} Failed to write to file '{}': {:?}\n",
                "Error:".red().bold(),
                args.output,
                e
            );
            std::process::exit(1);
        }
    }

    // Line and Word Count
    println!(
        "{}\n{}\n{}",
        "=====================".blue(),
        "File Statistics:".yellow().bold(),
        "=====================".blue()
    );
    println!(
        "{}\n  Lines: {}\n  Words: {}\n",
        "Original File:".green(),
        data.lines().count(),
        data.split_whitespace().count()
    );
    println!(
        "{}\n  Lines: {}\n  Words: {}\n",
        "Modified File:".green(),
        final_data.lines().count(),
        final_data.split_whitespace().count()
    );

    // Log File
    if let Some(log_file) = args.log_file {
        let log_data: String = log
            .iter()
            .map(|(pos, matched)| format!("Position: {}, Matched: {}", pos, matched))
            .collect::<Vec<_>>()
            .join("\n");
        match fs::write(&log_file, log_data) {
            Ok(_) => println!(
                "{} Log saved to '{}'\n",
                "Success:".green().bold(),
                log_file
            ),
            Err(e) => eprintln!(
                "{} Failed to write log to '{}': {:?}\n",
                "Error:".red().bold(),
                log_file,
                e
            ),
        }
    }
}
