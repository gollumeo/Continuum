use crate::infrastructure::cli::terminal_rendering::render_failure;
use crate::infrastructure::runtime::local_shell_runtime::run_local_shell_session_with_admission_hook;
use continuum::RawMission;
use crossterm::cursor::{MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use std::env;
use std::io::{self, Stdout, Write};
use std::process::ExitCode;

const UNDERSPECIFIED_DOCUMENT_PROMPT_REFUSAL: &str =
    "refused to act on an underspecified document prompt; add an explicit allowed file scope";

pub fn run_bootstrap_tui_shell() -> ExitCode {
    let mut shell = match BootstrapTuiShell::enter() {
        Ok(shell) => shell,
        Err(message) => return render_failure(None, None, None, Some(&message)),
    };

    let result = shell.run();

    drop(shell);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => render_failure(None, None, None, Some(&message)),
    }
}

#[derive(Clone)]
enum TuiSessionStatus {
    Active,
    Completed,
    Stopped,
}

#[derive(Clone)]
struct SessionRecord {
    prompt_preview: String,
    status: TuiSessionStatus,
}

fn make_preview(submission: &str) -> String {
    submission.chars().take(40).collect()
}

struct BootstrapTuiShell {
    stdout: Stdout,
    view: BootstrapView,
    sessions: Vec<SessionRecord>,
    selected_idx: Option<usize>,
}

impl BootstrapTuiShell {
    fn enter() -> Result<Self, String> {
        enable_raw_mode()
            .map_err(|error| format!("failed to enable bootstrap raw mode: {error}"))?;

        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, Show) {
            let _ = disable_raw_mode();
            return Err(format!(
                "failed to enter bootstrap alternate screen: {error}"
            ));
        }

