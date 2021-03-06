use seed::{prelude::*, *};
use seed_hooks::{*, topo::nested as view};
use seed_styles::{em, pc, rem, Style};
use seed_styles::*;
use web_sys::HtmlElement;
use regex::Regex;
use std::rc::Rc;
use stremio_core::runtime::msg::{Msg as CoreMsg, Action, ActionCtx, Event};
use stremio_core::models::ctx::CtxError;
use stremio_core::types::api::{AuthRequest, APIError};
use crate::{multi_select, Msg as RootMsg, Context, PageId, Actions, Urls as RootUrls};
use crate::basic_layout::{basic_layout, BasicLayoutArgs};
use crate::styles::{self, themes::{Color, Breakpoint}, global};

fn on_click_not_implemented() -> EventHandler<Msg> {
    ev(Ev::Click, |_| { window().alert_with_message("Not implemented!").unwrap(); })
}

// ---- url parts ----

const LOGIN: &str = "login";

// ------ ------
//     Init
// ------ ------

pub fn init(
    mut url: Url,
    model: &mut Option<Model>,
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
) -> Option<PageId> {
    let base_url = url.to_hash_base_url();

    orders.after_next_render(|_| Msg::FocusEmail);

    let form_type = match url.next_hash_path_part() {
        None => FormType::SignUp,
        Some(LOGIN) => FormType::LogIn,
        _ => return None,
    };

    let model = model.get_or_insert_with(move || Model {
        base_url,
        _core_msg_sub_handle: orders.subscribe_with_handle(Msg::CoreMsg),
        form_type,
        form_data: FormData::default(),
        email_input: ElRef::new(),
        form_error: None,
    });
    model.form_type = form_type;
    Some(PageId::Intro)
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    base_url: Url,
    _core_msg_sub_handle: SubHandle,
    form_type: FormType,
    form_data: FormData,
    email_input: ElRef<HtmlElement>,
    form_error: Option<FormError>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FormType {
    SignUp,
    LogIn,
} 

#[derive(Default)]
struct FormData {
    email: String,
    password: String,
    confirm_password: String,
    terms_and_conditions_checked: bool,
    privacy_policy_checked: bool,
    marketing_checked: bool,
}

enum FormError {
    InvalidEmail,
    InvalidPassword,
    APIError(APIError),
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    pub fn root(self) -> Url {
        self.base_url()
    }
    pub fn login(self) -> Url {
        self.base_url().add_hash_path_part(LOGIN)
    }
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    CoreMsg(Rc<CoreMsg>),
    EmailChanged(String),
    PasswordChanged(String),
    ConfirmPasswordChanged(String),
    TermsAndConditionsClicked,
    PrivacyPolicyClicked,
    MarketingClicked,
    FocusEmail,
    Login,
}

pub fn update(msg: Msg, model: &mut Model, context: &mut Context, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::CoreMsg(core_msg) => {
            match core_msg.as_ref() {
                CoreMsg::Event(Event::UserAuthenticated {..}) => {
                    model.form_data = FormData::default();
                    orders.request_url(RootUrls::new(&context.root_base_url).root());
                }
                CoreMsg::Event(Event::Error {error: CtxError::API(api_error), ..}) => {
                    model.form_error = Some(FormError::APIError(api_error.to_owned()));
                }
                _ => (),
            }
        }
        Msg::EmailChanged(email) => {
            model.form_data.email = email;
        }
        Msg::PasswordChanged(password) => {
            model.form_data.password = password;
        }
        Msg::ConfirmPasswordChanged(confirm_password) => {
            model.form_data.confirm_password = confirm_password;
        }
        Msg::TermsAndConditionsClicked => {
            model.form_data.terms_and_conditions_checked = !model.form_data.terms_and_conditions_checked;
        }
        Msg::PrivacyPolicyClicked => {
            model.form_data.privacy_policy_checked = !model.form_data.privacy_policy_checked;
        }
        Msg::MarketingClicked => {
            model.form_data.marketing_checked = !model.form_data.marketing_checked;
        }
        Msg::FocusEmail => {
            model.email_input.get().map(|input| input.focus().expect("focus email input"));
        }
        Msg::Login => {
            let email = &model.form_data.email;
            let password = &model.form_data.password;

            // Basic regex from https://www.w3schools.com/tags/att_input_pattern.asp
            // @TODO replace with RFC 5321/5322? ; compile the regex only once in a lazy static?
            let email_regex = Regex::new(r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap();

            if email.len() == 0 || not(email_regex.is_match(email)) {
                model.form_error = Some(FormError::InvalidEmail);
                return
            }

            if password.len() == 0 {
                model.form_error = Some(FormError::InvalidPassword);
                return
            }

            let auth_request = AuthRequest::Login {
                email: email.to_owned(),
                password: password.to_owned(),
                facebook: false,
            };
            orders.notify(Actions::UpdateCoreModel(Rc::new(CoreMsg::Action(Action::Ctx(
                ActionCtx::Authenticate(auth_request)
            )))));
        }
    }
}

// ------ ------
//     View
// ------ ------

#[view]
pub fn view(model: &Model) -> Node<Msg> {
    div![
        C!["route-content"],
        s()
            .bottom("0")
            .left("0")
            .overflow(CssOverflow::Hidden)
            .position(CssPosition::Absolute)
            .right("0")
            .top("0")
            .z_index("0"),
        div![
            C!["intro-container"],
            s()
                .background(format!("linear-gradient(hsla(243,24.4%,6%,0.8),hsla(243,24.4%,6%,0.8)),url({})", global::image_url("intro_background.jpg")).as_str())
                .background_origin("border-box")
                .background_position("center")
                .background_repeat(CssBackgroundRepeat::NoRepeat)
                .background_size("cover")
                .display(CssDisplay::Flex)
                .flex_direction(CssFlexDirection::Row)
                .height(pc(100))
                .overflow_y(CssOverflowY::Auto)
                .width(pc(100)),
            form_container(model),
        ]
    ]
}

#[view]
fn form_container(model: &Model) -> Node<Msg> {
    div![
        C!["form-container"],
        s()
            .flex(CssFlex::None)
            .margin(CssMargin::Auto)
            .padding("2rem 0")
            .width(rem(28)),
        logo_container(),
        facebook_button(),
        IF!(model.form_type == FormType::SignUp => {
            login_form_button(&model.base_url)
        }),
        email_input(&model.form_data.email, &model.email_input),
        password_input(&model.form_data.password),
        IF!(model.form_type == FormType::SignUp => {
            vec![
                confirm_password_input(&model.form_data.confirm_password),
                terms_and_conditions_checkbox(model.form_data.terms_and_conditions_checked),
                privacy_policy_checkbox(model.form_data.privacy_policy_checked),
                marketing_checkbox(model.form_data.marketing_checked),
                sign_up_button(),
                guest_login_button(),
            ]
        }),
        IF!(model.form_type == FormType::LogIn => {
            nodes![
                forgot_password_button(),
                model.form_error.as_ref().map(error_message),
                login_button(),
                sign_up_with_email_button(&model.base_url),
            ]
        }),
    ]
}

#[view]
fn logo_container() -> Node<Msg> {
    div![
        C!["logo-container"],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::Center)
            .margin_bottom(rem(3)),
        img![
            C!["logo"],
            s()
                .flex(CssFlex::None)
                .height(rem(4))
                .margin_right(rem(1))
                .opacity("0.9")
                .width(rem(4)),
            attrs!{
                At::Src => global::image_url("stremio_symbol.png"),
                At::Alt => " ",
            },
        ],
        svg![
            C!["name"],
            s()
                .fill(Color::SurfaceDark4_90)
                .flex(CssFlex::None)
                .height(rem(4))
                .width(rem(8))
                .overflow(CssOverflow::Visible),
            attrs!{
                At::ViewBox => "0 0 7866 1024",
                At::from("icon") => "ic_stremio",
            },
            path![
                attrs!{
                    At::D => "M837.873 169.864q-28.009-13.553-64.151-30.118c-22.624-9.744-50.3-19.218-78.771-26.732l-4.353-0.977c-28.074-7.414-63.793-14.472-100.14-19.573l-5.272-0.606c-37.207-5.188-80.194-8.15-123.872-8.15-2.087 0-4.172 0.007-6.256 0.020l0.32-0.002c-0.161-0-0.352-0-0.544-0-54.215 0-107.483 4.051-159.519 11.867l5.861-0.724c-42.371 5.197-80.924 15.787-116.936 31.178l2.79-1.060c-28.605 12.112-52.506 30.502-70.798 53.545l-0.28 0.366c-15.973 20.249-25.621 46.135-25.621 74.274 0 0.782 0.007 1.563 0.022 2.341l-0.002-0.117c-0.006 0.407-0.010 0.888-0.010 1.369 0 24.225 8.774 46.402 23.317 63.525l-0.117-0.141c17.379 20.037 38.268 36.5 61.79 48.554l1.156 0.538c26.819 14.561 58.146 27.754 90.831 37.857l3.738 0.995c35.238 11.746 72.885 22.588 112.64 32.828s80.715 20.781 122.88 30.118 82.824 21.986 122.579 34.033c46.328 13.952 83.795 27.773 120.268 43.463l-7.327-2.804c35.425 14.928 65.954 32.493 93.967 53.258l-1.205-0.853c25.043 18.687 45.915 41.382 62.045 67.332l0.6 1.036c14.623 24.358 23.271 53.749 23.271 85.16 0 1.826-0.029 3.646-0.087 5.458l0.007-0.264c0.032 1.384 0.051 3.014 0.051 4.649 0 33.215-7.573 64.662-21.087 92.707l0.556-1.28c-14.602 27.645-33.684 50.958-56.542 69.869l-0.38 0.305c-23.761 19.665-51.113 36.184-80.722 48.329l-2.101 0.763c-28.649 12.002-62.535 22.403-97.615 29.512l-3.58 0.606c-31.306 7.054-69.27 12.812-107.951 16.049l-3.183 0.215c-38.551 3.012-76.198 4.518-113.242 4.518-2.867 0.022-6.255 0.035-9.647 0.035-78.304 0-155.018-6.805-229.584-19.857l7.927 1.149c-76.402-13.243-142.954-30.488-207.299-52.495l9.426 2.801v-109.026c59 36.068 127.712 63.547 200.98 78.507l4.121 0.703c67.090 13.604 144.206 21.39 223.141 21.39 1.388 0 2.774-0.002 4.161-0.007l-0.214 0.001c1.71 0.009 3.731 0.015 5.754 0.015 52.773 0 104.701-3.618 155.55-10.618l-5.897 0.665c45.977-6.108 87.633-17.618 126.719-34.035l-3.237 1.206c32.133-12.5 59.223-31.847 80.503-56.371l0.212-0.25c18.75-22.313 30.141-51.355 30.141-83.057 0-0.871-0.009-1.74-0.026-2.607l0.002 0.13c0.009-0.532 0.014-1.16 0.014-1.789 0-26.243-8.718-50.449-23.415-69.881l0.211 0.291c-17.1-21.713-37.911-39.641-61.608-53.069l-1.037-0.54c-26.469-15.544-57.139-29.511-89.285-40.254l-3.477-1.008c-35.539-11.746-72.885-22.889-112.339-33.129s-80.113-20.179-122.278-30.118-83.125-19.878-123.482-30.72c-46.23-13.107-83.567-26.143-119.966-41.004l7.627 2.755c-35.44-14.215-65.997-31.108-94.104-51.208l1.341 0.912c-25.1-18.117-45.983-40.342-62.074-65.89l-0.571-0.971c-14.591-24.041-23.227-53.1-23.227-84.176 0-1.219 0.013-2.435 0.040-3.647l-0.003 0.181c-0.018-1.002-0.029-2.184-0.029-3.369 0-30.491 6.897-59.371 19.215-85.166l-0.513 1.194c12.782-25.961 29.897-47.89 50.63-65.732l0.269-0.226c21.804-18.515 46.84-34.184 73.967-45.938l1.93-0.745c26.41-11.926 57.494-22.29 89.768-29.499l3.295-0.618c28.926-6.814 64.062-12.455 99.88-15.732l3.122-0.231c35.84-3.313 70.776-4.819 105.713-4.819s73.788 0 107.821 4.819 66.861 7.831 98.485 13.252 62.645 11.746 92.762 18.974 60.235 14.758 90.353 22.889z",
                }
            ],
            path![
                attrs!{
                    At::D => "M1592.922 106.315v898.409h-87.040v-898.409h-446.946v-81.318h979.727v81.318z",
                }
            ],
            path![
                attrs!{
                    At::D => "M2899.727 86.739c-25.552-4.851-54.944-7.626-84.984-7.626-3.371 0-6.733 0.035-10.087 0.104l0.501-0.008c-1.031-0.007-2.25-0.011-3.469-0.011-52.336 0-103.001 7.15-151.073 20.526l3.955-0.939c-49.469 13.446-92.597 31.134-132.875 53.354l3.068-1.552c-42.347 23.797-78.93 49.725-112.535 78.987l0.799-0.681c-34.053 29.519-64.79 60.734-92.93 94.206l-1.037 1.267v680.358h-90.353v-979.727h90.353v199.68c31.569-32.247 65.5-61.879 101.621-88.729l2.285-1.624c34.537-25.728 73.656-49.411 114.982-69.227l4.284-1.85c37.045-18.267 80.379-34.16 125.547-45.419l4.562-0.962c42.070-10.716 90.368-16.866 140.101-16.866 0.298 0 0.597 0 0.895 0.001l-0.046-0c16.565 0 30.118 0 44.875 0s25.6 0 36.744 2.711l30.118 3.915c9.939 0 20.179 3.012 30.118 5.12v88.847c-11.746-4.216-31.021-10.24-55.416-13.854z",
                }
            ],
            path![
                attrs!{
                    At::D => "M4109.553 953.525q-60.235 18.372-111.134 31.624c-34.334 8.734-68.066 15.962-101.496 21.384s-67.162 9.638-101.496 12.047-72.282 3.915-113.845 3.915c-3.606 0.052-7.864 0.082-12.128 0.082-88.381 0-173.774-12.792-254.429-36.625l6.34 1.607c-76.143-21.789-142.441-56.081-200.082-100.969l1.305 0.978c-52.872-42.191-95.397-95.028-124.726-155.456l-1.166-2.662c-27.986-58.88-44.329-127.943-44.329-200.824 0-2.563 0.020-5.121 0.060-7.675l-0.005 0.385c-0.021-1.684-0.033-3.673-0.033-5.665 0-72.327 15.651-140.999 43.751-202.814l-1.252 3.077c28.995-63.268 69.156-116.734 118.439-160.074l0.526-0.453c51.66-44.672 112.692-80.166 179.569-103.062l3.848-1.145c67.496-23.513 145.297-37.094 226.271-37.094 2.935 0 5.865 0.018 8.791 0.053l-0.445-0.004c3.352-0.055 7.307-0.087 11.27-0.087 80.812 0 158.553 13.147 231.202 37.418l-5.146-1.49c67.994 22.24 126.582 56.632 176.042 100.996l-0.456-0.403c46.608 43.851 83.4 97.574 107.101 157.878l1.022 2.95c23.681 60.814 37.403 131.211 37.403 204.812 0 2.855-0.021 5.704-0.062 8.549l0.005-0.43v21.986h-1045.384c4.44 60.794 20.75 116.846 46.616 167.181l-1.138-2.437c26.458 50.328 62.705 92.16 106.516 124.286l1.004 0.702c48.753 34.999 105.893 62.073 167.556 77.845l3.512 0.762c66.8 17.487 143.488 27.527 222.522 27.527 5.206 0 10.401-0.044 15.586-0.13l-0.781 0.010c0.4 0.001 0.875 0.001 1.349 0.001 40.841 0 81.018-2.846 120.342-8.352l-4.534 0.52c44.621-5.744 83.148-13.026 120.888-22.309l-7.344 1.528c40.332-9.907 73.399-20.231 105.636-32.259l-6.549 2.141c28.246-10.076 52.496-21.759 75.26-35.64l-1.773 1.004zM4102.325 469.835c-3.433-52.269-15.399-100.855-34.529-145.651l1.098 2.894c-20.209-47.433-49.085-87.607-84.956-120.222l-0.277-0.249c-40.365-35.959-88.457-64.235-141.378-81.984l-2.885-0.839c-57.83-19.248-124.408-30.353-193.58-30.353-6.064 0-12.107 0.085-18.13 0.255l0.886-0.020c-3.954-0.094-8.612-0.147-13.283-0.147-67.203 0-131.818 11.077-192.12 31.505l4.217-1.24c-55.644 19.586-103.667 47.621-145.194 83.046l0.63-0.524c-38.569 33.165-70.2 73.166-93.251 118.279l-1.017 2.192c-21.563 41.701-36.872 90.245-43.161 141.577l-0.208 2.084z",
                }
            ],
            path![
                attrs!{
                    At::D => "M4540.235 24.998v150.588c21.082-18.071 44.273-37.045 69.873-57.525 24.902-20.561 52.68-39.698 82.062-56.331l2.87-1.494c28.675-15.906 62.015-30.034 96.903-40.602l3.69-0.961c33.879-10.897 72.854-17.179 113.296-17.179 1.040 0 2.079 0.004 3.118 0.012l-0.159-0.001c2.019-0.043 4.398-0.068 6.783-0.068 61.979 0 120.088 16.568 170.141 45.516l-1.639-0.875c53.634 33.113 95.874 80.096 122.355 135.999l0.826 1.939c28.31-26.221 58.956-51.132 91.155-74l2.812-1.896c28.689-20.38 61.38-39.553 95.712-55.828l3.977-1.696c29.896-14.21 65.024-26.706 101.549-35.643l3.863-0.799c33.256-8.040 71.436-12.651 110.694-12.651 0.367 0 0.733 0 1.1 0.001l-0.057-0c1.611-0.027 3.512-0.043 5.417-0.043 46.249 0 90.349 9.201 130.569 25.872l-2.263-0.831c40.001 16.263 74.086 39.427 102.347 68.313l0.053 0.054c27.653 28.543 49.809 62.598 64.64 100.335l0.715 2.065c14.682 36.669 23.198 79.173 23.198 123.667 0 0.782-0.003 1.564-0.008 2.344l0.001-0.12v681.562h-90.353v-658.974c0.139-3.786 0.218-8.233 0.218-12.698 0-34.374-4.697-67.652-13.485-99.225l0.617 2.596c-9.279-31.71-24.947-59.068-45.626-81.784l0.149 0.166c-21.425-22.535-47.987-39.979-77.823-50.474l-1.386-0.425c-32.231-11.326-69.393-17.869-108.083-17.869-2.873 0-5.738 0.036-8.595 0.108l0.423-0.008c-37.798 0.017-74.279 5.629-108.668 16.051l2.654-0.691c-38.534 11.395-71.783 25.221-103.167 42.069l2.875-1.41c-34.534 18.232-64.101 37.312-91.856 58.634l1.503-1.109c-26.733 20.635-50.453 42.349-72.238 65.881l-0.346 0.378c2.001 10.022 3.404 21.828 3.9 33.867l0.015 0.468c0 9.336 0 20.781 0 34.033v670.419h-90.353v-658.974c0.086-2.846 0.135-6.194 0.135-9.554 0-40.412-7.113-79.163-20.155-115.069l0.744 2.345c-21.859-60.886-71.357-107.010-132.955-123.767l-1.37-0.318c-29.71-8.42-63.831-13.262-99.082-13.262-0.955 0-1.908 0.004-2.861 0.011l0.146-0.001c-36.968 0.094-72.361 6.709-105.131 18.755l2.129-0.685c-36.107 13.229-67.204 28.861-96.162 47.524l1.894-1.143c-32.234 19.918-60.188 40.878-86.114 64.058l0.58-0.51c-24.386 21.806-46.554 44.832-66.91 69.413l-0.854 1.062v720.113h-90.353v-979.727z",
                }
            ],
            path![
                attrs!{
                    At::D => "M6270.795 1004.725v-979.727h90.353v979.727z",
                }
            ],
            path![
                attrs!{
                    At::D => "M7865.525 515.614c0.106 3.855 0.166 8.392 0.166 12.943 0 78-17.723 151.86-49.362 217.773l1.309-3.027c-31.602 63.704-75.887 116.759-129.908 157.619l-1.104 0.8c-55.924 41.117-121.627 72.858-192.693 91.285l-3.975 0.875c-71.606 19.142-153.815 30.136-238.592 30.136-2.097 0-4.192-0.007-6.286-0.020l0.322 0.002c-1.769 0.012-3.86 0.018-5.954 0.018-85.305 0-168.042-10.993-246.871-31.641l6.764 1.505c-75.485-19.371-141.589-51.216-199.499-93.596l1.626 1.135c-55.662-41.405-100.294-94.52-130.783-155.9l-1.132-2.519c-30.33-62.887-48.053-136.747-48.053-214.746 0-4.551 0.060-9.089 0.18-13.611l-0.014 0.668c-0.022-1.803-0.034-3.932-0.034-6.064 0-54.644 8.157-107.383 23.321-157.065l-0.999 3.807c14.904-49.007 36.049-91.713 63.001-130.040l-0.958 1.437c26.974-38.055 58.86-70.42 95.204-97.062l1.173-0.82c36.411-26.814 78.008-50.099 122.387-67.921l3.806-1.349c43.064-17.142 93.694-31.28 146.144-40.045l4.444-0.614c48.999-8.247 105.45-12.96 163.002-12.96 1.565 0 3.13 0.003 4.694 0.010l-0.242-0.001c1.771-0.012 3.867-0.019 5.963-0.019 84.777 0 166.986 10.994 245.279 31.635l-6.687-1.498c75.068 19.286 140.783 51.139 198.221 93.556l-1.553-1.095c55.055 41.62 99.324 94.561 129.857 155.57l1.155 2.547c29.623 62.394 46.92 135.572 46.92 212.789 0 5.452-0.086 10.883-0.257 16.293l0.020-0.791zM7775.172 515.614c0.032-2.047 0.051-4.463 0.051-6.884 0-63.99-12.859-124.978-36.133-180.519l1.146 3.082c-22.795-53.774-56.95-98.85-99.682-133.84l-0.61-0.484c-46.476-37.218-101.676-65.751-161.892-82.095l-3.153-0.729c-68.983-18.077-148.178-28.457-229.798-28.457s-160.814 10.379-236.336 29.89l6.538-1.434c-63.906 17.189-119.597 45.82-167.493 83.847l0.942-0.722c-44.232 35.665-79.104 81.166-101.539 133.281l-0.861 2.248c-21.762 51.622-34.407 111.633-34.407 174.598 0 2.889 0.027 5.772 0.080 8.648l-0.006-0.432c-0.053 2.619-0.084 5.708-0.084 8.803 0 63.178 12.648 123.402 35.551 178.277l-1.133-3.061c23.3 53.779 57.823 98.831 100.856 133.82l0.64 0.504c46.962 37.203 102.659 65.737 163.351 82.090l3.199 0.734c65.117 17.852 139.885 28.109 217.048 28.109 4.589 0 9.17-0.036 13.741-0.109l-0.691 0.009c3.943 0.066 8.594 0.104 13.254 0.104 76.884 0 151.371-10.26 222.168-29.484l-5.926 1.371c63.344-17.216 118.52-45.73 165.966-83.533l-0.922 0.709c43.556-35.518 77.978-80.567 100.324-132.071l0.871-2.254c21.578-51.938 34.11-112.264 34.11-175.518 0-2.989-0.028-5.972-0.084-8.947l0.007 0.446z",
                }
            ],
        ],
    ]
}

