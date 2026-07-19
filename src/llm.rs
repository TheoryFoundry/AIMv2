use crate::ui::{COLOR_YELLOW, Spinner, print_api_error, style};
use anyhow::{Context, Result, bail};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCallChunk, ChatCompletionMessageToolCalls,
        ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestDeveloperMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestToolMessageContent, ChatCompletionRequestUserMessageContent,
        ChatCompletionStreamOptions, ChatCompletionTool, ChatCompletionTools, CompletionUsage,
        CreateChatCompletionRequestArgs, FunctionObject, FunctionType, ReasoningEffort,
    },
    types::responses::{
        CreateResponseArgs, EasyInputContent, EasyInputMessage, FunctionCallOutput,
        FunctionCallOutputItemParam, FunctionTool, FunctionToolCall, InputItem, InputParam, Item,
        OutputItem, OutputMessageContent, Reasoning, Response, Role, Tool,
    },
};
use futures::StreamExt;
use serde::Serialize;
use serde_json::{Map, Value, json};

#[derive(Clone, Debug)]
pub(crate) struct LlmConfig {
    pub(crate) model: String,
    pub(crate) reasoning_effort: ReasoningEffort,
}

pub(crate) type LlmClient = Client<OpenAIConfig>;

#[derive(Debug)]
pub(crate) struct LlmReply {
    pub(crate) content: String,
    pub(crate) reasoning: Option<String>,
    pub(crate) tool_calls: Vec<ToolCall>,
    pub(crate) input_tokens: Option<u64>,
    pub(crate) output_tokens: Option<u64>,
    pub(crate) total_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolCall {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) arguments: Value,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ToolMode {
    Agent { enable_shell: bool },
}

#[derive(Debug, Default)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

pub(crate) fn build_client(api_key: &str, base_url: &str) -> LlmClient {
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(base_url);
    Client::with_config(config)
}

pub(crate) async fn call_model<F>(
    client: &LlmClient,
    config: &LlmConfig,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_mode: Option<ToolMode>,
    show_spinner: bool,
    mut on_text: F,
) -> Result<LlmReply>
where
    F: FnMut(&str),
{
    match call_responses_api(
        client,
        config,
        messages.clone(),
        tool_mode,
        show_spinner,
        &mut on_text,
    )
    .await
    {
        Ok(reply) => return Ok(reply),
        Err(responses_err) => {
            let chat_result =
                call_chat_completions(client, config, messages, tool_mode, show_spinner, on_text)
                    .await;
            return chat_result.with_context(|| {
                format!("responses API request failed before chat completions fallback: {responses_err:#}")
            });
        }
    }
}

async fn call_chat_completions<F>(
    client: &LlmClient,
    config: &LlmConfig,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_mode: Option<ToolMode>,
    show_spinner: bool,
    mut on_text: F,
) -> Result<LlmReply>
where
    F: FnMut(&str),
{
    let stream_request = build_request(config, messages.clone(), tool_mode, true)?
        .build()
        .context("failed to build chat completion request")?;

    let mut stream = client
        .chat()
        .create_stream(stream_request)
        .await
        .context("chat completion request failed")?;

    let mut spinner = if show_spinner {
        Some(Spinner::start())
    } else {
        None
    };
    let mut content = String::new();
    let mut refusal = String::new();
    let reasoning: Option<String> = None;
    let mut usage: Option<CompletionUsage> = None;
    let mut tool_calls: Vec<PartialToolCall> = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("failed to receive streaming response chunk")?;
        if let Some(chunk_usage) = chunk.usage {
            usage = Some(chunk_usage);
        }

        for choice in chunk.choices {
            if let Some(text) = choice.delta.content {
                stop_spinner(&mut spinner);
                on_text(&text);
                content.push_str(&text);
            }
            if let Some(text) = choice.delta.refusal {
                stop_spinner(&mut spinner);
                on_text(&text);
                refusal.push_str(&text);
            }
            if let Some(chunks) = choice.delta.tool_calls {
                merge_tool_call_chunks(&mut tool_calls, chunks)?;
            }
        }
    }

