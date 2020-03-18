use seed::{prelude::*, *, fetch};
use std:: str::FromStr;
use serde::Deserialize;
use web_sys::Performance;
use localsearch::{self, LocalSearch};

// ------ ------
//     Model
// ------ ------

pub struct Model {
    title: &'static str,
    download_url: &'static str,
    downloaded_records: Vec<Record>,
    local_search: LocalSearch<Record>,
    download_start: f64,
    download_time: Option<f64>,
    index_time: Option<f64>,
    query: String,
    search_time: Option<f64>,
    max_autocomplete_results: usize,
    max_results: usize,
    autocomplete_results: Vec<String>,
    results: Vec<localsearch::ResultItemOwned<Record>>,
    performance: Performance,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Record {
    id: String,
    name: String,
//    poster: String,
//    #[serde(rename(deserialize = "type"))]
//    type_: String,
}

// ------ ------
//     Init
// ------ ------

pub fn init(
    title: &'static str,
    download_url: &'static str,
) -> Model {
    Model {
        title,
        download_url,
        downloaded_records: Vec::new(),
        local_search: LocalSearch::new(|rec: &Record| &rec.name),
        download_start: 0.,
        download_time: None,
        index_time: None,
        query: "".to_owned(),
        search_time: None,
        max_autocomplete_results: 5,
        max_results: 5,
        autocomplete_results: Vec::new(),
        results: Vec::new(),
        performance: window().performance().expect("get `Performance`"),
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    Download,
    Downloaded(fetch::ResponseDataResult<Vec<Record>>),
    Index,
    MaxAutocompleteResultsChanged(String),
    MaxResultsChanged(String),
    QueryChanged(String),
    Search,
}

async fn fetch_records(url: &'static str) -> Result<Msg, Msg> {
    fetch::Request::new(url)
        .fetch_json_data(Msg::Downloaded)
        .await
}

fn index(downloaded_records: Vec<Record>) -> LocalSearch<Record> {
    let mut local_search = LocalSearch::new(|rec: &Record| &rec.name);
    local_search.set_documents(downloaded_records);
    local_search
}

fn search(query: &str, local_search: &LocalSearch<Record>, max_results: usize) -> Vec<localsearch::ResultItemOwned<Record>> {
    local_search
        .search(query, max_results)
        .iter()
        .map(|result| result.to_owned_result())
        .collect()
}

fn autocomplete(query: &str, local_search: &LocalSearch<Record>, max_results: usize) -> Vec<String> {
    if let Some(last_token) = localsearch::default_tokenizer(query).last() {
        local_search.autocomplete(last_token, max_results)
    } else {
        Vec::new()
    }
}

pub fn update<GMs>(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMs>) {
    match msg {
        Msg::Download => {
            model.download_start = model.performance.now();
            model.download_time = None;
            orders.perform_cmd(fetch_records(model.download_url));
        },
        Msg::Downloaded(Ok(records)) => {
            model.downloaded_records = records;
            model.download_time = Some(model.performance.now() - model.download_start);
        },
        Msg::Downloaded(Err(err)) => {
            log!("Download error", err);
        },
        Msg::Index => {
            let records = model.downloaded_records.clone();
            let index_start = model.performance.now();
            model.local_search = index(records);
            model.index_time = Some(model.performance.now() - index_start);
            orders.send_msg(Msg::Search);
        },
        Msg::MaxAutocompleteResultsChanged(max_results) => {
            if let Ok(max_results) = usize::from_str(&max_results) {
                model.max_autocomplete_results = max_results;
                orders.send_msg(Msg::Search);
            }
        },
        Msg::MaxResultsChanged(max_results) => {
            if let Ok(max_results) = usize::from_str(&max_results) {
                model.max_results = max_results;
                orders.send_msg(Msg::Search);
            }
        },
        Msg::QueryChanged(query) => {
            model.query = query;
            orders.send_msg(Msg::Search);
        },
        Msg::Search => {
            let search_start = model.performance.now();
            model.results = search(&model.query, &model.local_search, model.max_results);
            model.search_time = Some(model.performance.now() - search_start);
            model.autocomplete_results = autocomplete(&model.query, &model.local_search, model.max_autocomplete_results);
        }
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Padding => px(50) + " " + &px(10),
            St::MinWidth => px(320),
        },
        h2![
            model.title,
        ],
        view_download(model),
        view_index(model),
        view_max_autocomplete_results(model),
        view_max_results(model),
        view_query(model),
        view_autocomplete_results(model),
        view_results(model),
    ]
}

pub fn view_download(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::AlignItems => "center",
            St::Padding => "10px 0",
        },
        div![
            style!{
                St::Cursor => "pointer",
                St::Padding => "5px 15px",
                St::BackgroundColor => "lightgreen",
                St::BorderRadius => px(10),
            },
            simple_ev(Ev::Click, Msg::Download),
            "Download & Deserialize"
        ],
        div![
            style!{
                St::Padding => "0 10px",
            },
            format!("{} ms", model.download_time.as_ref().map_or("-".to_owned(), ToString::to_string)),
        ],
    ]
}