#[view]
fn facebook_button() -> Node<Msg> {
    div![
        C!["form-button", "facebook-button", "button-container"],
        s()
            .background(global::COLOR_FACEBOOK)
            .margin("1rem 0")
            .min_height(rem(4.5))
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::Center)
            .padding("0.5rem 1rem")
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => 0,
        },
        on_click_not_implemented(),
        svg![
            C!["icon"],
            s()
                .fill(Color::SurfaceLight5_90)
                .flex(CssFlex::None)
                .height(rem(2))
                .margin_right(rem(1))
                .width(rem(1))
                .overflow(CssOverflow::Visible),
            attrs!{
                At::ViewBox => "0 0 474 1024",
                At::from("icon") => "ic_facebook",
            },
            path![
                attrs!{
                    At::D => "M474.052 331.294h-161.431v-106.014c-0.245-1.731-0.385-3.731-0.385-5.764 0-23.952 19.417-43.369 43.369-43.369 0.665 0 1.326 0.015 1.984 0.045l-0.093-0.003h114.146v-176.188h-156.913c-174.381 0-213.835 131.012-213.835 214.739v116.555h-100.894v180.706h100.894v512h210.824v-512h143.059z",
                }
            ],
        ],
        div![
            s()
                .font_size(rem(1.2))
                .color(Color::SurfaceLight5_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("1")
                .font_weight("500")
                .text_align(CssTextAlign::Center),
            C!["label"],
            "Continue with Facebook",
        ],
    ]
}

