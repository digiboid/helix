use completion::{CompletionEvent, CompletionHandler};
use helix_event::send_blocking;
use tokio::sync::mpsc::Sender;

use crate::handlers::lsp::SignatureHelpInvoked;
use crate::{DocumentId, Editor, ViewId};

pub mod completion;
pub mod dap;
pub mod diagnostics;
pub mod lsp;

#[derive(Debug)]
pub enum AutoSaveEvent {
    DocumentChanged { save_after: u64 },
    LeftInsertMode,
}

#[derive(Debug)]
pub enum AutoReloadEvent {
    CheckForChanges { after: u64 },
    EditorFocused,
    LeftInsertMode,
}

pub struct Handlers {
    // only public because most of the actual implementation is in helix-term right now :/
    pub completions: CompletionHandler,
    pub signature_hints: Sender<lsp::SignatureHelpEvent>,
    pub auto_save: Sender<AutoSaveEvent>,
    pub auto_reload: Sender<AutoReloadEvent>,
    pub document_colors: Sender<lsp::DocumentColorsEvent>,
}

impl Handlers {
    /// Manually trigger completion (c-x)
    pub fn trigger_completions(&self, trigger_pos: usize, doc: DocumentId, view: ViewId) {
        self.completions.event(CompletionEvent::ManualTrigger {
            cursor: trigger_pos,
            doc,
            view,
        });
    }

    pub fn trigger_signature_help(&self, invocation: SignatureHelpInvoked, editor: &Editor) {
        let event = match invocation {
            SignatureHelpInvoked::Automatic => {
                if !editor.config().lsp.auto_signature_help {
                    return;
                }
                lsp::SignatureHelpEvent::Trigger
            }
            SignatureHelpInvoked::Manual => lsp::SignatureHelpEvent::Invoked,
        };
        send_blocking(&self.signature_hints, event)
    }
}

pub fn register_hooks(handlers: &Handlers) {
    lsp::register_hooks(handlers);
}
