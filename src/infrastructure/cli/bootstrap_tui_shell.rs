use crate::infrastructure::cli::terminal_rendering::render_failure;
use crossterm::cursor::{MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use std::env;
use std::io::{self, Stdout, Write};
use std::process::ExitCode;

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

struct BootstrapTuiShell {
    stdout: Stdout,
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

        Ok(Self { stdout })
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

    fn render(&mut self, prompt: &str) -> Result<(), String> {
        let layout = BootstrapLayout::current(prompt);

        queue!(self.stdout, MoveTo(0, 0), Clear(ClearType::All))
            .map_err(|error| format!("failed to clear bootstrap shell frame: {error}"))?;

        for line in layout.lines {
            queue!(self.stdout, Print(line), Print("\r\n"))
                .map_err(|error| format!("failed to render bootstrap shell frame: {error}"))?;
        }

        queue!(
            self.stdout,
            Print(layout.prompt_line),
            MoveTo(layout.cursor_x, layout.cursor_y),
            Show
        )
        .map_err(|error| format!("failed to position bootstrap shell cursor: {error}"))?;

        self.stdout
            .flush()
            .map_err(|error| format!("failed to flush bootstrap shell frame: {error}"))
    }
}

impl Drop for BootstrapTuiShell {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
}

struct BootstrapLayout {
    lines: Vec<&'static str>,
    prompt_line: String,
    cursor_x: u16,
    cursor_y: u16,
}

impl BootstrapLayout {
    fn current(prompt: &str) -> Self {
        let prompt_width = prompt.chars().count() as u16;
        let (columns, rows) = bootstrap_terminal_size();

        if columns < 40 || rows < 7 {
            return Self {
                lines: vec!["Continuum TUI", "Idle | Esc exits", "Supervision: none"],
                prompt_line: format!("> {prompt}"),
                cursor_x: 2 + prompt_width,
                cursor_y: 3,
            };
        }

        Self {
            lines: vec![
                "Continuum TUI",
                "State: Idle",
                "Next: Type a prompt. Esc exits.",
                "Supervision",
                "No session running.",
                "Prompt [focused]",
            ],
            prompt_line: format!("> {prompt}"),
            cursor_x: 2 + prompt_width,
            cursor_y: 6,
        }
    }
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
