use std::env;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_openai::Client;
use async_openai::types::{
    ChatCompletionRequestMessage, CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    CreateChatCompletionResponse, CreateCompletionRequest, CreateCompletionResponse, Prompt, Role,
};
use futures::{future, Stream, TryStreamExt};
use parking_lot::RwLock;
use crate::model::model::{CODE, GPT_3_5};

#[derive(Clone)]
pub enum State {
    // with closure
    Chat((Arc<dyn Fn(&mut String) + Send + Sync>, Arc<RwLock<Vec<ChatCompletionRequestMessage>>>)),
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
            State::Other(model) => model.as_str(),
        }
    }

    fn unwrap_completion_res(
        &self,
        res: CreateCompletionResponse,
    ) -> Result<Vec<ResponseData>, String> {
        Ok(res
            .choices
            .iter()
            .map(|choice| ResponseData {
                content: choice.text.to_string().trim().to_string(),
                ..Default::default()
            })
            .collect())
    }

    fn unwrap_chat_res(
        &self,
        res: CreateChatCompletionResponse,
    ) -> Result<Vec<ResponseData>, String> {
        Ok(res
            .choices
            .iter()
            .map(|choice| {
                ResponseData {
                    role: {
                        let role = choice.message.role.to_string();
                        // role[0] += 'A' - 'a';
                        role.to_string()
                    },
                    content: choice.message.content.to_string().trim().to_string(),
                }
            })
            .collect())
    }

    // fn unwrap_chat_res_stream(&self, res: ChatCompletionResponseStream) ->
    //                                                                     Result<Vec<ResponseData>,
    // String> {
    //     Ok(res
    //         .choices
    //         .iter()
    //         .map(|choice| {
    //             ResponseData {
    //                 role: {
    //                     let role = choice.message.role.to_string();
    //                     // role[0] += 'A' - 'a';
    //                     role.to_string()
    //                 },
    //                 content: choice.message.content.to_string().trim().to_string(),
    //             }
    //         })
    //         .collect())
    // }

    fn form_chat_request(
        &self,
        content: String,
        save_context: bool,
    ) -> Result<CreateChatCompletionRequest, String> {
        let model = self.get_model().to_string();
        match self {
            Self::Chat((f, context)) => {
                let mut content = content.to_string();
                f(&mut content);
                CreateChatCompletionRequestArgs::default()
                    .model(model)
                    .messages({
                        let new_message =
                            ChatCompletionRequestMessage { role: Role::User, content, name: None };
                        let mut new_context: Vec<_> =
                            context.read().iter().map(|message| message.clone()).collect();
                        if save_context {
                            context.write().push(new_message.clone());
                        }

                        new_context.push(new_message);
                        new_context
                    })
                    .temperature(0.0f32)
                    .build()
                    .map_err(|err| err.to_string())
            }
            _ => Err("invalid state".to_string()),
        }
    }

    fn form_completion_request(&self, content: String) -> Result<CreateCompletionRequest, String> {
        let model = self.get_model().to_string();
        match self {
            Self::CodeCompletion => Ok(CreateCompletionRequest {
                model,
                prompt: Some(Prompt::String(content)),
                ..Default::default()
            }),
            _ => Err("invalid state".to_string()),
        }
    }
}

/// Builder-style methods
impl State {
    pub fn chat() -> Self {
        Self::Chat((Arc::new(|_: &mut String| {}), Default::default()))
    }

    pub fn chat_with_prefix(mut self, prefix: &str) -> Self {
        let prefix = prefix.to_string();
        if let Self::Chat((f, context)) = self {
            let closure = Arc::new(move |original: &mut String| {
                f(original);
                original.insert_str(0, prefix.as_str());
            });
            self = Self::Chat((closure, context));
        }
        self
    }

    pub fn chat_with_closure(mut self, c: Arc<dyn Fn(&mut String) + Send + Sync>) -> Self {
        if let Self::Chat((f, context)) = self {
            let closure = Arc::new(move |original: &mut String| {
                f(original);
                c(original);
            });
            self = Self::Chat((closure, context));
        }
        self
    }

    // pub fn add_prefix(&mut self, prefix: String) {
    //     if let Self::Chat((ref mut f, context)) = self {
    //         let closure = Arc::new(move |original: &mut String| {
    //             f(original);
    //             original.insert_str(0, prefix.as_str());
    //         }) as Arc<dyn Fn(&mut String) + Send + Sync>;
    //         *f = closure;
    //     }
    // }