pub fn view_index(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::AlignItems => "center",
            St::Padding => "10px 0",
        },
        div![
            style!{
                St::Cursor => "pointer",
                St::Padding => "5px 15px",
                St::BackgroundColor => "lightblue",
                St::BorderRadius => px(10),
            },
            simple_ev(Ev::Click, Msg::Index),
            "Index"
        ],
        div![
            style!{
                St::Padding => "0 10px",
            },
            format!("{} ms", model.index_time.as_ref().map_or("-".to_owned(), ToString::to_string)),
        ],
    ]
}

pub fn view_max_autocomplete_results(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::AlignItems => "center",
            St::Padding => "10px 0",
        },
        div![
            "Max autocomplete results:"
        ],
        input![
            style!{
                St::Padding => "3px 8px",
                St::Margin => "0 10px",
                St::Border => "2px solid black",
            },
            attrs!{
                At::Value => model.max_autocomplete_results,
                At::Type => "number",
            },
            input_ev(Ev::Input, Msg::MaxAutocompleteResultsChanged),
        ],
    ]
}

pub fn view_max_results(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::AlignItems => "center",
            St::Padding => "10px 0",
        },
        div![
            "Max results:"
        ],
        input![
            style!{
                St::Padding => "3px 8px",
                St::Margin => "0 10px",
                St::Border => "2px solid black",
            },
            attrs!{
                At::Value => model.max_results,
                At::Type => "number",
            },
            input_ev(Ev::Input, Msg::MaxResultsChanged),
        ],
    ]
}

pub fn view_query(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::AlignItems => "center",
            St::Padding => "10px 0",
        },
        div![
            "Query:"
        ],
        input![
            style!{
                St::Padding => "3px 8px",
                St::Margin => "0 10px",
                St::Border => "2px solid black",
            },
            attrs!{
                At::Value => model.query,
            },
            input_ev(Ev::Input, Msg::QueryChanged),
        ],
        div![
            format!("{} ms", model.search_time.as_ref().map_or("-".to_owned(), ToString::to_string)),
        ],
    ]
}

pub fn view_autocomplete_results(model: &Model) -> Node<Msg> {
    div![
        model.autocomplete_results.join(" - ")
    ]
}


pub fn view_results(model: &Model) -> Node<Msg> {
    table![
        style!{
            St::Padding => "10px 0",
        },
        thead![
            tr![
                th!["Id"],
                th!["Name"],
                th!["Score"],
            ]
        ],
        tbody![
            model.results.iter().enumerate().map(view_result)
        ]
    ]
}

pub fn view_result(result_item_data: (usize, &localsearch::ResultItemOwned<Record>)) -> Node<Msg> {
    let (index, result_item) = result_item_data;
    tr![
        style!{
            St::BackgroundColor => if index % 2 == 0 { Some("aliceblue") } else { None },
        },
        td![
            style!{
                St::Padding => px(10),
            },
            result_item.document.id,
        ],
        td![
            style!{
                St::Padding => px(10),
            },
            result_item.document.name,
        ],
        td![
            style!{
                St::Padding => px(10),
            },
            result_item.score.to_string(),
        ],
    ]
}
