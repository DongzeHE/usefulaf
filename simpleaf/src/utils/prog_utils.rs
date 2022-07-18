use anyhow::{anyhow, Context, Result};
use cmd_lib::run_fun;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use which::which;

#[derive(Serialize, Deserialize, Clone)]
pub struct ProgInfo {
    pub exe_path: PathBuf,
    pub version: String,
}

impl Default for ProgInfo {
    fn default() -> Self {
        Self {
            exe_path: PathBuf::from(""),
            version: String::from("0.0.0"),
        }
    }
}

// Holds the paths to the
// programs we'll need to run
// the tool.
#[derive(Serialize, Deserialize)]
pub struct ReqProgs {
    pub salmon: Option<ProgInfo>,
    pub alevin_fry: Option<ProgInfo>,
    pub pyroe: Option<ProgInfo>,
}

pub fn check_version_constraints<S1: AsRef<str>>(
    req_string: S1,
    prog_output: std::result::Result<String, std::io::Error>,
) -> Result<Version> {
    match prog_output {
        Ok(vs) => {
            let x = vs.split_whitespace();
            if let Some(version) = x.last() {
                let parsed_version = Version::parse(version).unwrap();
                let req = VersionReq::parse(req_string.as_ref()).unwrap();
                if req.matches(&parsed_version) {
                    return Ok(parsed_version);
                } else {
                    return Err(anyhow!(
                        "parsed version {:?} does not satisfy constraints {:?}",
                        version,
                        req
                    ));
                }
            }
        }
        Err(e) => {
            eprintln!("Error running salmon {}", e);
            return Err(anyhow!("could not parse program output"));
        }
    }
    Err(anyhow!("invalid version string"))
}

pub fn search_for_executable(env_key: &str, prog_name: &str) -> Result<PathBuf> {
    match env::var(env_key) {
        Ok(p) => {
            return Ok(PathBuf::from(p));
        }
        Err(e) => {
            eprintln!("${} is unset {}, trying default path.", env_key, e);
            eprintln!(
                "If a satisfactory version is not found, consider setting the ${} variable.",
                env_key
            );
            match which(prog_name) {
                Ok(p) => {
                    println!("found `{}` in the PATH at {}", prog_name, p.display());
                    return Ok(p);
                }
                Err(e) => {
                    return Err(anyhow!(
                        "could not find `{}` in your path: {}",
                        prog_name,
                        e
                    ));
                }
            }
        }
    }
}
pub fn get_required_progs() -> Result<ReqProgs> {
    let mut rp = ReqProgs {
        salmon: None,
        alevin_fry: None,
        pyroe: None,
    };

    // First look for any environment variables
    // then check the path.
    let salmon_exe = Some(search_for_executable("SALMON", "salmon")?);
    let alevin_fry_exe = Some(search_for_executable("ALEVIN_FRY", "alevin-fry")?);
    let pyroe_exe = Some(search_for_executable("PYROE", "pyroe")?);

    if let Some(salmon) = salmon_exe.clone() {
        let st = salmon.display().to_string();
        let sr = run_fun!($st --version);
        let v = check_version_constraints(">=1.5.1, <2.0.0", sr)?;
        rp.salmon = Some(ProgInfo{ exe_path: salmon, version: format!("{}",v)});
    }

    if let Some(af) = alevin_fry_exe.clone() {
        let st = af.display().to_string();
        let sr = run_fun!($st --version);
        let v = check_version_constraints(">=0.4.1, <1.0.0", sr)?;
        rp.alevin_fry = Some(ProgInfo{ exe_path: af, version: format!("{}",v)});
    }

    if let Some(pr) = pyroe_exe.clone() {
        let st = pr.display().to_string();
        let sr = run_fun!($st --version);
        let v = check_version_constraints(">=0.6.2, <1.0.0", sr)?;
        rp.pyroe = Some(ProgInfo{ exe_path: pr, version: format!("{}",v)});
    }
    Ok(rp)
}