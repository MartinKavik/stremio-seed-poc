use crate::{PageId, Urls as RootUrls, Context};
use seed::{prelude::*, *};
use seed_styles::{pc, rem, em};
use seed_styles::*;
use crate::styles::{self, themes::{Color, Breakpoint}, global};
use serde::Deserialize;
use localsearch::LocalSearch;

const search_debounce_time: u32 = 400;

fn on_click_not_implemented() -> EventHandler<Msg> {
    ev(Ev::Click, |_| { window().alert_with_message("Not implemented!"); })
}

// ------ ------
//     Init
// ------ ------

pub fn init(
    mut url: Url,
    model: &mut Option<Model>,
    orders: &mut impl Orders<Msg>,
) -> Option<PageId> {
    let base_url = url.to_hash_base_url();
    let search_query = url.next_hash_path_part().map(ToOwned::to_owned);
    let input_search_query = search_query.clone().unwrap_or_default();

    if let Some(model) = model {
        if model.search_query != search_query {
            model.input_search_query = input_search_query;
            model.search_query = search_query;
            orders.send_msg(Msg::Search);
        }
    } else {
        *model = Some(Model {
            base_url,
            input_search_query,
            search_query,
            debounced_search_query_change: None,
            video_groups: Vec::new(),
            search_results: Vec::new(),
        });
        orders.perform_cmd(async { 
            Msg::VideosReceived(get_videos().await.unwrap()) 
        });
    }
    Some(PageId::Search)
}

async fn get_videos() -> Result<Vec<Video>, FetchError> {
    fetch("/data/cinemeta_20_000.json")
        .await?
        .check_status()?
        .json::<Vec<Video>>()
        .await
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    base_url: Url,
    input_search_query: String,
    search_query: Option<String>,
    debounced_search_query_change: Option<CmdHandle>,
    video_groups: Vec<VideoGroup>,
    search_results: Vec<VideoGroupResults>,
}

struct VideoGroup {
    label: String,
    videos: LocalSearch<Video>,
}

#[derive(Debug)]
struct VideoGroupResults {
    label: String,
    videos: Vec<Video>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Video {
    id: String,
    name: String,
    poster: String,
    #[serde(rename = "type")]
    type_: VideoType,
    imdb_rating: f64,
    popularity: f64,
}

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum VideoType {
    Movie,
    Series,
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    pub fn root(self) -> Url {
        self.base_url()
    }
    pub fn query(self, query: &str) -> Url {
        self.base_url().add_hash_path_part(query)
    }
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    SearchQueryInputChanged(String),
    UpdateSearchQuery,
    VideosReceived(Vec<Video>),
    Search,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SearchQueryInputChanged(query) => {
            model.input_search_query = query;
            model.debounced_search_query_change = Some(
                orders.perform_cmd_with_handle(cmds::timeout(search_debounce_time, || Msg::UpdateSearchQuery))
            );
        },
        Msg::UpdateSearchQuery => {
            orders.request_url(Urls::new(&model.base_url).query(&model.input_search_query));
        }
        Msg::VideosReceived(videos) => {
            let mut cinemeta_top_movie = Vec::new();
            let mut cinemeta_top_series = Vec::new();

            for video in videos {
                match video.type_ {
                    VideoType::Movie => cinemeta_top_movie.push(video),  
                    VideoType::Series => cinemeta_top_series.push(video),
                }
            }
            model.video_groups = vec![
                VideoGroup {
                    label: "Cinemeta - top movie".to_owned(),
                    videos: index(cinemeta_top_movie),
                },
                VideoGroup {
                    label: "Cinemeta - top series".to_owned(),
                    videos: index(cinemeta_top_series),
                },
            ];
            orders.send_msg(Msg::Search);
        }
        Msg::Search => {
            let mut search_results = Vec::new();
            if let Some(search_query) = &model.search_query {
                for group in &model.video_groups {

                    let group_results = group
                        .videos
                        .search(search_query, 10)
                        .into_iter()
                        .map(|(video, _)| video.clone())
                        .collect::<Vec<_>>();

                    if !group_results.is_empty() {
                        search_results.push(VideoGroupResults {
                            label: group.label.clone(),
                            videos: group_results,
                        });
                    }
                }
            }
            model.search_results = search_results;
            log!("SEARCH!", model.search_results.len());
        }
    }
}

fn index(videos: Vec<Video>) -> LocalSearch<Video> {
    let max_imdb_rating = 10.;
    let imdb_rating_weight = 1.0;
    let popularity_weight = 1.0;
    let score_threshold = 0.48;

    let max_popularity = videos
        .iter()
        .map(|video| video.popularity)
        .max_by(|popularity_a, popularity_b| popularity_a.partial_cmp(popularity_b).unwrap())
        .unwrap_or_default();

    let boost_computer = move |video: &Video| {
        let imdb_rating_boost = (video.imdb_rating / max_imdb_rating * imdb_rating_weight).exp();
        let popularity_boost = (video.popularity / max_popularity * popularity_weight).exp();
        imdb_rating_boost * popularity_boost
    };

    LocalSearch::builder(videos, |video: &Video| &video.name)
        .boost_computer(boost_computer)
        .score_threshold(score_threshold)
        .build()
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model, context: &Context ) -> Node<Msg> {
    div![
        C!["route-content"],
        s()
            .position(CssPosition::Absolute)
            .bottom("0")
            .left("0")
            .right("0")
            .top("0")
            .overflow(CssOverflow::Hidden)
            .z_index("0"),
        div![
            C!["search-container", "main-nav-bars-container"],
            s()
                .background_color(Color::BackgroundDark2)
                .height(pc(100))
                .width(pc(100))
                .position(CssPosition::Relative)
                .z_index("0"),
            horizontal_nav_bar(&context.root_base_url, &model.input_search_query),
            vertical_nav_bar(&context.root_base_url),
            nav_content_container(&model.search_results),
        ]
    ]
}

fn horizontal_nav_bar(root_base_url: &Url, input_search_query: &str) -> Node<Msg> {
    nav![
        C!["horizontal-nav-bar", "horizontal-nav-bar-container"],
        s()
            .left("0")
            .position(CssPosition::Absolute)
            .right("0")
            .top("0")
            .z_index("0")
            .align_items(CssAlignItems::Center)
            .background_color(Color::Background)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .height(global::HORIZONTAL_NAV_BAR_SIZE)
            .overflow(CssOverflow::Visible)
            .padding_right(rem(1)),
        logo_container(),
        spacer(None),
        search_bar(input_search_query),
        spacer(Some("11rem")),
        addons_top_button(root_base_url),
        fullscreen_button(),
        menu_button(),
    ]
}

