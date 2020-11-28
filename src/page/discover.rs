use crate::{entity::multi_select, Context, PageId, Actions, Urls as RootUrls};
use enclose::enc;
use seed::{prelude::*, *};
use std::rc::Rc;
use stremio_core::state_types::{
    Action, ActionLoad, CatalogEntry, CatalogError, Internal, Loadable, Msg as CoreMsg, TypeEntry,
};
use stremio_core::types::MetaPreview;
use stremio_core::types::{
    addons::{ResourceRef, ResourceRequest, ResourceResponse},
    PosterShape,
};
use seed_styles::{px, pc, rem, em};
use seed_styles::*;
use crate::styles::{self, themes::{Color, Breakpoint}, global};

mod catalog_selector;
mod extra_prop_selector;
mod type_selector;

type MetaPreviewId = String;
// @TODO add into stremio-core?
type ExtraPropOption = String;

const DEFAULT_CATALOG: &str = "top";
const DEFAULT_TYPE: &str = "movie";
const BASE: &str = "https://v3-cinemeta.strem.io/manifest.json";
const RESOURCE: &str = "catalog";

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

    let resource_request = match url.remaining_hash_path_parts().as_slice() {
        [base, path] => path
            .parse()
            .map_err(|error| error!(error))
            .map(|path| ResourceRequest {
                base: base.to_string(),
                path,
            })
            .ok(),
        _ => None,
    };

    load_catalog(resource_request, orders);

    model.get_or_insert_with(move || Model {
        base_url,
        _core_msg_sub_handle: orders.subscribe_with_handle(Msg::CoreMsg),
        type_selector_model: type_selector::init(),
        catalog_selector_model: catalog_selector::init(),
        extra_prop_selector_model: extra_prop_selector::init(),
        selected_meta_preview_id: None,
    });
    Some(PageId::Discover)
}

fn load_catalog(resource_request: Option<ResourceRequest>, orders: &mut impl Orders<Msg>) {
    orders.notify(Actions::UpdateCoreModel(Rc::new(CoreMsg::Action(Action::Load(
        ActionLoad::CatalogFiltered(resource_request.unwrap_or_else(default_resource_request)),
    )))));
}

pub fn default_resource_request() -> ResourceRequest {
    ResourceRequest::new(
        BASE,
        ResourceRef::without_extra(RESOURCE, DEFAULT_TYPE, DEFAULT_CATALOG),
    )
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    base_url: Url,
    _core_msg_sub_handle: SubHandle,
    selected_meta_preview_id: Option<MetaPreviewId>,
    type_selector_model: type_selector::Model,
    catalog_selector_model: catalog_selector::Model,
    extra_prop_selector_model: extra_prop_selector::Model,
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    pub fn root(self) -> Url {
        self.base_url()
    }
    pub fn res_req(self, res_req: &ResourceRequest) -> Url {
        self.base_url()
            .add_hash_path_part(&res_req.base)
            .add_hash_path_part(res_req.path.to_string())
    }
}

// ------ ------
//    Update
// ------ ------

#[allow(clippy::pub_enum_variant_names, clippy::large_enum_variant)]
pub enum Msg {
    CoreMsg(Rc<CoreMsg>),
    MetaPreviewClicked(MetaPreview),
    TypeSelectorMsg(type_selector::Msg),
    TypeSelectorChanged(Vec<multi_select::Group<TypeEntry>>),
    CatalogSelectorMsg(catalog_selector::Msg),
    CatalogSelectorChanged(Vec<multi_select::Group<CatalogEntry>>),
    ExtraPropSelectorMsg(extra_prop_selector::Msg),
    ExtraPropSelectorChanged(Vec<multi_select::Group<ExtraPropOption>>),
}