#[view]
fn login_form_button(base_url: &Url) -> Node<Msg> {
    a![
        C!["form-button", "login-form-button", "button-container"],
        s()
            .color(Color::SurfaceDark2_90)
            .display(CssDisplay::Block)
            .margin("1rem 0")
            .text_align(CssTextAlign::Center)
            .padding("0.5rem 1rem")
            .cursor(CssCursor::Pointer),
        s()
            .style_other(":hover .login-label")
            .text_decoration(CssTextDecoration::Underline),
        attrs!{
            At::TabIndex => 0,
            At::Href => Urls::new(base_url).login(),
        },
        "Already have an account? ",
        span![
            C!["login-label"],
            s()
                .color(Color::Accent4Light1_90)
                .font_weight("500"),
            "LOG IN",
        ],
    ]
}

#[view]
fn email_input(email: &str, email_input: &ElRef<HtmlElement>) -> Node<Msg> {
    input![
        el_ref(email_input),
        C!["credentials-text-input", "text-input"],
        s()
            .style_other("::placeholder")
            .color(Color::SurfaceDark2_90),
        s()
            .style_other(":focus::placeholder")
            .color(Color::SecondaryVariant2Light1_90),
        s()
            .border_bottom("thin solid hsla(0,0%,75%,0.9)")
            .color(Color::SurfaceLight5)
            .display(CssDisplay::Block)
            .margin("1rem 0")
            .padding(rem(1))
            .width(pc(100))
            .user_select("text"),
        s()
            .hover()
            .background_color(Color::SurfaceLight5_20),
        s()
            .focus()
            .border_bottom_color(Color::SecondaryVariant2Light1_90),
        attrs!{
            At::Size => 1,
            At::from("autocorrect") => "off",
            At::from("autocapitalize") => "none",
            At::AutoComplete => "off",
            At::SpellCheck => false,
            At::TabIndex => 0,
            At::Type => "email",
            At::Placeholder => "Email",
            At::Value => email,
        },
        input_ev(Ev::Input, Msg::EmailChanged),
    ]
}

