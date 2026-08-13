use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

use crate::{
    assets::{
        Asset, AssetDescription, InitialAssetDescription, core::{mesh::Mesh, shader::Shader}, manifest::Manifest, source_file::{AssetSourceFile, AssetSourceFileHandler}
    },
    error::{OptionNoneErr, ParsecError},
};

/// parsec-engine-cli add <path> // adds an asset
/// parsec-engine-cli rescan [<path>] // adds an asset
/// parsec-engine-cli remove <path> // removes an asset
/// parsec-engine-cli cook [<path>] // cooks all assets
///
/// Example project structure:
/// src/
/// Cargo.toml
/// assets/
///   asset1.asset
///   asset2.asset
/// assets.json
///
/// assets.json
///

#[derive(Debug, clap::Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Add { path: PathBuf },
    Remove { path: PathBuf },
    Rescan { path: Option<PathBuf> },
    Cook { path: Option<PathBuf> },
}

#[derive(Debug)]
pub enum ManifestWriteError {
    FailedToCreateFile(std::io::Error),
    FailedToWriteTemp(serde_json::Error),
    FailedToSwapFiles(std::io::Error),
}

fn write_manifest(manifest: &Manifest) -> Result<(), ManifestWriteError> {
    let write_file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./assets.json.tmp")
        .map_err(|err| ManifestWriteError::FailedToCreateFile(err))?;
    let writer = BufWriter::new(write_file);
    serde_json::to_writer_pretty(writer, manifest)
        .map_err(|err| ManifestWriteError::FailedToWriteTemp(err))?;
    let rename_op = std::fs::rename("./assets.json.tmp", "./assets.json")
        .map_err(|err| ManifestWriteError::FailedToSwapFiles(err));
    if rename_op.is_err() {
        std::fs::remove_file("./assets.json.tmp")
            .expect("Failed to delete assets.json.tmp");
    }
    Ok(())
}

fn get_cook_dir() -> PathBuf {
    fs::create_dir_all("./assets/").unwrap();
    PathBuf::from_str("./assets/").unwrap()
}

fn cook(
    name: &str,
    manifest: &Manifest,
    cooker: &Cooker,
) -> Result<(), ParsecError> {
    let in_path = manifest.assets.iter().find(|a| a.0 == name).none_err()?;
    let out_path = get_cook_dir().join(name).with_extension("asset");
    cooker.cook(&in_path.1, &out_path);
    Ok(())
}

fn cook_type_erased<T: Asset>(
    description: &AssetDescription,
    source_file: &AssetSourceFile,
) -> Vec<u8> {
    let out = T::cook(description, source_file);
    let out_bytes = postcard::to_stdvec(&out).unwrap();
    out_bytes
}

pub struct CookerAssetRegistation {
    cook_fn: Box<fn(&AssetDescription, &AssetSourceFile) -> Vec<u8>>,
}

pub struct CookerSourceFileRegistration {
    extract_fn: Box<fn(&Path) -> Vec<InitialAssetDescription>>,
}

#[derive(Debug, Default)]
pub struct Cooker {
    asset_handlers: HashMap<&'static str, CookerAssetRegistation>,
    source_file_handler: HashMap<&'static str, CookerSourceFileRegistration>,
}

impl Cooker {
    pub fn register_asset_type<T: Asset>(&mut self) {
        let registation = CookerAssetRegistation {
            cook_fn: Box::new(cook_type_erased::<T>),
        };
        self.asset_handlers.insert(T::ASSET_TYPE, registation);
    }

    pub fn register_source_file_type<T: AssetSourceFileHandler>(&mut self) {
        let registration = CookerSourceFileRegistration {
            extract_fn: Box::new(T::extract_assets),
        };
        self.source_file_handler.insert(T::EXTENSION, registration);
    }

    pub fn cook(&self, input: &AssetDescription, output: &Path) {}
}

fn rescan(
    manifest: &mut Manifest,
    cooker: &Cooker,
    path: &Path,
) -> Result<bool, CliError> {
    let file = manifest
        .tracked_files
        .iter()
        .find(|x| x.filepath == path)
        .ok_or(CliError::AssetSourceFileNotFound(path.to_path_buf()))?;
    let extension = file.filepath.extension().and_then(|x| x.to_str()).ok_or(
        CliError::AssetSourceFileExtensionInvalid(path.to_path_buf()),
    )?;
    let handler = cooker
        .source_file_handler
        .get(extension)
        .ok_or(CliError::HandlerNotFound(extension.to_string()))?;
    let asset_descriptions = (handler.extract_fn)(&file.filepath);
    for asset_description in asset_descriptions {
        asset_description.
    }
}

pub fn run_cli(mut cooker: Cooker) {
    cooker.register_asset_type::<Mesh>();
    cooker.register_asset_type::<Shader>();

    let args = Args::parse();
    let mut manifest = Manifest::load();

    match args.command {
        Commands::Add { path } => {
            let extension = path
                .extension()
                .expect("Asset source file should have an extension")
                .to_str()
                .expect("Asset source file extension should be valid UTF-8");
            let handler = cooker
                .source_file_handler
                .get(extension)
                .expect("Asset source file handler not provided");
            let asset_descriptions = (handler.extract_fn)(&path);
            let asset_ids = BTreeSet::from_iter(
                manifest.assets.len()
                    ..(manifest.assets.len() + asset_descriptions.len()),
            );
            manifest.tracked_files.push(AssetSourceFile {
                filepath: path,
                asset_ids,
            });
            manifest.assets.extend_from_slice(&asset_descriptions);
        },
        Commands::Remove { path } => {
            let (idx, file) = manifest
                .tracked_files
                .iter()
                .enumerate()
                .find(|(_, x)| x.filepath == path)
                .expect("Asset source file not found");
            for &asset in file.asset_ids.iter().rev() {
                manifest.assets.remove(asset);
            }
            manifest.tracked_files.swap_remove(idx);
        },
        Commands::Rescan { path } => {},
        Commands::Cook { path } => todo!(),
    }
}

pub enum CliError {
    AssetSourceFileNotFound(PathBuf),
    AssetSourceFileExtensionInvalid(PathBuf),
    HandlerNotFound(String),
}
