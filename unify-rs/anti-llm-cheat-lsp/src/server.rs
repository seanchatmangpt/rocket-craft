use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::engine;

pub struct AntiLlmServer {
    pub client: Client,
    pub workspace_root: Arc<Mutex<Option<String>>>,
}

impl AntiLlmServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_root: Arc::new(Mutex::new(None)),
        }
    }

    async fn run_scan_and_publish(&self, uri: &Url) {
        let root_dir = {
            let guard = self.workspace_root.lock().unwrap();
            guard.clone().unwrap_or_else(|| ".".to_string())
        };

        let obs = engine::scan_directory(&root_dir);
        let diags = engine::evaluate_diagnostics(&obs);

        let file_diags: Vec<Diagnostic> = diags
            .iter()
            .filter(|d| {
                let norm_path = d.file_path.replace('\\', "/");
                let norm_uri = uri.to_string().replace('\\', "/");
                norm_uri.ends_with(&norm_path)
            })
            .map(|d| d.to_lsp())
            .collect();

        self.client
            .publish_diagnostics(uri.clone(), file_diags, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AntiLlmServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(uri) = params.root_uri {
            if let Ok(url) = url::Url::parse(uri.as_str()) {
                if let Ok(path) = url.to_file_path() {
                    let mut root = self.workspace_root.lock().unwrap();
                    *root = Some(path.to_string_lossy().to_string());
                }
            }
        }

        let caps = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec!["#".to_string()]),
                ..Default::default()
            }),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
            }),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities: caps,
            server_info: Some(ServerInfo {
                name: "anti-llm-cheat-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "anti-llm-cheat-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.run_scan_and_publish(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.run_scan_and_publish(&params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.run_scan_and_publish(&params.text_document.uri).await;
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = vec![
            CompletionItem {
                label: "FAILSET_NONEMPTY".to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("Active blocking failset exists".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "CANDIDATE".to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("All requirements met, awaiting audit".to_string()),
                ..Default::default()
            },
        ];
        Ok(Some(CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items,
        })))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let _uri = params.text_document.uri;
        let actions = vec![CodeActionOrCommand::CodeAction(CodeAction {
            title: "Open anti-llm failset report".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            command: Some(Command {
                title: "Open Failset".to_string(),
                command: "anti-llm.openFailset".to_string(),
                arguments: None,
            }),
            ..Default::default()
        })];
        Ok(Some(actions))
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        let lens = vec![CodeLens {
            range: Range::new(Position::new(0, 0), Position::new(0, 5)),
            command: Some(Command {
                title: "Admissibility Check Active".to_string(),
                command: "anti-llm.check".to_string(),
                arguments: None,
            }),
            data: Some(serde_json::json!({ "uri": uri.as_str() })),
        }];
        Ok(Some(lens))
    }

    async fn code_lens_resolve(&self, mut code_lens: CodeLens) -> Result<CodeLens> {
        if let Some(data) = &code_lens.data {
            if let Some(uri) = data.get("uri").and_then(|u| u.as_str()) {
                code_lens.command = Some(Command {
                    title: format!("Check: {}", uri),
                    command: "anti-llm.check".to_string(),
                    arguments: None,
                });
            }
        }
        Ok(code_lens)
    }
}