#[view]
fn password_input(password: &str) -> Node<Msg> {
    input![
        C!["credentials-text-input", "text-input"],
        s()
            .style_other("::placeholder")
            .color(Color::SurfaceDark2_90),
        s()
            .style_other(":focus::placeholder")
            .color(Color::SecondaryVariant2Light1_90),
        s()
            .border_bottom("thin solid hsla(0,0%,75%,0.9)")
            .color(Color::SurfaceLight5)
            .display(CssDisplay::Block)
            .margin("1rem 0")
            .padding(rem(1))
            .width(pc(100))
            .user_select("text"),
        s()
            .hover()
            .background_color(Color::SurfaceLight5_20),
        s()
            .focus()
            .border_bottom_color(Color::SecondaryVariant2Light1_90),
        attrs!{
            At::Size => 1,
            At::from("autocorrect") => "off",
            At::from("autocapitalize") => "none",
            At::AutoComplete => "off",
            At::SpellCheck => false,
            At::TabIndex => 0,
            At::Type => "password",
            At::Placeholder => "Password",
            At::Value => password,
        },
        input_ev(Ev::Input, Msg::PasswordChanged),
    ]
}

#[view]
fn confirm_password_input(confirm_password: &str) -> Node<Msg> {
    input![
        C!["credentials-text-input", "text-input"],
        s()
            .style_other("::placeholder")
            .color(Color::SurfaceDark2_90),
        s()
            .style_other(":focus::placeholder")
            .color(Color::SecondaryVariant2Light1_90),
        s()
            .border_bottom("thin solid hsla(0,0%,75%,0.9)")
            .color(Color::SurfaceLight5)
            .display(CssDisplay::Block)
            .margin("1rem 0")
            .padding(rem(1))
            .width(pc(100))
            .user_select("text"),
        s()
            .hover()
            .background_color(Color::SurfaceLight5_20),
        s()
            .focus()
            .border_bottom_color(Color::SecondaryVariant2Light1_90),
        attrs!{
            At::Size => 1,
            At::from("autocorrect") => "off",
            At::from("autocapitalize") => "none",
            At::AutoComplete => "off",
            At::SpellCheck => false,
            At::TabIndex => 0,
            At::Type => "password",
            At::Placeholder => "Confirm Password",
            At::Value => confirm_password,
        },
        input_ev(Ev::Input, Msg::ConfirmPasswordChanged),
    ]
}