fn logo_container() -> Node<Msg> {
    div![
        C!["logo-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex(CssFlex::None)
            .height(global::HORIZONTAL_NAV_BAR_SIZE)
            .justify_content(CssJustifyContent::Center)
            .width(global::VERTICAL_NAV_BAR_SIZE),
        logo(),
    ]
}

fn logo() -> Node<Msg> {
    img![
        C!["logo"],
        s()
            .flex(CssFlex::None)
            .height(rem(2.5))
            .object_fit("contain")
            .opacity("0.9")
            .width(rem(2.5)),
        attrs!{
            At::Src => global::image_url("stremio_symbol.png"),
        }
    ]
}

fn spacer(max_width: Option<&str>) -> Node<Msg> {
    div![
        C!["spacing"],
        s()
            .flex("1 0 0"),
        max_width.map(|max_width| {
            s()
                .max_width(max_width)
        }),
    ]
}

fn search_bar(input_search_query: &str) -> Node<Msg> {
    label![
        C!["search-bar", "search-bar-container"],
        s()
            .flex("2 0 9.5rem")
            .max_width(rem(30))
            .background_color(Color::BackgroundLight2)
            .border_radius(global::SEARCH_BAR_SIZE)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .height(global::SEARCH_BAR_SIZE),
        s()
            .hover()
            .background_color(Color::BackgroundLight3),
        search_input(input_search_query),
        search_button(),
    ]
}

fn search_input(input_search_query: &str) -> Node<Msg> {
    input![
        C!["search-input", "text-input"],
        s()
            .style_other("::placeholder")
            .color(Color::SecondaryVariant1Light1_90)
            .max_height(em(1.2))
            .opacity("1"),
        s()
            .user_select("text")
            .align_items(CssAlignItems::Center)
            .align_self(CssAlignSelf::Stretch)
            .color(Color::SecondaryVariant1Light1_90)
            .display(CssDisplay::Flex)
            .flex("1")
            .flex_direction(CssFlexDirection::Row)
            .font_weight("500")
            .padding("0 0.5rem 0 1.5rem"),
        attrs!{
            At::from("autocorrect") => "off",
            At::from("autocapitalize") => "off",
            At::AutoComplete => "off",
            At::SpellCheck => "false",
            At::TabIndex => -1,
            At::Type => "text",
            At::Placeholder => "Search or paste link",
            At::Value => input_search_query,
        },
        input_ev(Ev::Input, Msg::SearchQueryInputChanged),
    ]
}

fn search_button() -> Node<Msg> {
    div![
        C!["submit-button-container", "button-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex(CssFlex::None)
            .flex_direction(CssFlexDirection::Row)
            .height(global::SEARCH_BAR_SIZE)
            .justify_content(CssJustifyContent::Center)
            .width(global::SEARCH_BAR_SIZE)
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => -1,
        },
        ev(Ev::Click, |_| Msg::Search),
        search_icon(),
    ]
}

fn search_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .overflow(CssOverflow::Visible)
            .fill(Color::SecondaryVariant1_90)
            .flex(CssFlex::None)
            .height(rem(1.7))
            .width(rem(1.7)),
        attrs!{
            At::ViewBox => "0 0 1443 1024",
            At::from("icon") => "ic_search_link",
        },
        path![
            attrs!{
                At::D => "M1033.035 774.927h-105.111c-0.013 0-0.027 0-0.042 0-10.802 0-21.14-1.988-30.667-5.619l0.591 0.198c-15.423-5.707-27.932-16.268-35.965-29.798l-0.176-0.32c-2.484-3.967-4.719-8.539-6.464-13.345l-0.162-0.509c-3.048-7.589-4.817-16.388-4.819-25.599l-0-0.001c0.67-42.233 35.063-76.212 77.393-76.212 0.533 0 1.064 0.005 1.594 0.016l-0.079-0.001h144.264c0.863-0.033 1.877-0.052 2.896-0.052 7.433 0 14.63 1.008 21.462 2.896l-0.565-0.133c11.866 3.986 21.976 10.503 30.094 18.95l0.023 0.024c13.553 13.793 21.92 32.721 21.92 53.602 0 3.187-0.195 6.328-0.573 9.412l0.037-0.37c-0.198 1.162-0.312 2.5-0.312 3.864 0 6.594 2.649 12.569 6.94 16.92l-0.003-0.003c3.716 3.783 8.767 6.245 14.389 6.622l0.068 0.004c0.278 0.011 0.605 0.018 0.932 0.018 13.056 0 23.716-10.256 24.364-23.151l0.002-0.058c0.649-4.698 1.020-10.125 1.020-15.64 0-33.301-13.512-63.447-35.352-85.253l-0.001-0.001c-21.066-21.097-50.071-34.263-82.15-34.635l-0.071-0.001c-52.104 0-103.906 0-156.009 0-49.554 2.528-91.243 33.695-109.027 77.175l-0.3 0.83c-2.498 6.628-4.885 14.795-6.704 23.173l-0.223 1.222c-2.090 8.002-3.29 17.188-3.29 26.654s1.2 18.652 3.456 27.414l-0.166-0.76c-0.065 0.722-0.103 1.561-0.103 2.409s0.037 1.688 0.11 2.517l-0.008-0.107c0 2.711 2.108 5.722 3.313 8.433 0.933 2.58 1.948 4.765 3.126 6.846l-0.115-0.22 3.614 7.228c1.752 3.103 3.546 5.761 5.523 8.266l-0.102-0.134c1.236 2.097 2.429 3.867 3.716 5.561l-0.102-0.14c3.598 4.93 7.154 9.25 10.937 13.356l-0.094-0.104c0.859 1.159 1.853 2.153 2.974 2.985l0.038 0.027c18.807 19.502 44.944 31.827 73.961 32.525l0.129 0.002c40.056 1.506 80.113 0 120.471 0 0.263 0.011 0.571 0.017 0.881 0.017 9.895 0 18.303-6.362 21.359-15.218l0.048-0.159c1.655-2.99 2.629-6.556 2.629-10.35 0-4.964-1.668-9.539-4.474-13.194l0.038 0.051c-4.974-5.048-11.885-8.176-19.527-8.176-0.547 0-1.090 0.016-1.63 0.048l0.074-0.003z",
            },
        ],
        path![
            attrs!{
                At::D => "M1407.398 611.689l-3.012-3.012c-17.962-18.55-42.498-30.641-69.842-32.509l-0.332-0.018c-19.576-1.506-39.454 0-60.235 0s-42.767 0-64.151 0c-0.38-0.022-0.825-0.035-1.273-0.035-9.786 0-18.157 6.062-21.562 14.636l-0.055 0.157c-1.435 2.772-2.276 6.052-2.276 9.528 0 5.366 2.005 10.264 5.307 13.986l-0.019-0.022 1.506 1.807c5.195 4.38 11.964 7.042 19.355 7.042 0.926 0 1.843-0.042 2.748-0.124l-0.117 0.009h104.508c0.17-0.001 0.37-0.002 0.571-0.002 21.491 0 40.967 8.624 55.157 22.6l-0.010-0.010c13.214 13.239 21.385 31.515 21.385 51.699 0 0.142-0 0.284-0.001 0.426l0-0.022c-0.842 42.098-35.167 75.902-77.388 75.902-0.323 0-0.645-0.002-0.967-0.006l0.049 0h-145.468c-0.821 0.030-1.785 0.047-2.754 0.047-7.045 0-13.88-0.896-20.399-2.58l0.565 0.124c-12.291-3.615-22.831-9.967-31.328-18.378l0.006 0.006c-13.459-13.864-21.756-32.803-21.756-53.68 0-3.586 0.245-7.115 0.719-10.571l-0.045 0.401c0.377-1.787 0.592-3.84 0.592-5.943 0-6.983-2.376-13.411-6.365-18.519l0.050 0.067c-1.77-2.045-3.862-3.753-6.208-5.060l-0.116-0.060c-16.264-6.626-30.118 3.614-33.129 23.793-0.783 5.16-1.23 11.115-1.23 17.173 0 66.534 53.937 120.471 120.471 120.471 0.433 0 0.865-0.002 1.296-0.007l-0.066 0.001c49.995 0 99.991 0 150.588 0 50.623-0.695 93.946-31.236 113.227-74.793l0.317-0.802c6.184-13.844 9.785-30.001 9.785-46.998 0-34.274-14.642-65.128-38.013-86.649l-0.083-0.075z",
            },
        ],
        path![
            attrs!{
                At::D => "M992.075 865.882c-25.6 0-51.802 0-78.005 0-40.714-1.196-77.196-18.374-103.573-45.445l-0.031-0.032-3.614-3.915c-28.592-29.766-46.199-70.27-46.199-114.887 0-60.965 32.875-114.252 81.865-143.1l0.777-0.423c12.528-38.704 19.791-83.241 19.878-129.462l0-0.044c-1.371-237.151-193.936-428.869-431.278-428.869-238.192 0-431.285 193.093-431.285 431.285 0 237.342 191.718 429.907 428.738 431.277l0.131 0.001c0.118 0 0.258 0 0.397 0 88.033 0 169.923-26.302 238.24-71.477l-1.612 1.002 200.885 202.089c13.51 18.524 35.139 30.425 59.548 30.425 2.363 0 4.699-0.112 7.005-0.33l-0.295 0.023c1.429 0.081 3.101 0.127 4.784 0.127 35.359 0 65.974-20.311 80.814-49.902l0.237-0.521c7.55-11.025 12.058-24.651 12.058-39.33 0-20.085-8.438-38.2-21.963-50.992l-0.033-0.031zM433.694 736.376c-166.335 0-301.176-134.841-301.176-301.176v0-7.529c1.449-166.068 136.41-300.133 302.682-300.133 167.173 0 302.693 135.52 302.693 302.693 0 0.9-0.004 1.799-0.012 2.698l0.001-0.138c-1.855 167.126-137.013 302.072-304.044 303.585l-0.144 0.001z",
            },
        ],
    ]
}

