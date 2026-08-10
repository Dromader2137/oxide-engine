use crate::assets::{AssetDescription, source_file::AssetSourceFile};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub verison: u8,
    pub files: Vec<AssetSourceFile>,
    pub assets: Vec<AssetDescription>
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            verison: 0,
            files: Vec::new(),
            assets: Vec::new()
        }
    }

    pub fn load() -> Manifest {
        let try_file = File::options().read(true).open("./assets.json");
        if matches!(
            try_file.as_ref().map_err(|err| err.kind()),
            Err(std::io::ErrorKind::NotFound)
        ) {
            Manifest::new()
        } else {
            let file = try_file.expect("Failed to open assets.json");
            let reader = BufReader::new(file);
            let manifest = serde_json::from_reader(reader)
                .expect("Failed to parse assets.json");
            manifest
        }
    }
}
