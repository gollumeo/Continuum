use continuum::{
    select_runtime_use_case_authority, Builder, BuilderIssue, BuilderRunReport,
    BuilderScopeStatus, ScholarOutput,
};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

pub struct CodexLocalBuilderAdapter {
    repository_root: PathBuf,
}

impl CodexLocalBuilderAdapter {
    pub fn new(repository_root: PathBuf) -> Self {
        Self { repository_root }
    }

    fn allowed_file_scope(&self, scholar_output: &ScholarOutput) -> Option<Vec<String>> {
        if let Some(authority) = select_runtime_use_case_authority(&scholar_output.selected_task_scope)
        {
            if let Some(builder_allowed_file_scope) = authority.builder_allowed_file_scope {
                return Some(
                    builder_allowed_file_scope
                        .iter()
                        .map(|path| (*path).to_string())
                        .collect(),
                );
            }
        }

        if scholar_output
            .selected_task_scope
            .contains("Modify only README.md and project-directives/index.md.")
        {
            Some(vec![
                "README.md".to_string(),
                "project-directives/index.md".to_string(),
            ])
        } else if scholar_output.selected_task_scope.contains("Modify only README.md.")
            && !scholar_output
                .selected_task_scope
                .contains("project-directives/index.md")
        {
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
                stderr: "builder requires an explicit allowed file scope; only README.md or README.md plus project-directives/index.md are admitted in this minimal adapter".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_increment_contract_fix_and_zero_confirm_prompt_injects_only_src_lib_rs_scope() {
        let builder = CodexLocalBuilderAdapter::new(PathBuf::from("/tmp/repo"));
        let scholar_output = ScholarOutput::new(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        );

        assert_eq!(builder.allowed_file_scope(&scholar_output), Some(vec!["src/lib.rs".to_string()]));
    }

    #[test]
    fn increment_contract_fix_and_zero_confirm_prompt_rejects_broader_scope_variation() {
        let builder = CodexLocalBuilderAdapter::new(PathBuf::from("/tmp/repo"));
        let scholar_output = ScholarOutput::new(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs and tests/increment_contract.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs and tests/increment_contract.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        );

        assert_eq!(builder.allowed_file_scope(&scholar_output), None);
    }

    #[test]
    fn exact_increment_contract_fix_prompt_injects_only_src_lib_rs_scope() {
        let builder = CodexLocalBuilderAdapter::new(PathBuf::from("/tmp/repo"));
        let scholar_output = ScholarOutput::new(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        );

        assert_eq!(builder.allowed_file_scope(&scholar_output), Some(vec!["src/lib.rs".to_string()]));
    }

    #[test]
    fn increment_contract_fix_prompt_rejects_any_non_src_lib_rs_scope_variation() {
        let builder = CodexLocalBuilderAdapter::new(PathBuf::from("/tmp/repo"));
        let scholar_output = ScholarOutput::new(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/main.rs.",
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/main.rs.",
        );

        assert_eq!(builder.allowed_file_scope(&scholar_output), None);
    }
}
