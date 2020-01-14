pub mod search_panel;

use seed::{prelude::*, *};
use crate::SharedModel;

// ------ ------
//     Model
// ------ ------

pub struct Model {
    shared: SharedModel,
    cinemeta_lite: search_panel::Model,
    cinemeta: search_panel::Model,
    cinemeta_simple: search_panel::Model,
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
        cinemeta_lite: search_panel::init("Cinemeta-lite", "/data/cinemeta-lite.json"),
        cinemeta: search_panel::init("Cinemeta", "/data/cinemeta.json"),
        cinemeta_simple: search_panel::init("Cinemeta (simple search)", "/data/cinemeta.json"),
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    CinemetaLite(search_panel::Msg),
    Cinemeta(search_panel::Msg),
    CinemetaSimple(search_panel::Msg),
}

pub fn update<GMs: 'static>(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMs>) {
    match msg {
        Msg::CinemetaLite(msg) => search_panel::update(msg, &mut model.cinemeta_lite, &mut orders.proxy(Msg::CinemetaLite)),
        Msg::Cinemeta(msg) => search_panel::update(msg, &mut model.cinemeta, &mut orders.proxy(Msg::Cinemeta)),
        Msg::CinemetaSimple(msg) => search_panel::update(msg, &mut model.cinemeta_simple, &mut orders.proxy(Msg::CinemetaSimple)),
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
        search_panel::view(&model.cinemeta_lite).map_msg(Msg::CinemetaLite),
        search_panel::view(&model.cinemeta).map_msg(Msg::Cinemeta),
        search_panel::view(&model.cinemeta_simple).map_msg(Msg::CinemetaSimple),
    ]
}