fn addons_top_button(root_base_url: &Url) -> Node<Msg> {
    a![
        C!["button-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex(CssFlex::None)
            .height(global::HORIZONTAL_NAV_BAR_SIZE)
            .justify_content(CssJustifyContent::Center)
            .width(global::HORIZONTAL_NAV_BAR_SIZE)
            .cursor(CssCursor::Pointer),
        s()
            .hover()
            .background_color(Color::BackgroundLight2),
        attrs!{
            At::TabIndex => -1,
            At::Title => "Addons",
            At::Href => RootUrls::new(root_base_url).addons_urls().root(),
        },
        addons_top_icon(),
    ]
}

fn addons_top_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .overflow(CssOverflow::Visible)
            .fill(Color::SecondaryVariant2Light1_90)
            .flex(CssFlex::None)
            .height(rem(1.7))
            .width(rem(1.7)),
        attrs!{
            At::ViewBox => "0 0 1043 1024",
            At::from("icon") => "ic_addons",
        },
        path![
            attrs!{
                At::D => "M145.468 679.454c-40.056-39.454-80.715-78.908-120.471-118.664-33.431-33.129-33.129-60.235 0-90.353l132.216-129.807c5.693-5.938 12.009-11.201 18.865-15.709l0.411-0.253c23.492-15.059 41.864-7.529 48.188 18.974 0 7.228 2.711 14.758 3.614 22.287 3.801 47.788 37.399 86.785 82.050 98.612l0.773 0.174c10.296 3.123 22.128 4.92 34.381 4.92 36.485 0 69.247-15.94 91.702-41.236l0.11-0.126c24.858-21.654 40.48-53.361 40.48-88.718 0-13.746-2.361-26.941-6.701-39.201l0.254 0.822c-14.354-43.689-53.204-75.339-99.907-78.885l-0.385-0.023c-18.372-2.409-41.562 0-48.188-23.492s11.445-34.635 24.998-47.887q65.054-62.946 130.409-126.795c32.527-31.925 60.235-32.226 90.353 0 40.659 39.153 80.715 78.908 120.471 118.362 8.348 8.594 17.297 16.493 26.82 23.671l0.587 0.424c8.609 7.946 20.158 12.819 32.846 12.819 24.823 0 45.29-18.653 48.148-42.707l0.022-0.229c3.012-13.252 4.518-26.805 8.734-39.755 12.103-42.212 50.358-72.582 95.705-72.582 3.844 0 7.637 0.218 11.368 0.643l-0.456-0.042c54.982 6.832 98.119 49.867 105.048 104.211l0.062 0.598c0.139 1.948 0.218 4.221 0.218 6.512 0 45.084-30.574 83.026-72.118 94.226l-0.683 0.157c-12.348 3.915-25.299 5.722-37.948 8.433-45.779 9.638-60.235 46.984-30.118 82.824 15.265 17.569 30.806 33.587 47.177 48.718l0.409 0.373c31.925 31.925 64.452 62.946 96.075 94.871 13.698 9.715 22.53 25.511 22.53 43.369s-8.832 33.655-22.366 43.259l-0.164 0.111c-45.176 45.176-90.353 90.353-137.035 134.325-5.672 5.996-12.106 11.184-19.169 15.434l-0.408 0.227c-4.663 3.903-10.725 6.273-17.341 6.273-13.891 0-25.341-10.449-26.92-23.915l-0.012-0.127c-2.019-7.447-3.714-16.45-4.742-25.655l-0.077-0.848c-4.119-47.717-38.088-86.476-82.967-97.721l-0.76-0.161c-9.584-2.63-20.589-4.141-31.947-4.141-39.149 0-74.105 17.956-97.080 46.081l-0.178 0.225c-21.801 21.801-35.285 51.918-35.285 85.185 0 1.182 0.017 2.36 0.051 3.533l-0.004-0.172c1.534 53.671 40.587 97.786 91.776 107.115l0.685 0.104c12.649 2.409 25.901 3.313 38.249 6.626 22.588 6.325 30.118 21.685 18.372 41.864-4.976 8.015-10.653 14.937-17.116 21.035l-0.051 0.047c-44.875 44.574-90.353 90.353-135.228 133.12-10.241 14.067-26.653 23.106-45.176 23.106s-34.935-9.039-45.066-22.946l-0.111-0.159c-40.659-38.852-80.414-78.908-120.471-118.362z",
            }
        ]
    ]
}

fn fullscreen_button() -> Node<Msg> {
    div![
        C!["button-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex(CssFlex::None)
            .height(global::HORIZONTAL_NAV_BAR_SIZE)
            .justify_content(CssJustifyContent::Center)
            .width(global::HORIZONTAL_NAV_BAR_SIZE)
            .cursor(CssCursor::Pointer),
        s()
            .hover()
            .background_color(Color::BackgroundLight2),
        attrs!{
            At::TabIndex => -1,
            At::Title => "Enter Fullscreen",
        },
        fullscreen_icon(),
        on_click_not_implemented(),
    ]
}

fn fullscreen_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .overflow(CssOverflow::Visible)
            .fill(Color::SecondaryVariant2Light1_90)
            .flex(CssFlex::None)
            .height(rem(1.7))
            .width(rem(1.7)),
        attrs!{
            At::ViewBox => "0 0 1016 1024",
            At::from("icon") => "ic_fullscreen",
        },
        path![
            attrs!{
                At::D => "M379.784 1.506l-316.235-1.506c-17.58 0.003-33.524 7.011-45.19 18.385l0.014-0.013c-11.345 11.55-18.354 27.393-18.372 44.872l-0 0.003 1.506 316.838c0.663 34.993 28.856 63.187 63.787 63.848l0.063 0.001c0.090 0 0.196 0.001 0.302 0.001 34.492 0 62.473-27.876 62.644-62.328l0-0.016v-253.591h252.386c0.271 0.004 0.59 0.007 0.91 0.007 34.598 0 62.645-28.047 62.645-62.645 0-0.32-0.002-0.639-0.007-0.958l0.001 0.048c-1.004-34.88-29.443-62.792-64.437-62.946l-0.015-0z",
            },
        ],
        path![
            attrs!{
                At::D => "M633.976 128.904h254.494v252.386c-0.004 0.269-0.007 0.586-0.007 0.904 0 34.598 28.047 62.645 62.645 62.645 0.002 0 0.005-0 0.007-0l-0 0c35.122-0.497 63.483-28.753 64.15-63.787l0.001-0.063v-316.838c0.019-0.581 0.030-1.264 0.030-1.95 0-16.946-6.54-32.364-17.233-43.869l0.037 0.040c-11.448-11.329-27.189-18.338-44.568-18.372l-0.007-0-317.139 1.506c-35.189 0.334-63.646 28.686-64.15 63.802l-0.001 0.048c-0.004 0.271-0.007 0.59-0.007 0.91 0 34.282 27.538 62.133 61.7 62.638l0.048 0.001z",
            },
        ],
        path![
            attrs!{
                At::D => "M380.386 895.096h-252.386v-252.386c0.005-0.282 0.007-0.616 0.007-0.95 0-33.753-26.694-61.271-60.122-62.595l-0.12-0.004c-0.448-0.011-0.976-0.018-1.506-0.018-35.762 0-64.753 28.991-64.753 64.753 0 0.006 0 0.012 0 0.018l-0-0.001-1.506 316.838c-0.002 0.18-0.003 0.392-0.003 0.605 0 34.387 27.706 62.303 62.013 62.642l0.032 0h317.139c35.189-0.334 63.646-28.686 64.15-63.802l0.001-0.048c-0.142-35.138-27.992-63.725-62.825-65.050l-0.121-0.004z",
            },
        ],
        path![
            attrs!{
                At::D => "M950.814 580.066c-0.002-0-0.004-0-0.007-0-34.598 0-62.645 28.047-62.645 62.645 0 0.318 0.002 0.635 0.007 0.951l-0.001-0.048v252.386h-252.687c-0.18-0.002-0.392-0.003-0.605-0.003-34.387 0-62.303 27.706-62.642 62.013l-0 0.032c-0.007 0.359-0.011 0.783-0.011 1.207 0 35.554 28.655 64.416 64.13 64.75l0.032 0h316.536c17.385-0.034 33.126-7.043 44.58-18.377l-0.005 0.005c11.345-11.55 18.354-27.393 18.372-44.872l0-0.003v-316.838c-0.677-35.406-29.538-63.849-65.043-63.849-0.004 0-0.008 0-0.012 0l0.001-0z",
            },
        ],
    ]
}

