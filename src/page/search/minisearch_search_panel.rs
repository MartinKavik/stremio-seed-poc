use seed::{prelude::*, *, fetch};
use std:: str::FromStr;
use web_sys::Performance;
use serde::Deserialize;

// https://github.com/lucaong/minisearch

// ------ ------
//     Model
// ------ ------

pub struct Model {
    title: &'static str,
    download_url: &'static str,
    downloaded_records: String,
    download_start: f64,
    download_time: Option<f64>,
    index_time: Option<f64>,
    query: String,
    search_time: Option<f64>,
    max_results: usize,
    results: Vec<ResultItem>,
    performance: Performance,
}

#[derive(Deserialize)]
pub struct ResultItem {
    id: String,
    name: String,
    score: f64,
    // terms
    // match
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
        downloaded_records: String::new(),
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
    Downloaded(fetch::ResponseDataResult<String>),
    Index,
    MaxResultsChanged(String),
    QueryChanged(String),
    Search,
}

async fn fetch_records(url: &'static str) -> Result<Msg, Msg> {
    fetch::Request::new(url)
        .fetch_string_data(Msg::Downloaded)
        .await
}

#[wasm_bindgen]
extern "C" {
    fn index_multisearch(documents: &str);
}

fn index(downloaded_records: &str) {
    index_multisearch(downloaded_records);
}

#[wasm_bindgen]
extern "C" {
    fn search_multisearch(query: &str) -> Box<[JsValue]>;
}

fn search(query: &str, max_results: usize) -> Vec<ResultItem> {
    search_multisearch(query)
        .into_iter()
        .take(max_results)
        .map(|result| result.into_serde().expect("deserialize `ResultItem`"))
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
        Msg::Index => {
            let index_start = model.performance.now();
            index(&model.downloaded_records);
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
            model.results = search(&model.query, model.max_results);
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