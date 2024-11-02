pub use super::{
    cancel::Cancel, client::Client, headers::Headers, request::Request, response::Response,
    url::Url,
};

#[rquickjs::module]
pub mod http_mod {
    pub use super::{Cancel, Client, Headers, Request, Response, Url};

    #[rquickjs::function(rename = "createCancel")]
    pub fn create_cancel() -> Cancel {
        Cancel::new()
    }
}