    let streamed_content = if content.trim().is_empty() {
        refusal.trim().to_string()
    } else {
        content.trim().to_string()
    };
    if let Some(mut spinner) = spinner.take() {
        spinner.stop();
    }
    let usage_input = usage.as_ref().map(|usage| u64::from(usage.prompt_tokens));
    let usage_output = usage
        .as_ref()
        .map(|usage| u64::from(usage.completion_tokens));
    let usage_total = usage.as_ref().map(|usage| u64::from(usage.total_tokens));

    let tool_calls = match finalize_tool_calls(tool_calls) {
        Ok(tool_calls) => tool_calls,
        Err(_) if tool_mode.is_some() => {
            let fallback = fallback_non_stream(client, config, messages, tool_mode).await?;
            if streamed_content.is_empty() && !fallback.content.is_empty() {
                stop_spinner(&mut spinner);
                on_text(&fallback.content);
            }
            let content = if streamed_content.is_empty() {
                fallback.content
            } else {
                streamed_content.clone()
            };
            return Ok(LlmReply {
                content,
                reasoning: fallback.reasoning,
                tool_calls: fallback.tool_calls,
                input_tokens: sum_token_usage(usage_input, fallback.input_tokens),
                output_tokens: sum_token_usage(usage_output, fallback.output_tokens),
                total_tokens: sum_token_usage(usage_total, fallback.total_tokens),
            });
        }
        Err(err) => return Err(err),
    };

    if streamed_content.is_empty() && tool_calls.is_empty() {
        let fallback = fallback_non_stream(client, config, messages, tool_mode).await?;
        if !fallback.content.is_empty() {
            stop_spinner(&mut spinner);
            on_text(&fallback.content);
        }
        if fallback.content.trim().is_empty() && fallback.tool_calls.is_empty() {
            bail!("model returned neither content nor tool calls");
        }
        return Ok(LlmReply {
            content: fallback.content,
            reasoning: fallback.reasoning,
            tool_calls: fallback.tool_calls,
            input_tokens: sum_token_usage(usage_input, fallback.input_tokens),
            output_tokens: sum_token_usage(usage_output, fallback.output_tokens),
            total_tokens: sum_token_usage(usage_total, fallback.total_tokens),
        });
    }

    Ok(LlmReply {
        content: streamed_content,
        reasoning,
        tool_calls,
        input_tokens: usage_input,
        output_tokens: usage_output,
        total_tokens: usage_total,
    })
}

async fn call_responses_api<F>(
    client: &LlmClient,
    config: &LlmConfig,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_mode: Option<ToolMode>,
    show_spinner: bool,
    on_text: &mut F,
) -> Result<LlmReply>
where
    F: FnMut(&str),
{
    let request = build_response_request(config, messages, tool_mode)?
        .build()
        .context("failed to build responses API request")?;

    let mut spinner = if show_spinner {
        Some(Spinner::start())
    } else {
        None
    };
    let response = client
        .responses()
        .create(request)
        .await
        .context("responses API request failed")?;
    stop_spinner(&mut spinner);

    let reply = parse_response_api_reply(response)?;
    if !reply.content.is_empty() {
        on_text(&reply.content);
    }
    Ok(reply)
}

fn build_response_request(
    config: &LlmConfig,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_mode: Option<ToolMode>,
) -> Result<CreateResponseArgs> {
    let mut request = CreateResponseArgs::default();
    request.model(&config.model);
    request.input(InputParam::Items(response_input_items(messages)?));
    request.reasoning(Reasoning {
        effort: Some(config.reasoning_effort.clone()),
        summary: None,
    });
    request.store(false);
    if let Some(mode) = tool_mode {
        request.tools(response_tool_definitions(mode));
        request.parallel_tool_calls(false);
    }
    Ok(request)
}

