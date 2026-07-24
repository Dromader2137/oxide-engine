use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

use crate::{
    assets::{
        Asset, AssetDescription, Manifest,
        core::{mesh::Mesh, shader::Shader}, source_file::{AssetSourceFile, AssetSourceFileHandler},
    },
    error::{OptionNoneErr, ParsecError},
};

/// parsec-engine-cli add <name> <path> // adds an asset
/// parsec-engine-cli remove <name> // removes an asset
/// parsec-engine-cli cook // cooks all assets
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
    Add { name: String, path: PathBuf },
    Remove { name: String },
    Cook,
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
    extract_fn: Box<fn(&Path) -> Vec<AssetDescription>>
}

#[derive(Debug, Default)]
pub struct Cooker {
    handlers: HashMap<&'static str, CookerAssetRegistation>,
    source_file_handler: HashMap<&'static str, CookerSourceFileRegistration>
}


impl Cooker {
    pub fn register<T: Asset>(&mut self) {
        let registation = CookerAssetRegistation {
            cook_fn: Box::new(cook_type_erased::<T>),
        };
        self.handlers.insert(T::ASSET_TYPE, registation);
    }

    pub fn cook(&self, input: &AssetDescription, output: &Path) {
        let ext = input
            .path
            .extension()
            .unwrap_or(&OsStr::new(""))
            .to_str()
            .unwrap();
        let (_, handler) = self
            .handlers
            .iter()
            .find(|(_, v)| v.extensions.contains(&ext))
            .unwrap();
        let bytes = std::fs::read(&input.path).unwrap();
        let out_bytes = (handler.cook_fn)(
            &bytes,
            input   
        );
        let mut out_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output)
            .unwrap();
        out_file.write_all(&out_bytes).unwrap();
    }
}

pub fn run_cli(mut cooker: Cooker) {
    let args = Args::parse();

    let mut manifest = Manifest::load();
    cooker.register::<Mesh>();
    cooker.register::<Shader>();

    match args.command {
        Commands::Add { name, path } => {
            if manifest.assets.contains_key(&name) {
                println!("Asset with this name already existst");
                return;
            }
            manifest.assets.insert(name.clone(), AssetDescription {
                name: name.clone(),
                path,
                last_cooked: None,
            });
            write_manifest(&manifest).unwrap();
            println!("Added asset {}", name);
        },
        Commands::Remove { name } => {
            if !manifest.assets.contains_key(&name) {
                println!("Asset not found");
                return;
            }
            manifest.assets.remove(&name);
            println!("Removed asset {}", name);
        },
        Commands::Cook => {
            println!("Cooking...");
            for name in manifest.assets.keys() {
                cook(name, &manifest, &cooker).unwrap();
            }
        },
    }
}
