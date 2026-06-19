mod calculator;
mod docker_ext;
mod emoji;
mod git;
mod integrations;
mod notes;
mod systemd_ext;
mod translate;

mod ai;

use std::sync::Arc;

use crate::extensions::SearchProvider;

pub fn all_builtin() -> Vec<Box<dyn SearchProvider>> {
    let mut exts: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(emoji::EmojiExtension),
        Box::new(notes::NotesExtension::new()),
        Box::new(git::GitExtension),
        Box::new(docker_ext::DockerExtension),
        Box::new(systemd_ext::SystemdExtension),
        Box::new(calculator::CalculatorExtension),
        Box::new(translate::TranslateExtension),
        Box::new(integrations::IntegrationsExtension),
    ];
    if crate::config::load().ai_enabled {
        exts.push(Box::new(ai::AiExtension::new()));
    }
    exts
}
