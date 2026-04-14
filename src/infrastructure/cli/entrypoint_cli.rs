use continuum::RawMission;
use std::env;
use std::path::PathBuf;

pub struct CliRuntimeRequest {
    pub mission: RawMission,
    pub repository_root: PathBuf,
}

pub enum CliEntrypoint {
    BootstrapTuiShell,
    RuntimeRequest(CliRuntimeRequest),
}

pub fn read_cli_entrypoint() -> Result<CliEntrypoint, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return Ok(CliEntrypoint::BootstrapTuiShell);
    }

    if args.len() != 1 || args[0].trim().is_empty() {
        return Err("expected exactly one non-empty prompt argument".to_string());
    }

    let repository_root = env::current_dir()
        .map_err(|error| format!("failed to resolve current repository root: {error}"))?;

    Ok(CliEntrypoint::RuntimeRequest(CliRuntimeRequest {
        mission: RawMission::new(&args[0]),
        repository_root,
    }))
}
