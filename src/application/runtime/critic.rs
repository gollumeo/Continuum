use crate::application::runtime::critic_signal::CriticSignal;
use crate::domain::ScholarOutput;

pub trait Critic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal;
}
