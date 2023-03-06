use chat_toy::chatapi::ChatBot;

use std::fmt::{Debug, format, Formatter, Pointer};
use std::io;
use reqwest::{Body, Client, get, Proxy};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, to_string, Value};
use chat_toy::chatapi::*;

#[tokio::main]
async fn main() {
    if let Ok(mut chat_bot) = ChatBot::new() {
        println!("Bot started");
        let help = r#"This is a naive chat bot, implemented in Rust as a wrapper around OpenAI's API. Feel free to use and submit suggestions!
Enter:
'-l' to list all openai models
'-c ${{model_name}}' to customize your model (Some of the listed models may not be released by OpenAI yet)
'code' to start code completion mode
'chat' to start chatting
'explain' to explain a piece of code
'q' to quit"#;
        println!("{}", help);
        let mut input = "".to_string();
        while let Ok(_input_size) = io::stdin().read_line(&mut input) {
            let raw_input = input.trim_end().to_string();
            // let args = cli::Args::try_parse_from([raw_input.to_string()].into_iter());


            if raw_input.is_empty() {} else if raw_input == "q" {
                // quiting
                return;
            } else if raw_input == "-h" {
                // print help
                println!("{}", help);
            } else if raw_input == "-l" {
                // listing models
                let _ = chat_bot.list_model().await;
            } else if raw_input.starts_with("-c") {
                // customizing models
                let model = raw_input.trim_start_matches("-c ");
                if model.is_empty() {
                    println!("invalid format. Please check your input")
                } else {
                    chat_bot.set_state(State::Other(model.to_string()));
                }
            } else if raw_input == "chat" {
                // code completion
                chat_bot.set_state(State::Chat(Box::new(|_| {})));
            } else if raw_input == "explain" {
                // code completion
                chat_bot.set_state(State::Chat(Box::new(|s| {
                    s.insert_str(0, "I need you to provide a short explanation(a paragraph) \
                    to describe the functionality of a piece of code, which will be shown in an \
                    IDE, provided to developers. The goal is to help developers quickly pick up \
                    the idea of that code. Please explain the following piece of code to me: \n")
                })));
            } else if raw_input.eq("code") {
                // code completion
                chat_bot.set_state(State::CodeCompletion);
            } else {
                match  chat_bot.input_with_state(raw_input.to_string()).await {
                    Ok(res) => println!("{}", res),
                    Err(res) => println!("{}", res),
                }
            }
            input.clear();
        }
    }
}
