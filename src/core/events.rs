//! # Event System
//!
//! Central event system for Xylux IDE that handles communication between
//! different components using an async event bus.

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, error, trace, warn};

use crate::core::error::Result;

/// Maximum number of events to keep in the broadcast channel.
const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Event priorities for handling order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Critical events that must be handled immediately.
    Critical = 0,
    /// High priority events (user input, LSP responses).
    High = 1,
    /// Normal priority events (file changes, UI updates).
    Normal = 2,
    /// Low priority events (background tasks, logging).
    Low = 3,
}

/// Main event types that can occur in the IDE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Editor-related events.
    Editor(EditorEvent),
    /// UI-related events.
    Ui(UiEvent),
    /// File system events.
    FileSystem(FileSystemEvent),
    /// LSP events.
    Lsp(LspEvent),
    /// Project events.
    Project(ProjectEvent),
    /// Build system events.
    Build(BuildEvent),
    /// Alux language events.
    Alux(AluxEvent),
    /// Xylux engine events.
    Xylux(XyluxEvent),
    /// Plugin events.
    Plugin(PluginEvent),
    /// System events.
    System(SystemEvent),
}

/// Editor-specific events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorEvent {
    /// Text was inserted at a specific position.
    TextInserted {
        /// Line number where text was inserted.
        line: usize,
        /// Column position where text was inserted.
        column: usize,
        /// The text that was inserted.
        text: String,
    },
    /// Text was deleted from a range.
    TextDeleted {
        /// Starting line of deleted text.
        start_line: usize,
        /// Starting column of deleted text.
        start_column: usize,
        /// Ending line of deleted text.
        end_line: usize,
        /// Ending column of deleted text.
        end_column: usize,
    },
    /// Cursor moved to a new position.
    CursorMoved {
        /// New cursor line position.
        line: usize,
        /// New cursor column position.
        column: usize,
    },
    /// File was opened.
    FileOpened {
        /// Path of the opened file.
        path: PathBuf,
    },
    /// File was closed.
    FileClosed {
        /// Path of the closed file.
        path: PathBuf,
    },
    /// File was saved.
    FileSaved {
        /// Path of the saved file.
        path: PathBuf,
    },
    /// Selection changed.
    SelectionChanged {
        /// Starting line of selection.
        start_line: usize,
        /// Starting column of selection.
        start_column: usize,
        /// Ending line of selection.
        end_line: usize,
        /// Ending column of selection.
        end_column: usize,
    },
    /// Find/replace operation.
    FindReplace {
        /// Search query string.
        query: String,
        /// Optional replacement string.
        replacement: Option<String>,
    },
    /// Undo operation performed.
    Undo,
    /// Redo operation performed.
    Redo,
}

/// UI-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiEvent {
    /// Window was resized.
    WindowResized {
        /// New window width.
        width: u16,
        /// New window height.
        height: u16,
    },
    /// Theme changed.
    ThemeChanged {
        /// Name of the new theme.
        theme: String,
    },
    /// Font changed.
    FontChanged {
        /// New font family name.
        family: String,
        /// New font size.
        size: u16,
    },
    /// Panel toggled.
    PanelToggled {
        /// Name of the panel that was toggled.
        panel: String,
        /// Whether the panel is now visible.
        visible: bool,
    },
    /// Status message displayed.
    StatusMessage {
        /// The status message text.
        message: String,
        /// The message level (info, warning, error).
        level: String,
    },
    /// Command executed.
    CommandExecuted {
        /// The command that was executed.
        command: String,
        /// Arguments passed to the command.
        args: Vec<String>,
    },
    /// Key was pressed.
    KeyPressed {
        /// The key that was pressed.
        key: String,
        /// List of modifier keys (ctrl, alt, shift, etc.).
        modifiers: Vec<String>,
    },
    /// Mouse event.
    MouseEvent {
        /// Type of mouse event (click, move, etc.).
        event_type: MouseEventType,
        /// X coordinate of mouse position.
        x: u16,
        /// Y coordinate of mouse position.
        y: u16,
    },
}

/// Message levels for status messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLevel {
    /// Informational message.
    Info,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
    /// Success message.
    Success,
}

/// Mouse event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseEventType {
    /// Single click event.
    Click,
    /// Double click event.
    DoubleClick,
    /// Right click event.
    RightClick,
    /// Scroll wheel event.
    Scroll {
        /// Scroll delta amount.
        delta: i32,
    },
    /// Drag event.
    Drag {
        /// Starting X coordinate.
        from_x: u16,
        /// Starting Y coordinate.
        from_y: u16,
    },
}

