use serde::Serialize;

#[derive(Clone, Copy, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    App,
    File,
    Setting,
    Recent,
    Web,
    Bookmark,
    Browser,
    Contact,
    Quicklink,
    Snippet,
    Script,
    Clipboard,
    Extension,
    Window,
}

impl ResultKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::File => "file",
            Self::Setting => "setting",
            Self::Recent => "recent",
            Self::Web => "web",
            Self::Bookmark => "bookmark",
            Self::Browser => "browser",
            Self::Contact => "contact",
            Self::Quicklink => "quicklink",
            Self::Snippet => "snippet",
            Self::Script => "script",
            Self::Clipboard => "clipboard",
            Self::Extension => "extension",
            Self::Window => "window",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "app" => Some(Self::App),
            "file" => Some(Self::File),
            "setting" => Some(Self::Setting),
            "recent" => Some(Self::Recent),
            "web" => Some(Self::Web),
            "bookmark" => Some(Self::Bookmark),
            "browser" => Some(Self::Browser),
            "contact" => Some(Self::Contact),
            "quicklink" => Some(Self::Quicklink),
            "snippet" => Some(Self::Snippet),
            "script" => Some(Self::Script),
            "clipboard" => Some(Self::Clipboard),
            "extension" => Some(Self::Extension),
            "window" => Some(Self::Window),
            _ => None,
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
    let kind = ResultKind::from_str(kind)?;
    Some((kind, key))
}
