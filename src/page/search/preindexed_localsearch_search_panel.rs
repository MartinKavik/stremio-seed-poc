use seed::{prelude::*, *, fetch};
use std:: str::FromStr;
use serde::Deserialize;
use web_sys::Performance;

// ------ ------
//     Model
// ------ ------

pub struct Model {
    title: &'static str,
    download_url: &'static str,
    downloaded_records: Vec<Record>,
    indexed_records: Vec<IndexedRecord>,
    download_start: f64,
    download_time: Option<f64>,
    query: String,
    search_time: Option<f64>,
    max_results: usize,
    results: Vec<ResultItem>,
    performance: Performance,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Record {
    id: String,
    name: String,
    poster: String,
    #[serde(rename(deserialize = "type"))]
    type_: String,
}

pub struct IndexedRecord {
    id: String,
    name: String,
    name_lowercase: String,
}

pub struct ResultItem {
    id: String,
    name: String,
    score: f64,
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
        indexed_records: Vec::new(),
        download_start: 0.,
        download_time: None,
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
    MaxResultsChanged(String),
    QueryChanged(String),
    Search,
    CreateFileWithSerializedLocalSearch,
}

async fn fetch_records(url: &'static str) -> Result<Msg, Msg> {
    fetch::Request::new(url)
        .fetch_json_data(Msg::Downloaded)
        .await
}

fn search(query: &str, indexed_records: &[IndexedRecord], max_results: usize) -> Vec<ResultItem> {
    let query = query.to_lowercase();
    indexed_records
        .iter()
        .filter_map(|record| {
            if record.name_lowercase.contains(&query) {
                Some(ResultItem {
                    id: record.id.clone(),
                    name: record.name.clone(),
                    score: 1.,
                })
            } else {
                None
            }
        })
        .take(max_results)
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
            model.downloaded_records = records;
            model.download_time = Some(model.performance.now() - model.download_start);
        },
        Msg::Downloaded(Err(err)) => {
            log!("Download error", err);
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
            model.results = search(&model.query, &model.indexed_records, model.max_results);
            model.search_time = Some(model.performance.now() - search_start);
        },
        Msg::CreateFileWithSerializedLocalSearch => { } // @TODO
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
        view_create_file_with_serialized_localsearch(),
        view_download(model),
        view_max_results(model),
        view_query(model),
        view_results(model),
    ]
}

pub fn view_create_file_with_serialized_localsearch() -> Node<Msg> {
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
            simple_ev(Ev::Click, Msg::CreateFileWithSerializedLocalSearch),
            "Create file with serialized LocalSearch"
        ],
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
                th!["Score"],
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
        td![
            style!{
                St::Padding => px(10),
            },
            result_item.score.to_string(),
        ],
    ]
}
