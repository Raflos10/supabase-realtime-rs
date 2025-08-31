use once_cell::sync::Lazy;
use regex::Regex;

static HTTP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)http://").unwrap());
static HTTPS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)https://").unwrap());
static WS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^ws").unwrap());
static PATH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(/socket/websocket|/socket|/websocket)/?$").unwrap());
static TRAILING_SLASH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/+$").unwrap());

pub(crate) fn is_ws_url(url: &str) -> bool {
    url.find(':').is_some_and(|colon_idx| {
        let scheme = &url[..colon_idx].to_ascii_lowercase();
        matches!(scheme.as_str(), "ws" | "wss" | "http" | "https")
    })
}

pub(crate) fn http_to_ws(http_url: &str) -> String {
    let replaced_http = HTTP_REGEX.replace_all(http_url, "ws://");
    let replaced_https = HTTPS_REGEX.replace_all(&replaced_http, "wss://");
    format!("{replaced_https}/realtime/v1/websocket")
}

pub(crate) fn http_endpoint_url(socket_url: &str) -> String {
    let result = WS_REGEX.replace(socket_url, "http");
    let result = PATH_REGEX.replace(&result, "");
    let result = TRAILING_SLASH_REGEX.replace(&result, "");

    String::from(result)
}

pub(crate) fn get_reply_event_name(_ref: &str) -> String {
    format!("chan_reply_{_ref}")
}
