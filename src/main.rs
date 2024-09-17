use std::{fs, path::PathBuf};

use clap::Parser;
use syn::{parse_file, spanned::Spanned, Attribute, Expr, ItemFn, Stmt};
use walkdir::WalkDir;

/// CLI tool to check for missing attributes on rust obecjts.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
	/// The target directory to check
	target_dir: PathBuf,

	/// Check if functions are missing `#[instrument]`
	#[arg(long)]
	instrument: bool,

	/// Check if all `unsafe` blocks have safety comments
	#[arg(long)]
	safety: bool,

	/// Check if all public functions have documentation
	#[arg(long)]
	all: bool,
}

fn has_instrument_attr(attrs: &[Attribute]) -> bool {
	attrs.iter().any(|attr| attr.path().is_ident("instrument"))
}

fn check_functions_in_file(file_content: &str, file_path: &PathBuf) -> Vec<String> {
	let syntax_tree = match parse_file(file_content) {
		Ok(tree) => tree,
		Err(e) => {
			eprintln!("Failed to parse file {:?}: {}", file_path, e);
			return Vec::new();
		}
	};

	let mut missing_instrument = Vec::new();

	for item in syntax_tree.items {
		if let syn::Item::Fn(ItemFn { attrs, sig, .. }) = item {
			if !has_instrument_attr(&attrs) {
				let span_start = sig.ident.span().start();
				missing_instrument.push(format!(
					"No #[instrument] on `{}` in {}:{}:{}",
					sig.ident,
					file_path.display(),
					span_start.line,
					span_start.column
				));
			}
		}
	}

	missing_instrument
}

fn check_safety_comments(file_content: &str, file_path: &PathBuf) -> Vec<String> {
	let syntax_tree = match parse_file(file_content) {
		Ok(tree) => tree,
		Err(e) => {
			eprintln!("Failed to parse file {:?}: {}", file_path, e);
			return Vec::new();
		}
	};

	let mut missing_safety_comments = Vec::new();

	for item in syntax_tree.items {
		if let syn::Item::Fn(ItemFn { block, .. }) = item {
			for stmt in block.stmts.iter() {
				if let Stmt::Expr(Expr::Unsafe(unsafe_block), _) = stmt {
					// Find if there is a comment with `SAFETY`
					let span_start = unsafe_block.unsafe_token.span().start();
					let byte_offset = file_content
                        .lines()
                        .take(span_start.line - 1) // take lines up to the start line
                        .map(|line| line.len() + 1) // +1 for the newline character
                        .sum::<usize>()
						+ span_start.column;
					let preceding_code = &file_content[..byte_offset];
					if !preceding_code.contains("// SAFETY") && !preceding_code.contains("//SAFETY") {
						missing_safety_comments.push(format!(
							"Unsafe block without `// SAFETY` in file {}:{}:{}",
							file_path.display(),
							span_start.line,
							span_start.column
						));
					}
				}
			}
		}
	}

	missing_safety_comments
}

fn main() {
	let mut cli = Cli::parse();
	if cli.all {
		cli.instrument = true;
		cli.safety = true;
	}
	if !cli.target_dir.exists() {
		eprintln!("Target directory does not exist: {:?}", cli.target_dir);
		return;
	}
	if !cli.instrument && !cli.safety {
		eprintln!("Please specify at least one check to conduct. Use --help for more information.");
		return;
	}

	for entry in WalkDir::new(&cli.target_dir).into_iter().filter_map(Result::ok) {
		let path = entry.path();
		if path.extension().map_or(false, |ext| ext == "rs") {
			if let Ok(file_content) = fs::read_to_string(path) {
				if cli.instrument {
					let missing_instrument = check_functions_in_file(&file_content, &path.to_path_buf());
					for message in missing_instrument {
						println!("{}", message);
					}
				}
				if cli.safety {
					let missing_safety_comments = check_safety_comments(&file_content, &path.to_path_buf());
					for message in missing_safety_comments {
						println!("{}", message);
					}
				}
			}
		}
	}
}
