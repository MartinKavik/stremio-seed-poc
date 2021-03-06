use seed::{prelude::*, *};
use seed_hooks::{*, topo::nested as view};
use seed_styles::{em, pc, rem, Style};
use seed_styles::*;
use web_sys::HtmlElement;
use std::rc::Rc;
use std::array;
use enclose::enc;
use serde::Serialize;
use crate::{PageId, Context, Actions, Events};
use crate::styles::{self, themes::{Color, Breakpoint}, global};
use stremio_core::types::resource::{Stream, StreamSource};
use stremio_core::models::player::Selected as PlayerSelected;
use stremio_core::runtime::msg::{Action, ActionLoad, Msg as CoreMsg, Internal};
use super::Msg;

#[view]
pub fn mute_button(muted: bool, volume: u32) -> Node<Msg> {
    div![
        C!["control-bar-button", "button-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex(CssFlex::None)
            .height(rem(4))
            .justify_content(CssJustifyContent::Center)
            .width(rem(4))
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => "-1",
            At::Title => if volume == 0 { "Unmute" } else { "Mute" },
        },
        ev(Ev::Click, |_| Msg::ToggleMute),
        if muted {
            volume_0_icon()
        } else {
            match volume {
                0..=33 => volume_1_icon(),
                34..=66 => volume_2_icon(),
                _ => volume_3_icon(),
            }
        }
    ]
}

#[view]
fn volume_0_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .fill(hsl(0, 0, 100))
            .flex(CssFlex::None)
            .height(rem(2))
            .width(rem(3)),
        attrs!{
            At::ViewBox => "0 0 1234 1024",
            At::from("icon") => "ic_volume0",
        },
        path![
            attrs!{
                At::D => "M903.529 512l111.736-112.038c4.905-4.72 7.951-11.341 7.951-18.673s-3.047-13.953-7.943-18.665l-0.008-0.008c-4.789-4.756-11.388-7.695-18.673-7.695s-13.884 2.939-18.674 7.697l0.002-0.002-111.736 112.038-113.242-113.242c-4.579-3.879-10.554-6.237-17.079-6.237-14.637 0-26.504 11.866-26.504 26.504 0 6.525 2.358 12.5 6.269 17.118l-0.032-0.039 112.038 113.242-112.038 112.038c-5.037 4.884-8.164 11.715-8.164 19.275 0 14.822 12.015 26.837 26.837 26.837 7.261 0 13.849-2.884 18.68-7.568l-0.007 0.007 112.038-112.038 111.736 112.038c4.789 4.756 11.388 7.695 18.673 7.695s13.884-2.939 18.674-7.697l-0.002 0.002c4.905-4.72 7.951-11.341 7.951-18.673s-3.047-13.953-7.943-18.665l-0.008-0.008z",
            }
        ],
        path![
            attrs!{
                At::D => "M597.835 512q0-193.656 0-387.614c0-17.167 0-36.141-18.372-45.176s-33.129 4.518-47.285 14.758l-13.854 10.541c-94.268 76.198-188.838 153.299-282.202 231.002-14.854 13.503-34.678 21.771-56.434 21.771-1.337 0-2.667-0.031-3.989-0.093l0.187 0.007c-38.852-1.807-77.402 0-116.254 0-2.637-0.482-5.672-0.757-8.771-0.757-28.277 0-51.2 22.923-51.2 51.2 0 2.071 0.123 4.113 0.362 6.12l-0.024-0.243q0 98.485 0 196.969c-0.218 1.779-0.342 3.838-0.342 5.926 0 28.443 23.058 51.501 51.501 51.501 3.208 0 6.347-0.293 9.393-0.855l-0.317 0.048c41.562 0 83.125 0 124.687 0 1.099-0.069 2.383-0.109 3.676-0.109 15.42 0 29.526 5.626 40.378 14.936l-0.083-0.070c98.786 81.016 198.475 161.732 297.864 242.447 15.661 12.348 30.118 30.118 53.609 20.179s18.673-33.732 18.974-53.308z",
            }
        ],
    ]
}

