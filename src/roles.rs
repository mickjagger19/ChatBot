use lazy_static::lazy_static;
use crate::chat_api::State;
use std::sync::Arc;

lazy_static! {
    /// roles
    pub static ref CODE_EXPLAINER: State = State::Chat(Arc::new(|code_snippet| {
        code_snippet.insert_str(0, r#"
I need you to provide a short explanation(a paragraph) to describe the functionality of a piece of code, which will be shown in an IDE, provided to developers.
The goal is to help developers quickly pick up the idea of that code.
Please explain the following piece of code to me: \n"#);
    }));
}