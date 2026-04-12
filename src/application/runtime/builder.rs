use crate::application::runtime::builder_run_report::BuilderRunReport;
use crate::domain::ScholarOutput;

pub trait Builder {
    fn run(&mut self, scholar_output: &ScholarOutput) -> BuilderRunReport;
}
