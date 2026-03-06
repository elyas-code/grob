use boa_engine::{Context, JsValue, Source};
use crate::dom::Dom;
use std::rc::Rc;
use std::cell::RefCell;

/// Manages the JavaScript runtime with DOM bindings
pub struct JsRuntime {
    context: Context,
    dom: Rc<RefCell<Option<Dom>>>,
}

impl JsRuntime {
    /// Create a new JavaScript runtime with DOM bindings
    pub fn new() -> Self {
        let mut runtime = JsRuntime {
            context: Context::default(),
            dom: Rc::new(RefCell::new(None)),
        };
        runtime.setup_dom_bindings();
        runtime
    }

    /// Set the DOM reference for JavaScript code to access
    pub fn set_dom(&self, dom: Dom) {
        *self.dom.borrow_mut() = Some(dom);
    }

    /// Take ownership of the DOM (for updating after script execution)
    pub fn take_dom(&self) -> Option<Dom> {
        self.dom.borrow_mut().take()
    }

    /// Execute inline JavaScript code
    pub fn execute_script(&mut self, code: &str) -> Result<JsValue, Box<boa_engine::JsError>> {
        let source = Source::from_bytes(code);
        self.context.eval(source).map_err(Box::new)
    }

    /// Get mutable reference to the context for advanced operations
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Get reference to the context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Setup global DOM bindings for JavaScript
    fn setup_dom_bindings(&mut self) {
        // Initialize basic globals with simple JavaScript code
        // This creates console, document, and window objects
        self.context
            .eval(Source::from_bytes(
                r#"
                // Console object for debugging
                globalThis.console = {
                    log: function(...args) {
                        // Simple no-op implementation
                    },
                    error: function(...args) {},
                    warn: function(...args) {},
                    info: function(...args) {}
                };

                // Document object - minimal API for now
                globalThis.document = {
                    title: "",
                    createElement: function(tag) {
                        return { tag: tag, textContent: "", attributes: {} };
                    },
                    createTextNode: function(text) {
                        return { nodeType: 3, textContent: text };
                    },
                    querySelector: function(selector) {
                        return null;
                    },
                    querySelectorAll: function(selector) {
                        return [];
                    }
                };

                // Window object
                globalThis.window = globalThis;

                // Timer functions (no-op stubs)
                globalThis.setTimeout = function(fn, delay) { return 0; };
                globalThis.setInterval = function(fn, delay) { return 0; };
                globalThis.clearTimeout = function(id) {};
                globalThis.clearInterval = function(id) {};
                "#
            ))
            .ok();
    }
}

impl Default for JsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_runtime_creation() {
        let _runtime = JsRuntime::new();
        // Basic smoke test
    }

    #[test]
    fn test_js_execution() {
        let mut runtime = JsRuntime::new();
        let result = runtime.execute_script("2 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_log() {
        let mut runtime = JsRuntime::new();
        let result = runtime.execute_script("console.log('test')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_document_access() {
        let mut runtime = JsRuntime::new();
        let result = runtime.execute_script("document.title = 'Test'; document.title");
        assert!(result.is_ok());
    }
}
