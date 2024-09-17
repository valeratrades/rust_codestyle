use std::{
	fs,
	path::{Path, PathBuf},
};

use clap::Parser;
use syn::{parse_file, spanned::Spanned, Expr, ItemFn, Stmt};
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

#[derive(Clone, Debug, Default, derive_new::new)]
struct Issues {
	instrument: Vec<String>,
	safety: Vec<String>,
}

fn check_instrument(fn_items: &[ItemFn], file_path: &Path) -> Vec<String> {
	let mut missing_instrument = Vec::new();
	for func in fn_items {
		if !func.attrs.iter().any(|attr| attr.path().is_ident("instrument")) {
			let span_start = func.sig.ident.span().start();
			missing_instrument.push(format!(
				"No #[instrument] on `{}` in {}:{}:{}",
				func.sig.ident,
				file_path.display(),
				span_start.line,
				span_start.column
			));
		}
	}
	missing_instrument
}

fn check_safety(fn_items: &[ItemFn], file_content: &str, file_path: &Path) -> Vec<String> {
	let mut missing_safety_comments = Vec::new();
	for func in fn_items {
		for stmt in func.block.stmts.iter() {
			if let Stmt::Expr(Expr::Unsafe(unsafe_block), _) = stmt {
				let span_start = unsafe_block.unsafe_token.span().start();
				let byte_offset = file_content
                    .lines()
                    .take(span_start.line - 1)
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
	missing_safety_comments
}

#[derive(Clone, Default, derive_new::new)]
struct FileInfo {
	contents: String,
	fn_items: Vec<ItemFn>,
	path: PathBuf,
}
fn fn_items_in_file(path: PathBuf) -> FileInfo {
	let file_contents = fs::read_to_string(&path).unwrap();
	let syntax_tree = match parse_file(&file_contents) {
		Ok(tree) => tree,
		Err(e) => {
			eprintln!("Failed to parse file {:?}: {}", path, e);
			return FileInfo::default(); //? Should I just panic, because we shouldn't work with files that are not compiling anyways?
		}
	};

	let fn_items = syntax_tree
		.items
		.iter()
		.filter_map(|item| if let syn::Item::Fn(func) = item { Some(func.clone()) } else { None })
		.collect();
	FileInfo {
		contents: file_contents,
		fn_items,
		path,
	}
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

	let mut relevant_file_infos = Vec::new();
	for entry in WalkDir::new(&cli.target_dir).into_iter().filter_map(Result::ok) {
		let path = entry.path().to_path_buf();
		if path.extension().map_or(false, |ext| ext == "rs") {
			relevant_file_infos.push(fn_items_in_file(path));
		}
	}

	if cli.instrument {
		for info in relevant_file_infos.iter() {
			let issues = check_instrument(&info.fn_items, &info.path);
			for message in issues {
				println!("{}", message);
			}
		}
	}
	if cli.safety {
		for info in relevant_file_infos.iter() {
			let issues = check_safety(&info.fn_items, &info.contents, &info.path);
			for message in issues {
				println!("{}", message);
			}
		}
	}
}
