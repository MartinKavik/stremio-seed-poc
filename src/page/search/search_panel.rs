use seed::{prelude::*, *};

// ------ ------
//     Model
// ------ ------

pub struct Model {
    title: &'static str,
    download_url: &'static str,
    downloaded_records: Vec<Record>,
    indexed_records: Vec<IndexedRecord>,
    download_time: Option<u32>,
    index_time: Option<u32>,
    query: String,
    search_time: Option<u32>,
    max_results: u32,
    results: Vec<ResultItem>,
}

struct Record {
    id: String,
    name: String,
    poster: String,
    type_: String,
}

struct IndexedRecord {

}

struct ResultItem {
    id: String,
    name: String,
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
        download_time: None,
        index_time: None,
        query: "".to_owned(),
        search_time: None,
        max_results: 5,
        results: Vec::new(),
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub struct Msg;

pub fn update<GMs>(_: Msg, _: &mut Model, _: &mut impl Orders<Msg, GMs>) {
    //
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Padding => px(10),
        },
        // label
        h2![
            "Cinemeta",
        ],
        // download
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
                    St::BackgroundColor => "aquamarine",
                    St::BorderRadius => px(10),
                },
                "Download"
            ],
            div![
                style!{
                    St::Padding => "0 10px",
                },
                "150 ms"
            ],
        ],
        // index
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
                "Index"
            ],
            div![
                style!{
                    St::Padding => "0 10px",
                },
                "25 ms"
            ],
        ],
        // max results
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
                    At::Value => "4",
                    At::Type => "number",
                }
            ],
        ],
        // query
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
                    At::Value => "Shazam",
                }
            ],
            div![
                style!{
                    St::Cursor => "pointer",
                    St::Padding => "5px 15px",
                    St::BackgroundColor => "lightgreen",
                    St::BorderRadius => px(10),
                },
                "Search"
            ],
            div![
                style!{
                    St::Padding => "0 10px",
                },
                "28 ms"
            ],
        ],
        // results
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
                tr![
                    style!{
                        St::BackgroundColor => "aliceblue",
                    },
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "tt0448115"
                    ],
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "Shazam!"
                    ],
                ],
                tr![
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "tt0448115"
                    ],
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "Shazam!"
                    ],
                ],
                tr![
                    style!{
                        St::BackgroundColor => "aliceblue",
                    },
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "tt0448115"],
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "Shazam!"
                    ],
                ],
                tr![
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "tt2386490"
                    ],
                    td![
                        style!{
                            St::Padding => px(10),
                        },
                        "How to Train Your Dragon: The Hidden World!"
                    ],
                ],
            ]
        ]
    ]
}