/// File system events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemEvent {
    /// File was created.
    FileCreated {
        /// Path to the created file.
        path: PathBuf,
    },
    /// File was modified.
    FileModified {
        /// Path to the modified file.
        path: PathBuf,
    },
    /// File was deleted.
    FileDeleted {
        /// Path to the deleted file.
        path: PathBuf,
    },
    /// File was renamed.
    FileRenamed {
        /// Original file path.
        old_path: PathBuf,
        /// New file path.
        new_path: PathBuf,
    },
    /// Directory was created.
    DirectoryCreated {
        /// Path to the created directory.
        path: PathBuf,
    },
    /// Directory was deleted.
    DirectoryDeleted {
        /// Path to the deleted directory.
        path: PathBuf,
    },
}

/// LSP events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LspEvent {
    /// LSP server started.
    ServerStarted {
        /// Programming language for the LSP server.
        language: String,
    },
    /// LSP server stopped.
    ServerStopped {
        /// Programming language for the LSP server.
        language: String,
    },
    /// Diagnostics received from LSP server.
    DiagnosticsReceived {
        /// Path of the file with diagnostics.
        path: PathBuf,
        /// List of diagnostic messages.
        diagnostics: Vec<LspDiagnostic>,
    },
    /// Completion items received.
    CompletionReceived {
        /// List of completion items.
        items: Vec<LspCompletionItem>,
    },
    /// Hover information received.
    HoverReceived {
        /// Hover content text.
        content: String,
    },
    /// Code actions received.
    CodeActionReceived {
        /// List of available code actions.
        actions: Vec<LspCodeAction>,
    },
    /// LSP error occurred.
    Error {
        /// Programming language for the LSP server.
        language: String,
        /// Error message.
        error: String,
    },
}

/// LSP diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    /// Line number where diagnostic starts.
    pub line: usize,
    /// Column number where diagnostic starts.
    pub column: usize,
    /// Line number where diagnostic ends.
    pub end_line: usize,
    /// Column number where diagnostic ends.
    pub end_column: usize,
    /// Severity level of the diagnostic.
    pub severity: DiagnosticSeverity,
    /// Diagnostic message text.
    pub message: String,
    /// Source of the diagnostic (e.g., compiler name).
    pub source: Option<String>,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Error level diagnostic.
    Error,
    /// Warning level diagnostic.
    Warning,
    /// Informational diagnostic.
    Info,
    /// Hint level diagnostic.
    Hint,
}

/// LSP completion item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCompletionItem {
    /// Display label for the completion item.
    pub label: String,
    /// Kind of completion item (function, variable, etc.).
    pub kind: Option<CompletionItemKind>,
    /// Additional detail information.
    pub detail: Option<String>,
    /// Documentation for the completion item.
    pub documentation: Option<String>,
    /// Text to insert when this completion is selected.
    pub insert_text: Option<String>,
}

/// LSP completion item kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemKind {
    /// Plain text completion.
    Text,
    /// Method completion.
    Method,
    /// Function completion.
    Function,
    /// Constructor completion.
    Constructor,
    /// Field completion.
    Field,
    /// Variable completion.
    Variable,
    /// Class completion.
    Class,
    /// Interface completion.
    Interface,
    /// Module completion.
    Module,
    /// Property completion.
    Property,
    /// Unit completion.
    Unit,
    /// Value completion.
    Value,
    /// Enum completion.
    Enum,
    /// Keyword completion.
    Keyword,
    /// Code snippet completion.
    Snippet,
    /// Color completion.
    Color,
    /// File completion.
    File,
    /// Reference completion.
    Reference,
}

/// LSP code action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCodeAction {
    /// Title of the code action.
    pub title: String,
    /// Kind of code action (refactor, quickfix, etc.).
    pub kind: Option<String>,
    /// Command to execute for this action.
    pub command: Option<LspCommand>,
}

/// LSP command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCommand {
    /// Human-readable title of the command.
    pub title: String,
    /// Command identifier.
    pub command: String,
    /// Arguments to pass to the command.
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// Project events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectEvent {
    /// Project opened.
    Opened {
        /// Path to the project root.
        path: PathBuf,
        /// Type of project detected.
        project_type: String,
    },
    /// Project closed.
    Closed {
        /// Path to the project that was closed.
        path: PathBuf,
    },
    /// Project configuration changed.
    ConfigChanged {
        /// Path to the project whose config changed.
        path: PathBuf,
    },
    /// Dependencies updated.
    DependenciesUpdated {
        /// Path to the project whose dependencies were updated.
        path: PathBuf,
    },
}