fn response_input_items(messages: Vec<ChatCompletionRequestMessage>) -> Result<Vec<InputItem>> {
    let mut items = Vec::new();
    for message in messages {
        match message {
            ChatCompletionRequestMessage::Developer(message) => {
                items.push(easy_message(
                    Role::Developer,
                    developer_content_text(message.content),
                ));
            }
            ChatCompletionRequestMessage::System(message) => {
                items.push(easy_message(
                    Role::System,
                    system_content_text(message.content),
                ));
            }
            ChatCompletionRequestMessage::User(message) => {
                items.push(easy_message(Role::User, user_content_text(message.content)));
            }
            ChatCompletionRequestMessage::Assistant(message) => {
                if let Some(content) = message.content {
                    let text = assistant_content_text(content);
                    if !text.trim().is_empty() {
                        items.push(easy_message(Role::Assistant, text));
                    }
                }
                for tool_call in message.tool_calls.unwrap_or_default() {
                    match tool_call {
                        ChatCompletionMessageToolCalls::Function(call) => {
                            items.push(InputItem::Item(Item::FunctionCall(FunctionToolCall {
                                arguments: call.function.arguments,
                                call_id: call.id,
                                namespace: None,
                                name: call.function.name,
                                id: None,
                                status: None,
                            })));
                        }
                        ChatCompletionMessageToolCalls::Custom(call) => {
                            bail!(
                                "custom chat tool call cannot be converted to responses input: {}",
                                call.custom_tool.name
                            );
                        }
                    }
                }
            }
            ChatCompletionRequestMessage::Tool(message) => {
                items.push(InputItem::Item(Item::FunctionCallOutput(
                    FunctionCallOutputItemParam {
                        call_id: message.tool_call_id,
                        output: FunctionCallOutput::Text(tool_content_text(message.content)),
                        id: None,
                        status: None,
                    },
                )));
            }
            ChatCompletionRequestMessage::Function(message) => {
                let output = message.content.unwrap_or_default();
                items.push(InputItem::Item(Item::FunctionCallOutput(
                    FunctionCallOutputItemParam {
                        call_id: message.name,
                        output: FunctionCallOutput::Text(output),
                        id: None,
                        status: None,
                    },
                )));
            }
        }
    }
    Ok(items)
}

fn easy_message(role: Role, content: String) -> InputItem {
    InputItem::EasyMessage(EasyInputMessage {
        r#type: Default::default(),
        role,
        content: EasyInputContent::Text(content),
        phase: None,
    })
}

fn developer_content_text(content: ChatCompletionRequestDeveloperMessageContent) -> String {
    match content {
        ChatCompletionRequestDeveloperMessageContent::Text(text) => text,
        ChatCompletionRequestDeveloperMessageContent::Array(parts) => content_parts_text(parts),
    }
}

fn system_content_text(content: ChatCompletionRequestSystemMessageContent) -> String {
    match content {
        ChatCompletionRequestSystemMessageContent::Text(text) => text,
        ChatCompletionRequestSystemMessageContent::Array(parts) => content_parts_text(parts),
    }
}

fn user_content_text(content: ChatCompletionRequestUserMessageContent) -> String {
    match content {
        ChatCompletionRequestUserMessageContent::Text(text) => text,
        ChatCompletionRequestUserMessageContent::Array(parts) => content_parts_text(parts),
    }
}

fn assistant_content_text(content: ChatCompletionRequestAssistantMessageContent) -> String {
    match content {
        ChatCompletionRequestAssistantMessageContent::Text(text) => text,
        ChatCompletionRequestAssistantMessageContent::Array(parts) => content_parts_text(parts),
    }
}

fn tool_content_text(content: ChatCompletionRequestToolMessageContent) -> String {
    match content {
        ChatCompletionRequestToolMessageContent::Text(text) => text,
        ChatCompletionRequestToolMessageContent::Array(parts) => content_parts_text(parts),
    }
}

