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
    let query = query.trim();
    if query.is_empty() {
        let recent = state.history.recent_results(5);
        let mut sections = Vec::new();
        if !recent.is_empty() {
            sections.push(ResultSection {
                id: "suggestions".to_string(),
                title: "Recentes".to_string(),
                results: recent,
            });
        }
        sections.push(default_apps_section(state, 8));
        return SearchResponse {
            quick_answer: None,
            sections,
        };
    }

    if let Some(cmd_q) = query.strip_prefix('>').map(str::trim) {
        let sys = crate::system_commands::search(cmd_q, &state.history, 8);
        let mut ext = state.extensions.search_all(cmd_q, &state.history, 8, None);
        let mut results = sys;
        results.append(&mut ext);
        return SearchResponse {
            quick_answer: None,
            sections: if results.is_empty() {
                Vec::new()
            } else {
                vec![ResultSection {
                    id: "commands".to_string(),
                    title: "Comandos".to_string(),
                    results,
                }]
            },
        };
    }

    if let Some(clip_q) = query.strip_prefix("clipboard:") {
        let clip_results = state.clipboard.search(clip_q.trim(), &state.history, 10);
        return SearchResponse {
            quick_answer: None,
            sections: vec![ResultSection {
                id: "clipboard".to_string(),
                title: "Clipboard".to_string(),
                results: clip_results,
            }],
        };
    }

    if query.eq_ignore_ascii_case("> settings") || query.eq_ignore_ascii_case("settings") {
        return SearchResponse {
            quick_answer: None,
            sections: vec![ResultSection {
                id: "settings_cmd".to_string(),
                title: "Comandos".to_string(),
                results: vec![crate::search::ranking::build_result(
                    crate::search::types::make_id(
                        crate::search::types::ResultKind::Setting,
                        "spotlight-settings",
                    ),
                    crate::search::types::ResultKind::Setting,
                    "Abrir configurações do Spotlight".to_string(),
                    Some("Settings UI".to_string()),
                    Some("preferences-system".to_string()),
                    2000,
                    query,
                    &state.history,
                )],
            }],
        };
    }

    let root_kw = crate::extensions::ExtensionRegistry::parse_root_search(query);
    let ext_query = root_kw.map(|(_, rest)| rest).unwrap_or(query);
    let ext_root = root_kw.map(|(kw, _)| kw);

    let quick = if root_kw.is_some() {
        None
    } else {
        crate::quick_answers::try_answer(query)
    };

    let skip_web = quick.as_ref().is_some_and(|q| {
        matches!(
            q.kind.as_str(),
            "currency" | "calculator" | "conversion" | "time"
        )
    });

    let web = if root_kw.is_some() || skip_web {
        None
    } else {
        crate::web_search::search_web(query)
    };

    let (productivity, (extensions, (browser, other))) = rayon::join(
        || {
            if root_kw.is_some() {
                (Vec::new(), Vec::new(), Vec::new())
            } else {
                (
                    state.quicklinks.search(query, &state.history, 5),
                    state.snippets.search(query, &state.history, 5),
                    state.scripts.search(query, &state.history, 5),
                )
            }
        },
        || {
            rayon::join(
                || {
                    let mut ext =
                        state.extensions.search_all(ext_query, &state.history, 8, ext_root);
                    if root_kw.is_none() {
                        if query.contains("window") || query.starts_with("win ") {
                            ext.extend(state.windows.search(query, &state.history, 4));
                            ext.extend(crate::windows::window_commands(&state.history, query, 5));
                        }
                    }
                    if skip_web {
                        ext.retain(|r| !r.id.starts_with("extension:calc:"));
                    }
                    ext
                },
                || {
                    rayon::join(
                        || {
                            if root_kw.is_some() {
                                (Vec::new(), Vec::new())
                            } else {
                                state.browser.search(query, &state.history, 5, 5)
                            }
                        },
                        || {
                            if root_kw.is_some() {
                                (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())
                            } else {
                                (
                                    state.contacts.search(query, &state.history, 4),
                                    state.apps.search_results(query, &state.history, 6),
                                    crate::files::search_files(query, &state.history, 6),
                                    state.settings.search(query, &state.history, 4),
                                    if query.len() >= 4 {
                                        state.clipboard.search(query, &state.history, 4)
                                    } else {
                                        Vec::new()
                                    },
                                )
                            }
                        },
                    )
                },
            )
        },
    );
    let (quicklinks, snippets, scripts) = productivity;
    let (bookmarks, browser_history) = browser;
    let (contacts, apps, files, settings, clipboard_hits) = other;

    let mut sections = Vec::new();

    if let Some(ref qa) = quick {
        sections.push(quick_answer_section(qa, query, &state.history));
    }

    if !extensions.is_empty() {
        sections.push(ResultSection {
            id: "extensions".to_string(),
            title: "Extensões".to_string(),
            results: extensions,
        });
    }
    if !quicklinks.is_empty() {
        sections.push(ResultSection {
            id: "quicklinks".to_string(),
            title: "Quicklinks".to_string(),
            results: quicklinks,
        });
    }
    if !snippets.is_empty() {
        sections.push(ResultSection {
            id: "snippets".to_string(),
            title: "Snippets".to_string(),
            results: snippets,
        });
    }
    if !scripts.is_empty() {
        sections.push(ResultSection {
            id: "scripts".to_string(),
            title: "Scripts".to_string(),
            results: scripts,
        });
    }

    if let Some(web_result) = web {
        sections.push(ResultSection {
            id: "web".to_string(),
            title: "Web".to_string(),
            results: vec![web_result],
        });
    }
    if !bookmarks.is_empty() {
        sections.push(ResultSection {
            id: "bookmarks".to_string(),
            title: "Favoritos".to_string(),
            results: bookmarks,
        });
    }
    if !browser_history.is_empty() {
        sections.push(ResultSection {
            id: "browser".to_string(),
            title: "Histórico do navegador".to_string(),
            results: browser_history,
        });
    }
    if !contacts.is_empty() {
        sections.push(ResultSection {
            id: "contacts".to_string(),
            title: "Contatos".to_string(),
            results: contacts,
        });
    }
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
    if !clipboard_hits.is_empty() {
        sections.push(ResultSection {
            id: "clipboard".to_string(),
            title: "Clipboard".to_string(),
            results: clipboard_hits,
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
        quick_answer: quick.map(|qa| crate::search::types::QuickAnswer {
            kind: qa.kind,
            label: qa.label,
            value: qa.value,
            hint: qa.hint,
        }),
        sections,
    }
}

fn quick_answer_section(
    qa: &crate::quick_answers::QuickAnswer,
    query: &str,
    history: &crate::history::HistoryDb,
) -> ResultSection {
    use crate::search::ranking::build_result;
    use crate::search::types::{make_id, ResultKind};

    let title = format!("{} {}", qa.label, qa.value);
    ResultSection {
        id: "quick_answer".to_string(),
        title: match qa.kind.as_str() {
            "currency" => "Conversão".to_string(),
            "calculator" => "Calculadora".to_string(),
            "time" => "Hora".to_string(),
            _ => "Resposta".to_string(),
        },
        results: vec![build_result(
            make_id(ResultKind::Recent, &format!("quick:{}", qa.kind)),
            ResultKind::Recent,
            title,
            qa.hint.clone(),
            Some("accessories-calculator".to_string()),
            3000,
            query,
            history,
        )],
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
        ResultKind::Web => {
            let engine = crate::config::load().web_search_engine;
            Some(PreviewData {
                title: format!("Pesquisar \"{key}\" na web"),
                subtitle: Some(crate::web_search::engine_label_cached().to_string()),
                description: Some(crate::web_search::search_url(key, &engine)),
                icon: Some("web-browser".to_string()),
                preview_text: None,
                preview_image: None,
                actions: vec![PreviewAction {
                    id: "open".to_string(),
                    label: "Pesquisar".to_string(),
                }],
            })
        }
        ResultKind::Bookmark | ResultKind::Browser => {
            let entry = state.browser.get_entry(id)?;
            Some(PreviewData {
                title: entry.title.clone(),
                subtitle: Some(entry.url.clone()),
                description: Some(format!("{} · {}", entry.source, entry.url)),
                icon: None,
                preview_text: None,
                preview_image: None,
                actions: vec![
                    PreviewAction {
                        id: "open".to_string(),
                        label: "Abrir no navegador".to_string(),
                    },
                    PreviewAction {
                        id: "copy_url".to_string(),
                        label: "Copiar URL".to_string(),
                    },
                ],
            })
        }
        ResultKind::Contact => {
            let contact = state.contacts.get(id)?;
            Some(PreviewData {
                title: contact.name.clone(),
                subtitle: contact.email.clone().or(contact.phone.clone()),
                description: contact.email.clone(),
                icon: Some("contact-new".to_string()),
                preview_text: contact.phone.clone(),
                preview_image: None,
                actions: vec![PreviewAction {
                    id: "open".to_string(),
                    label: if contact.email.is_some() {
                        "Enviar e-mail".to_string()
                    } else {
                        "Ligar".to_string()
                    },
                }],
            })
        }
        ResultKind::Quicklink => {
            let entry = state.quicklinks.get(key)?;
            Some(PreviewData {
                title: entry.title.clone(),
                subtitle: Some(entry.url.clone()),
                description: entry.alias.clone(),
                icon: entry.icon.clone().or(Some("web-browser".to_string())),
                preview_text: None,
                preview_image: None,
                actions: vec![PreviewAction {
                    id: "open".to_string(),
                    label: "Abrir URL".to_string(),
                }],
            })
        }
        ResultKind::Snippet => {
            let entry = state.snippets.get(key)?;
            Some(PreviewData {
                title: entry.name.clone(),
                subtitle: Some(entry.keyword.clone()),
                description: None,
                icon: Some("text-x-generic".to_string()),
                preview_text: Some(entry.text.clone()),
                preview_image: None,
                actions: vec![
                    PreviewAction {
                        id: "paste".to_string(),
                        label: "Colar".to_string(),
                    },
                    PreviewAction {
                        id: "copy".to_string(),
                        label: "Copiar".to_string(),
                    },
                ],
            })
        }
        ResultKind::Script => {
            let entry = state.scripts.get(key)?;
            Some(PreviewData {
                title: entry.title.clone(),
                subtitle: entry.keyword.clone(),
                description: Some(format!("{} {:?}", entry.command, entry.args)),
                icon: entry.icon.clone().or(Some("utilities-terminal".to_string())),
                preview_text: None,
                preview_image: None,
                actions: vec![PreviewAction {
                    id: "run".to_string(),
                    label: "Executar".to_string(),
                }],
            })
        }
        ResultKind::Clipboard => {
            let content_type = state.clipboard.get_type(key)?;
            let preview_image = state.clipboard.preview_image_path(key);
            let preview_text = if content_type == "text" {
                state.clipboard.get_content(key)
            } else {
                None
            };
            Some(PreviewData {
                title: if content_type == "image" {
                    "Imagem no clipboard".to_string()
                } else {
                    preview_text
                        .as_ref()
                        .map(|t| t.lines().next().unwrap_or(t).chars().take(60).collect())
                        .unwrap_or_else(|| "Clipboard".to_string())
                },
                subtitle: Some("Clipboard".to_string()),
                description: None,
                icon: Some("edit-copy".to_string()),
                preview_text,
                preview_image,
                actions: vec![PreviewAction {
                    id: "copy".to_string(),
                    label: "Copiar".to_string(),
                }],
            })
        }
        ResultKind::Extension | ResultKind::Window => {
            Some(PreviewData {
                title: key.to_string(),
                subtitle: Some(kind.as_str().to_string()),
                description: None,
                icon: None,
                preview_text: None,
                preview_image: None,
                actions: vec![PreviewAction {
                    id: "open".to_string(),
                    label: "Executar".to_string(),
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

pub fn open_result(state: &SpotlightState, id: &str, query: Option<&str>) -> Result<(), String> {
    use crate::apps::launcher;
    use crate::search::types::{parse_id, ResultKind};

    let (kind, key) = parse_id(id).ok_or_else(|| "Resultado inválido".to_string())?;
    let search_query = query.unwrap_or("");

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
            launcher::launch_app(&app)
        }
        ResultKind::File => {
            state.history.record_launch(id, kind);
            crate::files::open_file(key)
        }
        ResultKind::Setting if key == "spotlight-settings" => {
            state.history.record_launch(id, kind);
            let app = state
                .app_handle()
                .ok_or_else(|| "App não inicializado".to_string())?;
            crate::ui::show_settings_window(&app)
        }
        ResultKind::Setting if key.starts_with("syscmd:") => {
            state.history.record_launch(id, kind);
            let cmd = key.strip_prefix("syscmd:").unwrap_or("");
            if cmd == "settings" {
                let app = state
                    .app_handle()
                    .ok_or_else(|| "App não inicializado".to_string())?;
                crate::ui::show_settings_window(&app)
            } else {
                crate::system_commands::run(cmd)
            }
        }
        ResultKind::Setting => {
            state.history.record_launch(id, kind);
            crate::settings::open_setting(key)
        }
        ResultKind::Web => {
            state.history.save_meta(id, key, Some("Busca na web"), Some("web-browser"));
            state.history.record_launch(id, kind);
            crate::web_search::open_web_search(key)
        }
        ResultKind::Bookmark | ResultKind::Browser => {
            let entry = state
                .browser
                .get_entry(id)
                .ok_or_else(|| "URL não encontrada".to_string())?;
            state.history.save_meta(id, &entry.title, Some(&entry.url), None);
            state.history.record_launch(id, kind);
            crate::browser::open_url(&entry.url)
        }
        ResultKind::Contact => {
            let contact = state
                .contacts
                .get(id)
                .ok_or_else(|| "Contato não encontrado".to_string())?;
            state.history.save_meta(
                id,
                &contact.name,
                contact.email.as_deref(),
                Some("contact-new"),
            );
            state.history.record_launch(id, kind);
            crate::contacts::open_contact(&contact)
        }
        ResultKind::Quicklink => {
            let entry = state
                .quicklinks
                .get(key)
                .ok_or_else(|| "Quicklink não encontrado".to_string())?;
            let q = if search_query.is_empty() {
                key
            } else {
                search_query
            };
            let url = state
                .quicklinks
                .resolve_url(key, q)
                .unwrap_or(entry.url.clone());
            state.history.save_meta(id, &entry.title, Some(&url), entry.icon.as_deref());
            state.history.record_launch(id, kind);
            crate::browser::open_url(&url)
        }
        ResultKind::Snippet => {
            let entry = state
                .snippets
                .get(key)
                .ok_or_else(|| "Snippet não encontrado".to_string())?;
            state.history.save_meta(id, &entry.name, Some(&entry.keyword), Some("text-x-generic"));
            state.history.record_launch(id, kind);
            state.snippets.apply(key)?;
            Ok(())
        }
        ResultKind::Script => {
            let entry = state
                .scripts
                .get(key)
                .ok_or_else(|| "Script não encontrado".to_string())?;
            state.history.save_meta(id, &entry.title, entry.keyword.as_deref(), entry.icon.as_deref());
            state.history.record_launch(id, kind);
            let _ = state.scripts.run(key, search_query)?;
            Ok(())
        }
        ResultKind::Clipboard => {
            state.history.record_launch(id, kind);
            crate::clipboard::copy_item_to_clipboard(&state.clipboard, key)
        }
        ResultKind::Extension => {
            state.history.record_launch(id, kind);
            let (ext_id, action) = parse_extension_action(key)?;
            state.extensions.run_extension(ext_id, action, search_query)?;
            Ok(())
        }
        ResultKind::Window => {
            state.history.record_launch(id, kind);
            state.windows.run_action(key, search_query)
        }
        ResultKind::Recent => open_result(state, key, query),
    }
}

fn parse_extension_action(key: &str) -> Result<(&str, &str), String> {
    let ext_id = key
        .split(':')
        .next()
        .ok_or_else(|| "Extensão inválida".to_string())?;
    Ok((ext_id, key))
}

pub fn run_preview_action(
    state: &SpotlightState,
    id: &str,
    action: &str,
    query: Option<&str>,
) -> Result<(), String> {
    use crate::search::types::{parse_id, ResultKind};

    let (kind, key) = parse_id(id).ok_or_else(|| "Resultado inválido".to_string())?;

    match action {
        "copy_path" if kind == ResultKind::App => {
            let app = state
                .apps
                .get_by_id(key)
                .ok_or_else(|| "App não encontrado".to_string())?;
            crate::clipboard::write_to_clipboard(&app.desktop_path)
        }
        "copy_path" if kind == ResultKind::File => {
            crate::clipboard::write_to_clipboard(key)
        }
        "open" => open_result(state, id, query),
        "reveal" if kind == ResultKind::File => crate::files::reveal_in_folder(key),
        "copy_url" if matches!(kind, ResultKind::Bookmark | ResultKind::Browser) => {
            let url = state
                .browser
                .get_url(id)
                .ok_or_else(|| "URL não encontrada".to_string())?;
            crate::clipboard::write_to_clipboard(&url)
        }
        "paste" if kind == ResultKind::Snippet => {
            state.snippets.apply(key)
        }
        "copy" if kind == ResultKind::Snippet => {
            let entry = state
                .snippets
                .get(key)
                .ok_or_else(|| "Snippet não encontrado".to_string())?;
            crate::clipboard::write_to_clipboard(&entry.text)
        }
        "run" if kind == ResultKind::Script => {
            let _ = state.scripts.run(key, query.unwrap_or(""))?;
            Ok(())
        }
        "copy" if kind == ResultKind::Clipboard => {
            crate::clipboard::copy_item_to_clipboard(&state.clipboard, key)
        }
        _ => Err("Ação não suportada".to_string()),
    }
}