#[view]
fn volume_1_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .fill(hsl(0, 0, 100))
            .flex(CssFlex::None)
            .height(rem(2))
            .width(rem(3)),
        attrs!{
            At::ViewBox => "0 0 1234 1024",
            At::from("icon") => "ic_volume1",
        },
        path![
            attrs!{
                At::D => "M597.835 512q0-193.656 0-387.614c0-17.167 0-36.141-18.372-45.176s-33.129 4.518-47.285 14.758l-13.854 10.541c-94.268 76.198-188.838 153.299-282.202 231.002-14.854 13.503-34.678 21.771-56.434 21.771-1.337 0-2.667-0.031-3.989-0.093l0.187 0.007c-38.852-1.807-77.402 0-116.254 0-2.637-0.482-5.672-0.757-8.771-0.757-28.277 0-51.2 22.923-51.2 51.2 0 2.071 0.123 4.113 0.362 6.12l-0.024-0.243q0 98.485 0 196.969c-0.218 1.779-0.342 3.838-0.342 5.926 0 28.443 23.058 51.501 51.501 51.501 3.208 0 6.347-0.293 9.393-0.855l-0.317 0.048c41.562 0 83.125 0 124.687 0 1.099-0.069 2.383-0.109 3.676-0.109 15.42 0 29.526 5.626 40.378 14.936l-0.083-0.070c98.786 81.016 198.475 161.732 297.864 242.447 15.661 12.348 30.118 30.118 53.609 20.179s18.673-33.732 18.974-53.308z",
            }
        ],
        path![
            attrs!{
                At::D => "M873.412 473.449c-11.316-70.608-48.697-130.889-101.772-171.808l-0.628-0.465c-5.401-5.181-12.746-8.371-20.836-8.371-9.112 0-17.279 4.047-22.802 10.441l-0.033 0.039c-10.541 12.951-6.626 30.118 9.035 43.068 51.567 39.981 84.44 101.942 84.44 171.581 0 20.284-2.789 39.917-8.004 58.536l0.365-1.524c-10.544 44.811-37.899 81.899-74.888 105l-0.708 0.412c-9.308 4.683-15.583 14.16-15.583 25.102 0 5.422 1.54 10.484 4.208 14.772l-0.069-0.119c5.489 8.055 14.619 13.274 24.968 13.274 6.827 0 13.124-2.272 18.176-6.101l-0.075 0.055 4.819-4.518c63.36-44.040 104.316-116.467 104.316-198.46 0-2.547-0.040-5.085-0.118-7.613l0.009 0.369c-0.005-15.481-1.761-30.549-5.079-45.021l0.261 1.35z",
            }
        ],
    ]
}

#[view]
fn volume_2_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .fill(hsl(0, 0, 100))
            .flex(CssFlex::None)
            .height(rem(2))
            .width(rem(3)),
        attrs!{
            At::ViewBox => "0 0 1234 1024",
            At::from("icon") => "ic_volume2",
        },
        path![
            attrs!{
                At::D => "M597.835 512q0-193.656 0-387.614c0-17.167 0-36.141-18.372-45.176s-33.129 4.518-47.285 14.758l-13.854 10.541c-94.268 76.198-188.838 153.299-282.202 231.002-14.854 13.503-34.678 21.771-56.434 21.771-1.337 0-2.667-0.031-3.989-0.093l0.187 0.007c-38.852-1.807-77.402 0-116.254 0-2.637-0.482-5.672-0.757-8.771-0.757-28.277 0-51.2 22.923-51.2 51.2 0 2.071 0.123 4.113 0.362 6.12l-0.024-0.243q0 98.485 0 196.969c-0.218 1.779-0.342 3.838-0.342 5.926 0 28.443 23.058 51.501 51.501 51.501 3.208 0 6.347-0.293 9.393-0.855l-0.317 0.048c41.562 0 83.125 0 124.687 0 1.099-0.069 2.383-0.109 3.676-0.109 15.42 0 29.526 5.626 40.378 14.936l-0.083-0.070c98.786 81.016 198.475 161.732 297.864 242.447 15.661 12.348 30.118 30.118 53.609 20.179s18.673-33.732 18.974-53.308z",
            }
        ],
        path![
            attrs!{
                At::D => "M1050.504 427.369c-20.607-113.854-80.336-211.092-164.487-279.419l-0.858-0.675c-19.275-15.962-37.346-16.264-49.694-1.807s-9.638 32.527 10.24 48.489c22.45 18.023 42.212 37.785 59.656 59.49l0.579 0.745c57.394 71.924 92.096 164.158 92.096 264.497 0 13.55-0.633 26.953-1.871 40.179l0.128-1.693c-3.584 113.985-61.903 213.626-149.426 273.916l-1.162 0.757c-9.666 5.144-16.136 15.154-16.136 26.675 0 6.739 2.213 12.961 5.952 17.979l-0.057-0.080c5.476 8.151 14.66 13.443 25.080 13.443 7.594 0 14.532-2.811 19.83-7.449l-0.035 0.030c15.072-9.568 28.155-19.491 40.356-30.38l-0.299 0.262c98.485-87.944 139.746-199.078 139.746-339.125-0.255-30.545-3.753-60.135-10.165-88.622l0.527 2.786z",
            }
        ],
        path![
            attrs!{
                At::D => "M886.362 470.739c-11.887-76.996-52.81-142.713-111.017-186.809l-0.719-0.522c-5.541-6.197-13.559-10.079-22.484-10.079-9.881 0-18.65 4.758-24.142 12.108l-0.057 0.079c-3.656 5.124-5.845 11.513-5.845 18.414 0 11.559 6.143 21.683 15.342 27.286l0.141 0.080c55.223 43.106 90.395 109.678 90.395 184.465 0 22.576-3.205 44.403-9.185 65.052l0.409-1.649c-11.296 48.267-40.733 88.224-80.557 113.101l-0.761 0.443c-10.032 5.028-16.798 15.23-16.798 27.012 0 5.829 1.656 11.272 4.524 15.883l-0.074-0.128c5.931 8.67 15.775 14.286 26.931 14.286 7.33 0 14.094-2.425 19.533-6.516l-0.083 0.060 6.024-3.614c68.304-47.513 112.451-125.618 112.451-214.033 0-2.684-0.041-5.359-0.121-8.023l0.009 0.39c0.061-1.894 0.096-4.121 0.096-6.356 0-14.483-1.46-28.624-4.242-42.285l0.231 1.356z",
            }
        ],
    ]
}

