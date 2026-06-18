use serde::Serialize;

#[derive(Clone, Copy, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    App,
    File,
    Setting,
    Recent,
}

impl ResultKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::File => "file",
            Self::Setting => "setting",
            Self::Recent => "recent",
        }
    }
}

#[derive(Clone, Serialize)]
pub struct MatchRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub kind: ResultKind,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub score: f64,
    pub match_ranges: Vec<MatchRange>,
}

#[derive(Clone, Serialize)]
pub struct ResultSection {
    pub id: String,
    pub title: String,
    pub results: Vec<SearchResult>,
}

#[derive(Clone, Serialize)]
pub struct QuickAnswer {
    pub kind: String,
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct SearchResponse {
    pub quick_answer: Option<QuickAnswer>,
    pub sections: Vec<ResultSection>,
}

pub fn make_id(kind: ResultKind, key: &str) -> String {
    format!("{}:{}", kind.as_str(), key)
}

pub fn parse_id(id: &str) -> Option<(ResultKind, &str)> {
    let (kind, key) = id.split_once(':')?;
    let kind = match kind {
        "app" => ResultKind::App,
        "file" => ResultKind::File,
        "setting" => ResultKind::Setting,
        "recent" => ResultKind::Recent,
        _ => return None,
    };
    Some((kind, key))
}
