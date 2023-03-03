use std::process::CommandEnvs;
use std::sync::Arc;
use reqwest::{Body, Client, Proxy};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
// use tokio::
use tokio::*;

struct ChatBot {}

impl ChatBot {
    pub async fn new() -> Result<(), String> {
        const KEY: &str = "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";
        const URL: &str = "https://api.openai.com/v1/models";
        const HTTP_PROXY: &str = "";

        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        use reqwest::Body;


        // let https_proxy = Proxy::https(HTTPS_PROXY).map_err(|err| err.to_string())?;
        let http_proxy = Proxy::all(HTTP_PROXY).map_err(|err| err.to_string())?;
        let client = Client::builder().proxy(http_proxy).build().unwrap();

        let header_map = HeaderMap::from_iter(vec![
            (
                HeaderName::from_static("authorization"),
                HeaderValue::from_static("Bearer ${KEY}"),
            ),
            (
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/json"),
            ),
        ]);


        let req = client.get(URL).build().map_err(|err| err.to_string())?;
        match client.execute(req).await {
            Ok(res) =>
                println!("{}", res.text().await.map_err(|err| err.to_string())?),
            Err(err) => {
                println!("{}", err);
            }
        }


        let body = Body::from(
            r#"{"model": "text-davinci-003", "prompt": "Say this is a test", "temperature": 0, "max_tokens": 7}"#,
        );

        let resp = client.post(URL).headers(header_map).body(body).send().await.map_err(|err| err.to_string())?;

        println!("{}", resp.text().await.map_err(|err| err.to_string())?);

        Ok(())
    }
}


fn main() {
    let executor = Arc::new(tokio::runtime::Runtime::new().unwrap());

    executor.spawn(Box::pin((async move {
        let _ = ChatBot::new().await;
    })));
}
