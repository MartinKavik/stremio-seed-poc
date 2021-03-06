use crate::{PageId, Msg, Urls as RootUrls, Context};
use seed::{prelude::*, *};
use seed_styles::{pc, rem, em};
use seed_styles::*;
use crate::styles::{self, themes::{Color, Breakpoint}, global};
use std::rc::Rc;
use std::borrow::Cow;
use seed_hooks::{*, topo::nested as view, state_access::CloneState};
use stremio_core::types::profile::User;

fn on_click_not_implemented() -> EventHandler<Msg> {
    ev(Ev::Click, |_| { window().alert_with_message("Not implemented!").unwrap(); })
}

#[view]
pub fn menu_button(root_base_url: &Url, menu_visible: bool, fullscreen: bool, user: Option<&User>) -> Node<Msg> {
    label![
        id!("menu-toggle"),
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
        IF!(menu_visible => s().background_color(Color::BackgroundLight2)),
        attrs!{
            At::TabIndex => -1,
        },
        ev(Ev::Click, |event| {
            event.stop_propagation();
            Msg::ToggleMenu
        }),
        menu_icon(),
        IF!(menu_visible => {
            menu_container(root_base_url, fullscreen, user)
        }),
    ]
}

#[view]
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

#[view]
fn menu_container(root_base_url: &Url, fullscreen: bool, user: Option<&User>) -> Node<Msg> {
    div![
        C!["menu-container", "menu-direction-bottom-left"],
        s()
            .bottom("initial")
            .left("initial")
            .right("0")
            .top("100%")
            .visibility(CssVisibility::Visible)
            .box_shadow("0 1.35rem 2.7rem hsla(0,0%,0%,0.4),0 1.1rem 0.85rem hsla(0,0%,0%,0.2)")
            .cursor(CssCursor::Auto)
            .overflow(CssOverflow::Visible)
            .position(CssPosition::Absolute)
            .z_index("1"),
        ev(Ev::Click, |event| {
            event.stop_propagation();
        }),
        div![
            C!["nav-menu-container"],
            s()
                .background_color(Color::BackgroundDark1)
                .max_height(format!("calc(100vh - {})", global::HORIZONTAL_NAV_BAR_SIZE).as_str())
                .overflow_y("auto")
                .width(rem(20)),
            menu_section_user(root_base_url, user),
            menu_section_fullscreen(fullscreen),
            menu_section_general(root_base_url),
            menu_section_docs(),
        ]
    ]
}