fn menu_button() -> Node<Msg> {
    label![
        C!["button-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex(CssFlex::None)
            .height(global::HORIZONTAL_NAV_BAR_SIZE)
            .justify_content(CssJustifyContent::Center)
            .width(global::HORIZONTAL_NAV_BAR_SIZE)
            .cursor(CssCursor::Pointer)
            .overflow(CssOverflow::Visible)
            .position(CssPosition::Relative),
        s()
            .hover()
            .background_color(Color::BackgroundLight2),
        attrs!{
            At::TabIndex => -1,
        },
        menu_icon(),
        on_click_not_implemented(),
    ]
}

fn menu_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .overflow(CssOverflow::Visible)
            .fill(Color::SecondaryVariant2Light1_90)
            .flex(CssFlex::None)
            .height(rem(1.7))
            .width(rem(1.7)),
        attrs!{
            At::ViewBox => "0 0 216 1024",
            At::from("icon") => "ic_more",
        },
        path![
            attrs!{
                At::D => "M215.944 108.122c0-0.089 0-0.195 0-0.301 0-59.714-48.408-108.122-108.122-108.122s-108.122 48.408-108.122 108.122c0 59.714 48.408 108.122 108.122 108.122 0.106 0 0.211-0 0.317-0l-0.016 0c59.548 0 107.821-48.273 107.821-107.821v0z",
            },
        ],
        path![
            attrs!{
                At::D => "M215.944 507.181c-0-59.714-48.408-108.122-108.122-108.122s-108.122 48.408-108.122 108.122c0 59.714 48.408 108.122 108.122 108.122 0.106 0 0.212-0 0.318-0l-0.016 0c0 0 0 0 0 0 59.548 0 107.821-48.273 107.821-107.821 0-0.106-0-0.212-0-0.318l0 0.017z",
            },
        ],
        path![
            attrs!{
                At::D => "M215.944 915.878c-0-59.714-48.408-108.122-108.122-108.122s-108.122 48.408-108.122 108.122c0 59.714 48.408 108.122 108.122 108.122 0.106 0 0.212-0 0.318-0l-0.016 0c0 0 0 0 0 0 59.548 0 107.821-48.273 107.821-107.821 0-0.106-0-0.212-0-0.318l0 0.017z",
            },
        ],
    ]
}

fn vertical_nav_bar(root_base_url: &Url) -> Node<Msg> {
    nav![
        C!["vertical-nav-bar", "vertical-nav-bar-container"],
        s()
            .bottom("0")
            .left("0")
            .position(CssPosition::Absolute)
            .top(global::HORIZONTAL_NAV_BAR_SIZE)
            .z_index("1")
            .background_color(Color::BackgroundDark1)
            .overflow_y(CssOverflowY::Auto)
            .raw("scrollbar-width: none;")
            .width(global::VERTICAL_NAV_BAR_SIZE),
        vertical_nav_buttons(root_base_url),
    ]
}

