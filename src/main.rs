use continuum::{
    Builder,
    BuilderIssue,
    BuilderRunReport,
    BuilderScopeStatus,
    CriticSignal,
    Critic,
    MissionScholar,
    Planner,
    PostCriticPlanner,
    PostCriticSignal,
    RawMission,
    Scholar,
    ScholarOutput,
    SessionFlowDecision,
    SessionRunner,
    SessionStatus,
};
use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

struct ShellScholar {
    mission: RawMission,
    mission_scholar: MissionScholar,
}

impl ShellScholar {
    fn new(mission: RawMission) -> Self {
        Self {
            mission,
            mission_scholar: MissionScholar::new(),
        }
    }
}

impl Scholar for ShellScholar {
    fn run(&mut self) -> ScholarOutput {
        self.mission_scholar.transform(&self.mission)
    }
}

struct ShellPlanner {
    post_critic_planner: PostCriticPlanner,
}

impl ShellPlanner {
    fn new() -> Self {
        Self {
            post_critic_planner: PostCriticPlanner,
        }
    }
}

impl Planner for ShellPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        SessionFlowDecision::Build
    }

    fn decide_with_critic_signal(
        &mut self,
        scholar_output: &ScholarOutput,
        critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        self.post_critic_planner
            .decide_with_critic_signal(scholar_output, critic_signal)
    }
}

struct CodexLocalBuilderAdapter {
    repository_root: PathBuf,
}

impl CodexLocalBuilderAdapter {
    fn new(repository_root: PathBuf) -> Self {
        Self { repository_root }
    }

    fn allowed_file_scope(&self, scholar_output: &ScholarOutput) -> Option<Vec<String>> {
        if scholar_output.selected_task_scope.contains("README.md") {
            Some(vec!["README.md".to_string()])
        } else {
            None
        }
    }

    fn build_bounded_prompt(
        &self,
        scholar_output: &ScholarOutput,
        allowed_file_scope: &[String],
    ) -> String {
        format!(
            "Role: Builder\nMission: {}\nAllowed file scope: {}\nDo not modify any other files.\nDo not choose a new task, budget, or runtime transition.",
            scholar_output.mission_summary,
            allowed_file_scope.join(", ")
        )
    }

    fn capture_git_status(&self) -> Result<BTreeSet<String>, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repository_root)
            .args(["status", "--porcelain", "--untracked-files=all"])
            .output()
            .map_err(|error| format!("failed to capture git status: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "git status failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        Ok(parse_git_status_paths(&String::from_utf8_lossy(&output.stdout)))
    }

    fn scope_status(
        &self,
        before: &BTreeSet<String>,
        after: &BTreeSet<String>,
        allowed_file_scope: &[String],
    ) -> (BuilderScopeStatus, Vec<String>) {
        let changed_files: Vec<String> = after.difference(before).cloned().collect();
        let within_scope = changed_files.iter().all(|file| allowed_file_scope.contains(file));

        if within_scope {
            (BuilderScopeStatus::WithinScope, changed_files)
        } else {
            (BuilderScopeStatus::Violated, changed_files)
        }
    }
}

impl Builder for CodexLocalBuilderAdapter {
    fn run(&mut self, scholar_output: &ScholarOutput) -> BuilderRunReport {
        let Some(allowed_file_scope) = self.allowed_file_scope(scholar_output) else {
            return BuilderRunReport {
                issue: BuilderIssue::PreconditionFailed,
                scope_status: BuilderScopeStatus::NotChecked,
                allowed_file_scope: Vec::new(),
                changed_files: Vec::new(),
                stdout: String::new(),
                stderr: "builder requires an explicit allowed file scope; only README.md is admitted in this minimal adapter".to_string(),
            };
        };

        let before_status = match self.capture_git_status() {
            Ok(status) => status,
            Err(error) => {
                return BuilderRunReport {
                    issue: BuilderIssue::PreconditionFailed,
                    scope_status: BuilderScopeStatus::NotChecked,
                    allowed_file_scope,
                    changed_files: Vec::new(),
                    stdout: String::new(),
                    stderr: error,
                };
            }
        };

        let bounded_prompt = self.build_bounded_prompt(scholar_output, &allowed_file_scope);
        let output = match Command::new("codex")
            .current_dir(&self.repository_root)
            .arg("exec")
            .arg("-C")
            .arg(&self.repository_root)
            .arg(&bounded_prompt)
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                return BuilderRunReport {
                    issue: BuilderIssue::LaunchFailed,
                    scope_status: BuilderScopeStatus::NotChecked,
                    allowed_file_scope,
                    changed_files: Vec::new(),
                    stdout: String::new(),
                    stderr: error.to_string(),
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let after_status = match self.capture_git_status() {
            Ok(status) => status,
            Err(error) => {
                return BuilderRunReport {
                    issue: BuilderIssue::ProcessFailed,
                    scope_status: BuilderScopeStatus::NotChecked,
                    allowed_file_scope,
                    changed_files: Vec::new(),
                    stdout,
                    stderr: format!("{stderr}\nscope_check_error={error}"),
                };
            }
        };
        let (scope_status, changed_files) =
            self.scope_status(&before_status, &after_status, &allowed_file_scope);

        let issue = if scope_status == BuilderScopeStatus::Violated {
            BuilderIssue::ScopeViolated
        } else if output.status.success() {
            BuilderIssue::Completed
        } else {
            BuilderIssue::ProcessFailed
        };

        BuilderRunReport {
            issue,
            scope_status,
            allowed_file_scope,
            changed_files,
            stdout,
            stderr,
        }
    }
}

struct ShellCritic;

impl ShellCritic {
    fn new() -> Self {
        Self
    }
}

impl Critic for ShellCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        CriticSignal::Accepted
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args[0].trim().is_empty() {
        eprintln!("terminal_outcome=failure");
        eprintln!("error=expected exactly one non-empty prompt argument");
        return ExitCode::from(1);
    }

