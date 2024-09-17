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

	///UNIMPLEMENTED
	/// Check if all `unsafe` blocks have safety comments
	#[arg(long)]
	safety: bool,

	///UNIMPLEMENTED
	/// Check if all endless loops have a justification comment
	#[arg(long)]
	loops: bool,

	/// Run for all properties at once
	#[arg(long)]
	all: bool,
}

fn check_instrument(fn_items: &[ItemFn], file_path: &Path) -> Vec<String> {
	let mut missing_instrument = Vec::new();
	let filename = file_path.file_name().unwrap().to_str().unwrap();
	for func in fn_items {
		if !func.attrs.iter().any(|attr| attr.path().is_ident("instrument")) && filename != "utils.rs" && func.sig.ident != "main" {
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
				dbg!(&preceding_code);
				if !preceding_code.contains("// SAFETY") && !preceding_code.contains("//SAFETY") {
					missing_safety_comments.push(format!("Unsafe block without `// SAFETY` in {}:{}:{}", file_path.display(), span_start.line, span_start.column));
				}
			}
		}
	}
	missing_safety_comments
}

fn check_loops(fn_items: &[ItemFn], file_content: &str, file_path: &Path) -> Vec<String> {
	let mut missing_loop_comments = Vec::new();

	for func in fn_items {
		for stmt in func.block.stmts.iter() {
			if let Stmt::Expr(Expr::Loop(loop_expr), _) = stmt {
				let span_start = loop_expr.loop_token.span().start();
				let byte_offset = file_content
                    .lines()
                    .take(span_start.line - 1)
                    .map(|line| line.len() + 1) // +1 for the newline character
                    .sum::<usize>()
					+ span_start.column;
				let preceding_code = &file_content[..byte_offset];
				dbg!(&preceding_code);
				if !preceding_code.contains("// LOOP") && !preceding_code.contains("//LOOP") {
					missing_loop_comments.push(format!(
						"Endless loop without `// LOOP` in file {}:{}:{}",
						file_path.display(),
						span_start.line,
						span_start.column
					));
				}
			}
		}
	}

	missing_loop_comments
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
		cli.loops = true;
	}
	if !cli.target_dir.exists() {
		eprintln!("Target directory does not exist: {:?}", cli.target_dir);
		return;
	}
	if !cli.instrument && !cli.safety && !cli.loops {
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

	let mut instrument_issues = Vec::new();
	if cli.instrument {
		for info in relevant_file_infos.iter() {
			let issues = check_instrument(&info.fn_items, &info.path);
			instrument_issues.extend(issues);
		}
	}
	let instrument_issues_string = instrument_issues.join("\n");

	//let mut safety_issues = Vec::new();
	//if cli.safety {
	//	for info in relevant_file_infos.iter() {
	//		let issues = check_safety(&info.fn_items, &info.contents, &info.path);
	//		safety_issues.extend(issues);
	//	}
	//}
	//let safety_issues_string = safety_issues.join("\n");
	//
	//let mut loop_issues = Vec::new();
	//if cli.loops {
	//	for info in relevant_file_infos.iter() {
	//		let issues = check_loops(&info.fn_items, &info.contents, &info.path);
	//		loop_issues.extend(issues);
	//	}
	//}
	//let loop_issues_string = loop_issues.join("\n");

	let mut s = instrument_issues_string;
	//if !s.is_empty() && !safety_issues_string.is_empty() {
	//	s.push_str("\n\n");
	//}
	//s.push_str(&safety_issues_string);
	//
	//if !safety_issues_string.is_empty() {
	//	s.push_str("\n\n");
	//}
	//s.push_str(&loop_issues_string);

	match s.is_empty() {
		true => std::process::exit(0),
		false => {
			eprintln!("{s}");
			std::process::exit(1);
		}
	}
}
