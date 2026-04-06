use continuum::RawMission;
use std::env;
use std::path::PathBuf;

pub struct CliRuntimeRequest {
    pub mission: RawMission,
    pub repository_root: PathBuf,
}

pub fn read_cli_runtime_request() -> Result<CliRuntimeRequest, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args[0].trim().is_empty() {
        return Err("expected exactly one non-empty prompt argument".to_string());
    }

    let repository_root = env::current_dir()
        .map_err(|error| format!("failed to resolve current repository root: {error}"))?;

    Ok(CliRuntimeRequest {
        mission: RawMission::new(&args[0]),
        repository_root,
    })
}