fn vertical_nav_buttons(root_base_url: &Url) -> Vec<Node<Msg>> {
    vec![
        vertical_nav_button(
            "Board", 
            Some(RootUrls::new(root_base_url).board().to_string()), 
            true, 
            vertical_nav_icon("ic_board", "0 0 1395 1024", vec![
                path![attrs!{At::D => "M1308.009 174.381l-1220.668 0c-48.237 0-87.341-39.104-87.341-87.341v0.301c0-48.237 39.104-87.341 87.341-87.341l1220.668-0c48.237 0 87.341 39.104 87.341 87.341v-0.301c0 48.237-39.104 87.341-87.341 87.341z"}],
                path![attrs!{At::D => "M936.358 599.341l-849.016 0c-48.237 0-87.341-39.104-87.341-87.341v0.301c0-48.237 39.104-87.341 87.341-87.341l849.016-0c48.237 0 87.341 39.104 87.341 87.341v-0.301c0 48.237-39.104 87.341-87.341 87.341z"}],
                path![attrs!{At::D => "M1308.009 1024h-1220.668c-48.237-0-87.341-39.104-87.341-87.341v0.301c0-48.237 39.104-87.341 87.341-87.341l1220.668 0c48.237 0 87.341 39.104 87.341 87.341v-0.301c0 48.237-39.104 87.341-87.341 87.341z"}],
            ])
        ),
        vertical_nav_button(
            "Discover", 
            Some(RootUrls::new(root_base_url).discover_urls().root().to_string()), 
            false, 
            vertical_nav_icon("ic_discover", "0 0 1025 1024", vec![
                path![attrs!{At::D => "M602.353 575.849c49.694-96.075 99.991-192.151 150.588-288.226 3.012-6.024 10.842-13.252 4.819-19.275s-13.553 0-19.275 4.216l-291.84 150.588c-10.241 5.534-18.27 14.048-23.055 24.371l-0.135 0.326q-64.753 124.386-129.506 248.471c-8.734 16.866-17.468 33.129-25.901 49.995-2.711 4.819-6.024 11.445 4.518 12.951 4.819-2.108 10.24-4.216 15.36-6.927l289.732-150.588c10.78-5.894 19.287-14.788 24.546-25.559l0.151-0.342z"}],
                path![attrs!{At::D => "M883.351 161.732c-90.543-95.747-216.891-156.82-357.52-161.708l-0.88-0.024c-3.070-0.066-6.687-0.104-10.314-0.104-138.198 0-263.562 54.947-355.438 144.186l0.123-0.119c-98.26 92.852-159.424 224.071-159.424 369.575 0 142.717 58.843 271.691 153.591 363.984l0.111 0.107c88.622 88.958 210.672 144.561 345.709 146.368l0.343 0.004h24.094c277.633-5.364 500.641-231.69 500.641-510.104 0-136.661-53.732-260.772-141.221-352.36l0.185 0.195zM242.748 783.059c-70.126-69.135-113.568-165.177-113.568-271.364 0-210.414 170.574-380.988 380.988-380.988 0.644 0 1.288 0.002 1.931 0.005l-0.099-0c210.913 0 381.892 170.979 381.892 381.892s-170.979 381.892-381.892 381.892v0 0c-0.446 0.002-0.975 0.003-1.503 0.003-104.66 0-199.368-42.605-267.728-111.418l-0.020-0.021z"}],
            ])
        ),
        vertical_nav_button(
            "Library", 
            None, 
            false, 
            vertical_nav_icon("ic_library", "0 0 1209 1024", vec![
                path![attrs!{At::D => "M1204.706 917.082l-190.645-826.729c-9.055-39.42-43.838-68.374-85.384-68.374-48.324 0-87.499 39.175-87.499 87.499 0 6.779 0.771 13.378 2.23 19.714l-0.114-0.589 191.548 827.633c11.135 36.317 44.369 62.266 83.664 62.266 48.237 0 87.341-39.104 87.341-87.341 0-4.971-0.415-9.846-1.213-14.591l0.071 0.513z"}],
                path![attrs!{At::D => "M674.334 0c-0-0-0-0-0.001-0-48.071 0-87.040 38.969-87.040 87.040 0 0.106 0 0.212 0.001 0.318l-0-0.016v849.318c-0.096 1.532-0.151 3.323-0.151 5.127 0 48.237 39.104 87.341 87.341 87.341s87.341-39.104 87.341-87.341c0-1.804-0.055-3.594-0.162-5.371l0.012 0.244v-849.318c0-48.237-39.104-87.341-87.341-87.341v0z"}],
                path![attrs!{At::D => "M87.944 0c-0.179-0.001-0.391-0.002-0.602-0.002-48.237 0-87.341 39.104-87.341 87.341 0 0.001 0 0.002 0 0.002l-0-0v849.318c-0.096 1.532-0.151 3.323-0.151 5.127 0 48.237 39.104 87.341 87.341 87.341s87.341-39.104 87.341-87.341c0-1.804-0.055-3.594-0.162-5.371l0.012 0.244v-849.318c0-0.090 0.001-0.197 0.001-0.303 0-47.859-38.627-86.697-86.406-87.038l-0.032-0z"}],
                path![attrs!{At::D => "M380.988 171.369c-48.002 0.171-86.869 39.038-87.040 87.024l-0 0.016v678.249c-0.096 1.532-0.151 3.323-0.151 5.127 0 48.237 39.104 87.341 87.341 87.341s87.341-39.104 87.341-87.341c0-1.804-0.055-3.594-0.162-5.371l0.012 0.244v-678.249c-0.171-48.108-39.209-87.040-87.341-87.040-0 0-0 0-0.001 0l0-0z"}],
            ])
        ),
        vertical_nav_button(
            "Settings", 
            None, 
            false, 
            vertical_nav_icon("ic_settings", "0 0 1043 1024", vec![
                path![attrs!{At::D => "M791.492 901.421c-0.137 1.886-0.214 4.085-0.214 6.303 0 14.689 3.414 28.58 9.492 40.924l-0.242-0.544c1.442 2.027 2.306 4.553 2.306 7.281 0 5.548-3.572 10.262-8.542 11.967l-0.089 0.027c-37.735 21.585-81.411 40.158-127.33 53.451l-4.284 1.062c-2.114 1.002-4.593 1.587-7.209 1.587-7.903 0-14.559-5.341-16.556-12.61l-0.028-0.12c-20.88-43.535-64.606-73.060-115.229-73.060-26.819 0-51.703 8.287-72.23 22.44l0.428-0.279c-19.628 13.227-34.808 31.704-43.688 53.426l-0.284 0.786c-3.614 8.734-7.529 11.746-17.769 9.035-51.834-13.272-97.233-31.525-139.449-54.835l3.016 1.527c-14.758-7.831-8.734-16.866-5.12-26.805 4.846-12.398 7.654-26.752 7.654-41.762 0-32.050-12.804-61.11-33.576-82.344l0.021 0.021c-22.874-25.484-55.92-41.441-92.693-41.441-10.83 0-21.336 1.384-31.352 3.985l0.864-0.191h-5.722c-30.118 9.336-30.118 9.035-44.273-18.372-17.236-31.193-32.683-67.512-44.377-105.477l-1.101-4.152c-3.915-12.348-1.807-18.673 11.445-24.094 45.171-18.059 76.501-61.451 76.501-112.16 0-0.275-0.001-0.549-0.003-0.823l0 0.042c-0.157-51.84-32.003-96.203-77.176-114.748l-0.829-0.301c-13.553-4.819-15.962-10.842-12.047-23.793 13.962-48.504 31.914-90.674 54.24-130.036l-1.534 2.94c6.024-10.541 11.746-12.649 23.793-7.831 14.648 6.459 31.727 10.219 49.685 10.219 35.285 0 67.18-14.517 90.038-37.904l0.023-0.024c21.532-21.755 34.835-51.691 34.835-84.733 0-19.022-4.409-37.015-12.26-53.011l0.314 0.709c-4.216-9.638-3.012-15.059 6.024-20.48 39.702-23.013 85.609-42.536 133.977-56.195l4.263-1.029c13.252-3.614 14.758 5.12 18.372 13.252 16.261 41.325 53.282 71.221 97.87 77.036l0.614 0.065c6.241 1.121 13.425 1.762 20.759 1.762 40.852 0 77.059-19.886 99.469-50.507l0.242-0.347c7.452-9.232 13.404-20.047 17.264-31.809l0.204-0.718c3.012-8.433 8.132-9.939 16.264-8.132 52.584 13.65 98.681 32.83 141.232 57.456l-2.691-1.437c9.336 5.12 8.433 11.144 4.819 19.576-6.604 14.774-10.451 32.016-10.451 50.158 0 69.362 56.229 125.591 125.591 125.591 18.623 0 36.299-4.053 52.195-11.326l-0.784 0.321c10.24-4.518 15.962-3.012 21.384 6.927 22.212 37.657 40.917 81.17 53.87 127.095l0.944 3.916c2.711 10.24 0 15.36-10.24 19.878-46.208 16.823-78.61 60.371-78.61 111.487 0 0.299 0.001 0.599 0.003 0.898l-0-0.046c-0.106 1.871-0.166 4.060-0.166 6.264 0 49.766 30.792 92.34 74.362 109.71l0.797 0.28c12.951 6.024 16.264 11.746 12.047 25.6-14.446 47.781-32.562 89.199-54.858 127.907l1.55-2.918c-5.421 10.24-10.842 12.348-22.287 8.132-14.209-5.966-30.724-9.432-48.048-9.432-45.354 0-85.159 23.756-107.651 59.503l-0.31 0.527c-11.029 16.816-17.591 37.422-17.591 59.561 0 1.826 0.045 3.642 0.133 5.446l-0.010-0.254zM520.433 711.68c109.44-1.529 197.571-90.604 197.571-200.264 0-110.613-89.669-200.282-200.282-200.282s-200.282 89.669-200.282 200.282c0 0.205 0 0.411 0.001 0.616l-0-0.032c0.498 110.402 90.11 199.707 200.582 199.707 1.166 0 2.329-0.010 3.49-0.030l-0.175 0.002z"}],
            ])
        ),
        vertical_nav_button(
            "Addons", 
            Some(RootUrls::new(root_base_url).addons_urls().root().to_string()), 
            false, 
            vertical_nav_icon("ic_addons", "0 0 1043 1024", vec![
                path![attrs!{At::D => "M145.468 679.454c-40.056-39.454-80.715-78.908-120.471-118.664-33.431-33.129-33.129-60.235 0-90.353l132.216-129.807c5.693-5.938 12.009-11.201 18.865-15.709l0.411-0.253c23.492-15.059 41.864-7.529 48.188 18.974 0 7.228 2.711 14.758 3.614 22.287 3.801 47.788 37.399 86.785 82.050 98.612l0.773 0.174c10.296 3.123 22.128 4.92 34.381 4.92 36.485 0 69.247-15.94 91.702-41.236l0.11-0.126c24.858-21.654 40.48-53.361 40.48-88.718 0-13.746-2.361-26.941-6.701-39.201l0.254 0.822c-14.354-43.689-53.204-75.339-99.907-78.885l-0.385-0.023c-18.372-2.409-41.562 0-48.188-23.492s11.445-34.635 24.998-47.887q65.054-62.946 130.409-126.795c32.527-31.925 60.235-32.226 90.353 0 40.659 39.153 80.715 78.908 120.471 118.362 8.348 8.594 17.297 16.493 26.82 23.671l0.587 0.424c8.609 7.946 20.158 12.819 32.846 12.819 24.823 0 45.29-18.653 48.148-42.707l0.022-0.229c3.012-13.252 4.518-26.805 8.734-39.755 12.103-42.212 50.358-72.582 95.705-72.582 3.844 0 7.637 0.218 11.368 0.643l-0.456-0.042c54.982 6.832 98.119 49.867 105.048 104.211l0.062 0.598c0.139 1.948 0.218 4.221 0.218 6.512 0 45.084-30.574 83.026-72.118 94.226l-0.683 0.157c-12.348 3.915-25.299 5.722-37.948 8.433-45.779 9.638-60.235 46.984-30.118 82.824 15.265 17.569 30.806 33.587 47.177 48.718l0.409 0.373c31.925 31.925 64.452 62.946 96.075 94.871 13.698 9.715 22.53 25.511 22.53 43.369s-8.832 33.655-22.366 43.259l-0.164 0.111c-45.176 45.176-90.353 90.353-137.035 134.325-5.672 5.996-12.106 11.184-19.169 15.434l-0.408 0.227c-4.663 3.903-10.725 6.273-17.341 6.273-13.891 0-25.341-10.449-26.92-23.915l-0.012-0.127c-2.019-7.447-3.714-16.45-4.742-25.655l-0.077-0.848c-4.119-47.717-38.088-86.476-82.967-97.721l-0.76-0.161c-9.584-2.63-20.589-4.141-31.947-4.141-39.149 0-74.105 17.956-97.080 46.081l-0.178 0.225c-21.801 21.801-35.285 51.918-35.285 85.185 0 1.182 0.017 2.36 0.051 3.533l-0.004-0.172c1.534 53.671 40.587 97.786 91.776 107.115l0.685 0.104c12.649 2.409 25.901 3.313 38.249 6.626 22.588 6.325 30.118 21.685 18.372 41.864-4.976 8.015-10.653 14.937-17.116 21.035l-0.051 0.047c-44.875 44.574-90.353 90.353-135.228 133.12-10.241 14.067-26.653 23.106-45.176 23.106s-34.935-9.039-45.066-22.946l-0.111-0.159c-40.659-38.852-80.414-78.908-120.471-118.362z"}],
            ])
        ),
    ]
}

