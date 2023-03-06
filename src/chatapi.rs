
use std::fmt::{Debug, format, Formatter, Pointer};
use std::io;
use reqwest::{Body, Client, get, Proxy};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, to_string, Value};
use model::{CODE, GPT_3_5};
use url::{CHAT_COMPLETION, CODE_COMPLETION};



// const HTTP_PROXY: &str = "7.222.125.44:3128";
const HTTP_PROXY: &str = "http://q00569923:Heyjude19,.@proxyuk.huawei.com:8080";
const KEY: &str = "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";

pub mod url {
    // use lazy_static::lazy_static;

    // lazy_static! {
    pub(crate) const PREFIX: &'static str = "https://api.openai.com/";
    pub(crate) const MODELS: &'static str = "https://api.openai.com/v1/models";
    pub(crate) const CHAT_COMPLETION: &'static str = "https://api.openai.com/v1/chat/completions";
    pub(crate) const CODE_COMPLETION: &'static str = "https://api.openai.com/v1/completions";
}

pub mod model {
    pub(crate) const GPT_3_5: &'static str = "gpt-3.5-turbo";
    pub(crate) const CODE: &'static str = "code-davinci-002";
}

pub enum State {
    // with closure
    Chat(Box<dyn Fn(&mut String)>),
    CodeCompletion,
    Other(String),
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(self.get_model());
        Ok(())
    }
}

impl State {
    fn get_model(&self) -> &str {
        match self {
            Self::Chat(_) => GPT_3_5,
            Self::CodeCompletion => CODE,
            State::Other(model) => model.as_str()
        }
    }


    fn get_url(&self) -> &str {
        match self {
            Self::Chat(_) => CHAT_COMPLETION,
            Self::CodeCompletion => CODE_COMPLETION,
            State::Other(_) => CODE_COMPLETION,
        }
    }

    fn unwrap_result(&self, choice: &Map<String, Value>) -> Result<String, ()> {
        match self {
            State::Chat(_) => {
                choice.get("message").map(|message| {
                    let role = if let Some(Value::String(role)) = message.get("role") {
                        role.to_string()
                    } else {
                        "".to_string()
                    };
                    let content = if let Some(Value::String(content)) = &message.get("content") {
                        content.to_string()
                    } else {
                        "".to_string()
                    };
                    format!("{}:\n{}", role, content.trim_start())
                }).ok_or(())
            }
            State::CodeCompletion => {
                choice.get("text").map(|text| {
                    let content = if let Value::String(text) = text {
                        text.to_string()
                    } else {
                        "".to_string()
                    };
                    content
                }).ok_or(())
            }
            State::Other(_) => {
                choice.get("text").map(|text| {
                    let content = if let Value::String(text) = text {
                        text.to_string()
                    } else {
                        "".to_string()
                    };
                    content
                }).ok_or(())
            }
        }
    }

    fn form_request_body(&self, content: String) -> (String, Value) {
        match self {
            Self::Chat(f) => {
                let mut content = content.to_string();
                f(&mut content);
                ("messages".to_string(), Value::Array(vec![
                    Value::Object(serde_json::Map::from_iter([
                        ("role".to_string(), Value::String("user".to_string())),
                        ("content".to_string(), Value::String(format!("{content}").to_string())),
                    ].into_iter()))]))
            }
            Self::CodeCompletion => {
                ("prompt".to_string(), Value::String(content))
            }
            _ => {
                ("prompt".to_string(), Value::String(content))
            }
        }
    }
}

pub struct ChatBot {
    header: HeaderMap,
    client: Client,
    state: State,
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
            state: State::Chat(Box::new(|s| {})),
        })
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        println!("model changed to: {:?}", self.state.get_model());
    }


    pub async fn completions_with_model(&self, content: String, model: &str) -> Result<String,
        String> {
        let body_json = serde_json::Map::from_iter([
            ("model".to_string(), Value::String(model.to_string())),
            self.state.form_request_body(content)].into_iter());
        let body = Body::from(to_string(&body_json).map_err(|err| err.to_string())?);
        let req = self.client.post(self.state.get_url()).body(body).headers(self.header.clone())
            .build()
            .map_err(|err|
                err
                    .to_string
                    ())?;
        let res = self.client.execute(req).await.map_err(|err| err.to_string())?;
        let result = res.json::<Map<String, Value>>().await.map_err(|err| err.to_string())?;
        if let Value::Array(choices) = result.get("choices").ok_or("No choices returned"
            .to_string())? {
            let results: Vec<_> = choices.iter().map(|choice| {
                if let Value::Object(choice_map) = choice {
                    self.state.unwrap_result(choice_map).ok().unwrap_or_default()
                } else {
                    "".to_string()
                }
            }).collect();
            return Ok(results[0].to_string());
        }
        Ok("".to_string())
    }

    pub async fn input_with_state(&self, content: String) -> Result<String,
        String> {
        self.completions_with_model(content, self.state.get_model()).await
    }

    pub async fn chat(&self, content: String) -> Result<String, String> {
        self.completions_with_model(content, GPT_3_5).await
    }

    pub async fn code_completion(&self, content: String) -> Result<String, String> {
        self.completions_with_model(content, CHAT_COMPLETION).await
    }

    pub async fn list_model(&self) -> Result<(), String> {
        let req = self.client.get(url::MODELS.to_string()).headers(self.header.clone()).build()
            .map_err(|err|
                err
                    .to_string
                    ())?;
        println!("supported models:");
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