fn content_parts_text<T>(parts: Vec<T>) -> String
where
    T: Serialize,
{
    parts
        .into_iter()
        .map(|part| {
            let value = serde_json::to_value(part).unwrap_or(Value::Null);
            value
                .get("text")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn response_tool_definitions(mode: ToolMode) -> Vec<Tool> {
    tool_definitions(mode)
        .into_iter()
        .filter_map(|tool| match tool {
            ChatCompletionTools::Function(tool) => Some(Tool::Function(FunctionTool {
                name: tool.function.name,
                parameters: tool.function.parameters,
                strict: tool.function.strict,
                description: tool.function.description,
                defer_loading: None,
            })),
            ChatCompletionTools::Custom(_) => None,
        })
        .collect()
}

fn parse_response_api_reply(response: Response) -> Result<LlmReply> {
    if let Some(error) = response.error {
        bail!("responses API returned {}: {}", error.code, error.message);
    }

    let mut content = String::new();
    let mut refusal = String::new();
    let mut tool_calls = Vec::new();
    for item in response.output {
        match item {
            OutputItem::Message(message) => {
                for part in message.content {
                    match part {
                        OutputMessageContent::OutputText(text) => content.push_str(&text.text),
                        OutputMessageContent::Refusal(text) => refusal.push_str(&text.refusal),
                    }
                }
            }
            OutputItem::FunctionCall(call) => {
                let arguments = validate_tool_call_arguments(&call.name, &call.arguments)
                    .with_context(|| {
                        format!("tool call {} has malformed JSON arguments", call.call_id)
                    })?;
                tool_calls.push(ToolCall {
                    id: call.call_id,
                    name: call.name,
                    arguments,
                });
            }
            _ => {}
        }
    }

    let content = if content.trim().is_empty() {
        refusal.trim().to_string()
    } else {
        content.trim().to_string()
    };
    if content.is_empty() && tool_calls.is_empty() {
        bail!("responses API returned neither content nor tool calls");
    }

    let usage = response.usage;
    Ok(LlmReply {
        content,
        reasoning: None,
        tool_calls,
        input_tokens: usage.as_ref().map(|usage| u64::from(usage.input_tokens)),
        output_tokens: usage.as_ref().map(|usage| u64::from(usage.output_tokens)),
        total_tokens: usage.as_ref().map(|usage| u64::from(usage.total_tokens)),
    })
}

fn build_request(
    config: &LlmConfig,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_mode: Option<ToolMode>,
    stream: bool,
) -> Result<CreateChatCompletionRequestArgs> {
    let mut request = CreateChatCompletionRequestArgs::default();
    request.model(&config.model);
    request.messages(messages);
    request.reasoning_effort(config.reasoning_effort.clone());
    request.stream(stream);
    if stream {
        request.stream_options(ChatCompletionStreamOptions {
            include_usage: Some(true),
            include_obfuscation: None,
        });
    }
    if let Some(mode) = tool_mode {
        request.tools(tool_definitions(mode));
        request.parallel_tool_calls(false);
    }
    Ok(request)
}

async fn fallback_non_stream(
    client: &LlmClient,
    config: &LlmConfig,
    messages: Vec<ChatCompletionRequestMessage>,
    tool_mode: Option<ToolMode>,
) -> Result<LlmReply> {
    let request = build_request(config, messages, tool_mode, false)?
        .build()
        .context("failed to build fallback chat completion request")?;
    let response = client
        .chat()
        .create(request)
        .await
        .context("fallback chat completion request failed")?;
    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("fallback model response missing choices[0]"))?;
    let message = choice.message;
    let content = message
        .content
        .or(message.refusal)
        .unwrap_or_default()
        .trim()
        .to_string();
    let tool_calls = extract_tool_calls(message.tool_calls)?;
    let usage = response.usage;
    Ok(LlmReply {
        content,
        reasoning: None,
        tool_calls,
        input_tokens: usage.as_ref().map(|usage| u64::from(usage.prompt_tokens)),
        output_tokens: usage
            .as_ref()
            .map(|usage| u64::from(usage.completion_tokens)),
        total_tokens: usage.as_ref().map(|usage| u64::from(usage.total_tokens)),
    })
}

fn tool_definitions(mode: ToolMode) -> Vec<ChatCompletionTools> {
    match mode {
        ToolMode::Agent { enable_shell } => {
            let mut tools = vec![
                theorem_graph_push_tool_definition(),
                theorem_graph_list_tool_definition(),
                theorem_graph_list_deps_tool_definition(),
                theorem_graph_examine_tool_definition(),
                theorem_graph_review_tool_definition(),
                theorem_graph_revise_tool_definition(),
                theorem_graph_comment_tool_definition(),
            ];
            if enable_shell {
                tools.push(shell_tool_definition());
            }
            tools
        }
    }
}

fn shell_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "shell_tool".to_string(),
            description: Some(
                "Run a shell command inside the current workspace for inspection, editing, \
                 symbolic checks, numeric experiments, builds, and tests."
                    .to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The raw shell command to execute."
                    },
                    "workdir": {
                        "type": "string",
                        "description": "Optional relative working directory inside the workspace root."
                    }
                },
                "required": ["command"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_push_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_push".to_string(),
            description: Some(
                "Add a new theorem-graph entry. Use type=context for important facts supplied by the user or obtained from files, web search, or other external resources; in that case the proof should record the source or provenance. Use type=theorem only for important lemmas or theorems you have deduced yourself.".to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["context", "theorem"],
                        "description": "Whether this entry is prior context or a theorem established during the current exploration."
                    },
                    "statement": {
                        "type": "string",
                        "description": "The exact mathematical statement."
                    },
                    "proof": {
                        "type": "string",
                        "description": "A rigorous proof, or a reference note when the entry is context."
                    },
                    "dependencies": {
                        "type": "array",
                        "items": {"type": "integer", "minimum": 0},
                        "description": "Direct dependency ids in the theorem graph."
                    }
                },
                "required": ["type", "statement", "proof", "dependencies"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_list_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_list".to_string(),
            description: Some(
                "List theorem-graph entries in an id range, including statements, dependencies, and reviewer comments when present.".to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "start": {"type": "integer", "minimum": 0},
                    "end": {"type": "integer", "minimum": 0}
                },
                "required": ["start", "end"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_list_deps_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_list_deps".to_string(),
            description: Some(
                "Show a theorem entry together with its direct dependencies, including dependency statements, dependency links, review counts, and comments.".to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "minimum": 0}
                },
                "required": ["id"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_examine_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_examine".to_string(),
            description: Some(
                "Inspect one theorem-graph entry in full detail, including proof text, dependencies, derivations, reviews, and comments.".to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "minimum": 0}
                },
                "required": ["id"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_review_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_review".to_string(),
            description: Some(
                "Generate multiple subagents to review the current theorem and its dependencies in parallel."
                    .to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "minimum": 0}
                },
                "required": ["id"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_revise_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_revise".to_string(),
            description: Some(
                "Revise the proof and direct dependencies of an existing theorem-graph entry after a flaw has been identified.".to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "minimum": 0},
                    "proof": {
                        "type": "string",
                        "description": "The corrected proof text."
                    },
                    "dependencies": {
                        "type": "array",
                        "items": {"type": "integer", "minimum": 0},
                        "description": "The corrected direct dependency ids."
                    }
                },
                "required": ["id", "proof", "dependencies"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn theorem_graph_comment_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "theorem_graph_comment".to_string(),
            description: Some(
                "Append a reviewer comment to an existing theorem entry after detecting a proof error.".to_string(),
            ),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "minimum": 0},
                    "comment": {
                        "type": "string",
                        "description": "A concise description of the proof error that was found."
                    }
                },
                "required": ["id", "comment"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    })
}