        Ok(Self {
            stdout,
            view: BootstrapView::idle(),
            sessions: Vec::new(),
            selected_idx: None,
        })
    }

    fn run(&mut self) -> Result<(), String> {
        let mut prompt = String::new();

        loop {
            self.render(&prompt)?;

            match event::read()
                .map_err(|error| format!("failed to read bootstrap shell event: {error}"))?
            {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if should_exit_bootstrap_shell(key) {
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Enter => {
                            if prompt.is_empty() {
                                continue;
                            }

                            self.submit_prompt(&mut prompt)?;
                        }
                        KeyCode::Up => {
                            if let Some(idx) = self.selected_idx {
                                if let Some((min_visible, max_visible)) =
                                    visible_session_bounds(self.sessions.len())
                                {
                                    let clamped = idx.clamp(min_visible, max_visible);
                                    self.selected_idx =
                                        Some(clamped.saturating_sub(1).max(min_visible));
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let Some(idx) = self.selected_idx {
                                if let Some((min_visible, max_visible)) =
                                    visible_session_bounds(self.sessions.len())
                                {
                                    let clamped = idx.clamp(min_visible, max_visible);
                                    self.selected_idx = Some((clamped + 1).min(max_visible));
                                }
                            }
                        }
                        KeyCode::Char(character)
                            if !key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            prompt.push(character);
                        }
                        KeyCode::Backspace => {
                            prompt.pop();
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => continue,
                _ => {}
            }
        }
    }

    fn submit_prompt(&mut self, prompt: &mut String) -> Result<(), String> {
        let submission = prompt.clone();
        prompt.clear();

        if submission.starts_with('/') {
            self.view = BootstrapView::unsupported_command();
            return Ok(());
        }

        self.view = BootstrapView::mission_submitting();

        self.sessions.push(SessionRecord {
            prompt_preview: make_preview(&submission),
            status: TuiSessionStatus::Active,
        });
        let idx = self.sessions.len() - 1;
        self.selected_idx = Some(idx);

        self.render(prompt)?;

        let repository_root = env::current_dir()
            .map_err(|error| format!("failed to resolve current repository root: {error}"))?;

        let sessions_snapshot = self.sessions.clone();
        let selected_snapshot = self.selected_idx;
        let admitted_prompt = prompt.to_string();

        let outcome = run_local_shell_session_with_admission_hook(
            RawMission::new(&submission),
            repository_root,
            move || {
                render_bootstrap_frame(
                    &admitted_prompt,
                    &BootstrapView::mission_admitted(),
                    &sessions_snapshot,
                    selected_snapshot,
                )
            },
        )?;

        self.sessions[idx].status = if outcome.entered_admitted_path && outcome.result.is_ok() {
            TuiSessionStatus::Completed
        } else {
            TuiSessionStatus::Stopped
        };

        self.view = if outcome.entered_admitted_path {
            BootstrapView::mission_admitted()
        } else {
            match outcome.result {
                Ok(_) => BootstrapView::mission_admitted(),
                Err(report) => BootstrapView::mission_refused(report.error),
            }
        };

        Ok(())
    }

    fn render(&mut self, prompt: &str) -> Result<(), String> {
        let _ = &self.stdout;
        render_bootstrap_frame(prompt, &self.view, &self.sessions, self.selected_idx)
    }
}

struct BootstrapView {
    state_line: String,
    next_line: String,
    compact_state_line: String,
    compact_supervision_line: String,
}

impl BootstrapView {
    fn idle() -> Self {
        Self {
            state_line: "State: Idle".to_string(),
            next_line: "Next: Type a prompt. Esc exits.".to_string(),
            compact_state_line: "Idle | Esc exits".to_string(),
            compact_supervision_line: "Supervision: none".to_string(),
        }
    }

    fn mission_submitting() -> Self {
        Self {
            state_line: "State: Submitting mission".to_string(),
            next_line: "Mode: Mission".to_string(),
            compact_state_line: "Submitting | Mission".to_string(),
            compact_supervision_line: "Admission check".to_string(),
        }
    }

    fn mission_admitted() -> Self {
        Self {
            state_line: "State: Mission admitted".to_string(),
            next_line: "Mode: Mission".to_string(),
            compact_state_line: "Admitted | Mission".to_string(),
            compact_supervision_line: "Session initialized".to_string(),
        }
    }

    fn mission_refused(error: Option<&str>) -> Self {
        let next_line = match error {
            Some(UNDERSPECIFIED_DOCUMENT_PROMPT_REFUSAL) => {
                "Refusal: add an explicit allowed file scope.".to_string()
            }
            Some(error) => format!("Refusal: {error}"),
            None => "Refusal: mission not admitted.".to_string(),
        };

        Self {
            state_line: "State: Mission refused".to_string(),
            next_line,
            compact_state_line: "Refused | Retry".to_string(),
            compact_supervision_line: "No build started".to_string(),
        }
    }

    fn unsupported_command() -> Self {
        Self {
            state_line: "State: Command mode".to_string(),
            next_line: "Command unsupported in Story 1.2.".to_string(),
            compact_state_line: "Command | Unsupported".to_string(),
            compact_supervision_line: "Type mission".to_string(),
        }
    }
}

impl Drop for BootstrapTuiShell {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
}

struct BootstrapLayout {
    lines: Vec<String>,
    line_colors: Vec<Option<Color>>,
    prompt_line: String,
    cursor_x: u16,
    cursor_y: u16,
}

impl BootstrapLayout {
    fn current(
        prompt: &str,
        view: &BootstrapView,
        sessions: &[SessionRecord],
        selected_idx: Option<usize>,
    ) -> Self {
        let prompt_width = prompt.chars().count() as u16;
        let (columns, rows) = bootstrap_terminal_size();

        if columns < 40 || rows < 7 {
            return Self {
                lines: vec![
                    "Continuum TUI".to_string(),
                    view.compact_state_line.clone(),
                    view.compact_supervision_line.clone(),
                ],
                line_colors: vec![None, None, None],
                prompt_line: format!("> {prompt}"),
                cursor_x: 2 + prompt_width,
                cursor_y: 3,
            };
        }

        let max_session_rows = (rows as usize).saturating_sub(6);
        let start = sessions.len().saturating_sub(max_session_rows);
        let visible = &sessions[start..];

        let mut lines: Vec<String> = vec![
            "Continuum TUI".to_string(),
            view.state_line.clone(),
            view.next_line.clone(),
            "Sessions".to_string(),
        ];
        let mut line_colors: Vec<Option<Color>> = vec![None, None, None, None];

        if visible.is_empty() {
            lines.push("  No sessions yet.".to_string());
            line_colors.push(None);
        } else {
            for (i, session) in visible.iter().enumerate() {
                let absolute_idx = start + i;
                let prefix = if selected_idx == Some(absolute_idx) {
                    "> "
                } else {
                    "  "
                };
                let (label, color) = match session.status {
                    TuiSessionStatus::Active => ("[~]", Color::Yellow),
                    TuiSessionStatus::Completed => ("[+]", Color::Green),
                    TuiSessionStatus::Stopped => ("[!]", Color::Red),
                };
                lines.push(format!("{prefix}{label} {}", session.prompt_preview));
                line_colors.push(Some(color));
            }
        }

        lines.push("Prompt [focused]".to_string());
        line_colors.push(None);

        let cursor_y = lines.len() as u16;

        Self {
            lines,
            line_colors,
            prompt_line: format!("> {prompt}"),
            cursor_x: 2 + prompt_width,
            cursor_y,
        }
    }
}

fn render_bootstrap_frame(
    prompt: &str,
    view: &BootstrapView,
    sessions: &[SessionRecord],
    selected_idx: Option<usize>,
) -> Result<(), String> {
    let layout = BootstrapLayout::current(prompt, view, sessions, selected_idx);
    let mut stdout = io::stdout();

    queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))
        .map_err(|error| format!("failed to clear bootstrap shell frame: {error}"))?;

    for (line, maybe_color) in layout.lines.iter().zip(layout.line_colors.iter()) {
        if let Some(color) = maybe_color {
            queue!(
                stdout,
                SetForegroundColor(*color),
                Print(line),
                ResetColor,
                Print("\r\n")
            )
            .map_err(|error| format!("failed to render bootstrap shell frame: {error}"))?;
        } else {
            queue!(stdout, Print(line), Print("\r\n"))
                .map_err(|error| format!("failed to render bootstrap shell frame: {error}"))?;
        }
    }

    queue!(
        stdout,
        Print(layout.prompt_line),
        MoveTo(layout.cursor_x, layout.cursor_y),
        Show
    )
    .map_err(|error| format!("failed to position bootstrap shell cursor: {error}"))?;

    stdout
        .flush()
        .map_err(|error| format!("failed to flush bootstrap shell frame: {error}"))
}

fn should_exit_bootstrap_shell(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        || (key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL))
}

fn bootstrap_terminal_size() -> (u16, u16) {
    let env_columns = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok());
    let env_rows = env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<u16>().ok());

    if let (Some(columns), Some(rows)) = (env_columns, env_rows) {
        if columns > 0 && rows > 0 {
            return (columns, rows);
        }
    }

    if let Ok((columns, rows)) = terminal::size() {
        if columns > 0 && rows > 0 {
            return (columns, rows);
        }
    }

    (80, 24)
}

fn visible_session_bounds(session_count: usize) -> Option<(usize, usize)> {
    if session_count == 0 {
        return None;
    }

    let (_, rows) = bootstrap_terminal_size();
    let max_session_rows = (rows as usize).saturating_sub(6);

    if max_session_rows == 0 {
        return Some((0, session_count - 1));
    }

    let start = session_count.saturating_sub(max_session_rows);
    Some((start, session_count - 1))
}
