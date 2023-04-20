# ChatToy

This is a naive chat-bot, implemented in Rust as a wrapper around OpenAI's API. Feel free to use and suggest!


## Usages

The `State` data structure are designed as a convenient way to build prompts. Here are some examples:



```rust
impl StatesManager {
    /// Summarize the git change, returning a git commit command with summarization as the message
    pub(crate) fn summarizer() -> State {
        State::chat()
            .chat_with_prefix(
                "Summarize the changes of code in a project, in a git-commit message form. The changes are given in the format of git diff:\n"
            )
            .chat_with_suffix("\nPlease directly answer with the bash script together with the \
            summary in markdown format, with no filter words")
    }

    /// Find the error from where the error originates from, with the given error message
    pub(crate) fn error_message_file_finder() -> State {
        State::chat()
            .chat_with_prefix(
                "Find the name of the file from where the error originates, from the given \
                    error message:\n",
            )
            .chat_with_suffix("\n\nPlease directly answer with the path list")
    }

    /// Fix the error, with:
    /// 1. the fix suggestion
    /// 2. the corrected code
    pub(crate) fn error_fixer(file_content: String) -> State {
        State::chat()
            .chat_with_prefix(
                format!(
                    "Provide the appropriate way to fix the error, with the content of the file from \
                where the error originates:\n{}, and the error message:\n",
                    file_content,
                )
                    .as_str(),
            )
            .chat_with_suffix(
                "\n\nAnswer in correct markdown format, as a list of: 1. the fix 2. the corrected \
                code in markdown format",
            )
    }

    /// Translate the bash command to target shell command
    ///
    /// ## Input:
    /// ```bash
    /// while read -r fname lname a b c d;
    ///     do
    ///         x="$fname $lname"
    ///
    ///         if [ "$x" = "$1" ] then
    ///             avg=$(( (a + b + c + d) / 4 ))
    ///             echo $avg
    ///             break
    ///         fi
    /// done <class.txt
    /// ```
    /// translated into powershell command:
    ///
    /// ## Output
    /// ```powershell
    /// Get-Content class.txt | ForEach-Object {
    ///     $fname, $lname, $a, $b, $c, $d = $_ -split ' '
    ///     $x = "$fname $lname"
    ///     if ($x -eq $args[0]) {
    ///         $avg = [int]($a + $b + $c + $d) / 4
    ///         Write-Output $avg
    ///         break
    ///     }
    /// }
    /// ```
    pub(crate) fn code_translator(shell_name: &str) -> State {
        State::chat()
            .chat_with_prefix(
                format!(
                    "Translate the following shell command, into the same command in {}:\n",
                    shell_name
                )
                    .as_str(),
            )
            .with_additional_context(ChatCompletionRequestMessage {
                role: Role::User,
                content: r#"
while read -r fname lname a b c d;
    do
        x="$fname $lname"

        if [ "$x" = "$1" ] then
            avg=$(( (a + b + c + d) / 4 ))
            echo $avg
            break
        fi
done <class.txt
        "#
                    .to_string(),
                name: None,
            })
            .with_additional_context(ChatCompletionRequestMessage {
                role: Role::Assistant,
                content: r#"Get-Content class.txt | ForEach-Object {
    $fname, $lname, $a, $b, $c, $d = $_ -split ' '
    $x = "$fname $lname"
    if ($x -eq $args[0]) {
        $avg = [int]($a + $b + $c + $d) / 4
        Write-Output $avg
        break
    }
}"#
                    .to_string(),
                name: None,
            })
    }
}
```