fn merge_tool_call_chunks(
    tool_calls: &mut Vec<PartialToolCall>,
    chunks: Vec<ChatCompletionMessageToolCallChunk>,
) -> Result<()> {
    for chunk in chunks {
        let index = usize::try_from(chunk.index)
            .with_context(|| format!("invalid tool call index: {}", chunk.index))?;
        while tool_calls.len() <= index {
            tool_calls.push(PartialToolCall::default());
        }
        let entry = &mut tool_calls[index];
        if let Some(id) = chunk.id {
            entry.id = id;
        }
        if let Some(kind) = chunk.r#type {
            match kind {
                FunctionType::Function => {}
            }
        }
        if let Some(function) = chunk.function {
            if let Some(name) = function.name {
                entry.name = name;
            }
            if let Some(arguments) = function.arguments {
                entry.arguments.push_str(&arguments);
            }
        }
    }
    Ok(())
}

fn finalize_tool_calls(tool_calls: Vec<PartialToolCall>) -> Result<Vec<ToolCall>> {
    let mut finalized = Vec::with_capacity(tool_calls.len());
    for (index, call) in tool_calls.into_iter().enumerate() {
        if call.id.is_empty() && call.name.is_empty() && call.arguments.is_empty() {
            continue;
        }
        if call.id.is_empty() {
            bail!("tool call {index} missing id in streaming response");
        }
        if call.name.is_empty() {
            bail!(
                "tool call {} missing function name in streaming response",
                call.id
            );
        }
        let arguments = validate_tool_call_arguments(&call.name, &call.arguments)
            .with_context(|| format!("tool call {} has malformed JSON arguments", call.id))?;
        finalized.push(ToolCall {
            id: call.id,
            name: call.name,
            arguments,
        });
    }
    Ok(finalized)
}

