use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

type GenericResult<T> = Result<T, failure::Error>;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct MappedFolderItemIn {
    HostFolder: String,
    ReadOnly: Option<bool>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct ConfigurationIn {
    VGpu: Option<bool>,
    Networking: Option<bool>,
    LogonCommand: Option<String>,
    MappedFolders: Vec<MappedFolderItemIn>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct MappedFolder {
    HostFolder: String,
    ReadOnly: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct Configuration {
    VGpu: String,
    Networking: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    MappedFolders: Option<Vec<MappedFolder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    LogonCommand: Option<HashMap<String, String>>,
}

impl From<MappedFolderItemIn> for MappedFolder {
    fn from(m: MappedFolderItemIn) -> Self {
        MappedFolder {
            HostFolder: m.HostFolder,
            ReadOnly: match m.ReadOnly {
                Some(x) if !x => "true".to_owned(),
                _ => "false".to_owned(),
            },
        }
    }
}

fn trans_mapped_folders(x: Vec<MappedFolderItemIn>) -> Vec<MappedFolder> {
    x.into_iter().map(MappedFolder::from).collect()
}

impl From<ConfigurationIn> for Configuration {
    fn from(c: ConfigurationIn) -> Self {
        Configuration {
            VGpu: match c.VGpu {
                Some(x) if x => "Enable".to_owned(),
                Some(_) => "Disable".to_owned(),
                _ => "Default".to_owned(),
            },
            Networking: match c.Networking {
                Some(x) if x => "Enable".to_owned(),
                Some(_) => "Disable".to_owned(),
                _ => "Default".to_owned(),
            },
            MappedFolders: Some(trans_mapped_folders(c.MappedFolders)),
            LogonCommand: match c.LogonCommand {
                Some(x) => {
                    let mut hm = HashMap::new();
                    hm.insert("Command".to_owned(), x);
                    Some(hm)
                }
                _ => None,
            },
        }
    }
}

#[derive(StructOpt, Debug)]
enum Commands {
    /// generate TOML template
    Generate {
        /// disable virtual GPU in Sandbox
        #[structopt(long = "disable-vgpu")]
        disable_vgpu: bool,

        /// disable Network in Sandbox
        #[structopt(long = "disable-network")]
        disable_network: bool,

        /// generate wsb file directly
        #[structopt(short = "d", long = "direct")]
        generate_wsb: bool,

        /// add readonly shared directory.  
        /// these must be present and allowed relative path(will be canonicalized).
        #[structopt(long = "shared-dir")]
        readonly_dirs: Option<Vec<String>>,

        /// add shared directory. please note that these will be writable from sandbox.
        ///
        ///these must be present. allowed relative path(will be canonicalized).
        #[structopt(long = "shared-writable-dir")]
        readwrite_dirs: Option<Vec<String>>,

        #[structopt(
            long = "logon-command",
            default_value = "cmd /c C:\\Users\\WDAGUtilityAccount\\Desktop\\tools\\init.bat"
        )]
        logon_command: String,

        /// output TOML path
        outpath: PathBuf,
    },
    /// build .wsb from
    Build {
        #[structopt(short = "o", long = "output-path", default_value = "sandbox.wsb")]
        outpath: PathBuf,

        target_config: PathBuf,
    },
}

impl Commands {
    fn run(&self) -> GenericResult<()> {
        match self {
            Commands::Generate {
                disable_network,
                disable_vgpu,
                readonly_dirs,
                readwrite_dirs,
                logon_command,
                generate_wsb,
                outpath,
            } => {
                let rodirs = readonly_dirs.clone().map(|x| {
                    x.into_iter()
                        .map(|x| MappedFolderItemIn {
                            HostFolder: std::fs::canonicalize(&x)
                                .expect("failed to canonicalize path")
                                .to_str()
                                .expect("failed to convert path")
                                .to_owned(),
                            ReadOnly: Some(true),
                        })
                        .collect::<Vec<_>>()
                });
                let rwdirs = readwrite_dirs.clone().map(|x| {
                    x.into_iter()
                        .map(|x| MappedFolderItemIn {
                            HostFolder: std::fs::canonicalize(&x)
                                .expect("failed to canonicalize path")
                                .to_str()
                                .expect("failed to convert path")
                                .to_owned(),
                            ReadOnly: Some(false),
                        })
                        .collect::<Vec<_>>()
                });
                let targets: Vec<MappedFolderItemIn> = rodirs
                    .into_iter()
                    .chain(rwdirs.into_iter())
                    .flatten()
                    .collect();

                let c = ConfigurationIn {
                    VGpu: Some(!*disable_vgpu),
                    Networking: Some(!*disable_network),
                    MappedFolders: targets,
                    LogonCommand: Some(logon_command.clone()),
                };

                if *generate_wsb {
                    self.generate_direct(outpath, c)
                } else {
                    self.generate(outpath, c)
                }
            }
            Commands::Build {
                outpath,
                target_config,
            } => self.build(outpath, target_config),
        }
    }

    fn generate(&self, outpath: &PathBuf, config: ConfigurationIn) -> GenericResult<()> {
        let mut f = std::fs::File::create(outpath)?;
        let s = toml::to_string(&config)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }

    fn generate_direct(&self, outpath: &PathBuf, c: ConfigurationIn) -> GenericResult<()> {
        store_config(Configuration::from(c), outpath)
    }

    fn build(&self, outpath: &PathBuf, config_path: &PathBuf) -> GenericResult<()> {
        let c = load_config(config_path)?;
        store_config(Configuration::from(c), outpath)
    }
}

fn load_config(inputpath: &PathBuf) -> GenericResult<ConfigurationIn> {
    let mut f = std::fs::File::open(inputpath)?;
    let mut input = String::new();
    f.read_to_string(&mut input)?;
    let c: ConfigurationIn = toml::from_str(&input)?;
    Ok(c)
}

fn store_config(c: Configuration, outpath: &PathBuf) -> GenericResult<()> {
    let mut f = std::fs::File::create(outpath)?;
    let output = quick_xml::se::to_string(&c)?;
    f.write_all(output.as_bytes())?;
    Ok(())
}

fn main() -> GenericResult<()> {
    let command = Commands::from_args();
    command.run()
}
