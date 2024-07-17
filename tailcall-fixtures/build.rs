mod error;

use std::fmt::Write;
use std::path::Path;
use std::{env, fs};

use convert_case::{Case, Casing};
use error::{Error, Result};
use indenter::CodeFormatter;

fn write_mod(path: &Path, f: &mut CodeFormatter<String>, dir_name: Option<&str>) -> Result<()> {
    let files = fs::read_dir(path)?;

    if let Some(dir_name) = dir_name {
        writeln!(
            f,
            r#"
			pub mod {dir_name} {{
				pub const SELF: &str = r"{}";
				"#,
            path.display()
        )?;
        f.indent(1);
    }

    for file in files {
        let file = file?;

        if file.metadata()?.is_dir() {
            write_mod(&file.path(), f, Some(&file.file_name().to_string_lossy()))?;
            writeln!(f)?;
        } else {
            let name = file.file_name();
            let name = Path::new(&name)
                .file_stem()
                .ok_or_else(|| Error::FilenameNotResolved(file.file_name().into_string().unwrap()))?
                .to_string_lossy();
            let name = name.as_ref().to_case(Case::UpperSnake);
            let path = file.path();
            let path = path.display();

            writeln!(
                f,
                r#"
				pub const {name}: &str = r"{path}";
				"#
            )?;
        }
    }

    if dir_name.is_some() {
        f.dedent(1);
        writeln!(f, "\n}}")?;
    }

    Ok(())
}

// walk over `fixtures` directory and collect all files as const &str inside
// modules according to nested directories
fn main() -> Result<()> {
    let fixtures_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let dest_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("fixtures.rs");

    let mut buffer = String::new();
    let formatter = &mut CodeFormatter::new(&mut buffer, "  ");

    write_mod(&fixtures_path, formatter, None)?;

    fs::write(dest_path, buffer).unwrap();
    println!("cargo:rerun-if-changed=fixtures");

    Ok(())
}