/// Build system events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildEvent {
    /// Build started.
    Started {
        /// Build target name.
        target: String,
    },
    /// Build completed successfully.
    Completed {
        /// Build target that completed.
        target: String,
        /// How long the build took.
        duration: Duration,
    },
    /// Build failed.
    Failed {
        /// Build target that failed.
        target: String,
        /// Error message describing the failure.
        error: String,
    },
    /// Build output received.
    Output {
        /// Build target name.
        target: String,
        /// Output text.
        output: String,
        /// Whether this is error output.
        is_error: bool,
    },
    /// Tests started.
    TestsStarted,
    /// Tests completed.
    TestsCompleted {
        /// Number of tests that passed.
        passed: usize,
        /// Number of tests that failed.
        failed: usize,
    },
}

/// Alux language events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AluxEvent {
    /// Script compiled.
    ScriptCompiled {
        /// Path to the source script file.
        path: PathBuf,
        /// Path to the compiled bytecode file.
        bytecode_path: PathBuf,
    },
    /// Script execution started.
    ExecutionStarted {
        /// Path to the script being executed.
        path: PathBuf,
    },
    /// Script execution completed.
    ExecutionCompleted {
        /// Path to the executed script.
        path: PathBuf,
        /// Result of the script execution.
        result: AluxExecutionResult,
    },
    /// Hot reload triggered.
    HotReload {
        /// Path to the reloaded script.
        path: PathBuf,
    },
    /// VM log message.
    VmLog {
        /// Log level.
        level: String,
        /// Log message content.
        message: String,
    },
}

/// Alux script execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AluxExecutionResult {
    /// Script executed successfully.
    Success,
    /// Script execution failed with an error.
    Error {
        /// Error message describing the failure.
        message: String,
    },
    /// Script execution timed out.
    Timeout,
}

/// Xylux engine events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XyluxEvent {
    /// Engine started.
    EngineStarted {
        /// Engine mode (debug, release, etc.).
        mode: String,
    },
    /// Engine stopped.
    EngineStopped,
    /// Asset loaded.
    AssetLoaded {
        /// Path to the loaded asset.
        path: PathBuf,
        /// Type of asset (texture, model, etc.).
        asset_type: String,
    },
    /// Shader compiled.
    ShaderCompiled {
        /// Path to the shader file.
        path: PathBuf,
        /// Whether compilation was successful.
        success: bool,
    },
    /// Frame rendered.
    FrameRendered {
        /// Time taken to render the frame.
        frame_time: Duration,
    },
    /// WebAssembly export completed.
    WasmExported {
        /// Path to the exported WebAssembly file.
        path: PathBuf,
    },
}

/// Plugin events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// Plugin loaded.
    Loaded { name: String, version: String },
    /// Plugin unloaded.
    Unloaded { name: String },
    /// Plugin error.
    Error { name: String, error: String },
    /// Plugin custom event.
    Custom { name: String, data: serde_json::Value },
}

/// System events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// Shutdown requested.
    ShutdownRequested,
    /// Configuration reloaded.
    ConfigReloaded,
    /// Memory usage warning.
    MemoryWarning {
        /// Memory used in megabytes.
        used_mb: u64,
        /// Available memory in megabytes.
        available_mb: u64,
    },
    /// Performance warning.
    PerformanceWarning {
        /// Component that had performance issues.
        component: String,
        /// Duration of the slow operation.
        duration: Duration,
    },
    /// Log message.
    Log {
        /// Log level (debug, info, warn, error).
        level: String,
        /// Log message content.
        message: String,
        /// Timestamp when the log was created.
        timestamp: String,
    },
}

/// Event wrapper with metadata.
#[derive(Debug, Clone)]
pub struct EventMessage {
    /// The event type name.
    pub event_type: String,
    /// Event data as JSON.
    pub data: serde_json::Value,
    /// Event priority.
    pub priority: EventPriority,
    /// Timestamp when the event was created (for performance tracking).
    pub timestamp: Instant,
    /// Optional correlation ID for tracking related events.
    pub correlation_id: Option<String>,
    /// Optional source component that generated the event.
    pub source: Option<String>,
}

