use std::path::PathBuf;
use std::sync::Mutex;
use lsp_max::jsonrpc::Result;
use lsp_max::lsp_types::*;
use lsp_max::{Client, LanguageServer};

use crate::diagnostics::{find_asset_root, run_diagnostics};
use crate::ocel::log_event;

pub struct GgenAssetLspServer {
    pub client: Client,
    pub workspace_root: Mutex<Option<PathBuf>>,
}

impl GgenAssetLspServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_root: Mutex::new(None),
        }
    }

    async fn update_diagnostics(&self, doc_uri: Url, current_content: Option<&str>) {
        let url = match url::Url::parse(doc_uri.as_str()) {
            Ok(u) => u,
            Err(_) => return,
        };
        let doc_path = match url.to_file_path() {
            Ok(p) => p,
            Err(_) => return,
        };

        let asset_root = match find_asset_root(&doc_path) {
            Some(root) => root,
            None => return,
        };

        // Scan the asset_root for all `.usda` files
        for entry in walkdir::WalkDir::new(&asset_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("usda") {
                let is_current = path == doc_path;
                let content = if is_current {
                    if let Some(text) = current_content {
                        Some(text.to_string())
                    } else {
                        std::fs::read_to_string(path).ok()
                    }
                } else {
                    std::fs::read_to_string(path).ok()
                };

                if let Some(text) = content {
                    let diags = run_diagnostics(path, &text);
                    if let Some(url_obj) = url::Url::from_file_path(path).ok() {
                        if let Ok(uri) = url_obj.as_str().parse::<Url>() {
                            self.client
                                .publish_diagnostics(uri, diags.clone(), None)
                                .await;

                            // Log Validate activity to OCEL
                            log_event(
                                &asset_root,
                                "Validate",
                                path,
                                vec![(
                                    "diagnostic_count".to_string(),
                                    serde_json::json!(diags.len()),
                                )],
                            );
                        }
                    }
                }
            }
        }
    }
}

#[lsp_max::async_trait]
impl LanguageServer for GgenAssetLspServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri {
            let url = url::Url::parse(root_uri.as_str()).ok();
            if let Some(u) = url {
                if let Ok(path) = u.to_file_path() {
                    let mut lock = self.workspace_root.lock().unwrap();
                    *lock = Some(path);
                }
            }
        } else if let Some(root_path) = params.root_path {
            let mut lock = self.workspace_root.lock().unwrap();
            *lock = Some(PathBuf::from(root_path));
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "ggen-asset-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ggen-asset-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_diagnostics(
            params.text_document.uri,
            Some(&params.text_document.text),
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.update_diagnostics(
                params.text_document.uri,
                Some(&change.text),
            )
            .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.update_diagnostics(params.text_document.uri, None).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let actions = crate::code_actions::handle_code_action(params);
        Ok(actions)
    }
}
