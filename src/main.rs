use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        if let Some((line, character)) = find_main_function_position(&text) {
            let diagnostic = Diagnostic {
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + 7,
                    }, // "fn main" is 7 characters long
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                code: None,
                code_description: None,
                source: Some("Hofmeister LSP".to_string()),
                message: "Greetings Commander".to_string(),
                related_information: None,
                tags: None,
                data: None,
            };

            self.client
                .publish_diagnostics(uri, vec![diagnostic], None)
                .await;
        }
    }
}

fn find_main_function_position(text: &str) -> Option<(u32, u32)> {
    for (line_number, line) in text.lines().enumerate() {
        if let Some(character) = line.find("fn main()") {
            return Some((line_number as u32, character as u32));
        }
    }
    None
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