    let repository_root = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("terminal_outcome=failure");
            eprintln!("error=failed to resolve current repository root: {error}");
            return ExitCode::from(1);
        }
    };

    let mission = RawMission::new(&args[0]);
    let mut session_runner = SessionRunner::new(
        Box::new(ShellScholar::new(mission)),
        Box::new(ShellPlanner::new()),
        Box::new(CodexLocalBuilderAdapter::new(repository_root.clone())),
        Box::new(ShellCritic::new()),
    );

    let result = session_runner.run();
    let builder_report = session_runner.last_builder_report().cloned();

    match result {
        Ok(summary) => {
            println!("terminal_outcome=success");
            println!("repository_root={}", repository_root.display());
            if let Some(report) = builder_report.as_ref() {
                render_builder_report_stdout(report);
            }
            println!(
                "session_status={}",
                render_session_status(summary.final_session_status)
            );
            ExitCode::SUCCESS
        }
        Err(report) => {
            eprintln!("terminal_outcome=failure");
            eprintln!("repository_root={}", repository_root.display());
            if let Some(builder_report) = builder_report.as_ref() {
                render_builder_report_stderr(builder_report);
            }
            eprintln!(
                "session_status={}",
                render_session_status(report.final_session_status)
            );
            ExitCode::from(1)
        }
    }
}

fn parse_git_status_paths(status_output: &str) -> BTreeSet<String> {
    status_output
        .lines()
        .filter_map(|line| line.get(3..))
        .map(|path| {
            path.split_once(" -> ")
                .map(|(_, new_path)| new_path)
                .unwrap_or(path)
                .to_string()
        })
        .collect()
}

fn render_builder_report_stdout(report: &BuilderRunReport) {
    println!("builder_issue={}", render_builder_issue(&report.issue));
    println!(
        "builder_scope_status={}",
        render_builder_scope_status(&report.scope_status)
    );
    println!("builder_allowed_file_scope={}", report.allowed_file_scope.join(","));
    println!("builder_changed_files={}", report.changed_files.join(","));
    println!("builder_stdout={}", render_terminal_field(&report.stdout));
    println!("builder_stderr={}", render_terminal_field(&report.stderr));
}

fn render_builder_report_stderr(report: &BuilderRunReport) {
    eprintln!("builder_issue={}", render_builder_issue(&report.issue));
    eprintln!(
        "builder_scope_status={}",
        render_builder_scope_status(&report.scope_status)
    );
    eprintln!("builder_allowed_file_scope={}", report.allowed_file_scope.join(","));
    eprintln!("builder_changed_files={}", report.changed_files.join(","));
    eprintln!("builder_stdout={}", render_terminal_field(&report.stdout));
    eprintln!("builder_stderr={}", render_terminal_field(&report.stderr));
}

fn render_builder_issue(issue: &BuilderIssue) -> &'static str {
    match issue {
        BuilderIssue::Completed => "completed",
        BuilderIssue::PreconditionFailed => "precondition_failed",
        BuilderIssue::LaunchFailed => "launch_failed",
        BuilderIssue::ProcessFailed => "process_failed",
        BuilderIssue::ScopeViolated => "scope_violated",
    }
}

fn render_builder_scope_status(status: &BuilderScopeStatus) -> &'static str {
    match status {
        BuilderScopeStatus::NotChecked => "not_checked",
        BuilderScopeStatus::WithinScope => "within_scope",
        BuilderScopeStatus::Violated => "violated",
    }
}

fn render_terminal_field(value: &str) -> String {
    value.replace('\r', "").replace('\n', "\\n")
}

fn render_session_status(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Completed => "completed",
        SessionStatus::Stopped => "stopped",
    }
}