#[view]
fn terms_and_conditions_checkbox(terms_and_conditions_checked: bool) -> Node<Msg> {
    checkbox(CheckBoxArgs {
        checked: terms_and_conditions_checked,
        text: "I have read and agree with the Stremio ",
        link_text: Some("Terms and conditions"),
        link: Some("https://www.stremio.com/tos"),
        on_click: || Msg::TermsAndConditionsClicked,
    })
}

#[view]
fn privacy_policy_checkbox(privacy_policy_checked: bool) -> Node<Msg> {
    checkbox(CheckBoxArgs {
        checked: privacy_policy_checked,
        text: "I have read and agree with the Stremio ",
        link_text: Some("Privacy Policy"),
        link: Some("https://www.stremio.com/privacy"),
        on_click: || Msg::PrivacyPolicyClicked,
    })
}

#[view]
fn marketing_checkbox(marketing_checked: bool) -> Node<Msg> {
    checkbox(CheckBoxArgs {
        checked: marketing_checked,
        text: "I have read and agree with the Stremio ",
        link_text: None,
        link: None,
        on_click: || Msg::MarketingClicked,
    })
}

struct CheckBoxArgs<OC: FnOnce() -> Msg + Copy + 'static> {
    checked: bool,
    text: &'static str,
    link_text: Option<&'static str>,
    link: Option<&'static str>,
    on_click: OC, 
}

