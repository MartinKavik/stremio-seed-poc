// Copied from stremio-ng-example
use stremio_core::state_types::*;
use futures::{future, Future};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use web_sys::{Response, RequestInit};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use std::error::Error;
use std::fmt;
#[derive(Debug)]
pub enum EnvError {
    Js(String),
    Serde(serde_json::error::Error),
    StorageMissing,
}
impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for EnvError {
    fn description(&self) -> &str {
        match self {
            EnvError::Js(msg) => &msg,
            EnvError::Serde(e) => &e.description(),
            EnvError::StorageMissing => "localStorage is missing",
        }
    }
}
impl From<JsValue> for EnvError {
    fn from(e: JsValue) -> EnvError {
        let err_str: String = e.dyn_into::<js_sys::Error>()
            .map(|s| s.to_string().into())
            .unwrap_or_else(|_| "unknown JS error".into());
        EnvError::Js(err_str)
    }
}
impl From<serde_json::error::Error> for EnvError {
    fn from(e: serde_json::error::Error) -> EnvError {
        EnvError::Serde(e)
    }
}


// By creating an empty enum, we ensure that this type cannot be initialized
pub enum Env {}
impl Env {
    fn wrap_to_fut<T: 'static>(res: Result<T, EnvError>) -> EnvFuture<T> {
        Box::new(match res {
            Ok(res) => future::ok(res),
            Err(e) => future::err(e.into()),
        })
    }
    // @TODO maybe this be optimized by keeping the `storage` in the struct; this can only be done
    // if this becomes a non-static method
    fn get_storage() -> Result<web_sys::Storage, EnvError> {
        web_sys::window().unwrap().local_storage()?.ok_or(EnvError::StorageMissing)
    }
    fn get_storage_sync<T: 'static + DeserializeOwned>(key: &str) -> Result<Option<T>, EnvError> {
        let storage = Self::get_storage()?;
        let val = storage.get_item(key)?;
        Ok(match val {
            Some(r) => Some(serde_json::from_str(&r)?),
            None => None,
        })
    }
    fn set_storage_sync<T: Serialize>(key: &str, value: Option<&T>) -> Result<(), EnvError> {
        let storage = Self::get_storage()?;
        Ok(match value {
            Some(v) => {
                let serialized = serde_json::to_string(v)?;
                storage.set_item(key, &serialized)?
            },
            None => {
                storage.remove_item(key)?
            }
        })
    }
}
impl Environment for Env {
    fn fetch_serde<IN, OUT>(in_request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: 'static + Serialize,
        OUT: 'static + DeserializeOwned
    {
        let window = web_sys::window().unwrap();
        let mut opts = RequestInit::new();
        let (parts, body) = in_request.into_parts();
        opts.method(&parts.method.as_str());
        let headers: HashMap<&str, String> = parts
            .headers
            .iter()
            .map(|(k, v)| {
                (k.as_str(), String::from_utf8_lossy(v.as_ref()).into_owned())
            })
            .collect();
        opts.headers(&JsValue::from_serde(&headers).unwrap());
        // @TODO since HEAD/GET cannot have a body, consider checking that
        // @TODO consider adding Content-Type "application/json"
        // @TODO is there a better way to do this? we need to pass in a stringified value
        match serde_json::to_string(&body) {
            Ok(ref v) if v != "null" => {
                opts.body(Some(&JsValue::from_str(v)));
            },
            _ => {},
        };
        let req = web_sys::Request::new_with_str_and_init(&parts.uri.to_string(), &opts)
            .expect("failed building request");
        let pr = window.fetch_with_request(&req);
        let fut = JsFuture::from(pr)
            .and_then(|resp_value| {
                // @TODO: if the response code is not 200, return an error related to that
                // @TODO: optimize this, as this is basically deserializing in JS -> serializing in
                // JS -> deserializing in rust
                // NOTE: there's no realistic scenario those unwraps fail
                let resp: Response = resp_value.dyn_into().unwrap();
                JsFuture::from(resp.json().unwrap())
            })
            .or_else(|e| {
                future::err(EnvError::from(e).into())
            })
            .and_then(|json| {
                match json.into_serde() {
                    Ok(r) => future::ok(r),
                    Err(e) => future::err(EnvError::from(e).into()),
                }
            });
        Box::new(fut)
    }
    fn exec(fut: Box<dyn Future<Item=(), Error=()>>) {
        spawn_local(fut)
    }
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<T>> {
        Self::wrap_to_fut(Self::get_storage_sync(key))
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
        Self::wrap_to_fut(Self::set_storage_sync(key, value))
    }
}
