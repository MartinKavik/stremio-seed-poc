use seed::{prelude::*, *, fetch};
use std::{str::FromStr, collections::HashMap};
use serde::Deserialize;
use web_sys::Performance;
use simsearch::SimSearch;

// ------ ------
//     Model
// ------ ------

type Id = String;
type Name = String;

pub struct Model {
    title: &'static str,
    download_url: &'static str,
    downloaded_records: HashMap<Id, Name>,
    simsearch: SimSearch<Id>,
    download_start: f64,
    download_time: Option<f64>,
    index_time: Option<f64>,
    query: String,
    search_time: Option<f64>,
    max_results: usize,
    results: Vec<ResultItem>,
    performance: Performance,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Record {
    id: Id,
    name: Name,
    poster: String,
    #[serde(rename(deserialize = "type"))]
    type_: String,
}

pub struct ResultItem {
    id: Id,
    name: Name,
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
        downloaded_records: HashMap::new(),
        simsearch: SimSearch::default(),
        download_start: 0.,
        download_time: None,
        index_time: None,
        query: "".to_owned(),
        search_time: None,
        max_results: 5,
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
    MaxResultsChanged(String),
    QueryChanged(String),
    Search,
}

async fn fetch_records(url: &'static str) -> Result<Msg, Msg> {
    fetch::Request::new(url)
        .fetch_json_data(Msg::Downloaded)
        .await
}

fn index(downloaded_records: &HashMap<Id, Name>) -> SimSearch<Id> {
    let mut simsearch = SimSearch::new();
    for (id, name) in downloaded_records {
        simsearch.insert(id.clone(), name);
    }
    simsearch
}

fn search(query: &str, simsearch: &SimSearch<Id>, max_results: usize, downloaded_records: &HashMap<Id, Name>) -> Vec<ResultItem> {
    simsearch
        .search(query)
        .into_iter()
        .take(max_results) //@TODO
        .map(|id| ResultItem { name: downloaded_records[&id].clone(), id })
        .collect()
}

pub fn update<GMs>(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMs>) {
    match msg {
        Msg::Download => {
            model.download_start = model.performance.now();
            model.download_time = None;
            orders.perform_cmd(fetch_records(model.download_url));
        },
        Msg::Downloaded(Ok(records)) => {
            model.downloaded_records =
                records
                    .into_iter()
                    .map(|record| (record.id, record.name))
                    .collect();
            model.download_time = Some(model.performance.now() - model.download_start);
        },
        Msg::Downloaded(Err(err)) => {
            log!("Download error", err);
        },
        Msg::Index => {
            let index_start = model.performance.now();
            model.simsearch = index(&model.downloaded_records);
            model.index_time = Some(model.performance.now() - index_start);
            orders.send_msg(Msg::Search);
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
            model.results = search(&model.query, &model.simsearch, model.max_results, &model.downloaded_records);
            model.search_time = Some(model.performance.now() - search_start);
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
        view_max_results(model),
        view_query(model),
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

pub fn view_results(model: &Model) -> Node<Msg> {
    table![
        style!{
            St::Padding => "10px 0",
        },
        thead![
            tr![
                th!["Id"],
                th!["Name"],
            ]
        ],
        tbody![
            model.results.iter().enumerate().map(view_result)
        ]
    ]
}

pub fn view_result(result_item_data: (usize, &ResultItem)) -> Node<Msg> {
    let (index, result_item) = result_item_data;
    tr![
        style!{
            St::BackgroundColor => if index % 2 == 0 { Some("aliceblue") } else { None },
        },
        td![
            style!{
                St::Padding => px(10),
            },
            result_item.id,
        ],
        td![
            style!{
                St::Padding => px(10),
            },
            result_item.name,
        ],
    ]
}
