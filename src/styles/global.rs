use seed_styles::{px, em, pc, rem};
use seed_styles::*;
use crate::styles::themes::{self, Breakpoint, Color, get_color_value};

mod font_faces;
use font_faces::GlobalStyleFontFaces;

pub const LANDSCAPE_SHAPE_RATIO: f64 = 0.5625;
pub const POSTER_SHAPE_RATIO: f64 = 1.464;
pub const SCROLL_BAR_WIDTH: &str = "6px";
pub const FOCUS_OUTLINE_SIZE: &str = "2px";
pub const COLOR_FACEBOOK: &str = "#4267b2";
pub const COLOR_TWITTER: &str = "#1DA1F2";
pub const ITEM_SIZE: &str = "28rem";
pub const NAV_BAR_SIZE: &str = "3.2rem";
pub const HORIZONTAL_NAV_BAR_SIZE: &str = "4rem";
pub const VERTICAL_NAV_BAR_SIZE: &str = "5.2rem";
// @TODO 4rem is HORIZONTAL_NAV_BAR_SIZE - refactor
pub const SEARCH_BAR_SIZE: &str = "calc(4rem - 1.2rem)";
pub const THUMB_SIZE: &str = "1.3rem";
pub const TRACK_SIZE: &str = "0.4rem";

pub fn image_url(image: &str) -> String {
    format!("/images/{}", image)
}

pub fn init() {
    load_app_themes(&[themes::default_color_theme, themes::default_breakpoint_theme]);

    GlobalStyle::new()
        .add_font_faces()
        .style(
            "html",
            s()
                .width(pc(100))
                .height(pc(100))
                .min_width(px(640))
                .min_height(px(480))
                .font_family("'Roboto', 'sans-serif'")
                .overflow(CssOverflow::Auto)
        )
        .style(
            "html",
            s()
                .only_and_above(Breakpoint::XLarge)
                .font_size(px(18))
        )
        .style(
            "html",
            s()
                .only_and_below(Breakpoint::XLarge)
                .font_size(px(16))
        )
        .style(
            "html",
            s()
                .only_and_below(Breakpoint::Medium)
                .font_size(px(15))
        )
        .style(
            "html",
            s()
                .only_and_below(Breakpoint::Small)
                .font_size(px(14))
        )
        .style(
            "body",
            s()
                .width(pc(100))
                .height(pc(100))
        )
        .style(
            "svg",
            s()
                .overflow(CssOverflow::Visible)
        )
        .style(
            "::-webkit-scrollbar",
            s()
                .width(SCROLL_BAR_WIDTH)
        )
        .style(
            "::-webkit-scrollbar-thumb",
            s()
                .background_color(Color::SecondaryLighter80)
        )
        .style(
            "::-webkit-scrollbar-track",
            s()
            .background_color(Color::SecondaryLight)
        )
        .style(
            "*",
            s()
                .raw("appearance: none;")
                .margin("0")
                .padding("0")
                .box_sizing(CssBoxSizing::BorderBox)
                .font_size(rem(1))
                .line_height(em(1.2))
                .font_family(CssFontFamily::Inherit)
                .border("none")
                .outline(CssOutline::None)
                .list_style("none")
                .user_select("none")
                .text_decoration(CssTextDecoration::None)
                .raw("appearance: none;")
                .background("none")
                .box_shadow(CssBoxShadow::None)
                .overflow(CssOverflow::Hidden)
                .word_break("break-word")
                .raw("scrollbar-width: thin;")
                .raw(format!(
                    "scrollbar-color: {} {};", 
                    get_color_value(Color::SecondaryVariant2Light1), 
                    get_color_value(Color::BackgroundDark2)
                ).as_str())
        )
        .style(
            "#app",
            s()
                .position(CssPosition::Relative)
                .z_index(CssZIndex::Integer(0))
                .width(pc(100))
                .height(pc(100))
        )
        .activate_init_styles();
}
