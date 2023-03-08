use chat_toy::chat_api::ChatBot;

use std::io;
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
                chat_bot.set_state(State::Chat(Arc::new(|_| {})));
            } else if raw_input == "explain" {
                // code completion
                let explainer = State::Chat(Arc::new(|code_snippet| {
                    code_snippet.insert_str(0, "I need you to provide a short, \
                    summarized explanation(a few sentences) \
                    to describe the functionality of a piece of code, which will be shown in an \
                    IDE, provided to developers. The goal is to help developers quickly pick up \
                    the idea of that code. Please explain the following piece of code to me: \n");
                }));
                chat_bot.set_state(explainer);
            } else if raw_input.eq("code") {
                // code completion
                chat_bot.set_state(State::CodeCompletion);
            } else {
                match chat_bot.input_with_state(raw_input.to_string()).await {
                    Ok(res) => println!("{}:\n\n{}", res[0].role, res[0].content),
                    Err(res) => println!("{}", res),
                }
            }
            input.clear();
        }
    }
}
