set -e
cargo build --example bot
sudo cp ./target/debug/examples/bot /usr/local/bin/chatbot