#[view]
fn checkbox<OC: FnOnce() -> Msg + Copy + 'static>(args: CheckBoxArgs<OC>) -> Node<Msg> {
    div![
        C!["consent-checkbox-container", "checkbox-container", "button-container",
            IF!(args.checked => "checked"),
        ],
        s()
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .padding("0.5rem 1rem")
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => 0,
        },
        {
            let on_click = args.on_click;
            ev(Ev::Click, move |_| on_click())
        },
        if args.checked {
            svg![
                C!["icon"],
                s()
                    .background_color(Color::PrimaryVariant1)
                    .fill(Color::SurfaceLight5)
                    .flex(CssFlex::None)
                    .height(rem(1.2))
                    .width(rem(1.2))
                    .display(CssDisplay::Block)
                    .overflow(CssOverflow::Visible),
                attrs!{
                    At::ViewBox => "0 0 100 100",
                },
                svg![
                    s()
                        .overflow(CssOverflow::Visible),
                    attrs!{
                        At::ViewBox => "0 0 1331 1024",
                        At::X => 10,
                        At::Y => 10,
                        At::Width => 80,
                        At::Height => 80,
                        At::from("icon") => "ic_check", 
                    },
                    path![
                        attrs!{
                            At::D => "M545.129 1024c-40.334-0.026-76.847-16.363-103.306-42.769l-398.755-397.551c-24.752-26.158-39.97-61.56-39.97-100.516 0-80.839 65.533-146.372 146.372-146.372 38.806 0 74.085 15.101 100.281 39.748l-0.075-0.070 288.226 286.118 536.395-612.593c27.002-30.81 66.432-50.158 110.381-50.158 80.929 0 146.535 65.606 146.535 146.535 0 36.98-13.698 70.761-36.298 96.544l0.144-0.168-639.699 731.256c-25.909 29.451-63.15 48.401-104.838 49.987l-0.272 0.008z",
                        }
                    ]
                ]
            ]
        } else {
            svg![
                C!["icon"],
                s()
                    .fill(Color::SurfaceDark5)
                    .flex(CssFlex::None)
                    .height(rem(1.2))
                    .width(rem(1.2))
                    .display(CssDisplay::Block)
                    .overflow(CssOverflow::Visible),
                attrs!{
                    At::ViewBox => "0 0 1024 1024",
                    At::from("icon") => "ic_box_empty",
                },
                path![
                    attrs!{
                        At::D => "M843.294 180.706v662.588h-662.588v-662.588h662.588zM1024 0h-1024v1024h1024v-1024z",
                    }
                ]
            ]
        },
        div![
            C!["label"],
            s()
                .color(Color::Surface90)
                .flex("1")
                .font_size(rem(0.9))
                .margin_left(rem(0.5)),
            args.text,
            if let (Some(link_text), Some(link)) = (args.link_text, args.link) {
                Some(a![
                    C!["link", "button-container"],
                    s()
                        .color(Color::SurfaceLight5_90)
                        .font_size(rem(0.9))
                        .cursor(CssCursor::Pointer),
                    s()
                        .hover()
                        .text_decoration(CssTextDecoration::Underline),
                    attrs!{
                        At::TabIndex => -1,
                        At::Target => "_blank",
                        At::Href => link,
                    },
                    ev(Ev::Click, |event| event.stop_propagation()),
                    link_text,
                ])
            } else {
                None
            }
        ],
    ]
}

