mod formatter;

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{
    DocumentRangeFormattingParams, InitializeResult, Position, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
};
use std::collections::HashMap;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the transport over stdio
    let (connection, io_threads) = Connection::stdio();

    // Run the server
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_range_formatting_provider: Some(lsp_types::OneOf::Left(true)),
        ..Default::default()
    })?;

    let initialization_result = serde_json::to_value(InitializeResult {
        capabilities: serde_json::from_value(server_capabilities)?,
        server_info: Some(ServerInfo {
            name: "tablefmt-lsp".to_string(),
            version: Some("0.1.0".to_string()),
        }),
    })?;

    connection.initialize(initialization_result)?;

    // Store document contents
    let mut documents: HashMap<String, String> = HashMap::new();

    // Main message loop
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                handle_request(&connection, req, &documents)?;
            }
            Message::Notification(notif) => {
                handle_notification(&mut documents, notif);
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn handle_request(
    connection: &Connection,
    req: Request,
    documents: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    match req.method.as_str() {
        "textDocument/rangeFormatting" => {
            let (id, params): (RequestId, DocumentRangeFormattingParams) =
                req.extract("textDocument/rangeFormatting")?;

            let uri = params.text_document.uri.to_string();
            let range = params.range;

            let result = if let Some(content) = documents.get(&uri) {
                let edits = format_range(content, range);
                Some(edits)
            } else {
                None
            };

            let response = Response::new_ok(id, result);
            connection.sender.send(Message::Response(response))?;
        }
        _ => {
            // Unknown request - respond with method not found
            let response = Response::new_err(
                req.id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                format!("Method not found: {}", req.method),
            );
            connection.sender.send(Message::Response(response))?;
        }
    }
    Ok(())
}

fn handle_notification(documents: &mut HashMap<String, String>, notif: lsp_server::Notification) {
    match notif.method.as_str() {
        "textDocument/didOpen" => {
            if let Ok(params) =
                serde_json::from_value::<lsp_types::DidOpenTextDocumentParams>(notif.params)
            {
                let uri = params.text_document.uri.to_string();
                let text = params.text_document.text;
                documents.insert(uri, text);
            }
        }
        "textDocument/didChange" => {
            if let Ok(params) =
                serde_json::from_value::<lsp_types::DidChangeTextDocumentParams>(notif.params)
            {
                let uri = params.text_document.uri.to_string();
                // With full sync, the entire content is in the first change
                if let Some(change) = params.content_changes.into_iter().next() {
                    documents.insert(uri, change.text);
                }
            }
        }
        "textDocument/didClose" => {
            if let Ok(params) =
                serde_json::from_value::<lsp_types::DidCloseTextDocumentParams>(notif.params)
            {
                let uri = params.text_document.uri.to_string();
                documents.remove(&uri);
            }
        }
        _ => {}
    }
}

fn format_range(content: &str, range: Range) -> Vec<TextEdit> {
    let lines: Vec<&str> = content.lines().collect();

    // Extract the selected text based on range
    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;

    if start_line >= lines.len() {
        return vec![];
    }

    let end_line = end_line.min(lines.len() - 1);

    // Get the selected lines
    let selected_lines: Vec<&str> = lines[start_line..=end_line].to_vec();
    let selected_text = selected_lines.join("\n");

    // Format the table
    let formatted = formatter::format_table(&selected_text);

    if formatted == selected_text {
        return vec![];
    }

    // Calculate the end position
    let last_line_len = lines.get(end_line).map(|l| l.len()).unwrap_or(0);

    vec![TextEdit {
        range: Range {
            start: Position {
                line: start_line as u32,
                character: 0,
            },
            end: Position {
                line: end_line as u32,
                character: last_line_len as u32,
            },
        },
        new_text: formatted,
    }]
}
