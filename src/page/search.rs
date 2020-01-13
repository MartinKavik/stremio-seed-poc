pub mod search_panel;

use seed::{prelude::*, *};
use crate::SharedModel;

// ------ ------
//     Model
// ------ ------

pub struct Model {
    shared: SharedModel,
    cinemeta: search_panel::Model,
    cinemeta_lite: search_panel::Model,
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
        cinemeta: search_panel::init("Cinemeta", "/data/cinemeta.json"),
        cinemeta_lite: search_panel::init("Cinemeta-lite", "/data/cinemeta-lite.json"),
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    Cinemeta(search_panel::Msg),
    CinemetaLite(search_panel::Msg),
}

pub fn update<GMs: 'static>(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMs>) {
    match msg {
        Msg::Cinemeta(msg) => search_panel::update(msg, &mut model.cinemeta, &mut orders.proxy(Msg::Cinemeta)),
        Msg::CinemetaLite(msg) => search_panel::update(msg, &mut model.cinemeta_lite, &mut orders.proxy(Msg::CinemetaLite)),
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> impl View<Msg> {
    vec![
        view_search_panels(model),
        view_background(),
    ]
}

fn view_background() -> Node<Msg> {
    div![
        style!{
            St::Position => "fixed",
            St::Width => unit!(100, %),
            St::Height => unit!(100, %),
        }
    ]
}

fn view_search_panels(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::Width => unit!(100, %),
        },
        search_panel::view(&model.cinemeta).map_msg(Msg::Cinemeta),
        search_panel::view(&model.cinemeta_lite).map_msg(Msg::CinemetaLite),
    ]
}