#[view]
fn sign_up_button() -> Node<Msg> {
    div![
        C!["form-button", "submit-button", "button-container"],
        s()
            .background_color(Color::Accent3)
            .margin("1rem 0")
            .min_height(rem(4))
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::Center)
            .padding("0.5rem 1rem")
            .cursor(CssCursor::Pointer),
        s()
            .hover()
            .background_color(Color::Accent3Light1),
        attrs!{
            At::TabIndex => 0,
        },
        on_click_not_implemented(),
        div![
            C!["label"],
            s()
                .font_size(rem(1.2))
                .color(Color::SurfaceLight5_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("0")
                .font_weight("500")
                .text_align(CssTextAlign::Center),
            "Sign up",
        ]
    ]
}

#[view]
fn guest_login_button()-> Node<Msg> {
    div![
        C!["form-button", "guest-login-button", "button-container"],
        s()
            .margin_top(rem(1))
            .padding(rem(1))
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::Center)
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => 0,
        },
        on_click_not_implemented(),
        div![
            C!["label"],
            s()
                .color(Color::SurfaceLight5_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("1")
                .font_size(rem(1.1))
                .font_weight("500")
                .text_align(CssTextAlign::Center),
            s()
                .hover()
                .text_decoration(CssTextDecoration::Underline),
            "GUEST LOGIN",
        ]
    ]
}

