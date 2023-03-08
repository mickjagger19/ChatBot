use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use lazy_static::lazy_static;
use reqwest::{Body, Client, Proxy, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, to_string, Value};
use crate::model::model::{CODE, GPT_3_5};
use crate::url::url::{CHAT_COMPLETION, CODE_COMPLETION};
use crate::url::url::MODELS;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest, CreateChatCompletionResponse, CreateCompletionRequest, CreateCompletionResponse, Prompt, Role};

// const HTTP_PROXY: &str = "http://p_vnextcie:vNext49!@proxyuk.huawei.com:8080";
const HTTP_PROXY: &str = "http://q00569923:Heyjude19,.@proxyuk.huawei.com:8080";
const KEY: &str = "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";

pub enum State {
    // with closure
    Chat(Arc<dyn Fn(&mut String) + Send + Sync>),
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

    async fn unwrap_result(&self, res: Response) -> Result<Vec<ResponseData>, String> {
        match self {
            State::Chat(_) => {
                let res = res.json::<CreateChatCompletionResponse>().await.map_err(|err| err
                    .to_string())?;
                Ok(res.choices.iter().map(|choice| {
                    ResponseData {
                        role: choice.message.role.to_string(),
                        content: choice.message
                            .content.to_string().trim().to_string(),
                    }
                }).collect())
            }
            State::CodeCompletion => {
                let res = res.json::<CreateCompletionResponse>().await.map_err(|err| err
                    .to_string())?;
                Ok(res.choices.iter().map(|choice| {
                    ResponseData {
                        content: choice.text.to_string().trim().to_string(),
                        ..Default::default()
                    }
                }).collect())
            }
            State::Other(_) => {
                let res = res.json::<CreateCompletionResponse>().await.map_err(|err| err
                    .to_string())?;
                Ok(res.choices.iter().map(|choice| {
                    ResponseData {
                        content: choice.text.to_string().trim().to_string(),
                        ..Default::default()
                    }
                }).collect())
            }
        }
    }

    fn form_request_body(&self, content: String) -> Result<String, String> {
        let model = self.get_model().to_string();
        match self {
            Self::Chat(f) => {
                let mut content = content.to_string();
                f(&mut content);
                let req = CreateChatCompletionRequest {
                    model,
                    messages: vec![ChatCompletionRequestMessage {
                        role: Role::User,
                        content,
                        name: None,
                    }],
                    ..Default::default()
                };
                to_string(&req).map_err(|err| err.to_string())
            }
            Self::CodeCompletion => {
                let req = CreateCompletionRequest {
                    model,
                    prompt: Some(Prompt::String(content)),
                    ..Default::default()
                };
                to_string(&req).map_err(|err| err.to_string())
            }
            _ => {
                let req = CreateCompletionRequest {
                    prompt: Some(Prompt::String(content)),
                    ..Default::default()
                };
                to_string(&req).map_err(|err| err.to_string())
            }
        }
    }
}

pub struct ChatBot {
    header: HeaderMap,
    client: Client,
    state: State,
}


#[derive(Clone, Default, Debug)]
pub struct ResponseData {
    pub role: String,
    pub content: String,
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
            state: State::Chat(Arc::new(|_s| {})),
        })
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        println!("model changed to: {:?}", self.state.get_model());
    }

    pub async fn completions_with_model(&self, content: String) -> Result<Vec<ResponseData>,
        String> {
        let body_str =
            self.state.form_request_body(content)?;
        let body = Body::from(body_str);
        let req = self.client.post(self.state.get_url()).body(body).headers(self.header.clone())
            .build()
            .map_err(|err|
                err
                    .to_string
                    ())?;
        let res = self.client.execute(req).await.map_err(|err| err.to_string())?;

        self.state.unwrap_result(res).await
    }


    pub async fn input_with_state(&self, content: String) -> Result<Vec<ResponseData>,
        String> {
        self.completions_with_model(content).await
    }

    pub async fn chat(&self, content: String) -> Result<Vec<ResponseData>, String> {
        self.completions_with_model(content).await
    }

    pub async fn code_completion(&self, content: String) -> Result<Vec<ResponseData>, String> {
        self.completions_with_model(content).await
    }

    pub async fn list_model(&self) -> Result<(), String> {
        let req = self.client.get(MODELS.to_string()).headers(self.header.clone()).build()
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