#[view]
fn menu_section_user(root_base_url: &Url, user: Option<&User>) -> Node<Msg> {
    let button_title = if user.is_some() { "Log out "} else { "Log in / Sign up" };

    let avatar = user.and_then(|user| user.avatar.as_ref());
    let image_url: Cow<_> = match (user, avatar) {
        (None, _) => global::image_url("anonymous.png").into(),
        (Some(_), None) => global::image_url("default_avatar.png").into(),
        (Some(_), Some(avatar)) => avatar.into(),
    };

    div![
        C!["user-info-container"],
        s()
            .display(CssDisplay::Grid)
            .grid_template_areas(r#""avatar-area email-area" "avatar-area logout-button-area""#)
            .grid_template_columns("7rem 1fr")
            .grid_template_rows("50% 50%")
            .height(rem(7)),
        div![
            C!["avatar-container"],
            s()
                .background_image(format!(r#"url("{}")"#, image_url).as_str())
                .background_clip("content-box")
                .background_origin("content-box")
                .background_position(CssBackgroundPosition::Center)
                .background_repeat(CssBackgroundRepeat::NoRepeat)
                .background_size("cover")
                .border_radius(pc(50))
                .grid_area("avatar-area")
                .opacity("0.9")
                .padding(rem(1)),
        ],
        div![
            C!["email-container"],
            s()
                .align_items(CssAlignItems::Center)
                .display(CssDisplay::Flex)
                .flex_direction(CssFlexDirection::Row)
                .grid_area("email-area")
                .padding("1rem 1rem 0 0"),
            div![
                C!["email-label"],
                s()
                    .color(Color::SurfaceLight5_90)
                    .flex("1")
                    .max_height(em(2.4)),
                if let Some(user) = user {
                    &user.email
                } else {
                    "Anonymous user"
                },
            ]
        ],
        a![
            C!["logout-button-container", "button-container"],
            attrs!{
                At::TabIndex => 0,
                At::Title => button_title,
                At::Href => RootUrls::new(root_base_url).intro(),
            },
            s()
                .align_items(CssAlignItems::Center)
                .display(CssDisplay::Flex)
                .flex_direction(CssFlexDirection::Row)
                .grid_area("logout-button-area")
                .padding("0 1rem 1rem 0")
                .cursor(CssCursor::Pointer),
            ev(Ev::Click, |_| Msg::HideMenu),
            user.map(|_| ev(Ev::Click, |_| Msg::Logout)),
            div![
                C!["logout-label"],
                s()
                    .color(Color::SurfaceLight3_90)
                    .flex("1")
                    .max_height(em(2.4)),
                s()
                    .hover()
                    .color(Color::SurfaceLight5_90)
                    .text_decoration(CssTextDecoration::Underline),
                button_title,
            ]
        ]
    ]
}

#[view]
fn menu_section_fullscreen(fullscreen: bool) -> Node<Msg> {
    div![
        C!["nav-menu-section"],
        s()
            .border_top("thin solid hsla(0,0%,100%,0.2)"),
        menu_option(MenuOptionArgs { 
            title: if fullscreen { "Exit Fullscreen" } else { "Enter Fullscreen" }, 
            link: None, 
            icon: Some(fullscreen_icon(fullscreen)), 
            enabled: true,
            on_click: Some(ev(Ev::Click, |_| Msg::ToggleFullscreen)),
        }),
    ]
}

#[view]
fn menu_section_general(root_base_url: &Url) -> Node<Msg> {
    div![
        C!["nav-menu-section"],
        s()
            .border_top("thin solid hsla(0,0%,100%,0.2)"),
        menu_option(MenuOptionArgs { 
            title: "Settings", 
            link: Some(LinkArgs { url: &RootUrls::new(root_base_url).settings().to_string(), target_blank: false }),
            icon: Some(settings_icon()),
            enabled: true,
            on_click: None,
        }),
        menu_option(MenuOptionArgs { 
            title: "Addons", 
            link: Some(LinkArgs { url: &RootUrls::new(root_base_url).addons_urls().root().to_string(), target_blank: false }),
            icon: Some(addons_icon()),
            enabled: true,
            on_click: None,
        }),
        menu_option(MenuOptionArgs { 
            title: "Remote Control", 
            link: None, 
            icon: Some(remote_control_icon()), 
            enabled: false, 
            on_click: None,
        }),
        menu_option(MenuOptionArgs { 
            title: "Play Magnet Link", 
            link: None, 
            icon: Some(play_magnet_link_icon()), 
            enabled: false,  
            on_click: None,
        }),
        menu_option(MenuOptionArgs { 
            title: "Help & Feedback", 
            link: Some(LinkArgs { url: "https://stremio.zendesk.com/", target_blank: true }),
            icon: Some(help_and_feedback_icon()),
            enabled: true,
            on_click: None,
        }),
    ]
}

#[view]
fn menu_section_docs() -> Node<Msg> {
    div![
        C!["nav-menu-section"],
        s()
            .border_top("thin solid hsla(0,0%,100%,0.2)"),
        menu_option(MenuOptionArgs { 
            title: "Terms of Service", 
            link: Some(LinkArgs { url: "https://www.stremio.com/tos", target_blank: true }), 
            icon: None,
            enabled: true,
            on_click: None,
        }),
        menu_option(MenuOptionArgs { 
            title: "Privacy Policy", 
            link: Some(LinkArgs { url: "https://www.stremio.com/privacy", target_blank: true }), 
            icon: None,
            enabled: true,
            on_click: None,
        }),
        menu_option(MenuOptionArgs { 
            title: "About Stremio", 
            link: Some(LinkArgs { url: "https://www.stremio.com/", target_blank: true }),
            icon: None,
            enabled: true,
            on_click: None,
        }),
    ]
}

struct MenuOptionArgs<'a> {
    title: &'a str,
    link: Option<LinkArgs<'a>>,
    icon: Option<Node<Msg>>,
    enabled: bool,
    on_click: Option<EventHandler<Msg>>,
}

struct LinkArgs<'a> {
    url: &'a str,
    target_blank: bool,
}

#[view]
fn menu_option(args: MenuOptionArgs) -> Node<Msg> {
    custom![
        Tag::from(if args.link.is_some() { "a" } else { "div" }),
        C!["nav-menu-option-container", "button-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .height(rem(4)),
        IF!(args.enabled => {
            s()
                .cursor(CssCursor::Pointer)
        }),
        IF!(args.enabled => {
            s()
                .hover()
                .background_color(Color::BackgroundLight2)
        }),
        attrs!{
            At::TabIndex => 0,
            At::Title => args.title,
        },
        args.link.as_ref().map(|link| {
            attrs!{
                At::Href => link.url,
            }
        }),
        args.link.as_ref().map(|link| {
            if !link.target_blank {
                return None
            }
            Some (attrs!{
                At::Target => "_blank",
            })
        }),
        args.on_click,
        IF!(args.enabled => {
            ev(Ev::Click, |_| Msg::HideMenu)
        }),
        args.icon,
        div![
            C!["nav-menu-option-label"],
            s()
                .padding_left(rem(1.3))
                .color(Color::SurfaceLight5_90)
                .flex("1")
                .max_height(em(2.4))
                .padding_right(rem(1.3)),
            args.title,
        ]
    ]
}

#[view]
fn fullscreen_icon(fullscreen: bool) -> Node<Msg> {
    if fullscreen {
        option_icon("0 0 1016 1024", "ic_exit_fullscreen", vec![
            path![
                attrs!{
                    At::D => "M63.793 442.038l316.257 1.505c0.090 0 0.196 0.001 0.302 0.001 34.462 0 62.418-27.851 62.589-62.273l0-0.016v-316.257c0-35.897-29.1-64.997-64.997-64.997v0c-17.37 0.034-33.097 7.037-44.54 18.361l0.005-0.005c-11.337 10.959-18.375 26.303-18.375 43.291 0 0.543 0.007 1.085 0.022 1.625l-0.002-0.080v252.464h-252.163c-0.548-0.018-1.192-0.028-1.838-0.028-16.992 0-32.339 7.042-43.282 18.366l-0.017 0.017c-10.984 11.354-17.754 26.844-17.754 43.915 0 0.006 0 0.012 0 0.018l-0-0.001c0.497 35.091 28.727 63.427 63.73 64.093l0.063 0.001z",
                },
            ],
            path![
                attrs!{
                    At::D => "M634.621 443.543l316.257-1.505c35.195-0.501 63.593-28.899 64.094-64.046l0.001-0.048c-0.018-17.467-7.021-33.296-18.365-44.845l0.009 0.010c-10.96-11.342-26.307-18.384-43.299-18.384-0.646 0-1.29 0.010-1.932 0.030l0.094-0.002h-252.464v-252.464c0.004-0.28 0.007-0.61 0.007-0.941 0-16.569-6.287-31.669-16.605-43.045l0.048 0.054c-11.438-11.319-27.165-18.322-44.528-18.356l-0.006-0c-34.962 0.662-63.131 28.831-63.792 63.73l-0.001 0.063v316.257c-0.005 0.289-0.008 0.631-0.008 0.973 0 17.26 7.020 32.88 18.36 44.161l0.003 0.003c10.844 10.833 25.624 17.728 42.011 18.352l0.117 0.004z",
                },
            ],
            path![
                attrs!{
                    At::D => "M382.458 580.457v0 0h-318.063c-35.533 0.339-64.227 29.139-64.395 64.68l-0 0.016c0.018 17.467 7.021 33.296 18.365 44.845l-0.009-0.010c10.96 11.342 26.307 18.384 43.299 18.384 0.646 0 1.29-0.010 1.932-0.030l-0.094 0.002h252.163v252.464c-0.013 0.46-0.020 1.002-0.020 1.545 0 16.988 7.039 32.332 18.358 43.274l0.017 0.016c11.438 11.319 27.165 18.322 44.528 18.356l0.006 0c35.091-0.497 63.427-28.727 64.093-63.73l0.001-0.063v-316.257c0.005-0.282 0.007-0.615 0.007-0.949 0-33.723-26.67-61.217-60.069-62.54l-0.12-0.004z",
                },
            ],
            path![
                attrs!{
                    At::D => "M951.48 581.059h-316.257c-0.27-0.004-0.59-0.007-0.909-0.007-34.567 0-62.589 28.022-62.589 62.589 0 0.32 0.002 0.639 0.007 0.957l-0.001-0.048v316.257c1.163 34.652 29.533 62.3 64.361 62.3 0.435 0 0.869-0.004 1.303-0.013l-0.065 0.001c17.37-0.034 33.097-7.037 44.54-18.361l-0.005 0.005c11.335-11.54 18.337-27.369 18.356-44.832l0-0.003v-252.464h252.464c0.548 0.018 1.192 0.028 1.838 0.028 16.992 0 32.339-7.042 43.282-18.366l0.017-0.017c11.281-11.454 18.248-27.185 18.248-44.544 0-35.066-28.426-63.492-63.492-63.492-0.385 0-0.77 0.003-1.153 0.010l0.058-0.001z",
                },
            ],
        ])
    } else {
        option_icon("0 0 1016 1024", "ic_fullscreen", vec![
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
        ])
    }
}

#[view]
fn settings_icon() -> Node<Msg> {
    option_icon("0 0 1043 1024", "ic_settings", vec![
        path![
            attrs!{
                At::D => "M791.492 901.421c-0.137 1.886-0.214 4.085-0.214 6.303 0 14.689 3.414 28.58 9.492 40.924l-0.242-0.544c1.442 2.027 2.306 4.553 2.306 7.281 0 5.548-3.572 10.262-8.542 11.967l-0.089 0.027c-37.735 21.585-81.411 40.158-127.33 53.451l-4.284 1.062c-2.114 1.002-4.593 1.587-7.209 1.587-7.903 0-14.559-5.341-16.556-12.61l-0.028-0.12c-20.88-43.535-64.606-73.060-115.229-73.060-26.819 0-51.703 8.287-72.23 22.44l0.428-0.279c-19.628 13.227-34.808 31.704-43.688 53.426l-0.284 0.786c-3.614 8.734-7.529 11.746-17.769 9.035-51.834-13.272-97.233-31.525-139.449-54.835l3.016 1.527c-14.758-7.831-8.734-16.866-5.12-26.805 4.846-12.398 7.654-26.752 7.654-41.762 0-32.050-12.804-61.11-33.576-82.344l0.021 0.021c-22.874-25.484-55.92-41.441-92.693-41.441-10.83 0-21.336 1.384-31.352 3.985l0.864-0.191h-5.722c-30.118 9.336-30.118 9.035-44.273-18.372-17.236-31.193-32.683-67.512-44.377-105.477l-1.101-4.152c-3.915-12.348-1.807-18.673 11.445-24.094 45.171-18.059 76.501-61.451 76.501-112.16 0-0.275-0.001-0.549-0.003-0.823l0 0.042c-0.157-51.84-32.003-96.203-77.176-114.748l-0.829-0.301c-13.553-4.819-15.962-10.842-12.047-23.793 13.962-48.504 31.914-90.674 54.24-130.036l-1.534 2.94c6.024-10.541 11.746-12.649 23.793-7.831 14.648 6.459 31.727 10.219 49.685 10.219 35.285 0 67.18-14.517 90.038-37.904l0.023-0.024c21.532-21.755 34.835-51.691 34.835-84.733 0-19.022-4.409-37.015-12.26-53.011l0.314 0.709c-4.216-9.638-3.012-15.059 6.024-20.48 39.702-23.013 85.609-42.536 133.977-56.195l4.263-1.029c13.252-3.614 14.758 5.12 18.372 13.252 16.261 41.325 53.282 71.221 97.87 77.036l0.614 0.065c6.241 1.121 13.425 1.762 20.759 1.762 40.852 0 77.059-19.886 99.469-50.507l0.242-0.347c7.452-9.232 13.404-20.047 17.264-31.809l0.204-0.718c3.012-8.433 8.132-9.939 16.264-8.132 52.584 13.65 98.681 32.83 141.232 57.456l-2.691-1.437c9.336 5.12 8.433 11.144 4.819 19.576-6.604 14.774-10.451 32.016-10.451 50.158 0 69.362 56.229 125.591 125.591 125.591 18.623 0 36.299-4.053 52.195-11.326l-0.784 0.321c10.24-4.518 15.962-3.012 21.384 6.927 22.212 37.657 40.917 81.17 53.87 127.095l0.944 3.916c2.711 10.24 0 15.36-10.24 19.878-46.208 16.823-78.61 60.371-78.61 111.487 0 0.299 0.001 0.599 0.003 0.898l-0-0.046c-0.106 1.871-0.166 4.060-0.166 6.264 0 49.766 30.792 92.34 74.362 109.71l0.797 0.28c12.951 6.024 16.264 11.746 12.047 25.6-14.446 47.781-32.562 89.199-54.858 127.907l1.55-2.918c-5.421 10.24-10.842 12.348-22.287 8.132-14.209-5.966-30.724-9.432-48.048-9.432-45.354 0-85.159 23.756-107.651 59.503l-0.31 0.527c-11.029 16.816-17.591 37.422-17.591 59.561 0 1.826 0.045 3.642 0.133 5.446l-0.010-0.254zM520.433 711.68c109.44-1.529 197.571-90.604 197.571-200.264 0-110.613-89.669-200.282-200.282-200.282s-200.282 89.669-200.282 200.282c0 0.205 0 0.411 0.001 0.616l-0-0.032c0.498 110.402 90.11 199.707 200.582 199.707 1.166 0 2.329-0.010 3.49-0.030l-0.175 0.002z",
            },
        ],
    ])
}

#[view]
fn addons_icon() -> Node<Msg> {
    option_icon("0 0 1043 1024", "ic_addons", vec![
        path![
            attrs!{
                At::D => "M145.468 679.454c-40.056-39.454-80.715-78.908-120.471-118.664-33.431-33.129-33.129-60.235 0-90.353l132.216-129.807c5.693-5.938 12.009-11.201 18.865-15.709l0.411-0.253c23.492-15.059 41.864-7.529 48.188 18.974 0 7.228 2.711 14.758 3.614 22.287 3.801 47.788 37.399 86.785 82.050 98.612l0.773 0.174c10.296 3.123 22.128 4.92 34.381 4.92 36.485 0 69.247-15.94 91.702-41.236l0.11-0.126c24.858-21.654 40.48-53.361 40.48-88.718 0-13.746-2.361-26.941-6.701-39.201l0.254 0.822c-14.354-43.689-53.204-75.339-99.907-78.885l-0.385-0.023c-18.372-2.409-41.562 0-48.188-23.492s11.445-34.635 24.998-47.887q65.054-62.946 130.409-126.795c32.527-31.925 60.235-32.226 90.353 0 40.659 39.153 80.715 78.908 120.471 118.362 8.348 8.594 17.297 16.493 26.82 23.671l0.587 0.424c8.609 7.946 20.158 12.819 32.846 12.819 24.823 0 45.29-18.653 48.148-42.707l0.022-0.229c3.012-13.252 4.518-26.805 8.734-39.755 12.103-42.212 50.358-72.582 95.705-72.582 3.844 0 7.637 0.218 11.368 0.643l-0.456-0.042c54.982 6.832 98.119 49.867 105.048 104.211l0.062 0.598c0.139 1.948 0.218 4.221 0.218 6.512 0 45.084-30.574 83.026-72.118 94.226l-0.683 0.157c-12.348 3.915-25.299 5.722-37.948 8.433-45.779 9.638-60.235 46.984-30.118 82.824 15.265 17.569 30.806 33.587 47.177 48.718l0.409 0.373c31.925 31.925 64.452 62.946 96.075 94.871 13.698 9.715 22.53 25.511 22.53 43.369s-8.832 33.655-22.366 43.259l-0.164 0.111c-45.176 45.176-90.353 90.353-137.035 134.325-5.672 5.996-12.106 11.184-19.169 15.434l-0.408 0.227c-4.663 3.903-10.725 6.273-17.341 6.273-13.891 0-25.341-10.449-26.92-23.915l-0.012-0.127c-2.019-7.447-3.714-16.45-4.742-25.655l-0.077-0.848c-4.119-47.717-38.088-86.476-82.967-97.721l-0.76-0.161c-9.584-2.63-20.589-4.141-31.947-4.141-39.149 0-74.105 17.956-97.080 46.081l-0.178 0.225c-21.801 21.801-35.285 51.918-35.285 85.185 0 1.182 0.017 2.36 0.051 3.533l-0.004-0.172c1.534 53.671 40.587 97.786 91.776 107.115l0.685 0.104c12.649 2.409 25.901 3.313 38.249 6.626 22.588 6.325 30.118 21.685 18.372 41.864-4.976 8.015-10.653 14.937-17.116 21.035l-0.051 0.047c-44.875 44.574-90.353 90.353-135.228 133.12-10.241 14.067-26.653 23.106-45.176 23.106s-34.935-9.039-45.066-22.946l-0.111-0.159c-40.659-38.852-80.414-78.908-120.471-118.362z",
            },
        ],
    ])
}

#[view]
fn remote_control_icon() -> Node<Msg> {
    option_icon("0 0 1022 1024", "ic_remote", vec![
        path![
            attrs!{
                At::D => "M624.941 175.586c-13.057-13.353-31.097-21.786-51.107-22.285l-0.093-0.002c-0.040-0-0.088-0-0.135-0-20.908 0-39.826 8.522-53.469 22.282l-0.005 0.005c-16.866 16.565-33.732 33.431-50.296 50.296l-439.718 439.115c-36.141 37.346-40.96 76.8-8.433 110.833q112.038 115.652 227.388 227.388c12.53 12.789 29.979 20.717 49.28 20.717 20.031 0 38.068-8.54 50.668-22.177l0.042-0.046c14.155-12.649 27.106-26.805 40.659-40.056 150.588-150.588 301.176-301.176 451.765-451.765 37.948-37.948 40.659-77.402 6.024-112.941-74.391-74.391-148.781-147.576-222.569-221.365zM180.706 776.734c-1.178 0.096-2.55 0.151-3.936 0.151-28.443 0-51.501-23.058-51.501-51.501s23.058-51.501 51.501-51.501c1.385 0 2.757 0.055 4.115 0.162l-0.179-0.011c0.902-0.058 1.955-0.091 3.017-0.091 27.778 0 50.296 22.518 50.296 50.296 0 0.244-0.002 0.487-0.005 0.73l0-0.037c0.101 1.169 0.158 2.53 0.158 3.904 0 26.614-21.575 48.188-48.188 48.188-1.86 0-3.695-0.105-5.499-0.31l0.221 0.020zM297.562 897.205c-28.62-0.424-51.657-23.724-51.657-52.405 0-28.945 23.465-52.41 52.41-52.41 0.265 0 0.529 0.002 0.793 0.006l-0.040-0c0.116-0.001 0.254-0.002 0.391-0.002 27.944 0 50.598 22.653 50.598 50.598 0 1.060-0.033 2.113-0.097 3.156l0.007-0.143c0.012 0.399 0.018 0.868 0.018 1.339 0 27.113-21.979 49.092-49.092 49.092-1.278 0-2.545-0.049-3.799-0.145l0.167 0.010zM297.562 660.781c-0.703 0.036-1.526 0.057-2.354 0.057-26.78 0-48.489-21.709-48.489-48.489 0-1.717 0.089-3.414 0.263-5.085l-0.018 0.209c-0.027-0.633-0.043-1.376-0.043-2.123 0-29.275 23.732-53.007 53.007-53.007 0.439 0 0.876 0.005 1.312 0.016l-0.065-0.001c27.941 2.397 49.712 25.668 49.712 54.025 0 0.489-0.006 0.977-0.019 1.464l0.002-0.072c0.010 0.372 0.015 0.81 0.015 1.249 0 28.111-22.788 50.899-50.899 50.899-0.747 0-1.49-0.016-2.229-0.048l0.105 0.004zM414.118 777.638c-0.671 0.033-1.456 0.052-2.246 0.052-26.946 0-48.791-21.844-48.791-48.791 0-1.29 0.050-2.569 0.148-3.833l-0.010 0.168c0.13-28.012 22.868-50.67 50.898-50.67 0.954 0 1.901 0.026 2.842 0.078l-0.131-0.006c28.759 0.293 52.137 22.935 53.604 51.369l0.005 0.132c-1.277 28.259-24.497 50.686-52.956 50.686-1.077 0-2.147-0.032-3.209-0.096l0.146 0.007zM553.261 621.628c-83.676-0.427-151.343-68.359-151.343-152.094 0-84 68.096-152.096 152.096-152.096 0.265 0 0.529 0.001 0.794 0.002l-0.041-0c0.006-0 0.013-0 0.019-0 83.168 0 150.588 67.421 150.588 150.588 0 0.847-0.007 1.693-0.021 2.537l0.002-0.127c-0.169 83.040-67.525 150.292-150.588 150.292-0.424 0-0.847-0.002-1.27-0.005l0.065 0zM970.692 128.602c-44.849-66.251-113.901-113.214-194.197-128.31l-1.869-0.292c-39.454 0-57.224 9.939-57.826 31.925 0 26.805 16.565 35.84 39.454 39.153 25.97 3.55 49.371 12.055 70.092 24.554l-0.821-0.46c65.543 35.169 112.467 97.757 125.675 172.009l0.217 1.469c3.614 21.986 13.553 37.948 38.249 37.647s34.334-17.468 34.334-44.574c-3.974-51.025-23.438-96.864-53.636-133.53l0.328 0.41zM738.184 118.362c-20.48 0-36.442 7.831-40.056 28.612-0.555 2.311-0.873 4.965-0.873 7.693 0 17.785 13.522 32.411 30.846 34.159l0.144 0.012c54.205 8.944 96.459 50.995 105.606 104.354l0.107 0.757c2.215 17.355 16.892 30.635 34.671 30.635 2.116 0 4.188-0.188 6.2-0.548l-0.212 0.031c16.96-1.407 30.192-15.519 30.192-32.722 0-0.779-0.027-1.552-0.081-2.317l0.006 0.103c-1.205-81.016-91.558-170.767-166.551-170.767zM555.671 386.409c-1.188-0.064-2.578-0.1-3.977-0.1-43.58 0-78.908 35.328-78.908 78.908 0 1.094 0.022 2.184 0.066 3.267l-0.005-0.155c-0.13 1.672-0.203 3.62-0.203 5.586 0 42.415 34.385 76.8 76.8 76.8 1.767 0 3.52-0.060 5.258-0.177l-0.235 0.013c1.014 0.047 2.204 0.073 3.399 0.073 43.58 0 78.908-35.328 78.908-78.908 0-1.297-0.031-2.587-0.093-3.868l0.007 0.181c0.018-0.632 0.028-1.376 0.028-2.123 0-43.912-35.598-79.511-79.511-79.511-0.539 0-1.078 0.005-1.615 0.016l0.080-0.001z",
            },
        ],
    ])
}

#[view]
fn play_magnet_link_icon() -> Node<Msg> {
    option_icon("0 0 1024 1024", "ic_magnet", vec![
        path![
            attrs!{
                At::D => "M574.645 745.412c-35.51 28.152-80.966 45.162-130.395 45.162-116.435 0-210.824-94.389-210.824-210.824 0-49.429 17.011-94.885 45.497-130.833l-0.335 0.438 216.546-216.847-153.6-153.6-216.847 216.847c-66.233 74.833-106.675 173.824-106.675 282.261 0 235.697 191.070 426.767 426.767 426.767 108.437 0 207.428-40.443 282.713-107.068l-0.452 0.392 216.847-216.847-153.6-153.6z",
            },
        ],
        path![
            attrs!{
                At::D => "M715.294 357.195l-148.179 24.696c-15.567 2.765-27.235 16.194-27.235 32.348 0 1.871 0.157 3.706 0.457 5.492l-0.027-0.193c2.482 15.543 15.792 27.278 31.844 27.278 1.944 0 3.848-0.172 5.697-0.502l-0.195 0.029 210.824-34.936c15.719-2.32 27.646-15.717 27.646-31.899 0-9.115-3.784-17.345-9.866-23.207l-0.010-0.010-72.282-72.282 148.781-27.708c15.195-2.935 26.514-16.129 26.514-31.966 0-2.223-0.223-4.394-0.648-6.492l0.035 0.209c-1.294-6.665-4.506-12.43-9.030-16.861l-0.005-0.005c-5.953-6.017-14.211-9.743-23.34-9.743-2.4 0-4.74 0.258-6.994 0.747l0.216-0.039-206.908 39.454c-15.206 2.925-26.538 16.125-26.538 31.971 0 9.054 3.699 17.244 9.669 23.141l0.003 0.003z",
            },
        ],
        path![
            attrs!{
                At::D => "M1014.362 567.115l-109.026-109.327c-5.896-5.896-14.042-9.543-23.040-9.543-17.995 0-32.583 14.588-32.583 32.583 0 8.998 3.647 17.144 9.543 23.040l-0-0 109.327 109.026c5.842 5.893 13.94 9.541 22.889 9.541s17.048-3.648 22.887-9.539l0.002-0.002c5.968-5.807 9.67-13.916 9.67-22.889s-3.702-17.083-9.663-22.882l-0.007-0.007z",
            },
        ],
        path![
            attrs!{
                At::D => "M520.132 164.744c5.896 5.896 14.042 9.543 23.040 9.543 17.995 0 32.583-14.588 32.583-32.583 0-8.998-3.647-17.144-9.543-23.040l-0-0-109.327-109.026c-5.807-5.968-13.916-9.67-22.889-9.67s-17.083 3.702-22.882 9.663l-0.007 0.007c-5.893 5.842-9.541 13.94-9.541 22.889s3.648 17.048 9.539 22.887l0.002 0.002z",
            },
        ],
    ])
}

#[view]
fn help_and_feedback_icon() -> Node<Msg> {
    option_icon("0 0 596 1024", "ic_help", vec![
        path![
            attrs!{
                At::D => "M153.901 626.748c-0.109-2.593-0.17-5.636-0.17-8.694 0-38.876 9.994-75.418 27.554-107.196l-0.579 1.142c27.163-40.735 60.963-74.728 100.21-101.279l1.286-0.82c35.996-23.578 66.927-50.967 93.458-82.209l0.509-0.615c14.307-19.568 22.892-44.107 22.892-70.651 0-0.256-0.001-0.512-0.002-0.767l0 0.039c0.056-1.105 0.089-2.399 0.089-3.701 0-25.433-12.266-48.001-31.206-62.111l-0.205-0.146c-23.278-14.516-51.542-23.123-81.817-23.123-3.003 0-5.986 0.085-8.947 0.252l0.411-0.018c-58.672 1.407-114.055 13.896-164.656 35.441l2.924-1.106c-9.304 3.845-20.108 6.077-31.433 6.077-32.637 0-60.943-18.54-74.962-45.663l-0.221-0.471c-5.738-11.020-9.104-24.065-9.104-37.896 0-34.353 20.764-63.856 50.426-76.645l0.542-0.208c70.629-29.343 152.665-46.384 238.681-46.384 0.689 0 1.377 0.001 2.065 0.003l-0.107-0c5.501-0.317 11.936-0.498 18.413-0.498 76.827 0 147.72 25.412 204.729 68.289l-0.874-0.63c50.383 40.518 82.354 102.157 82.354 171.263 0 2.685-0.048 5.359-0.144 8.021l0.011-0.385c0.060 1.998 0.095 4.348 0.095 6.707 0 45.126-12.579 87.314-34.421 123.249l0.594-1.053c-35.418 48.351-78.169 88.896-127.051 120.84l-1.853 1.136c-31.417 21.25-58.543 45.505-82.065 72.94l-0.458 0.547c-10.853 17.093-17.296 37.913-17.296 60.238 0 1.906 0.047 3.801 0.14 5.683l-0.010-0.265c0.005 0.302 0.007 0.659 0.007 1.017 0 33.756-23.631 61.993-55.253 69.069l-0.472 0.089c-8.337 1.775-17.915 2.791-27.73 2.791-12.19 0-24.014-1.568-35.281-4.512l0.968 0.215c-34.846-10.040-60.224-40.827-62.035-77.807l-0.008-0.198zM132.216 908.649c-0.215-2.62-0.338-5.671-0.338-8.751 0-29.936 11.585-57.166 30.517-77.452l-0.061 0.066c21.565-18.849 49.978-30.344 81.075-30.344 2.628 0 5.238 0.082 7.825 0.244l-0.354-0.018c2.212-0.145 4.797-0.227 7.4-0.227 30.734 0 58.779 11.509 80.062 30.451l-0.121-0.106c18.834 20.337 30.388 47.653 30.388 77.668 0 2.768-0.098 5.513-0.291 8.232l0.021-0.365c0.147 2.166 0.231 4.694 0.231 7.242 0 29.918-11.541 57.141-30.414 77.46l0.066-0.071c-20.88 18.848-48.681 30.379-79.175 30.379-2.767 0-5.513-0.095-8.233-0.282l0.367 0.020c-2.381 0.168-5.161 0.263-7.962 0.263-30.753 0-58.814-11.523-80.102-30.486l0.12 0.105c-19.303-20.27-31.18-47.766-31.18-78.036 0-2.108 0.058-4.202 0.171-6.282l-0.013 0.289z",
            },
        ],
    ])
}

#[view]
fn option_icon(view_box: &str, icon: &str, paths: Vec<Node<Msg>>) -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .fill(Color::SecondaryVariant2Light1_90)
            .flex("none")
            .height(rem(1.4))
            .margin(rem(1.3))
            .margin_right("0")
            .width(rem(1.4))
            .overflow(CssOverflow::Visible),
        attrs!{
            At::ViewBox => view_box,
            At::from("icon") => icon,
        },
        paths
    ]
}


