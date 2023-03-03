use std::io;
use reqwest::{Body, Client, Proxy};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, Value};

const HTTP_PROXY: &str = "http://q00569923:Heyjude19,.@proxyuk.huawei.com:8080";
const KEY: &str = "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";

pub(crate) mod url {
    // use lazy_static::lazy_static;

    // lazy_static! {
    pub(crate) const PREFIX: &'static str = "https://api.openai.com/";
    pub(crate) const MODELS: &'static str = "https://api.openai.com/v1/models";
    pub(crate) const CHAT_COMPLETION: &'static str = "https://api.openai.com/v1/chat/completions";
}

mod model {
    pub(crate) const GPT_3_5: &'static str = "gpt-3.5-turbo";
    pub(crate) const GPT: &'static str = "gpt-3.5-turbo";
}


struct ChatBot {
    header: HeaderMap,
    client: Client,
}

impl ChatBot {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            header: HeaderMap::from_iter(vec![
                (
                    HeaderName::from_static("authorization"),
                    HeaderValue::from_str(format!("Bearer {KEY}").as_str()).map_err(|err| err
                        .to_string())?,
                ),
                (
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("application/json"),
                ),
            ]),
            client: {
                let http_proxy = Proxy::all(HTTP_PROXY).map_err(|err| err.to_string())?;
                let client = Client::builder().proxy(http_proxy).build().unwrap();
                client
            },
        })
    }

    pub async fn chat(&self, content: String) -> Result<(), String> {
        // println!("user : {}\n", content);
        let body_json = serde_json::Map::from_iter([
            ("model".to_string(), Value::String("gpt-3.5-turbo".to_string())),
            ("messages".to_string(), Value::Array(vec![
                Value::Object(serde_json::Map::from_iter([
                    ("role".to_string(), Value::String("user".to_string())),
                    ("content".to_string(), Value::String(format!("{content}").to_string())),
                ].into_iter()))]))].into_iter());
        let body = Body::from(serde_json::to_string(&body_json).map_err(|err| err.to_string())?);
        let req = self.client.post(url::CHAT_COMPLETION).body(body).headers(self.header.clone())
            .build()
            .map_err(|err|
                err
                    .to_string
                    ())?;
        let res = self.client.execute(req).await.map_err(|err| err.to_string())?;
        let result = res.json::<Map<String, Value>>().await.map_err(|err| err.to_string())?;
        if let Value::Array(choices) = result.get("choices").ok_or("No choices returned"
            .to_string())? {
            choices.iter().for_each(|choice| {
                if let Value::Object(choice_map) = choice {
                    if let Some(Value::Object(message)) = &choice_map.get("message") {
                        let role = if let Some(Value::String(role)) = message.get("role") {
                            role.to_string()
                        } else {
                            "gpt".to_string()
                        };
                        if let Some(Value::String(content)) = &message.get("content") {
                            println!("{}:{}", role, content);
                        }
                    }
                }
            })
        }
        Ok(())
    }

    pub async fn list_model(&self) -> Result<(), String> {
        let req = self.client.get(url::MODELS.to_string()).headers(self.header.clone()).build()
            .map_err(|err|
                err
                    .to_string
                    ())?;
        let res = self.client.execute(req).await.map_err(|err| err.to_string())?;
        let result = res.json::<Map<String, Value>>().await.map_err(|err| err.to_string())?;
        if let Some(Value::Array(models)) = result.get("data") {
            models.iter().for_each(|model| {
                if let Value::Object(model) = model {
                    if let Some(Value::String(model)) = model.get("id") {
                        println!("{}", model);
                    }
                }
            })
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    if let Ok(chat_bot) = ChatBot::new() {
        println!("Chat started");
        println!("Press q to quit");
        let mut input = "".to_string();
        while let Ok(_input_size) = io::stdin().read_line(&mut input) {
            if input.strip_suffix(&['\r', '\n']) == Some("q") {
                return;
            }
            if input.strip_suffix(&['\r', '\n']) == Some("-l") {
                let _ = chat_bot.list_model().await;
                continue;
            }
            let _ = chat_bot.chat(input.to_string()).await;
        }
    }
}

