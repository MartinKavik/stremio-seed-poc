use seed::{prelude::*, *};
use seed_hooks::{*, topo::nested as view};
use seed_styles::{em, pc, rem, Style};
use seed_styles::*;
use std::borrow::Cow;
use std::rc::Rc;
use stremio_core::types::profile::{User, Settings};
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::models::common::Loadable;
use crate::Urls as RootUrls;
use crate::styles::{self, themes::Color, global};
use crate::page::settings::Msg;
use crate::page::settings::section::{
    section_option,
    section,
    control::{label, dropdown, connect_button, link_label, large_button, url, status}
};
use web_sys::Element;

#[view]
pub fn streaming_server_section(
    settings: &Settings, 
    section_ref: &ElRef<Element>, 
    streaming_server: &StreamingServer
) -> Node<Msg> {
    let is_streming_server_ready = matches!(streaming_server.settings, Loadable::Ready(_));

    let mut options = vec![
        section_option(None, vec![
            large_button("Reload", None, Some(|| Msg::ReloadStreamingServer))
        ]),
        section_option(None, vec![
            label("Status"),
            status(streaming_server),
        ]),
        section_option(not(is_streming_server_ready).then(|| s().margin_bottom("0")), vec![
            label("Url"),
            url(&settings.streaming_server_url.to_string(), "Configure server url")
        ]),
    ];
    if let Loadable::Ready(settings) = &streaming_server.settings {
        let cache_size_value = settings.cache_size.map_or_else(
            || "No caching".to_owned(), 
            |size| format!("{} GiB", size / 1_073_741_824.)
        );

        options.append(&mut vec![
            section_option(None, vec![
                label("Cache size"),
                dropdown(&cache_size_value)
            ]),
            section_option(Some(s().margin_bottom("0")), vec![
                label("Torrent profile"),
                dropdown("default")
            ]),
        ]);
    }
    section("Streaming Server", false, section_ref, options)
}