#[view]
fn forgot_password_button() -> Node<Msg> {
    div![
        C!["forgot-password-link-container"],
        s()
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::FlexEnd)
            .margin("1rem 0")
            .text_align(CssTextAlign::Right),
        div![
            C!["forgot-password-link", "button-container"],
            s()
                .color(Color::SurfaceLight3_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("1")
                .padding("0.5rem 1rem")
                .cursor(CssCursor::Pointer),
            s()
                .hover()
                .color(Color::SecondaryVariant2Light1_90)
                .text_decoration(CssTextDecoration::Underline),
            attrs!{
                At::TabIndex => 0,
            },
            on_click_not_implemented(),
            "Forgot password?",
        ]
    ]
}

#[view]
fn error_message(error: &FormError) -> Node<Msg> {
    div![
        s()
            .color(Color::Accent5_90)
            .margin("1rem 0")
            .padding("0 1rem")
            .text_align(CssTextAlign::Center),
        match error {
            FormError::InvalidEmail => "Invalid email",
            FormError::InvalidPassword => "Invalid password",
            FormError::APIError(api_error) => &api_error.message,
        }
    ]
}

#[view]
fn login_button() -> Node<Msg> {
    div![
        C!["form-button", "submit-button", "button-container"],
        s()
            .background_color(Color::Accent3)
            .margin("1rem 0")
            .min_height(rem(4))
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::Center)
            .padding("0.5rem 1rem")
            .cursor(CssCursor::Pointer),
        s()
            .hover()
            .background_color(Color::Accent3Light1),
        attrs!{
            At::TabIndex => 0,
        },
        ev(Ev::Click, |_| Msg::Login),
        div![
            C!["label"],
            s()
                .font_size(rem(1.2))
                .color(Color::SurfaceLight5_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("0")
                .font_weight("500")
                .text_align(CssTextAlign::Center),
            "Log in",
        ]
    ]
}

#[view]
fn sign_up_with_email_button(base_url: &Url)-> Node<Msg> {
    a![
        C!["form-button", "guest-login-button", "button-container"],
        s()
            .margin_top(rem(1))
            .padding(rem(1))
            .align_items(CssAlignItems::Center)
            .display(CssDisplay::Flex)
            .flex_direction(CssFlexDirection::Row)
            .justify_content(CssJustifyContent::Center)
            .cursor(CssCursor::Pointer),
        attrs!{
            At::TabIndex => 0,
            At::Href => Urls::new(base_url).root(),
        },
        div![
            C!["label"],
            s()
                .color(Color::SurfaceLight5_90)
                .flex_basis(CssFlexBasis::Auto)
                .flex_grow("0")
                .flex_shrink("1")
                .font_size(rem(1.1))
                .font_weight("500")
                .text_align(CssTextAlign::Center),
            s()
                .hover()
                .text_decoration(CssTextDecoration::Underline),
            "SIGN UP WITH EMAIL",
        ]
    ]
}