    pub fn chat_with_suffix(mut self, suffix: &str) -> Self {
        let suffix = suffix.to_string();
        if let Self::Chat((f, context)) = self {
            let closure = Arc::new(move |original: &mut String| {
                f(original);
                original.push_str(suffix.as_str());
            });
            self = Self::Chat((closure, context));
        }
        self
    }

    pub fn with_additional_context(self, message: ChatCompletionRequestMessage) -> Self {
        if let Self::Chat((_, ref context)) = self {
            context.write().push(message);
        }
        self
    }

    pub fn append_additional_context(&self, message: ChatCompletionRequestMessage) {
        if let Self::Chat((_, ref context)) = self {
            context.write().push(message);
        }
    }
}

#[derive(Clone)]
pub struct ChatBot {
    client: Client,
    state: State,
    save_context: bool,
}

#[derive(Clone, Default, Debug)]
pub struct ResponseData {
    pub role: String,
    pub content: String,
}

impl ChatBot {
    pub fn new() -> Result<Self, String> {
        // env::set_var("HTTP_PROXY", HTTP_PROXY);
        // env::set_var("HTTPS_PROXY", HTTP_PROXY);
        let key = env::var("OPENAI_API_KEY").map_err(|err| "Please set OPENAI_API_KEY environment variable")?;
        Ok(Self {
            client: Client::new().with_api_key(key.to_string()),
            state: State::chat(),
            save_context: false,
        })
    }

    pub fn save_context(mut self) -> Self {
        self.save_context = true;
        self
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        println!("model changed to: {:?}", self.state.get_model());
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub async fn completion(
        &self,
        content: String,
        state: &State,
    ) -> Result<Vec<ResponseData>, String> {
        let req = state.form_completion_request(content)?;
        let res = self.client.completions().create(req).await.map_err(|err| err.to_string())?;
        println!("{:?}", res);
        state.unwrap_completion_res(res)
    }

    pub async fn chat(&self, content: String, state: &State) -> Result<Vec<ResponseData>, String> {
        let req = state.form_chat_request(content.clone(), self.save_context)?;
        let res = self.client.chat().create(req).await.map_err(|err| err.to_string())?;
        if let Some(choice) = res.choices.first() {
            state.append_additional_context(ChatCompletionRequestMessage {
                role: choice.message.role.clone(),
                content: choice.message.content.clone(),
                name: None,
            });
        }
        println!("{:?}", res);
        state.unwrap_chat_res(res)
    }

    pub async fn chat_stream(
        &self,
        content: String,
    ) -> Result<impl Stream<Item=Result<Vec<(Option<String>, Option<String>)>, String>>, String>
    {
        let req = self
            .state
            .form_chat_request(content, self.save_context)
            .map_err(|err| err.to_string())?;
        let stream = self.client.chat().create_stream(req).await.map_err(|err| err.to_string())?;
        Ok(stream
            .and_then(|res| {
                future::ok(
                    res.choices
                        .into_iter()
                        .map(|choice| {
                            (choice.delta.content, choice.delta.role.map(|role| role.to_string()))
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .map_err(|err| err.to_string())
            .into_stream())
        // while let Some(res) = res.next().await {
        //     res.map(|res| res)
        // }
        // Ok(res)
    }

    pub async fn input(&self, content: String) -> Result<Vec<ResponseData>, String> {
        let input = content.trim();
        if input.is_empty() {
            return Err("empty input".to_string());
        } else {
            match self.state {
                State::Chat(_) => self.chat(content, &self.state).await,
                State::CodeCompletion => self.completion(content, &self.state).await,
                _ => self.completion(content, &self.state).await,
            }
        }
    }

    /// Input with an additional temporary state
    pub async fn input_with_state(
        &self,
        content: String,
        state: State,
    ) -> Result<Vec<ResponseData>, String> {
        let input = content.trim();
        if input.is_empty() {
            return Err("empty input".to_string());
        } else {
            match self.state {
                State::Chat(_) => self.chat(content, &state).await,
                State::CodeCompletion => self.completion(content, &state).await,
                _ => self.completion(content, &state).await,
            }
        }
    }

    pub async fn list_model(&self) -> Result<(), String> {
        let req = self.client.models().list().await.map_err(|err| err.to_string())?;
        println!("supported models:");
        req.data.iter().for_each(|model| {
            println!("{}", model.id);
        });
        Ok(())
    }
}