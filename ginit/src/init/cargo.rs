use crate::{android, config::Config, ios, target::TargetTrait};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CargoTarget {
    pub ar: Option<String>,
    pub linker: Option<String>,
    pub rustflags: Vec<String>,
}

impl CargoTarget {
    fn is_empty(&self) -> bool {
        self.ar.is_none() && self.linker.is_none() && self.rustflags.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfig {
    target: BTreeMap<String, CargoTarget>,
}

impl CargoConfig {
    pub fn generate() -> Self {
        let mut target = BTreeMap::new();
        for android_target in android::target::Target::all().values() {
            target.insert(
                android_target.triple.to_owned(),
                android_target.generate_cargo_config(
                    &android::env::Env::new().expect("failed to init android env"),
                ),
            );
        }
        for ios_target in ios::target::Target::all().values() {
            target.insert(
                ios_target.triple.to_owned(),
                ios_target.generate_cargo_config(),
            );
        }
        target.insert(
            "x86_64-apple-darwin".to_owned(),
            CargoTarget {
                ar: None,
                linker: None,
                rustflags: vec![
                    "-C".to_owned(),
                    "target-cpu=native".to_owned(),
                    // this makes sure we'll be able to change dylib IDs
                    // (needed for dylib hot reloading)
                    "-C".to_owned(),
                    "link-arg=-headerpad_max_install_names".to_owned(),
                ],
            },
        );
        CargoConfig {
            target: target
                .into_iter()
                .filter(|(_, target)| !target.is_empty())
                .collect(),
        }
    }

    pub fn write(&self, config: &Config) {
        let serialized = toml::to_string_pretty(self).expect("Failed to serialize cargo config");
        let dir = config.prefix_path(".cargo");
        fs::create_dir_all(&dir).expect("Failed to create `.cargo` directory");
        let path = dir.join("config");
        let mut file = File::create(path).expect("Failed to create cargo config file");
        file.write_all(serialized.as_bytes())
            .expect("Failed to write to cargo config file");
    }
}