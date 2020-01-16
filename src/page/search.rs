pub mod simsearch_search_panel;
pub mod simple_search_panel;
pub mod minisearch_search_panel;
pub mod localsearch_search_panel;
pub mod preindexed_localsearch_search_panel;

use seed::{prelude::*, *};
use crate::SharedModel;

// ------ ------
//     Model
// ------ ------

pub struct Model {
    shared: SharedModel,
    cinemeta_lite_simsearch: simsearch_search_panel::Model,
    cinemeta_simsearch: simsearch_search_panel::Model,
    cinemeta_simple: simple_search_panel::Model,
    cinemeta_lite_minisearch: minisearch_search_panel::Model,
    cinemeta_minisearch: minisearch_search_panel::Model,
    cinemeta_lite_localsearch: localsearch_search_panel::Model,
    cinemeta_localsearch: localsearch_search_panel::Model,
    cinemeta_preindexed_localsearch: preindexed_localsearch_search_panel::Model,
}

impl Model {
    pub fn shared(&mut self) -> &mut SharedModel {
        &mut self.shared
    }
}

impl From<Model> for SharedModel {
    fn from(model: Model) -> Self {
        model.shared
    }
}

// ------ ------
//     Init
// ------ ------

pub fn init(
    shared: SharedModel,
) -> Model {
    Model {
        shared,
        cinemeta_lite_simsearch: simsearch_search_panel::init("Cinemeta-lite (simsearch)", "/data/cinemeta-lite.json"),
        cinemeta_simsearch: simsearch_search_panel::init("Cinemeta (simsearch)", "/data/cinemeta.json"),
        cinemeta_simple: simple_search_panel::init("Cinemeta (simple search with .contains(..))", "/data/cinemeta.json"),
        cinemeta_lite_minisearch: minisearch_search_panel::init("Cinemeta-lite (minisearch js)", "/data/cinemeta-lite.json"),
        cinemeta_minisearch: minisearch_search_panel::init("Cinemeta (minisearch js)", "/data/cinemeta.json"),
        cinemeta_lite_localsearch: localsearch_search_panel::init("Cinemeta-lite (localsearch) WIP", "/data/cinemeta-lite.json"),
        cinemeta_localsearch: localsearch_search_panel::init("Cinemeta (localsearch) WIP", "/data/cinemeta.json"),
        cinemeta_preindexed_localsearch: preindexed_localsearch_search_panel::init("Cinemeta (preindexed localsearch) NOT IMPLEMENTED (YET)", "/data/cinemeta.json"),
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    CinemetaLiteSimsearch(simsearch_search_panel::Msg),
    CinemetaSimsearch(simsearch_search_panel::Msg),
    CinemetaSimple(simple_search_panel::Msg),
    CinemetaMinisearch(minisearch_search_panel::Msg),
    CinemetaLiteMinisearch(minisearch_search_panel::Msg),
    CinemetaLiteLocalsearch(localsearch_search_panel::Msg),
    CinemetaLocalsearch(localsearch_search_panel::Msg),
    CinemetaPreindexedLocalsearch(preindexed_localsearch_search_panel::Msg),
}

pub fn update<GMs: 'static>(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMs>) {
    match msg {
        Msg::CinemetaLiteSimsearch(msg) => simsearch_search_panel::update(msg, &mut model.cinemeta_lite_simsearch, &mut orders.proxy(Msg::CinemetaLiteSimsearch)),
        Msg::CinemetaSimsearch(msg) => simsearch_search_panel::update(msg, &mut model.cinemeta_simsearch, &mut orders.proxy(Msg::CinemetaSimsearch)),
        Msg::CinemetaSimple(msg) => simple_search_panel::update(msg, &mut model.cinemeta_simple, &mut orders.proxy(Msg::CinemetaSimple)),
        Msg::CinemetaLiteMinisearch(msg) => minisearch_search_panel::update(msg, &mut model.cinemeta_lite_minisearch, &mut orders.proxy(Msg::CinemetaLiteMinisearch)),
        Msg::CinemetaMinisearch(msg) => minisearch_search_panel::update(msg, &mut model.cinemeta_minisearch, &mut orders.proxy(Msg::CinemetaMinisearch)),
        Msg::CinemetaLiteLocalsearch(msg) => localsearch_search_panel::update(msg, &mut model.cinemeta_lite_localsearch, &mut orders.proxy(Msg::CinemetaLiteLocalsearch)),
        Msg::CinemetaLocalsearch(msg) => localsearch_search_panel::update(msg, &mut model.cinemeta_localsearch, &mut orders.proxy(Msg::CinemetaLocalsearch)),
        Msg::CinemetaPreindexedLocalsearch(msg) => preindexed_localsearch_search_panel::update(msg, &mut model.cinemeta_preindexed_localsearch, &mut orders.proxy(Msg::CinemetaPreindexedLocalsearch)),
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> impl View<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::FlexWrap => "wrap",
            St::MaxWidth => vw(100),
            St::MaxHeight => vh(100),
            St::Overflow => "auto",
        },
        simsearch_search_panel::view(&model.cinemeta_lite_simsearch).map_msg(Msg::CinemetaLiteSimsearch),
        simsearch_search_panel::view(&model.cinemeta_simsearch).map_msg(Msg::CinemetaSimsearch),
        simple_search_panel::view(&model.cinemeta_simple).map_msg(Msg::CinemetaSimple),
        minisearch_search_panel::view(&model.cinemeta_lite_minisearch).map_msg(Msg::CinemetaLiteMinisearch),
        minisearch_search_panel::view(&model.cinemeta_minisearch).map_msg(Msg::CinemetaMinisearch),
        localsearch_search_panel::view(&model.cinemeta_lite_localsearch).map_msg(Msg::CinemetaLiteLocalsearch),
        localsearch_search_panel::view(&model.cinemeta_localsearch).map_msg(Msg::CinemetaLocalsearch),
        preindexed_localsearch_search_panel::view(&model.cinemeta_preindexed_localsearch).map_msg(Msg::CinemetaPreindexedLocalsearch),
    ]
}