fn vertical_nav_button(title: &str, href: Option<String>, margin_top: bool, icon: Node<Msg>) -> Node<Msg> {
    a![
        C!["nav-tab-button", "nav-tab-button-container"],
        attrs!{
            At::TabIndex => -1,
            At::Title => title,
        },
        IF!(href.is_none() => on_click_not_implemented()),
        href.map(|href| attrs!{At::Href => href}),
        IF!(margin_top => s().margin_top(rem(1))),
        s()
            .height(global::VERTICAL_NAV_BAR_SIZE)
            .width(global::VERTICAL_NAV_BAR_SIZE)
            .align_items(CssAlignItems::Center)
            .background_color(Color::BackgroundDark1)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Column)
            .justify_content(CssJustifyContent::Center)
            .cursor(CssCursor::Pointer),
        s()
            .hover()
            .background_color(Color::BackgroundLight2),
        icon,
        vertical_nav_label(title),
    ]
}

fn vertical_nav_icon(icon: &str, view_box: &str, paths: Vec<Node<Msg>>) -> Node<Msg> {
    svg![
        C!["icon"],
        attrs!{
            At::from("icon") => icon,
            At::ViewBox => view_box,
        },
        s()
            .overflow(CssOverflow::Visible)
            .fill(Color::SecondaryLight5_90)
            .flex(CssFlex::None)
            .height(rem(1.7))
            .margin_bottom(rem(0.5))
            .width(rem(1.7)),
        paths,
    ]
}

fn vertical_nav_label(title: &str) -> Node<Msg> {
    div![
        C!["label"],
        s()
            .color(Color::SecondaryVariant1_90)
            .flex(CssFlex::None)
            .font_size(rem(0.9))
            .font_weight("500")
            .letter_spacing(rem(0.01))
            .max_height(em(2.4))
            .padding("0 0.2rem")
            .text_align(CssTextAlign::Center),
        title,
    ]
}

fn nav_content_container(search_results: &[VideoGroupResults]) -> Node<Msg> {
    div![
        C!["nav-content-container"],
        s()
            .bottom("0")
            .left(global::VERTICAL_NAV_BAR_SIZE)
            .position(CssPosition::Absolute)
            .right("0")
            .top(global::HORIZONTAL_NAV_BAR_SIZE)
            .z_index("0"),
        search_content(search_results),
    ]
}

fn search_content(search_results: &[VideoGroupResults]) -> Node<Msg> {
    div![
        C!["search-content"],
        s()
            .height(pc(100))
            .overflow_y(CssOverflowY::Auto)
            .width(pc(100)),
        if search_results.is_empty() {
            vec![search_hints_container()]
        } else {
            search_rows(search_results)
        }
    ]
}

