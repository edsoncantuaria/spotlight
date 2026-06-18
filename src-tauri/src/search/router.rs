use serde::Serialize;

use crate::SpotlightState;
use crate::search::types::{ResultSection, SearchResponse};

#[derive(Clone, Serialize)]
pub struct PreviewAction {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Serialize)]
pub struct PreviewData {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub preview_text: Option<String>,
    pub preview_image: Option<String>,
    pub actions: Vec<PreviewAction>,
}

pub fn search(state: &SpotlightState, query: &str) -> SearchResponse {
    let quick = crate::quick_answers::try_answer(query).map(|qa| crate::search::types::QuickAnswer {
        kind: qa.kind,
        label: qa.label,
        value: qa.value,
        hint: qa.hint,
    });

    let query = query.trim();
    if query.is_empty() {
        let recent = state.history.recent_results(5);
        let sections = if recent.is_empty() {
            vec![default_apps_section(state, 8)]
        } else {
            vec![ResultSection {
                id: "suggestions".to_string(),
                title: "Sugestões".to_string(),
                results: recent,
            }]
        };
        return SearchResponse {
            quick_answer: None,
            sections,
        };
    }

    let apps = state.apps.search_results(query, &state.history, 6);
    let files = crate::files::search_files(query, &state.history, 6);
    let settings = state.settings.search(query, &state.history, 4);

    let mut sections = Vec::new();
    if !apps.is_empty() {
        sections.push(ResultSection {
            id: "apps".to_string(),
            title: "Aplicativos".to_string(),
            results: apps,
        });
    }
    if !files.is_empty() {
        sections.push(ResultSection {
            id: "files".to_string(),
            title: "Documentos".to_string(),
            results: files,
        });
    }
    if !settings.is_empty() {
        sections.push(ResultSection {
            id: "settings".to_string(),
            title: "Configurações".to_string(),
            results: settings,
        });
    }

    SearchResponse {
        quick_answer: quick,
        sections,
    }
}

fn default_apps_section(state: &SpotlightState, limit: usize) -> ResultSection {
    ResultSection {
        id: "apps".to_string(),
        title: "Aplicativos".to_string(),
        results: state.apps.search_results("", &state.history, limit),
    }
}

pub fn get_preview(state: &SpotlightState, id: &str) -> Option<PreviewData> {
    use crate::search::types::{parse_id, ResultKind};

    let (kind, key) = parse_id(id)?;

    match kind {
        ResultKind::App => {
            let app = state.apps.get_by_id(key)?;
            Some(PreviewData {
                title: app.name.clone(),
                subtitle: Some("Aplicativo".to_string()),
                description: app.comment.clone(),
                icon: app.icon.clone(),
                preview_text: None,
                preview_image: None,
                actions: vec![
                    PreviewAction {
                        id: "open".to_string(),
                        label: "Abrir".to_string(),
                    },
                    PreviewAction {
                        id: "copy_path".to_string(),
                        label: "Copiar caminho".to_string(),
                    },
                ],
            })
        }
        ResultKind::File => {
            let path = key;
            let name = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();
            let preview_text = crate::files::read_preview_text(path, 20);
            let preview_image = if crate::files::is_image(path) {
                Some(path.to_string())
            } else {
                None
            };
            Some(PreviewData {
                title: name,
                subtitle: std::path::Path::new(path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string()),
                description: None,
                icon: None,
                preview_text,
                preview_image,
                actions: vec![
                    PreviewAction {
                        id: "open".to_string(),
                        label: "Abrir".to_string(),
                    },
                    PreviewAction {
                        id: "reveal".to_string(),
                        label: "Mostrar na pasta".to_string(),
                    },
                    PreviewAction {
                        id: "copy_path".to_string(),
                        label: "Copiar caminho".to_string(),
                    },
                ],
            })
        }
        ResultKind::Setting => {
            let setting = state.settings.get(key)?;
            Some(PreviewData {
                title: setting.name.clone(),
                subtitle: Some("Configurações do sistema".to_string()),
                description: None,
                icon: setting.icon.clone(),
                preview_text: None,
                preview_image: None,
                actions: vec![PreviewAction {
                    id: "open".to_string(),
                    label: "Abrir".to_string(),
                }],
            })
        }
        ResultKind::Recent => state.history.recent_results(1).first().map(|r| PreviewData {
            title: r.title.clone(),
            subtitle: r.subtitle.clone(),
            description: None,
            icon: r.icon.clone(),
            preview_text: None,
            preview_image: None,
            actions: vec![PreviewAction {
                id: "open".to_string(),
                label: "Abrir".to_string(),
            }],
        }),
    }
}

pub fn open_result(state: &SpotlightState, id: &str) -> Result<(), String> {
    use crate::apps::launcher;
    use crate::search::types::{parse_id, ResultKind};

    let (kind, key) = parse_id(id).ok_or_else(|| "Resultado inválido".to_string())?;

    match kind {
        ResultKind::App => {
            let app = state
                .apps
                .get_by_id(key)
                .ok_or_else(|| "Aplicativo não encontrado".to_string())?;
            state.history.save_meta(
                id,
                &app.name,
                app.comment.as_deref(),
                app.icon.as_deref(),
            );
            state.history.record_launch(id, kind);
            launcher::launch_app(app)
        }
        ResultKind::File => {
            state.history.record_launch(id, kind);
            crate::files::open_file(key)
        }
        ResultKind::Setting => {
            state.history.record_launch(id, kind);
            crate::settings::open_setting(key)
        }
        ResultKind::Recent => open_result(state, key),
    }
}

pub fn run_preview_action(state: &SpotlightState, id: &str, action: &str) -> Result<(), String> {
    use crate::search::types::{parse_id, ResultKind};

    let (kind, key) = parse_id(id).ok_or_else(|| "Resultado inválido".to_string())?;

    match action {
        "open" => open_result(state, id),
        "reveal" if kind == ResultKind::File => crate::files::reveal_in_folder(key),
        _ => Err("Ação não suportada".to_string()),
    }
}