#[view]
fn volume_3_icon() -> Node<Msg> {
    svg![
        C!["icon"],
        s()
            .fill(hsl(0, 0, 100))
            .flex(CssFlex::None)
            .height(rem(2))
            .width(rem(3)),
        attrs!{
            At::ViewBox => "0 0 1234 1024",
            At::from("icon") => "ic_volume3",
        },
        path![
            attrs!{
                At::D => "M597.835 516.216q0-193.958 0-387.614c0-17.167 0-36.141-18.372-45.176s-33.129 4.216-47.285 14.758l-13.854 10.541c-93.967 76.8-188.536 153.299-281.901 230.701-14.772 13.681-34.613 22.074-56.414 22.074-1.344 0-2.681-0.032-4.009-0.095l0.188 0.007c-40.056-1.807-78.607-0-115.953-0-2.709-0.505-5.825-0.794-9.009-0.794-28.443 0-51.501 23.058-51.501 51.501 0 1.871 0.1 3.719 0.294 5.538l-0.020-0.226q0 98.485 0 196.969c-0.241 1.872-0.379 4.037-0.379 6.234 0 28.443 23.058 51.501 51.501 51.501 3.221 0 6.373-0.296 9.43-0.861l-0.317 0.049c41.562 0 83.125 0 124.687 0 0.944-0.050 2.049-0.079 3.161-0.079 15.582 0 29.853 5.608 40.907 14.915l-0.096-0.079c98.786 81.318 198.475 162.033 297.864 242.447 15.661 12.649 30.118 30.118 53.609 20.48s18.673-33.732 18.974-53.609z",
            }
        ],
        path![
            attrs!{
                At::D => "M1056.226 512c0.040-2.032 0.062-4.427 0.062-6.828 0-27.557-2.985-54.417-8.648-80.274l0.454 2.471c-20.736-112.48-79.851-208.501-162.993-276.114l-0.847-0.667c-19.275-15.661-37.045-15.962-49.694-1.807s-9.638 32.226 10.24 48.489c21.936 17.519 41.211 36.794 58.168 58.002l0.562 0.727c57.34 71.651 92.017 163.607 92.017 263.663 0 13.207-0.604 26.272-1.786 39.171l0.123-1.657c-4.035 113.334-62.261 212.262-149.419 272.108l-1.169 0.758c-9.322 5.241-15.516 15.068-15.516 26.341 0 6.363 1.973 12.265 5.341 17.128l-0.065-0.1c5.502 7.948 14.571 13.088 24.841 13.088 7.566 0 14.48-2.79 19.77-7.396l-0.036 0.031c15.073-9.548 28.156-19.471 40.347-30.373l-0.29 0.255c97.882-86.438 138.842-196.969 138.541-337.016z",
            }
        ],
        path![
            attrs!{
                At::D => "M885.158 471.040c-11.898-76.433-52.456-141.682-110.13-185.613l-0.703-0.514c-5.069-3.928-11.519-6.297-18.522-6.297-16.784 0-30.391 13.606-30.391 30.391 0 9.781 4.621 18.483 11.799 24.042l0.070 0.052c54.99 42.709 90.037 108.838 90.037 183.153 0 22.4-3.184 44.057-9.125 64.542l0.406-1.634c-10.991 48.32-40.388 88.344-80.256 113.104l-0.76 0.44c-9.905 5.070-16.567 15.205-16.567 26.897 0 5.877 1.683 11.361 4.594 15.995l-0.074-0.125c5.954 8.471 15.685 13.939 26.692 13.939 7.303 0 14.044-2.407 19.472-6.47l-0.085 0.061 6.024-3.614c67.758-47.063 111.56-124.493 111.56-212.153 0-2.816-0.045-5.621-0.135-8.415l0.010 0.408c0.087-2.237 0.136-4.864 0.136-7.502 0-14.4-1.475-28.455-4.283-42.022l0.231 1.336z",
            }
        ],
        path![
            attrs!{
                At::D => "M993.882 5.722c-4.81-3.4-10.797-5.435-17.258-5.435-9.423 0-17.835 4.327-23.357 11.103l-0.043 0.055c-3.4 4.81-5.435 10.797-5.435 17.258 0 9.423 4.327 17.835 11.103 23.357l0.055 0.043c129.72 110.558 212.383 272.999 215.634 454.822l0.008 0.557c0 260.518-124.386 391.529-217.751 465.016-5.549 5.466-8.987 13.062-8.987 21.461 0 15.875 12.282 28.881 27.862 30.034l0.099 0.006c6.849-0.093 13.128-2.459 18.137-6.375l-0.066 0.050c60.235-47.586 240.941-191.849 240.941-510.193-2.605-201.84-95.391-381.493-239.828-500.866l-1.113-0.894z",
            }
        ],
    ]
}
