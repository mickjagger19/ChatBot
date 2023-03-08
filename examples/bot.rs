use chat_toy::chat_api::ChatBot;

use std::io;
use std::io::{BufRead, Read};
use std::sync::Arc;
use chat_toy::chat_api::*;

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
        let mut user_input = String::new();
        loop {
            let mut lines = io::stdin().lock().lines();
            while let Some(line) = lines.next() {
                let last_input = line.unwrap();

                // stop reading
                if last_input.len() == 0 {
                    break;
                }

                // add a new line once user_input starts storing user input
                if user_input.len() > 0 {
                    user_input.push_str("\n");
                }

                // store user input
                user_input.push_str(&last_input);
            }
            let user_input = user_input.trim_end();
            // let args = cli::Args::try_parse_from([raw_input.to_string()].into_iter());
            if user_input.is_empty() {} else if user_input == "q" {
                // quiting
                return;
            } else if user_input == "-h" {
                // print help
                println!("{}", help);
            } else if user_input == "-l" {
                // listing models
                let _ = chat_bot.list_model().await;
            } else if user_input.starts_with("-c") {
                // customizing models
                let model = user_input.trim_start_matches("-c ");
                if model.is_empty() {
                    println!("invalid format. Please check your input")
                } else {
                    chat_bot.set_state(State::Other(model.to_string()));
                }
            } else if user_input == "chat" {
                // code completion
                chat_bot.set_state(State::Chat(Arc::new(|_| {})));
            } else if user_input == "explain" {
                // code completion
                let explainer = State::Chat(Arc::new(|code_snippet| {
                    code_snippet.insert_str(0, "I need you to provide a short, \
                    summarized explanation(a few sentences) \
                    to describe the functionality of a piece of code, which will be shown in an \
                    IDE, provided to developers. The goal is to help developers quickly pick up \
                    the idea of that code. Please explain the following piece of code to me: \n");
                }));
                chat_bot.set_state(explainer);
            } else if user_input.eq("code") {
                // code completion
                chat_bot.set_state(State::CodeCompletion);
            } else {
                if !user_input.trim().is_empty() {
                    match chat_bot.input_with_state(user_input.to_string()).await
                    {
                        Ok(res) => println!("{}:\n\n{}", res[0].role, res[0].content),
                        Err(res) => println!("{}", res),
                    }
                }
            }
            input.clear();
        }
    }
}
