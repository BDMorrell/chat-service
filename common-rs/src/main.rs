use schemars::schema::RootSchema;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Error as IoError, ErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};

/// Where to place the schema files.
///
/// This path is relative to [`MANIFEST_DIR`].
const OUTPUT_DIR: &str = "../common-js/json-schemas";

/// The file where [`OUTPUT_DIR`] is specified.
///
/// This item, and the [`OUTPUT_DIR`] item SHOULD be specified in the same file.
const WHERE_TO_CHANGE_OUTPUT_DIR: &str = file!();

/// Where this script package is specified.
const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// Type that represents all errors.
type GenericError = Box<dyn Error>;

/// Result that uses [`GenericError`].
type GenericResult<T> = Result<T, GenericError>;

fn main() -> GenericResult<()> {
    print_warning_information();

    let output = get_output_directory()?;

    write_schema_dir(&output)?;

    Ok(())
}

/// Get the directory to output to.
fn get_output_directory() -> IoResult<PathBuf> {
    let manifest = PathBuf::from(MANIFEST_DIR);
    let output = manifest.join(OUTPUT_DIR);
    ensure_directory_exists(&output)?;
    output.canonicalize()
}

/// Print nonportability and output information.
fn print_warning_information() {
    println!(
        "Warning: the {} script is not portable! (As in, moving file locations.)
If you want to change the output directory, please see `{}`.",
        env!("CARGO_BIN_NAME"),
        get_this_source_path().display()
    );
}

/// Export the schema informatino to the specified directory.
fn write_schema_dir(directory: &Path) -> GenericResult<()> {
    write_gitignore(directory)?;
    for (name, schema) in common::get_schemas_to_export() {
        write_schema(directory, name, schema)?;
    }
    Ok(())
}

/// Write the `.gitignore` file.
fn write_gitignore(directory: &Path) -> IoResult<()> {
    let mut writer = open_file_for_writing(directory, ".gitignore")?;
    writer.write_fmt(format_args!(
        "# This directory is automatically generated.
# If you want to change the generation of this directory, please see
# {}
*
",
        get_this_source_path().display()
    ))
}

/// Write schema information file for the specified schema.
fn write_schema(directory: &Path, name: &str, schema: RootSchema) -> GenericResult<()> {
    let writer = open_file_for_writing(directory, &format!("{}.json", name))?;
    serde_json::to_writer_pretty(writer, &schema)?;
    Ok(())
}

/// Open the [`directory`/`file_name`] file for writing to.
fn open_file_for_writing(directory: &Path, file_name: &str) -> IoResult<impl Write> {
    let file_path: PathBuf = directory.join(file_name);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)?;
    let writer = BufWriter::new(file);
    Ok(writer)
}

/// Make sure that a directory exists
///
/// Directories are _NOT_ created recursively here to prevent a strange
/// directory path to be created if the repository is moved or the binary is put
/// on another machine.
fn ensure_directory_exists(output: &Path) -> IoResult<()> {
    if !output.exists() {
        if let Err(e) = fs::create_dir(&output) {
            eprintln!("Could not create the directory {}!", output.display());
            return Err(e);
        }
    } else if !output.is_dir() {
        eprintln!("{} is not a directory!", output.display());
        return Err(IoError::new(ErrorKind::Other, "Not a directory"));
    }
    Ok(())
}

/// Get the path to *THIS* source file
fn get_this_source_path() -> PathBuf {
    PathBuf::from(MANIFEST_DIR)
        .parent()
        .unwrap()
        .join(WHERE_TO_CHANGE_OUTPUT_DIR)
}