fn search_hints_container() -> Node<Msg> {
    div![
        C!["search-hints-container"],
        s()
            .align_content(CssAlignContent::FlexStart)
            .align_items(CssAlignItems::FlexStart)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .flex_wrap(CssFlexWrap::Wrap)
            .justify_content(CssJustifyContent::FlexStart)
            .padding(rem(4)),
        search_hint_container(
            "Search for movies, series, YouTube and TV channels",
            "ic_movies",
            "0 0 840 1024",
            vec![
                path![attrs!{At::D => "M813.176 1024h-708.969c-14.3-3.367-24.781-16.017-24.781-31.115 0-0.815 0.031-1.623 0.090-2.422l-0.006 0.107q0-215.642 0-430.984v-4.819c0.015 0 0.033 0 0.051 0 30.976 0 58.991-12.673 79.146-33.116l0.013-0.013c19.218-19.773 31.069-46.796 31.069-76.586 0-1.134-0.017-2.265-0.051-3.391l0.004 0.165h649.939v558.381c-1.037 2.541-2.047 4.621-3.168 6.63l0.157-0.306c-4.8 8.938-13.235 15.394-23.273 17.431l-0.219 0.037zM796.612 481.882h-126.795c-1.944 0.438-3.547 1.646-4.5 3.28l-0.018 0.033-60.235 95.473c-0.466 0.866-0.972 1.957-1.422 3.076l-0.084 0.237h128.301c3.012 0 3.915 0 5.421-3.313l56.922-95.172c0.887-1.056 1.687-2.24 2.356-3.505l0.053-0.11zM393.638 583.078h128.602c0.156 0.017 0.337 0.026 0.52 0.026 2.3 0 4.246-1.517 4.892-3.604l0.010-0.036c18.974-30.118 37.948-62.645 56.621-94.268l2.711-4.518h-125.892c-0.179-0.018-0.387-0.028-0.597-0.028-2.519 0-4.694 1.473-5.711 3.604l-0.016 0.038-58.428 94.268zM377.675 481.882h-126.193c-0.024-0-0.052-0.001-0.080-0.001-2.57 0-4.763 1.609-5.629 3.875l-0.014 0.041-58.428 93.064-2.711 4.216h124.386c0.165 0.018 0.357 0.028 0.551 0.028 2.127 0 3.968-1.225 4.856-3.008l0.014-0.031 60.235-95.473z"}],
                path![attrs!{At::D => "M707.464 0c4.931 1.519 9.225 3.567 13.143 6.142l-0.192-0.119c4.632 3.831 8.386 8.548 11.033 13.909l0.11 0.247c18.372 44.574 36.442 90.353 54.814 134.325l-602.353 243.652c-18.275-41.26-58.864-69.523-106.054-69.523-14.706 0-28.77 2.745-41.71 7.75l0.79-0.269c-4.819-12.047-10.842-24.094-14.758-37.045-0.883-2.705-1.392-5.818-1.392-9.050 0-13.254 8.561-24.508 20.455-28.534l0.212-0.062c18.673-6.626 39.153-14.456 58.428-20.48l542.118-217.751 43.972-19.275 10.24-3.915zM123.181 271.059h1.807l93.064 67.464c0.846 0.357 1.829 0.565 2.861 0.565s2.015-0.208 2.911-0.583l-0.050 0.018 90.353-35.84 26.504-10.842-2.409-1.807-91.859-65.656c-0.846-0.572-1.889-0.914-3.012-0.914s-2.166 0.341-3.031 0.926l0.019-0.012-77.402 30.118zM535.793 214.739l-2.711-2.108-90.353-66.56c-0.933-0.622-2.080-0.993-3.313-0.993s-2.38 0.371-3.335 1.007l0.022-0.014-118.061 45.779 2.108 1.807 92.461 67.162c0.846 0.357 1.829 0.565 2.861 0.565s2.015-0.208 2.911-0.583l-0.050 0.018 87.341-34.635zM730.353 135.529h-1.807l-91.859-68.969c-0.803-0.547-1.794-0.874-2.861-0.874s-2.059 0.327-2.879 0.885l0.018-0.011-90.353 36.744c-8.433 3.012-16.565 6.325-24.998 9.939l2.409 2.108 90.353 65.355c0.846 0.357 1.829 0.565 2.861 0.565s2.015-0.208 2.911-0.583l-0.050 0.018 75.294-30.118z"}],
                path![attrs!{At::D => "M0 433.393c0-3.614 1.506-7.228 2.409-10.541 8.935-34.682 39.932-59.894 76.818-59.894 4.782 0 9.465 0.424 14.014 1.236l-0.48-0.071c37.902 5.909 66.564 38.317 66.564 77.421 0 2.432-0.111 4.839-0.328 7.214l0.023-0.305c-3.944 40.578-37.878 72.037-79.159 72.037-39.144 0-71.681-28.287-78.286-65.534l-0.070-0.48c-0.474-1.046-0.977-1.935-1.547-2.775l0.041 0.064z"}],
            ]
        ),
        search_hint_container(
            "Search for actors, directors and writers",
            "ic_actor",
            "0 0 1085 1024",
            vec![
                path![attrs!{At::D => "M1079.416 397.252c-11.403-64.785-36.251-122.282-71.634-171.727l0.858 1.261c-55.818-86.588-135.669-153.436-230.111-191.866l-3.301-1.188c-51.351-21.358-111-33.763-173.546-33.763-1.882 0-3.762 0.011-5.639 0.034l0.286-0.003c-74.242 1.759-143.267 22.563-202.841 57.688l1.956-1.067c-2.088 1.58-4.728 2.53-7.59 2.53-0.616 0-1.221-0.044-1.814-0.129l0.068 0.008c-16.962-3.079-37.648-5.545-58.648-6.848l-1.588-0.079c-9.62-0.825-20.817-1.296-32.124-1.296-48.771 0-95.497 8.756-138.692 24.781l2.759-0.897c-55.32 21.387-99.741 60.74-127.065 110.769l-0.634 1.268c0 2.409-3.915 5.12-1.807 7.529s5.12 0 7.529 0c20.216-6.76 44.065-12.15 68.646-15.176l1.829-0.184c2.919-0.427 6.289-0.67 9.716-0.67 9.865 0 19.258 2.018 27.79 5.664l-0.462-0.175c1.807 0 4.216 2.108 3.915 4.819s-2.409 2.409-4.216 3.012-11.746 5.12-17.468 8.132c-57.246 31.332-98.926 85.046-113.552 149.064l-0.293 1.524c-6.173 26.883-9.711 57.753-9.711 89.449s3.538 62.567 10.24 92.237l-0.529-2.788c20.112 99.687 51.459 188.161 93.388 270.336l-2.734-5.903c0 1.807 0 4.518 4.819 3.614 0.069-1.080 0.109-2.343 0.109-3.614s-0.039-2.534-0.117-3.786l0.009 0.172c-2.122-23.756-3.332-51.39-3.332-79.306 0-16.916 0.444-33.729 1.322-50.427l-0.098 2.335c2.143-41.776 8.279-81.046 18.068-118.845l-0.901 4.097c6.237-25.012 15.119-46.977 26.591-67.286l-0.69 1.328c10.556 50.436 44.321 91.249 89.362 111.342l0.991 0.395c6.927 3.915 9.939 2.108 10.842-5.421 2.446-16.541 6.335-31.358 11.641-45.481l-0.497 1.509c24.206-77.879 83.745-138.211 159.382-163.042l1.748-0.497c13.713-5.728 29.646-9.055 46.357-9.055 21.655 0 42.004 5.588 59.685 15.4l-0.63-0.321c30.563 19.089 55.771 43.912 74.731 73.162l0.563 0.928c29.693 44.54 54.732 95.808 72.53 150.348l1.258 4.456c3.614 10.24 4.518 10.842 13.252 4.819 37.504-25.775 69.958-54.976 98.226-87.878l0.56-0.667c35.014-36.387 56.734-85.784 57.223-140.253l0.001-0.096c0-5.12 2.108-5.722 6.024-3.614 11.716 5.659 21.692 13.036 30.070 21.935l0.048 0.051c22.879 25.437 41.269 55.452 53.583 88.431l0.629 1.922c30.128 75.686 53.532 163.968 66.179 255.684l0.682 6.038c0 3.313 0 7.831 3.614 8.734s4.216-3.915 5.722-6.626c25.167-40.726 44.986-87.981 56.877-138.3l0.648-3.253c10.527-41.368 16.569-88.858 16.569-137.759 0-30.89-2.411-61.216-7.054-90.802l0.424 3.281z"}],
                path![attrs!{At::D => "M756.555 634.278c-77.097 7.493-140.17 60.141-162.865 130.873l-0.372 1.343c-3.012 7.529-4.819 9.638-12.649 5.421-7.816-4.402-17.158-6.995-27.106-6.995s-19.29 2.593-27.388 7.14l0.282-0.145c-9.035 4.518-10.541 0-13.252-6.325-27.343-76.927-99.515-131.018-184.32-131.018-107.785 0-195.162 87.377-195.162 195.162 0 0.002 0 0.004 0 0.006l-0-0c0.177 107.652 87.486 194.852 195.162 194.852 58.836 0 111.592-26.036 147.374-67.215l0.203-0.239c29.71-32.853 47.891-76.621 47.891-124.636 0-0.442-0.002-0.883-0.005-1.324l0 0.068c-0.165-1.105-0.259-2.379-0.259-3.676 0-8.942 4.479-16.837 11.315-21.565l0.087-0.057c5.139-3.437 11.459-5.485 18.258-5.485 5.541 0 10.765 1.36 15.354 3.765l-0.182-0.087c8.284 4.103 13.879 12.499 13.879 22.201 0 1.413-0.119 2.798-0.347 4.146l0.020-0.145c-0.008 0.56-0.012 1.222-0.012 1.885 0 7.411 0.552 14.692 1.617 21.806l-0.099-0.802c12.467 97.023 94.545 171.237 193.956 171.237 107.952 0 195.464-87.512 195.464-195.464 0-10.799-0.876-21.393-2.56-31.716l0.152 1.128c-14.378-94.161-94.789-165.461-191.853-165.461-7.959 0-15.806 0.479-23.513 1.411l0.928-0.091zM326.475 988.762c-87.611-1.361-158.111-72.702-158.111-160.509 0-88.657 71.87-160.527 160.527-160.527s160.527 71.87 160.527 160.527c0 0.523-0.003 1.046-0.007 1.567l0.001-0.080c-1.183 88.082-72.864 159.031-161.116 159.031-0.64 0-1.279-0.004-1.918-0.011l0.097 0.001zM778.24 988.762c-88.136-0.684-159.32-72.29-159.32-160.523 0-88.657 71.87-160.527 160.527-160.527s160.527 71.87 160.527 160.527c0 0.316-0.001 0.632-0.003 0.948l0-0.049c-0.675 88.309-72.419 159.637-160.824 159.637-0.743 0-1.484-0.005-2.225-0.015l0.112 0.001z"}],
                path![attrs!{At::D => "M486.701 652.047c3.028 4.352 8.005 7.164 13.639 7.164 3.71 0 7.135-1.22 9.897-3.28l-0.044 0.031c4.286-3.098 7.042-8.082 7.042-13.709 0-3.669-1.172-7.065-3.161-9.833l0.034 0.050-53.609-74.993 76.499-20.179-93.967-114.146c-3.117-3.818-7.823-6.237-13.095-6.237-4.075 0-7.812 1.445-10.727 3.85l0.029-0.023c-3.751 3.17-6.117 7.877-6.117 13.138 0 4.042 1.397 7.757 3.734 10.69l-0.027-0.035 60.235 73.487-72.885 19.576z"}],
            ]
        ),
    ]
}

