pub use super::{
    cancel::Cancel, client::Client, headers::Headers, request::Request, response::Response,
};

#[rquickjs::module(rename_vars = "camelCase")]
pub mod http_mod {
    pub use super::{Cancel, Client, Headers, Request, Response};

    #[rquickjs::function(rename = "createCancel")]
    pub fn create_cancel() -> Cancel {
        Cancel::new()
    }
}