pub fn update(msg: Msg, model: &mut Model, context: &mut Context, orders: &mut impl Orders<Msg>) {
    let catalog = &context.core_model.catalog;

    match msg {
        Msg::CoreMsg(core_msg) => {
            if let CoreMsg::Internal(Internal::AddonResponse(_, result)) = core_msg.as_ref() {
                if let Ok(ResourceResponse::Metas { metas }) = result.as_ref() {
                    model.selected_meta_preview_id = metas.first().map(|meta| meta.id.clone());
                }
            }
        }
        Msg::MetaPreviewClicked(meta_preview) => {
            if model.selected_meta_preview_id.as_ref() == Some(&meta_preview.id) {
                let id = &meta_preview.id;
                let type_name = &meta_preview.type_name;

                let detail_urls = RootUrls::new(&context.root_base_url).detail_urls();

                orders.request_url(if meta_preview.type_name == "movie" {
                    detail_urls.with_video_id(type_name, id, id)
                } else {
                    detail_urls.without_video_id(type_name, id)
                });
            } else {
                model.selected_meta_preview_id = Some(meta_preview.id);
            }
        }

        // ------ TypeSelector  ------
        Msg::TypeSelectorMsg(msg) => {
            let msg_to_parent = type_selector::update(
                msg,
                &mut model.type_selector_model,
                &mut orders.proxy(Msg::TypeSelectorMsg),
                type_selector::groups(&catalog.types),
                Msg::TypeSelectorChanged,
            );
            if let Some(msg) = msg_to_parent {
                orders.send_msg(msg);
            }
        }
        Msg::TypeSelectorChanged(groups_with_selected_items) => {
            let res_req = type_selector::resource_request(groups_with_selected_items);
            orders.request_url(Urls::new(&model.base_url).res_req(&res_req));
        }

        // ------ CatalogSelector  ------
        Msg::CatalogSelectorMsg(msg) => {
            let msg_to_parent = catalog_selector::update(
                msg,
                &mut model.catalog_selector_model,
                &mut orders.proxy(Msg::CatalogSelectorMsg),
                catalog_selector::groups(&catalog.catalogs, &catalog.selected),
                Msg::CatalogSelectorChanged,
            );
            if let Some(msg) = msg_to_parent {
                orders.send_msg(msg);
            }
        }
        Msg::CatalogSelectorChanged(groups_with_selected_items) => {
            let res_req = catalog_selector::resource_request(groups_with_selected_items);
            orders.request_url(Urls::new(&model.base_url).res_req(&res_req));
        }

        // ------ ExtraPropSelector  ------
        Msg::ExtraPropSelectorMsg(msg) => {
            let msg_to_parent = extra_prop_selector::update(
                msg,
                &mut model.extra_prop_selector_model,
                &mut orders.proxy(Msg::ExtraPropSelectorMsg),
                extra_prop_selector::groups(&catalog.selectable_extra, &catalog.selected),
                Msg::ExtraPropSelectorChanged,
            );
            if let Some(msg) = msg_to_parent {
                orders.send_msg(msg);
            }
        }
        Msg::ExtraPropSelectorChanged(groups_with_selected_items) => {
            if let Some(res_req) =
                extra_prop_selector::resource_request(groups_with_selected_items, &catalog.selected)
            {
                orders.request_url(Urls::new(&model.base_url).res_req(&res_req));
            }
        }
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model, context: &Context) -> Node<Msg> {
    let catalog = &context.core_model.catalog;

    div![
        C!["discover- "],
        s()
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Column)
            .width(pc(100))
            .height(pc(100))
            .background_color(Color::BackgroundDark2),
        div![
            C!["discover-content"],
            s()
                .flex("1")
                .align_self(CssAlignSelf::Stretch)
                .display(CssDisplay::Grid)
                .grid_template_columns("1fr 28rem")
                .grid_template_rows("7rem 1fr")
                .grid_template_areas(r#""controls-area meta-preview-area" "catalog-content-area meta-preview-area""#),
            s()
                .only_and_below(Breakpoint::Minimum)
                .grid_template_columns("1fr")
                .grid_template_rows("fit-content(19rem) 1fr")
                .grid_template_areas(r#""controls-area" "catalog-content-area""#),
            div![
                C!["selectable-inputs-container"],
                s()
                    .align_self(CssAlignSelf::Stretch)
                    .display(CssDisplay::Flex)
                    .flex(CssFlex::None)
                    .flex_direction(CssFlexDirection::Row)
                    .overflow(CssOverflow::Visible)
                    .padding(rem(1.5)),
                // type selector
                type_selector::view(
                    &model.type_selector_model,
                    &type_selector::groups(&catalog.types)
                )
                .map_msg(Msg::TypeSelectorMsg),
                // catalog selector
                catalog_selector::view(
                    &model.catalog_selector_model,
                    &catalog_selector::groups(&catalog.catalogs, &catalog.selected)
                )
                .map_msg(Msg::CatalogSelectorMsg),
                // extra prop selector
                extra_prop_selector::view(
                    &model.extra_prop_selector_model,
                    &extra_prop_selector::groups(&catalog.selectable_extra, &catalog.selected)
                )
                .map_msg(Msg::ExtraPropSelectorMsg),
            ],
            div![
                C!["catalog-content-container"],
                s()
                    .grid_area("catalog-content-area")
                    .margin_right(rem(2)),
                s()
                    .only_and_below(Breakpoint::Minimum)
                    .margin_right("0"),
                view_content(
                    &context.core_model.catalog.content,
                    model.selected_meta_preview_id.as_ref()
                ),
            ]
        ],
    ]
}

fn view_content(
    content: &Loadable<Vec<MetaPreview>, CatalogError>,
    selected_meta_preview_id: Option<&MetaPreviewId>,
) -> Node<Msg> {
    let message_container_style = s()
        .padding("0 2rem")
        .font_size(rem(2))
        .color(Color::SurfaceLighter);

    match content {
        Loadable::Err(catalog_error) => {
            div![C!["message-container",], message_container_style, format!("{:#?}", catalog_error)]
        }
        Loadable::Loading => div![C!["message-container",], message_container_style, "Loading"],
        Loadable::Ready(meta_previews) if meta_previews.is_empty() => empty![],
        Loadable::Ready(meta_previews) => div![
            C!["meta-items-container",],
            s()
                .display(CssDisplay::Grid)
                .max_height(pc(100))
                .grid_auto_rows("max-content")
                .grid_gap(rem(1.5))
                .align_items(CssAlignItems::Center)
                .padding("0 2rem")
                .overflow_y(CssOverflowY::Auto),
            s()
                .only_and_above(Breakpoint::XXLarge)
                .grid_template_columns("repeat(8, 1fr)"),
            s()
                .only_and_below(Breakpoint::XLarge)
                .grid_template_columns("repeat(7, 1fr)"),
            s()
                .only_and_below(Breakpoint::Medium)
                .grid_template_columns("repeat(6, 1fr)"),
            s()
                .only_and_below(Breakpoint::Small)
                .grid_template_columns("repeat(5, 1fr)"),
            s()
                .only_and_below(Breakpoint::XSmall)
                .grid_template_columns("repeat(4, 1fr)"),
            s()
                .only_and_below(Breakpoint::Minimum)
                .grid_template_columns("repeat(5, 1fr)"),
            meta_previews
                .iter()
                .map(|meta_preview| meta_item(meta_preview, selected_meta_preview_id)),
        ],
    }
}

fn meta_item(meta: &MetaPreview, selected_meta_preview_id: Option<&MetaPreviewId>) -> Node<Msg> {
    a![
        el_key(&meta.id),
        C!["meta-item", "poster-shape-poster", "meta-item-container", "button-container"],
        s()
            .flex(format!("calc(1 / {});", global::POSTER_SHAPE_RATIO).as_str())
            .padding(rem(1))
            .overflow(CssOverflow::Visible)
            .cursor(CssCursor::Pointer),
        s()
            .hover()
            .background_color(Color::BackgroundLight3)
            .transition("background-color 100ms ease-out"),
        attrs!{
            At::TabIndex => 0,
            At::Title => meta.name,
        },
        on_click_not_implemented(),
        poster_container(&meta.poster),
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
                &meta.name,
            ]
        ]
    ]
}

fn poster_container(poster: &Option<String>) -> Node<Msg> {
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
                    At::Src => poster.clone().unwrap_or_default(),
                },
            ]
        ]
    ]
}