fn search_hint_container(
    label: &str,
    icon: &str, 
    view_box: &str, 
    paths: Vec<Node<Msg>>,
) -> Node<Msg> {
    div![
        C!["search-hint-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex("0 0 50%")
            .flex_direction(CssFlexDirection::Column)
            .justify_content(CssJustifyContent::Center)
            .margin_bottom(rem(4))
            .padding("0 2rem"),
        svg![
            C!["icon"],
            s()
                .overflow(CssOverflow::Visible)
                .fill(Color::SurfaceLight5_90)
                .flex(CssFlex::None)
                .height(rem(6))
                .margin_bottom(rem(2))
                .width(rem(6)),
            attrs!{
                At::from("icon") => icon,
                At::ViewBox => view_box,
            },
            paths,
        ],
        div![
            C!["label"],
            s()
                .color(Color::SurfaceLight5_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("1")
                .font_size(rem(1.2))
                .text_align(CssTextAlign::Center),
            label,
        ]
    ]
}

fn search_rows(search_results: &[VideoGroupResults]) -> Vec<Node<Msg>> {
    search_results.iter().enumerate().map(search_row).collect()
}

fn search_row((index, group): (usize, &VideoGroupResults)) -> Node<Msg> {
    div![
        C!["search-row", "search-row-poster", "meta-row-container"],
        s()
            .margin("4rem 2rem")
            .overflow(CssOverflow::Visible),
        IF!(index == 0 => s().margin_top(rem(2))),
        search_row_header_container(group),
        search_row_meta_items_container(group),
    ]
}

fn search_row_header_container(group: &VideoGroupResults) -> Node<Msg> {
    let see_all_title = "SEE ALL";
    div![
        C!["header-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::FlexEnd)
            .margin_bottom(rem(1))
            .padding("0 1rem"),
        div![
            C!["title-container"],
            s()
                .color(Color::SecondaryVariant2Light1_90)
                .flex("1")
                .font_size(rem(1.8))
                .max_height(em(2.4)),
            attrs!{
                At::Title => &group.label,
            },
            &group.label,
        ],
        a![
            C!["see-all-container", "button-container"],
            s()
                .align_items(CssAlignItems::Center)
                .display(CssDisplay::Flex)
                .flex(CssFlex::None)
                .flex_direction(CssFlexDirection::Row)
                .max_width(rem(12))
                .padding(rem(0.2))
                .cursor(CssCursor::Pointer),
            s()
                .style_other(":hover .label, :hover .icon")
                .color(Color::SecondaryVariant2Light2_90),
            attrs!{
                At::TabIndex => 0,
                At::Title => see_all_title,
            },
            on_click_not_implemented(),
            div![
                C!["label"],
                s()
                    .color(Color::SecondaryVariant2Light1_90)
                    .flex("0 1 auto")
                    .font_size(rem(1.3))
                    .font_weight("500")
                    .max_height(em(1.2))
                    .text_transform(CssTextTransform::Uppercase),
                see_all_title,
            ],
            see_all_icon(),
        ]
    ]
}

fn see_all_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .overflow(CssOverflow::Visible)
            .fill(Color::SecondaryVariant2Light1_90)
            .flex(CssFlex::None)
            .height(rem(1.3))
            .margin_left(rem(0.5)),
        attrs!{
            At::ViewBox => "0 0 565 1024",
            At::from("icon") => "ic_arrow_thin_right",
        },
        path![
            attrs!{
                At::D => "M84.932 14.155l465.016 463.511c8.963 8.73 14.578 20.859 14.757 34.301l0 0.033c-0.021 13.598-5.67 25.873-14.743 34.621l-0.015 0.014-464.113 463.209c-9.052 8.82-21.434 14.26-35.087 14.26s-26.035-5.44-35.098-14.27l0.011 0.010c-9.355-8.799-15.292-21.14-15.66-34.87l-0.001-0.066c-0.001-0.103-0.001-0.225-0.001-0.348 0-13.437 5.534-25.582 14.448-34.278l0.010-0.009 430.080-428.273-429.779-427.972c-9.101-8.684-14.76-20.907-14.76-34.451 0-0.171 0.001-0.341 0.003-0.511l-0 0.026c-0-0.043-0-0.094-0-0.145 0-13.595 5.526-25.899 14.455-34.789l0.002-0.002c9.099-8.838 21.532-14.287 35.238-14.287s26.138 5.449 35.25 14.299l-0.012-0.012z",
            }
        ]
    ]
}

fn search_row_meta_items_container(group: &VideoGroupResults) -> Node<Msg> {
    div![
        C!["meta-items-container"],
        s()
            .align_items(CssAlignItems::Stretch)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .overflow(CssOverflow::Visible),
        group.videos.iter().map(meta_item),
    ]
}

fn meta_item(video: &Video) -> Node<Msg> {
    a![
        C!["meta-item", "poster-shape-poster", "meta-item-container", "button-container"],
        s()
            .flex(format!("calc(1 / {})", global::POSTER_SHAPE_RATIO).as_str())
            .padding(rem(1))
            .overflow(CssOverflow::Visible)
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => 0,
            At::Title => video.name,
        },
        on_click_not_implemented(),
        poster_container(&video.poster),
        div![
            C!["title-bar-container"],
            s()
                .align_items(CssAlignItems::Center)
                .display(CssDisplay::Flex)
                .flex_direction(CssFlexDirection::Row)
                .height(rem(2.8))
                .overflow(CssOverflow::Visible),
            div![
                C!["title-label"],
                s()
                    .padding_right(rem(0.5))
                    .color(Color::SurfaceLight5_90)
                    .flex("1")
                    .max_height(em(2.4))
                    .padding_left(rem(0.5)),
                &video.name,
            ]
        ]
    ]
}

fn poster_container(poster: &str) -> Node<Msg> {
    div![
        C!["poster-container"],
        s()
            .padding_top(format!("calc(100% * {})", global::POSTER_SHAPE_RATIO).as_str())
            .background_color(Color::Background)
            .position(CssPosition::Relative)
            .z_index("0"),
        div![
            C!["poster-image-layer"],
            s()
                .align_items(CssAlignItems::Center)
                .bottom("0")
                .display(CssDisplay::Flex)
                .flex_direction(CssFlexDirection::Row)
                .justify_content(CssJustifyContent::Center)
                .left("0")
                .position(CssPosition::Absolute)
                .right("0")
                .top("0")
                .z_index("-3"),
            img![
                C!["poster-image"],
                s()
                    .flex(CssFlex::None)
                    .height(pc(100))
                    .object_fit("cover")
                    .object_position("center")
                    .opacity("0.9")
                    .width(pc(100)),
                attrs!{
                    At::Alt => " ",
                    At::Src => poster,
                },
            ]
        ]
    ]
}