impl EventMessage {
    /// Create a new event message.
    pub fn new(event_type: String, data: serde_json::Value) -> Self {
        Self {
            event_type,
            data,
            priority: EventPriority::Normal,
            timestamp: Instant::now(),
            correlation_id: None,
            source: None,
        }
    }

    /// Create a new event message from an Event.
    pub fn from_event(event: Event) -> Self {
        let event_type = match &event {
            Event::Editor(_) => "editor",
            Event::Ui(_) => "ui",
            Event::FileSystem(_) => "filesystem",
            Event::Lsp(_) => "lsp",
            Event::Project(_) => "project",
            Event::Build(_) => "build",
            Event::Alux(_) => "alux",
            Event::Xylux(_) => "xylux",
            Event::Plugin(_) => "plugin",
            Event::System(_) => "system",
        }
        .to_string();

        let data = serde_json::to_value(&event).unwrap_or(serde_json::Value::Null);

        Self {
            event_type,
            data,
            priority: EventPriority::Normal,
            timestamp: Instant::now(),
            correlation_id: None,
            source: None,
        }
    }

    /// Set the priority of the event.
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the correlation ID.
    pub fn with_correlation_id<S: Into<String>>(mut self, id: S) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set the source component.
    pub fn with_source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Event handler trait.
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event.
    async fn handle(&self, event: &EventMessage) -> Result<()>;

    /// Check if this handler can process the given event.
    fn can_handle(&self, event_type: &str) -> bool;

    /// Get the priority for this handler.
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
}

/// Event subscription for filtering events.
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Event types to subscribe to.
    pub event_types: Vec<String>,
    /// Minimum priority level.
    pub min_priority: EventPriority,
    /// Optional source filter.
    pub source_filter: Option<String>,
}

impl EventSubscription {
    /// Create a new subscription for all events.
    pub fn all() -> Self {
        Self {
            event_types: vec!["*".to_string()],
            min_priority: EventPriority::Low,
            source_filter: None,
        }
    }

    /// Create a subscription for specific event types.
    pub fn for_types(types: Vec<String>) -> Self {
        Self { event_types: types, min_priority: EventPriority::Low, source_filter: None }
    }

    /// Set minimum priority.
    pub fn with_min_priority(mut self, priority: EventPriority) -> Self {
        self.min_priority = priority;
        self
    }

    /// Set source filter.
    pub fn with_source_filter<S: Into<String>>(mut self, source: S) -> Self {
        self.source_filter = Some(source.into());
        self
    }

    /// Check if this subscription matches an event.
    pub fn matches(&self, event: &EventMessage) -> bool {
        // Check priority
        if event.priority > self.min_priority {
            return false;
        }

        // Check source filter
        if let Some(ref filter) = self.source_filter {
            if event.source.as_ref() != Some(filter) {
                return false;
            }
        }

        // Check event types
        if self.event_types.contains(&"*".to_string()) {
            return true;
        }

        self.event_types.contains(&event.event_type)
    }
}

/// Central event bus for the IDE.
pub struct EventBus {
    /// Broadcast sender for events.
    sender: broadcast::Sender<EventMessage>,
    /// Registered event handlers.
    handlers: Arc<RwLock<HashMap<String, Arc<dyn EventHandler + Send + Sync>>>>,
    /// Event statistics.
    stats: Arc<RwLock<EventStats>>,
}

/// Event statistics for monitoring.
#[derive(Debug, Default)]
pub struct EventStats {
    /// Total events processed.
    pub total_events: u64,
    /// Events by type.
    pub events_by_type: HashMap<String, u64>,
    /// Events by priority.
    pub events_by_priority: HashMap<String, u64>,
    /// Processing times.
    pub avg_processing_time: Duration,
    /// Errors encountered.
    pub error_count: u64,
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            sender,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// Publish an event to the bus.
    pub async fn publish(&self, event: EventMessage) -> Result<()> {
        trace!("Publishing event: {:?}", event.event_type);

        // Update statistics
        self.update_stats(&event).await;

        // Send to broadcast channel
        if let Err(e) = self.sender.send(event.clone()) {
            warn!("Failed to send event to broadcast channel: {}", e);
        }

        // Process with registered handlers
        self.process_with_handlers(&event).await?;

        Ok(())
    }