#[allow(dead_code)]
fn extract_tool_calls(
    tool_calls: Option<Vec<ChatCompletionMessageToolCalls>>,
) -> Result<Vec<ToolCall>> {
    let mut parsed = Vec::new();
    for tool_call in tool_calls.unwrap_or_default() {
        match tool_call {
            ChatCompletionMessageToolCalls::Function(call) => {
                let arguments =
                    validate_tool_call_arguments(&call.function.name, &call.function.arguments)
                        .with_context(|| {
                            format!("tool call {} has malformed JSON arguments", call.id)
                        })?;
                parsed.push(ToolCall {
                    id: call.id,
                    name: call.function.name,
                    arguments,
                });
            }
            ChatCompletionMessageToolCalls::Custom(call) => {
                bail!("unsupported custom tool call: {}", call.custom_tool.name);
            }
        }
    }
    Ok(parsed)
}

fn validate_tool_call_arguments(tool_name: &str, arguments: &str) -> Result<Value> {
    let trimmed = arguments.trim();
    if trimmed.is_empty() {
        bail!("tool {tool_name} returned empty arguments");
    }
    let value = serde_json::from_str::<Value>(trimmed)
        .with_context(|| format!("tool {tool_name} arguments are not valid JSON: {trimmed}"))?;
    match value {
        Value::Object(_) => Ok(value),
        other => bail!(
            "tool {tool_name} arguments must decode to a JSON object, got {}",
            json_type_name(&other)
        ),
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub(crate) fn tool_arguments_as_object(arguments: &Value) -> Result<Map<String, Value>> {
    match arguments {
        Value::Object(map) => Ok(map.clone()),
        other => bail!(
            "tool arguments must be a JSON object, got {}",
            json_type_name(other)
        ),
    }
}

fn sum_token_usage(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn stop_spinner(spinner: &mut Option<Spinner>) {
    if let Some(mut spinner) = spinner.take() {
        spinner.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::{stop_spinner, sum_token_usage};
    use crate::ui::Spinner;

    #[test]
    fn sums_usage_across_stream_and_fallback_requests() {
        assert_eq!(sum_token_usage(Some(12), Some(8)), Some(20));
        assert_eq!(sum_token_usage(Some(12), None), Some(12));
        assert_eq!(sum_token_usage(None, Some(8)), Some(8));
        assert_eq!(sum_token_usage(None, None), None);
    }

    #[test]
    fn stop_spinner_consumes_spinner_once() {
        let mut spinner = Some(Spinner::start());
        stop_spinner(&mut spinner);
        assert!(spinner.is_none());
        stop_spinner(&mut spinner);
        assert!(spinner.is_none());
    }
}

pub(crate) fn report_api_error(err: &anyhow::Error) {
    print_api_error(&format!("{err:#}"));
    println!(
        "{} request failed; inspect configuration, model name, and API compatibility",
        style(COLOR_YELLOW, "warning>")
    );
}