    /// Subscribe to events with a filter.
    pub fn subscribe(&self, _subscription: EventSubscription) -> broadcast::Receiver<EventMessage> {
        let receiver = self.sender.subscribe();

        // For now, return the raw receiver
        // TODO: Implement filtering based on subscription
        receiver
    }

    /// Register an event handler.
    pub async fn register_handler<S: Into<String>>(
        &self,
        name: S,
        handler: Arc<dyn EventHandler + Send + Sync>,
    ) -> Result<()> {
        let name = name.into();
        debug!("Registering event handler: {}", name);

        let mut handlers = self.handlers.write().await;
        handlers.insert(name, handler);

        Ok(())
    }

    /// Unregister an event handler.
    pub async fn unregister_handler<S: AsRef<str>>(&self, name: S) -> Result<()> {
        let name = name.as_ref();
        debug!("Unregistering event handler: {}", name);

        let mut handlers = self.handlers.write().await;
        handlers.remove(name);

        Ok(())
    }

    /// Get event statistics.
    pub async fn get_stats(&self) -> EventStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Process event with registered handlers.
    async fn process_with_handlers(&self, event: &EventMessage) -> Result<()> {
        let handlers = self.handlers.read().await;
        let start_time = Instant::now();

        for (name, handler) in handlers.iter() {
            if handler.can_handle(&event.event_type) {
                if let Err(e) = handler.handle(event).await {
                    error!("Handler '{}' failed to process event: {}", name, e);
                    // Continue processing other handlers
                }
            }
        }

        let processing_time = start_time.elapsed();
        trace!("Event processing completed in {:?}", processing_time);

        Ok(())
    }

    /// Update event statistics.
    async fn update_stats(&self, event: &EventMessage) {
        let mut stats = self.stats.write().await;
        stats.total_events += 1;

        // Update events by type
        *stats.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;

        // Update events by priority
        let priority_str = match event.priority {
            EventPriority::Critical => "critical",
            EventPriority::High => "high",
            EventPriority::Normal => "normal",
            EventPriority::Low => "low",
        };
        *stats.events_by_priority.entry(priority_str.to_string()).or_insert(0) += 1;
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventStats {
    fn clone(&self) -> Self {
        Self {
            total_events: self.total_events,
            events_by_type: self.events_by_type.clone(),
            events_by_priority: self.events_by_priority.clone(),
            avg_processing_time: self.avg_processing_time,
            error_count: self.error_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    struct TestHandler {
        name: String,
    }

    #[async_trait::async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, event: &EventMessage) -> Result<()> {
            println!("Handler '{}' received event: {:?}", self.name, event.event_type);
            Ok(())
        }

        fn can_handle(&self, event_type: &str) -> bool {
            event_type == "editor"
        }
    }

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new();

        // Register a test handler
        let handler = Arc::new(TestHandler { name: "test_handler".to_string() });
        bus.register_handler("test", handler).await.unwrap();

        // Create and publish an event
        let event = EventMessage::from_event(Event::Editor(EditorEvent::FileOpened {
            path: PathBuf::from("test.rs"),
        }));

        bus.publish(event).await.unwrap();

        // Give some time for processing
        sleep(Duration::from_millis(10)).await;

        // Check statistics
        let stats = bus.get_stats().await;
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.events_by_type.get("editor"), Some(&1));
    }

    #[test]
    fn test_event_subscription() {
        let subscription = EventSubscription::for_types(vec!["editor".to_string()])
            .with_min_priority(EventPriority::High);

        let event = EventMessage::from_event(Event::Editor(EditorEvent::FileOpened {
            path: PathBuf::from("test.rs"),
        }))
        .with_priority(EventPriority::High);

        assert!(subscription.matches(&event));

        let low_priority_event = EventMessage::from_event(Event::Editor(EditorEvent::FileOpened {
            path: PathBuf::from("test.rs"),
        }))
        .with_priority(EventPriority::Low);

        assert!(!subscription.matches(&low_priority_event));
    }

    #[test]
    fn test_event_message_creation() {
        let event = EventMessage::from_event(Event::Editor(EditorEvent::FileOpened {
            path: PathBuf::from("test.rs"),
        }))
        .with_priority(EventPriority::High)
        .with_correlation_id("test-123")
        .with_source("editor");

        assert_eq!(event.priority, EventPriority::High);
        assert_eq!(event.correlation_id, Some("test-123".to_string()));
        assert_eq!(event.source, Some("editor".to_string()));
    }
}
