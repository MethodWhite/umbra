#![allow(deprecated)]
use eframe::egui::{self, Color32, Vec2, RichText, Button, Stroke, Pos2, Align2, FontId, Rect, Rounding, TextEdit, scroll_area::ScrollBarVisibility};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;
use crate::sphere::SphereRenderer;
use crate::agent_memory::{AgentMemory, CognitiveBehavior, EmotionalState};
use crate::agent_personality::*;
use crate::ai_client::{OllamaClient, ChatMessage, SttClient};

const HOVER_PURPLE: Color32 = Color32::from_rgba_premultiplied(167, 139, 250, 40);

#[derive(Default, PartialEq, Clone, Debug)]
pub enum View { #[default] Hud, Trading, Conversations }

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub enum State { #[default] Idle, Listening, Thinking, Working, Speaking }

#[derive(Clone)]
struct Message { sender: String, text: String, is_user: bool }

#[derive(Clone)]
#[allow(dead_code)]
struct Conversation { id: usize, title: String, messages: Vec<Message> }

struct ShortcutEntry { action: String, key: String, recording: bool }

struct ProviderEntry { name: String, configured: bool, key: String }
struct AgentEntry { name: String, active: bool, agent_type: String, gender: AiGender }

#[allow(dead_code)]
pub struct App {
    view: View, state: State, start: Instant, frames: u64,
    sidebar_open: bool, sidebar_tab: usize, settings_tab: usize, vault_open: bool,
    opacity: f32,
    theme: usize,
    primary_color_hex: String,
    main_sphere: SphereRenderer,
    agent_memory: AgentMemory,
    current_emotion: EmotionalState,
    providers: Vec<ProviderEntry>, agents: Vec<AgentEntry>,
    cognitive: CognitiveBehavior,
    muted: bool, use_local_tts: bool,
    conversations: Vec<Conversation>,
    selected_conv: Option<usize>,
    conv_input: String,
    conv_counter: usize,
    conv_thinking: Option<Instant>,
    mt5_balance: f64, mt5_equity: f64, mt5_margin: f64,
    trading_input: String,
    trading_messages: Vec<Message>,
    sphere_selected: bool,
    emotion_last_change: Instant,
    last_emotion_str: String,
    umbra_analysis: String,
    chat_messages: Vec<Message>,
    hud_input: String,
    shortcuts: Vec<ShortcutEntry>,
    recording_shortcut: Option<usize>,
    hf_model_name: String, hf_downloading: bool, hf_status: String,
    new_agent_name: String, new_agent_type: usize, new_agent_gender: usize,
    local_tts_detected: bool, fish_api_detected: bool,
    logo_texture: Option<egui::TextureHandle>,
    deps: HashMap<String, bool>,
    voice_tone: String,
    user_gender: String,
    chart_type: i32,
    trading_filter: usize,
    trading_strategy: usize,
    trading_active: bool,
    trading_timeframe: String,
    trading_symbol: String,
    trading_volume: f64,
    broker_name: String,
    broker_account: String,
    broker_password: String,
    broker_server: String,
    ai_strategy: String,
    needs_repaint: bool,
    ollama: Option<OllamaClient>,
    inference_result: Arc<Mutex<Option<(usize, String)>>>,
    stt: Option<SttClient>,
    stt_result: Arc<Mutex<Option<String>>>,
}

const TRADING_FILTERS: &[&str] = &["All", "Forex", "Crypto", "Commodities", "Indices"];
const TRADING_STRATEGIES: &[&str] = &["144K Method", "3.4 Unification", "Manual"];

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            view: View::Hud, state: State::Idle, start: Instant::now(), frames: 0,
            sidebar_open: false, sidebar_tab: 0, settings_tab: 0, vault_open: false,
            opacity: 0.85, muted: false, use_local_tts: false,
            theme: 0, primary_color_hex: "#00DCFF".into(),
            main_sphere: SphereRenderer::new(250),
            agent_memory: AgentMemory::new(),
            current_emotion: EmotionalState::calm(),
            cognitive: CognitiveBehavior::new(),
            providers: vec![
                ProviderEntry { name: "OpenAI".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Anthropic".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Google".into(), configured: false, key: String::new() },
                ProviderEntry { name: "DeepSeek".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Qwen (Alibaba)".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Zhipu (GLM)".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Moonshot".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Yi (01.AI)".into(), configured: false, key: String::new() },
                ProviderEntry { name: "StepFun".into(), configured: false, key: String::new() },
                ProviderEntry { name: "MiniMax".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Baidu (ERNIE)".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Mistral".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Groq".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Perplexity".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Together".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Cohere".into(), configured: false, key: String::new() },
                ProviderEntry { name: "OpenRouter".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Ollama".into(), configured: false, key: String::new() },
                ProviderEntry { name: "llama.cpp".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Fish Audio".into(), configured: false, key: String::new() },
                ProviderEntry { name: "OpenCode Go".into(), configured: false, key: String::new() },
                ProviderEntry { name: "HuggingFace".into(), configured: false, key: String::new() },
                ProviderEntry { name: "Cursor".into(), configured: false, key: String::new() },
            ],
            agents: vec![
                AgentEntry { name: "trader".into(), active: false, agent_type: "LLM".into(), gender: AiGender::Neutral },
                AgentEntry { name: "analyst".into(), active: false, agent_type: "JEPA".into(), gender: AiGender::Neutral },
                AgentEntry { name: "voice".into(), active: false, agent_type: "SSM".into(), gender: AiGender::Neutral },
                AgentEntry { name: "monitor".into(), active: false, agent_type: "SNN".into(), gender: AiGender::Neutral },
            ],
            conversations: vec![],
            selected_conv: None,
            conv_input: String::new(),
            conv_counter: 0,
            conv_thinking: None,
            mt5_balance: 12450.75, mt5_equity: 12890.30, mt5_margin: 2340.50,
            trading_input: String::new(),
            trading_messages: vec![
                Message { sender: "System".into(), text: "Trading AI initialized. Awaiting commands.".into(), is_user: false },
            ],
            sphere_selected: false,
            emotion_last_change: Instant::now(),
            last_emotion_str: String::new(),
            umbra_analysis: "Analysing neural patterns...\nCorrelation detected: 0.87\nOptimal decision path identified.".into(),
            chat_messages: Vec::new(),
            hud_input: String::new(),
            shortcuts: vec![
                ShortcutEntry { action: "Toggle Sidebar".into(), key: "Ctrl+B".into(), recording: false },
                ShortcutEntry { action: "Focus Input".into(), key: "Ctrl+I".into(), recording: false },
                ShortcutEntry { action: "Toggle Mute".into(), key: "Ctrl+M".into(), recording: false },
                ShortcutEntry { action: "New Conversation".into(), key: "Ctrl+N".into(), recording: false },
                ShortcutEntry { action: "Recenter Window".into(), key: "Ctrl+R".into(), recording: false },
            ],
            recording_shortcut: None,
            hf_model_name: String::new(), hf_downloading: false, hf_status: String::new(),
            new_agent_name: String::new(), new_agent_type: 0, new_agent_gender: 0,
            local_tts_detected: false, fish_api_detected: false,
            logo_texture: None,
            deps: Self::check_dependencies(),
            voice_tone: "neutral".into(),
            user_gender: "male".into(),
            chart_type: 0,
            trading_filter: 0,
            trading_strategy: 0,
            trading_active: false,
            trading_timeframe: "1H".into(),
            trading_symbol: "EURUSD".into(),
            trading_volume: 0.01,
            broker_name: "Pepperstone".into(),
            broker_account: String::new(),
            broker_password: String::new(),
            broker_server: "demo.pepperstone.com".into(),
            ai_strategy: "144K Method".into(),
            needs_repaint: false,
            ollama: None,
            inference_result: Arc::new(Mutex::new(None)),
            stt: None,
            stt_result: Arc::new(Mutex::new(None)),
        };
        app.detect_local_models();
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames += 1;

        // Track emotion changes for fade display
        let current_emo = self.current_emotion.to_string();
        if current_emo != self.last_emotion_str {
            self.last_emotion_str = current_emo;
            self.emotion_last_change = Instant::now();
        }

        self.load_logo_texture(ctx);
        self.update_voice_tone();

        let has_activity = ctx.input(|i| i.pointer.any_down() || !i.events.is_empty());
        self.needs_repaint = self.needs_repaint || has_activity || self.sphere_selected;
        if self.needs_repaint {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
            self.needs_repaint = false;
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        let primary = self.primary_color();
        let alpha = (self.opacity * 200.0) as u8;

        self.handle_shortcuts(ctx);

        // Check for completed STT transcription
        if let Ok(mut stt) = self.stt_result.lock() {
            if let Some(text) = stt.take() {
                self.hud_input = text;
                self.needs_repaint = true;
            }
        }

        // Check for completed inference
        if let Ok(mut result) = self.inference_result.lock() {
            if let Some((conv_id, response)) = result.take() {
                if conv_id < self.conversations.len() {
                    self.conversations[conv_id].messages.push(Message {
                        sender: "umbra".into(),
                        text: response,
                        is_user: false,
                    });
                    self.current_emotion = EmotionalState::analytical();
                }
                self.needs_repaint = true;
            }
        }

        if let Some(started) = self.conv_thinking {
            if started.elapsed() > std::time::Duration::from_millis(1200) {
                self.conv_thinking = None;
                self.needs_repaint = true;
                if let Some(idx) = self.selected_conv {
                    if idx < self.conversations.len() {
                        let last_msg = self.conversations[idx].messages.last()
                            .map(|m| m.text.clone())
                            .unwrap_or_default();
                        if self.ollama.is_some() {
                            let result_arc = self.inference_result.clone();
                            let conv_id = idx;
                            let msgs = vec![
                                ChatMessage { role: "system".into(), content: "You are Umbra, an advanced AI assistant. Be concise and helpful.".into() },
                                ChatMessage { role: "user".into(), content: last_msg.clone() },
                            ];
                            tokio::spawn(async move {
                                let client = OllamaClient::new();
                                if let Ok(response) = client.chat_completion("llama3.2", msgs).await {
                                    if let Ok(mut guard) = result_arc.lock() {
                                        *guard = Some((conv_id, response));
                                    }
                                }
                            });
                        } else {
                            let response = self.generate_ai_response();
                            self.conversations[idx].messages.push(Message {
                                sender: "umbra".into(),
                                text: response,
                                is_user: false,
                            });
                            self.current_emotion = EmotionalState::analytical();
                        }
                    }
                }
            }
        }

        // Clock format string is re-formatted every frame (~60fps).
        // This is fine for performance since simple integer formatting is cheap.
        self.render_top_bar(ctx, primary);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.sidebar_open = false;
                    self.sphere_selected = false;
                }
                let r = ui.max_rect();
                let t = self.start.elapsed().as_secs_f32();

                let sphere_cx = if self.sphere_selected { r.right() - 140.0 } else { r.center().x };
                let sphere_cy = if self.sphere_selected { r.top() + 180.0 } else { r.top() + 200.0 };
                let sphere_rect = Rect::from_center_size(Pos2::new(sphere_cx, sphere_cy), Vec2::splat(280.0));
                let sphere_response = ui.allocate_rect(sphere_rect, egui::Sense::click());

                if sphere_response.clicked() {
                    self.sphere_selected = !self.sphere_selected;
                }

                match self.view {
                    View::Trading => {
                        self.sphere_selected = false;
                        self.render_trading_view(ui, r, t, primary)
                    },
                    View::Conversations => {
                        self.sphere_selected = false;
                        self.render_conversations_view(ui, r, alpha, primary)
                    },
                    View::Hud => self.render_hud_view(ui, r, t, sphere_cx, sphere_cy, primary),
                }

                if self.sphere_selected {
                    self.render_sphere_view(ui, r, t, sphere_cx, sphere_cy, primary);
                }

                if self.sidebar_open {
                    self.render_sidebar_menu(ui, r, primary);
                }
            });
    }
}
impl App {
    fn load_logo_texture(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() { return; }
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".local/share/icons/hicolor/128x128/apps/umbra.png");
            if let Ok(img_data) = std::fs::read(&path) {
                if let Ok(img) = image::load_from_memory(&img_data) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let pixels = rgba.into_raw();
                    let color_img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
                    let handle = ctx.load_texture("umbra_logo", color_img, egui::TextureOptions::LINEAR);
                    self.logo_texture = Some(handle);
                }
            }
        }
    }

    fn primary_color(&self) -> Color32 {
        let hex = &self.primary_color_hex;
        if hex.len() == 7 && hex.starts_with('#') {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[1..3], 16),
                u8::from_str_radix(&hex[3..5], 16),
                u8::from_str_radix(&hex[5..7], 16),
            ) {
                return Color32::from_rgb(r, g, b);
            }
        }
        Color32::from_rgb(167, 139, 250)
    }

    fn check_dependencies() -> HashMap<String, bool> {
        let mut deps = HashMap::new();
        for bin in &["ollama", "piper", "espeak", "aplay"] {
            deps.insert(bin.to_string(), which_exists(bin));
        }
        deps
    }

    fn update_voice_tone(&mut self) {
        self.voice_tone = match self.current_emotion.to_string().to_lowercase().as_str() {
            "joy" | "happy" | "excited" | "creative" | "flow" => "cheerful",
            "angry" => "angry",
            "sad" | "depressed" | "tired" | "ashamed" => "sad",
            "fearful" | "anxious" | "surprised" => "worried",
            _ => "neutral",
        }.to_string();
    }

    pub fn detect_local_models(&mut self) {
        if let Ok(resp) = reqwest::blocking::get("http://localhost:11434/api/tags") {
            if resp.status().is_success() {
                if let Some(ollama) = self.providers.iter_mut().find(|p| p.name == "Ollama") {
                    ollama.configured = true;
                }
                self.ollama = Some(OllamaClient::new());
            }
        }
        if let Ok(resp) = reqwest::blocking::get("http://localhost:8080/v1/models") {
            if resp.status().is_success() {
                if let Some(cpp) = self.providers.iter_mut().find(|p| p.name == "llama.cpp") {
                    cpp.configured = true;
                }
            }
        }
        // Detect whisper.cpp STT server
        if let Ok(resp) = reqwest::blocking::get("http://localhost:8080/health") {
            if resp.status().is_success() {
                self.stt = Some(SttClient::new_local());
            }
        }
        self.detect_tts();
    }

    fn detect_tts(&mut self) {
        self.local_tts_detected = false;
        self.fish_api_detected = false;
        for bin in &["piper", "espeak", "espeak-ng"] {
            if which_exists(bin) {
                self.local_tts_detected = true;
                self.use_local_tts = true;
                break;
            }
        }
        if let Some(fish) = self.providers.iter().find(|p| p.name == "Fish Audio") {
            if fish.configured {
                self.fish_api_detected = true;
            }
        }
    }

    #[allow(dead_code)]
    fn send_hud_message(&mut self) {
        let text = self.hud_input.trim().to_string();
        if text.is_empty() { return; }
        self.hud_input.clear();

        let idx = if let Some(idx) = self.selected_conv {
            if idx < self.conversations.len() { idx } else {
                self.conv_counter += 1;
                self.conversations.push(Conversation { id: self.conv_counter, title: "HUD Chat".into(), messages: vec![] });
                self.conversations.len() - 1
            }
        } else {
            self.conv_counter += 1;
            self.conversations.push(Conversation { id: self.conv_counter, title: "HUD Chat".into(), messages: vec![] });
            self.selected_conv = Some(self.conversations.len() - 1);
            self.conversations.len() - 1
        };
        self.selected_conv = Some(idx);

        self.conversations[idx].messages.push(Message { sender: "user".into(), text: text.clone(), is_user: true });
        self.state = State::Thinking;
        self.current_emotion = EmotionalState::curious();
        self.conv_thinking = Some(Instant::now());
    }

    fn send_conv_message(&mut self) {
        let text = self.conv_input.trim().to_string();
        if text.is_empty() { return; }
        self.conv_input.clear();

        if let Some(idx) = self.selected_conv {
            if idx < self.conversations.len() {
                self.conversations[idx].messages.push(Message { sender: "user".into(), text: text.clone(), is_user: true });
                self.state = State::Thinking;
                self.current_emotion = EmotionalState::curious();
                self.conv_thinking = Some(Instant::now());
            }
        }
    }

    fn send_trading_message(&mut self) {
        let text = self.trading_input.trim().to_string();
        if text.is_empty() { return; }
        self.trading_input.clear();

        self.trading_messages.push(Message { sender: "user".into(), text: text.clone(), is_user: true });
        self.state = State::Thinking;

        let response = match text.to_lowercase().as_str() {
            t if t.contains("balance") || t.contains("account") => {
                format!("Account balance: ${:.2}, Equity: ${:.2}, Margin: ${:.2}. All within normal parameters. Leverage: 1:30.", self.mt5_balance, self.mt5_equity, self.mt5_margin)
            }
            t if t.contains("eurusd") || t.contains("market") => {
                "EURUSD showing bullish momentum on the 4H chart. RSI at 58, MACD positive cross. Support at 1.2300, resistance at 1.2420.".into()
            }
            t if t.contains("position") || t.contains("open") => {
                "I recommend a conservative approach. Consider a 0.5 lot BUY on EURUSD with a 20-pip stop loss. Market conditions are favourable.".into()
            }
            _ => {
                let responses = [
                    "Analysing market conditions... Volatility is moderate. No unusual patterns detected.",
                    "Cross-referencing multiple timeframes. Short-term bullish, long-term neutral.",
                    "Running predictive models... Probability of upward movement: 62% over next 4 hours.",
                    "Economic calendar shows no major events in the next 2 hours. Normal trading conditions.",
                    "Technical indicators suggest consolidation. Recommend waiting for a clear breakout signal.",
                ];
                responses[self.frames as usize % responses.len()].into()
            }
        };

        self.trading_messages.push(Message { sender: "Trader AI".into(), text: response, is_user: false });
        self.state = State::Idle;
    }

    fn generate_ai_response(&self) -> String {
        let responses = [
            "I've analysed the input. The optimal approach involves leveraging existing patterns in the data.",
            "Processing complete. I recommend a multi-step strategy with continuous monitoring.",
            "Correlation analysis shows a 0.73 confidence level. Proceeding with suggested parameters.",
            "Insight: The underlying structure suggests emergent properties we can exploit.",
            "Analysis complete. Three possible paths identified. Path A has the highest probability of success.",
            "Neural assessment: Patterns detected across 4 dimensions. Confidence threshold exceeded.",
            "Cognitive processing finished. The most efficient solution utilises parallel sub-agent coordination.",
        ];
        responses[self.frames as usize % responses.len()].into()
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.recording_shortcut.is_some() {
            ctx.input_mut(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                        let mod_str = {
                            let mut s = String::new();
                            if modifiers.ctrl { s.push_str("Ctrl+"); }
                            if modifiers.alt { s.push_str("Alt+"); }
                            if modifiers.shift { s.push_str("Shift+"); }
                            if modifiers.mac_cmd { s.push_str("Cmd+"); }
                            s
                        };
                        let mut full = format!("{}{:?}", mod_str, key);
                        if full == "Ctrl+" { full = format!("{:?}", key); }
                        if let Some(idx) = self.recording_shortcut {
                            if idx < self.shortcuts.len() {
                                let dup = self.shortcuts.iter().any(|s| s.key == full);
                                if !dup {
                                    self.shortcuts[idx].key = full;
                                }
                            }
                        }
                        self.recording_shortcut = None;
                    }
                }
            });
        }

        let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());
        for event in &events {
            if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                let mod_str = {
                    let mut m = String::new();
                    if modifiers.ctrl { m.push_str("Ctrl+"); }
                    if modifiers.shift { m.push_str("Shift+"); }
                    if modifiers.alt { m.push_str("Alt+"); }
                    m.push_str(&format!("{:?}", key).to_uppercase());
                    m
                };
                for s in &self.shortcuts {
                    if s.key == mod_str && !s.recording {
                        match s.action.as_str() {
                            "Toggle Sidebar" => self.sidebar_open = !self.sidebar_open,
                            "Toggle Mute" => self.muted = !self.muted,
                            "Recenter Window" => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(1400.0, 900.0)));
                                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::Pos2::new(200.0, 50.0)));
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
        }
    }

    fn render_top_bar(&mut self, ctx: &egui::Context, primary: Color32) {
        egui::TopBottomPanel::top("top")
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let drag_resp = ui.interact(ui.max_rect(), ui.next_auto_id(), egui::Sense::click());
                if drag_resp.dragged_by(egui::PointerButton::Primary) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.horizontal(|ui| {
                    if let Some(tex) = &self.logo_texture {
                        let logo_size = Vec2::new(22.0, 22.0);
                        let (logo_rect, _) = ui.allocate_exact_size(logo_size, egui::Sense::click());
                        ui.put(logo_rect, egui::Image::new(tex).fit_to_exact_size(logo_size));
                        if logo_rect.contains(ui.ctx().pointer_interact_pos().unwrap_or(Pos2::ZERO))
                            && ui.input(|i| i.pointer.any_click()) {
                            self.sidebar_open = !self.sidebar_open;
                        }
                    } else {
                        if btn(ui, RichText::new("☰").size(18.0).color(primary)).clicked() {
                            self.sidebar_open = !self.sidebar_open;
                        }
                    }
                    ui.add_space(4.0);

                    let tabs = [("HUD", View::Hud), ("TRADING", View::Trading), ("CONVERSATIONS", View::Conversations)];
                    for (label, view) in &tabs {
                        let active = self.view == *view;
                        let text_color = if active { primary } else { Color32::from_rgb(80, 60, 130) };
                        if btn_rounded(ui, RichText::new(*label).color(text_color).size(11.0), Rounding::same(12.0), Vec2::new(0.0, 28.0)).clicked() {
                            self.view = view.clone();
                            if *view == View::Hud { self.sphere_selected = false; }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if btn(ui, RichText::new("✕").size(14.0).color(Color32::from_rgb(200, 80, 80)))
                            .on_hover_text("Close")
                            .clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.add_space(2.0);
                        if btn(ui, RichText::new("─").size(14.0).color(Color32::from_rgb(160, 160, 180)))
                            .on_hover_text("Minimize")
                            .clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        ui.add_space(6.0);
                        let s = self.start.elapsed().as_secs();
                        ui.label(RichText::new(format!("{:02}:{:02}:{:02}", s/3600, (s%3600)/60, s%60))
                            .color(Color32::from_rgb(180, 150, 230)).size(11.0).monospace());
                        ui.add_space(4.0);
                        let mute_icon = if self.muted { "🔇" } else { "🔊" };
                        if btn(ui, RichText::new(mute_icon).size(16.0))
                            .on_hover_text("Toggle mute")
                            .clicked() {
                            self.muted = !self.muted;
                        }
                    });
                });
            });
    }

    fn render_emotion_label(&self, ui: &egui::Ui, sphere_cx: f32, sphere_cy: f32) {
        let emotion_elapsed = self.emotion_last_change.elapsed().as_secs_f32();
        let emotion_alpha = if emotion_elapsed < 2.0 {
            ((1.0 - emotion_elapsed / 2.0) * 120.0) as u8
        } else { 0 };
        if emotion_alpha > 5 {
            ui.painter().text(Pos2::new(sphere_cx, sphere_cy + 100.0), Align2::CENTER_CENTER, &self.current_emotion.to_string(), FontId::monospace(10.0), Color32::from_rgba_premultiplied(0, 200, 255, emotion_alpha));
        }
    }

    fn render_hud_view(&mut self, ui: &mut egui::Ui, _r: Rect, t: f32, sphere_cx: f32, sphere_cy: f32, _primary: Color32) {
        let hue = self.current_emotion.hue();
        let sat = self.current_emotion.saturation();
        let intensity = self.current_emotion.intensity();
        if !self.sphere_selected {
            let activity = match self.state {
                State::Idle => 0.3,
                State::Listening => 0.6,
                State::Thinking => 0.8,
                State::Working => 1.0,
                State::Speaking => 0.9,
            };
            self.main_sphere.render(&ui.painter(), Pos2::new(sphere_cx, sphere_cy), hue, sat, intensity, activity, t, 0.8, 1.0, 0.2, 0.0, 1.0);
            self.render_emotion_label(ui, sphere_cx, sphere_cy);
        } else {
            self.main_sphere.render(&ui.painter(), Pos2::new(sphere_cx, sphere_cy), hue, sat, intensity, 0.6, t, 0.4, 0.6, 0.1, 0.0, 0.8);
        }
    }

    fn render_trading_view(&mut self, ui: &mut egui::Ui, r: Rect, t: f32, primary: Color32) {
        let left_panel_w = 180.0;
        let right_panel_w = r.width() - left_panel_w - 20.0;

        let top_bar_rect = Rect::from_min_size(Pos2::new(r.left() + 10.0, r.top() + 5.0), Vec2::new(r.width() - 20.0, 36.0));
        ui.painter().rect_filled(top_bar_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 6, 16, 200));
        ui.painter().rect_stroke(top_bar_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));
        ui.allocate_ui_at_rect(top_bar_rect.shrink(4.0), |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📊 SIMULATED ACCOUNT").color(primary).size(9.0).strong());
                ui.separator();
                ui.label(RichText::new(format!("Balance: ${:.2}", self.mt5_balance)).color(Color32::from_rgb(124, 58, 237)).size(9.0).monospace());
                ui.label(RichText::new(format!("Equity: ${:.2}", self.mt5_equity)).color(Color32::from_rgb(0, 220, 100)).size(9.0).monospace());
                ui.label(RichText::new(format!("Margin: ${:.2}", self.mt5_margin)).color(Color32::from_rgb(200, 180, 0)).size(9.0).monospace());
                let free_margin = self.mt5_equity - self.mt5_margin;
                ui.label(RichText::new(format!("Free: ${:.2}", free_margin)).color(Color32::from_rgb(0, 200, 255)).size(9.0).monospace());
                 if btn(ui, RichText::new(format!("{} • 1:30 ⚙", self.broker_name)).color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace()).clicked() {
                     self.settings_tab = 8;
                     self.sidebar_open = true;
                 }
                 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (i, f) in TRADING_FILTERS.iter().enumerate() {
                        let sel = self.trading_filter == i;
                        let c = if sel { primary } else { Color32::from_rgb(130, 100, 190) };
                        if btn(ui, RichText::new(*f).color(c).size(9.0).monospace()).clicked() {
                            self.trading_filter = i;
                        }
                    }
                });
            });
        });

        let left_rect = Rect::from_min_size(Pos2::new(r.left() + 10.0, r.top() + 46.0), Vec2::new(left_panel_w, r.height() - 56.0));
        ui.painter().rect_filled(left_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 6, 16, 200));
        ui.painter().rect_stroke(left_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));

        let sphere_y = left_rect.top() + 50.0;
        let sphere_center = Pos2::new(left_rect.left() + left_panel_w / 2.0, sphere_y);
        let sph = self.current_emotion.hue();
        let trading_activity = if self.trading_active { 0.5 + (t * 0.3).sin() * 0.2 } else { 0.2 };
        self.main_sphere.set_radius(28.0);
        self.main_sphere.render(&ui.painter(), sphere_center, sph, 0.7, 0.5, trading_activity, t, 0.6, 0.5, 0.15, 0.05, 1.0);

        ui.allocate_ui_at_rect(Rect::from_min_size(Pos2::new(left_rect.left() + 6.0, sphere_y + 55.0), Vec2::new(left_panel_w - 12.0, 170.0)), |ui| {
            ui.label(RichText::new("SYMBOLS").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
            ui.separator();
            let prices: Vec<(&str, f64)> = vec![
                ("EURUSD", 1.08432),
                ("GBPUSD", 1.26418),
                ("BTCUSD", 64127.50),
                ("XAUUSD", 2335.80),
                ("SP500", 5342.10),
            ];
            for (sym, price) in &prices {
                let sel = self.trading_symbol == *sym;
                let c = if sel { primary } else { Color32::from_rgb(160, 180, 210) };
                let bg = if sel { Color32::from_rgba_premultiplied(60, 30, 120, 60) } else { Color32::TRANSPARENT };
                let price_str = if *price > 1000.0 { format!("{:.2}", price) } else { format!("{:.5}", price) };
                let id = ui.next_auto_id();
                let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
                let fill = if sel { bg } else if prev_hovered { HOVER_PURPLE } else { Color32::TRANSPARENT };
                let resp = ui.add(Button::new(RichText::new(format!("{}  {}", sym, price_str)).color(c).size(9.0).monospace())
                    .fill(fill).min_size(Vec2::new(left_panel_w - 12.0, 16.0)));
                ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
                if resp.clicked() {
                    self.trading_symbol = sym.to_string();
                }
            }
        });

        ui.allocate_ui_at_rect(Rect::from_min_size(Pos2::new(left_rect.left() + 6.0, left_rect.top() + 260.0), Vec2::new(left_panel_w - 12.0, 120.0)), |ui| {
            ui.label(RichText::new("POSITIONS").color(Color32::from_rgb(0, 180, 220)).size(9.0).monospace());
            ui.separator();
            let positions = vec![
                ("EURUSD", "BUY", 1.08432, 1.08800, 36.8),
                ("BTCUSD", "SELL", 64200.0, 63850.0, -350.0),
                ("XAUUSD", "BUY", 2330.0, 2342.5, 125.0),
            ];
            for (pair, dir, entry, _current, pl) in &positions {
                let pnl_c = if *pl > 0.0 { Color32::from_rgb(0, 220, 100) } else { Color32::from_rgb(220, 50, 50) };
                let dir_arrow = if *dir == "BUY" { "▲" } else { "▼" };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} {} {}", pair, dir_arrow, entry)).color(Color32::from_rgb(150, 180, 200)).size(9.0).monospace());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{}{:.1}", if *pl > 0.0 { "+" } else { "" }, pl)).color(pnl_c).size(9.0).monospace());
                    });
                });
            }
        });

        let right_x = r.left() + left_panel_w + 20.0;

        let chart_rect = Rect::from_min_size(Pos2::new(right_x, r.top() + 46.0), Vec2::new(right_panel_w, r.height() * 0.42));
        ui.painter().rect_filled(chart_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 6, 18, 180));
        ui.painter().rect_stroke(chart_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));

        ui.allocate_ui_at_rect(Rect::from_min_size(Pos2::new(chart_rect.left() + 6.0, chart_rect.top() + 4.0), Vec2::new(chart_rect.width() - 12.0, 22.0)), |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.trading_symbol).color(primary).size(11.0).strong());
                ui.separator();
                let timeframes = ["1m", "5m", "15m", "1H", "4H", "1D", "1W"];
                for tf in &timeframes {
                    let sel = self.trading_timeframe == *tf;
                    let c = if sel { Color32::from_rgb(0, 220, 100) } else { Color32::from_rgb(160, 160, 200) };
                    let bg = if sel { Color32::from_rgba_premultiplied(0, 60, 30, 80) } else { Color32::TRANSPARENT };
                    let id = ui.next_auto_id();
                    let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
                    let fill = if sel { bg } else if prev_hovered { HOVER_PURPLE } else { Color32::TRANSPARENT };
                    let resp = ui.add(Button::new(RichText::new(*tf).color(c).size(9.0).monospace())
                        .fill(fill).rounding(Rounding::same(3.0)));
                    ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
                    if resp.clicked() {
                        self.trading_timeframe = tf.to_string();
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let chart_types = [("Line", 0), ("Candle", 1)];
                    for (lbl, idx) in &chart_types {
                        let sel = self.chart_type == *idx;
                        let c = if sel { primary } else { Color32::from_rgb(100, 100, 140) };
                        if btn(ui, RichText::new(*lbl).color(c).size(9.0).monospace()).clicked() { self.chart_type = *idx; }
                    }
                });
            });
        });

        let chart_area = chart_rect.shrink(12.0);
        let chart_area_top = chart_rect.top() + 30.0;
        let chart_draw = Rect::from_min_size(Pos2::new(chart_area.left(), chart_area_top), Vec2::new(chart_area.width(), chart_rect.bottom() - chart_area_top - 8.0));

        for i in 0..4 {
            let y = chart_draw.top() + chart_draw.height() * (i as f32 + 0.5) / 4.0;
            let alpha = 20 + i as u8 * 8;
            ui.painter().line_segment(
                [Pos2::new(chart_draw.left(), y), Pos2::new(chart_draw.right(), y)],
                Stroke::new(1.0, Color32::from_rgba_premultiplied(100, 100, 140, alpha)),
            );
        }

        let base_price: f64 = match self.trading_symbol.as_str() {
            "EURUSD" => 1.08432,
            "GBPUSD" => 1.26418,
            "BTCUSD" => 64127.50,
            "XAUUSD" => 2335.80,
            "SP500" => 5342.10,
            _ => 1.0000,
        };
        let volatility = if self.trading_symbol == "BTCUSD" { 0.03 } else { 0.005 };
        let n_points = 60;
        let seed = self.frames as f64 * 0.1;
        let prices: Vec<f64> = (0..n_points).map(|i| {
            let angle = (i as f64 + seed) * 0.3;
            let noise = angle.sin() * 0.5 + (angle * 2.3).sin() * 0.3 + (angle * 5.7).sin() * 0.2;
            base_price * (1.0 + noise * volatility)
        }).collect();

        let max_p = prices.iter().cloned().fold(0.0_f64, f64::max);
        let min_p = prices.iter().cloned().fold(f64::MAX, f64::min);
        let range = (max_p - min_p).max(0.0001);

        for i in 0..3 {
            let frac = (i as f64 + 0.5) / 3.0;
            let val = max_p - frac * range;
            let y = chart_draw.bottom() - (frac as f32) * chart_draw.height();
            let label = if val > 100.0 { format!("{:.1}", val) } else { format!("{:.5}", val) };
            ui.painter().text(Pos2::new(chart_draw.right() - 4.0, y), Align2::RIGHT_CENTER, &label, FontId::monospace(9.0), Color32::from_rgb(140, 140, 180));
        }

        let cw = chart_draw.width() / (n_points - 1) as f32;
        for i in 0..n_points - 1 {
            let y1 = chart_draw.bottom() - ((prices[i] - min_p) / range) as f32 * chart_draw.height();
            let y2 = chart_draw.bottom() - ((prices[i + 1] - min_p) / range) as f32 * chart_draw.height();
            let line_color = if prices[i + 1] > prices[i] { Color32::from_rgb(0, 220, 100) } else { Color32::from_rgb(220, 50, 50) };
            ui.painter().line_segment(
                [Pos2::new(chart_draw.left() + i as f32 * cw, y1), Pos2::new(chart_draw.left() + (i + 1) as f32 * cw, y2)],
                Stroke::new(2.0, line_color),
            );
        }

        let last_price = prices[n_points - 1];
        let last_str = if last_price > 100.0 { format!("{:.2}", last_price) } else { format!("{:.5}", last_price) };
        let last_y = chart_draw.bottom() - ((last_price - min_p) / range) as f32 * chart_draw.height();
        ui.painter().text(Pos2::new(chart_draw.right() - 4.0, last_y - 10.0), Align2::RIGHT_BOTTOM, &format!("{} {}", self.trading_symbol, last_str), FontId::monospace(9.0), primary);
        ui.painter().text(Pos2::new(chart_draw.left() + 4.0, chart_draw.bottom() - 4.0), Align2::LEFT_BOTTOM, &format!("{} · {} bars", self.trading_timeframe, n_points), FontId::monospace(9.0), Color32::from_rgb(140, 140, 180));

        let order_rect = Rect::from_min_size(Pos2::new(right_x, chart_rect.bottom() + 5.0), Vec2::new(right_panel_w, 110.0));
        ui.painter().rect_filled(order_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 6, 18, 180));
        ui.painter().rect_stroke(order_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));
        ui.allocate_ui_at_rect(order_rect.shrink(8.0), |ui| {
            ui.label(RichText::new("ORDER ENTRY").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Vol:").color(Color32::from_rgb(100, 130, 160)).size(9.0).monospace());
                let mut vol_str = format!("{:.2}", self.trading_volume);
                let resp = ui.add(TextEdit::singleline(&mut vol_str).desired_width(55.0).font(FontId::monospace(10.0)));
                if resp.changed() {
                    if let Ok(v) = vol_str.parse::<f64>() {
                        self.trading_volume = v.clamp(0.01, 100.0);
                    }
                }
                ui.separator();
                ui.label(RichText::new("SL:").color(Color32::from_rgb(220, 80, 80)).size(9.0).monospace());
                let mut sl_str = String::new();
                ui.add(TextEdit::singleline(&mut sl_str).desired_width(50.0).font(FontId::monospace(10.0)).hint_text("pips"));
                ui.label(RichText::new("TP:").color(Color32::from_rgb(0, 220, 100)).size(9.0).monospace());
                let mut tp_str = String::new();
                ui.add(TextEdit::singleline(&mut tp_str).desired_width(50.0).font(FontId::monospace(10.0)).hint_text("pips"));
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let buy_bg = Color32::from_rgba_premultiplied(0, 60, 30, 100);
                if ui.add(Button::new(RichText::new("BUY  ▲").color(Color32::from_rgb(0, 220, 100)).size(12.0).monospace().strong())
                    .fill(buy_bg).min_size(Vec2::new(110.0, 30.0)).rounding(Rounding::same(4.0))).clicked() {
                    self.trading_messages.push(Message { sender: "System".into(), text: format!("BUY {} {} at market", self.trading_volume, self.trading_symbol), is_user: false });
                }
                ui.add_space(8.0);
                let sell_bg = Color32::from_rgba_premultiplied(60, 0, 0, 100);
                if ui.add(Button::new(RichText::new("SELL ▼").color(Color32::from_rgb(220, 50, 50)).size(12.0).monospace().strong())
                    .fill(sell_bg).min_size(Vec2::new(110.0, 30.0)).rounding(Rounding::same(4.0))).clicked() {
                    self.trading_messages.push(Message { sender: "System".into(), text: format!("SELL {} {} at market", self.trading_volume, self.trading_symbol), is_user: false });
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} · {}", self.trading_symbol, self.trading_timeframe)).color(Color32::from_rgb(80, 80, 120)).size(9.0).monospace());
                });
            });
        });

        let strat_rect = Rect::from_min_size(Pos2::new(right_x, order_rect.bottom() + 5.0), Vec2::new(right_panel_w, 24.0));
        ui.painter().rect_filled(strat_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 6, 18, 180));
        ui.painter().rect_stroke(strat_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));
        ui.allocate_ui_at_rect(strat_rect.shrink(4.0), |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("AI:").color(primary).size(9.0).monospace());
                ui.label(RichText::new(&self.ai_strategy).color(primary).size(9.0).monospace());
                ui.separator();
                ui.label(RichText::new("Manual:").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
                for (i, s) in TRADING_STRATEGIES.iter().enumerate() {
                    let sel = self.trading_strategy == i;
                    let c = if sel { primary } else { Color32::from_rgb(80, 60, 130) };
                    if btn(ui, RichText::new(*s).color(c).size(9.0).monospace()).clicked() {
                        self.trading_strategy = i;
                    }
                }
            });
        });

        let chat_top = strat_rect.bottom() + 5.0;
        let chat_h = r.bottom() - chat_top - 5.0;
        let chat_rect = Rect::from_min_size(Pos2::new(right_x, chat_top), Vec2::new(right_panel_w, chat_h));
        ui.painter().rect_filled(chat_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 8, 20, 200));
        ui.painter().rect_stroke(chat_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));
        ui.allocate_ui_at_rect(chat_rect.shrink(6.0), |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("💬 Trading AI").color(primary).size(9.0).monospace());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("IC Markets (AU)").color(Color32::from_rgb(80, 80, 120)).size(9.0).monospace());
                });
            });
            ui.separator();
            scroll_area().max_height(chat_h - 56.0).show(ui, |ui| {
                for msg in &self.trading_messages {
                    let c = if msg.is_user { Color32::from_rgb(124, 58, 237) } else { Color32::from_rgb(100, 220, 150) };
                    ui.label(RichText::new(format!("{}: {}", msg.sender, msg.text)).color(c).size(9.0).monospace());
                }
            });
            ui.separator();
            let input_h = 22.0;
            let input_rect = Rect::from_min_size(Pos2::new(chat_rect.left() + 6.0, chat_rect.bottom() - input_h - 6.0), Vec2::new(chat_rect.width() - 64.0, input_h));
            ui.allocate_ui_at_rect(input_rect, |ui| {
                let resp = ui.add(TextEdit::singleline(&mut self.trading_input)
                    .font(FontId::monospace(9.0))
                    .hint_text("Ask Trading AI...")
                    .desired_width(f32::INFINITY));
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.send_trading_message();
                }
            });
            let snd_rect = Rect::from_min_size(Pos2::new(chat_rect.right() - 52.0, chat_rect.bottom() - input_h - 6.0), Vec2::new(46.0, input_h));
            ui.allocate_ui_at_rect(snd_rect, |ui| {
                if btn(ui, RichText::new("⏎").size(9.0).color(primary)).on_hover_text("Send message").clicked() {
                    self.send_trading_message();
                }
            });
        });
    }

    fn render_conversations_view(&mut self, ui: &mut egui::Ui, r: Rect, alpha: u8, primary: Color32) {
        ui.painter().rect_filled(r, Rounding::ZERO, Color32::from_rgba_premultiplied(0, 4, 12, alpha));

        let side_panel_w = 180.0;

        let list_rect = Rect::from_min_size(Pos2::new(r.left() + 5.0, r.top() + 5.0), Vec2::new(side_panel_w, r.height() - 50.0));
        ui.painter().rect_filled(list_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 8, 20, 200));
        ui.painter().rect_stroke(list_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));
        ui.allocate_ui_at_rect(list_rect.shrink(6.0), |ui| {
            ui.label(RichText::new("CONVERSATIONS").color(primary).size(11.0).monospace());
            ui.separator();
            if btn(ui, RichText::new("➕ New").size(9.0).color(Color32::from_rgb(0, 200, 100))).clicked() {
                self.conv_counter += 1;
                let conv = Conversation { id: self.conv_counter, title: format!("Chat {}", self.conv_counter), messages: vec![] };
                self.conversations.push(conv);
                self.selected_conv = Some(self.conversations.len() - 1);
            }
            ui.separator();
            scroll_area().show(ui, |ui| {
                let mut del_idx = None;
                for (i, conv) in self.conversations.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let sel = self.selected_conv == Some(i);
                        let c = if sel { primary } else { Color32::from_rgb(120, 150, 180) };
                        let bg = if sel { Color32::from_rgba_premultiplied(0, 40, 80, 100) } else { Color32::TRANSPARENT };
                        let id = ui.next_auto_id();
                        let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
                        let fill = if sel { bg } else if prev_hovered { HOVER_PURPLE } else { Color32::TRANSPARENT };
                        let resp = ui.add(Button::new(RichText::new(&conv.title).color(c).size(9.0).monospace()).fill(fill).min_size(Vec2::new(side_panel_w - 50.0, 20.0)));
                        ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
                        if resp.clicked() { self.selected_conv = Some(i); }
                        if btn(ui, RichText::new("✕").size(9.0).color(Color32::from_rgb(200, 50, 50))).on_hover_text("Delete conversation").clicked() {
                            del_idx = Some(i);
                        }
                    });
                }
                if let Some(idx) = del_idx {
                    if self.selected_conv == Some(idx) {
                        self.selected_conv = if idx < self.conversations.len().saturating_sub(1) { Some(idx) } else if idx > 0 { Some(idx - 1) } else { None };
                    }
                    self.conversations.remove(idx);
                }
            });
        });

        let msg_x = r.left() + 5.0 + side_panel_w + 5.0;
        let msg_rect = Rect::from_min_size(Pos2::new(msg_x, r.top() + 5.0), Vec2::new(r.right() - msg_x - 5.0, r.height() - 50.0));
        ui.painter().rect_filled(msg_rect, Rounding::same(6.0), Color32::from_rgba_premultiplied(0, 6, 18, 200));
        ui.painter().rect_stroke(msg_rect, Rounding::same(6.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 30, 80, 80)));
        ui.allocate_ui_at_rect(msg_rect.shrink(8.0), |ui| {
            if let Some(idx) = self.selected_conv {
                if idx < self.conversations.len() {
                    ui.label(RichText::new(&self.conversations[idx].title).color(primary).size(12.0).strong());
                    ui.separator();
                    let scroll_h = msg_rect.height() - 80.0;
                    let scroll_rect = Rect::from_min_size(Pos2::new(msg_rect.left() + 8.0, msg_rect.top() + 40.0), Vec2::new(msg_rect.width() - 16.0, scroll_h));
                    ui.allocate_ui_at_rect(scroll_rect, |ui| {
                        scroll_area().show(ui, |ui| {
                            for msg in &self.conversations[idx].messages {
                                let icon = if msg.is_user { "♂️" } else { "♀️" };
                                let c = if msg.is_user { Color32::from_rgb(124, 58, 237) } else { Color32::from_rgb(100, 220, 150) };
                                ui.label(RichText::new(format!("{} {}: {}", icon, msg.sender, msg.text)).color(c).size(10.0).monospace());
                            }
                            if self.conv_thinking.is_some() {
                                ui.label(RichText::new("♀️ umbra: thinking...").color(Color32::from_rgb(100, 150, 180)).size(10.0).monospace());
                            }
                        });
                    });

                    let in_y = msg_rect.bottom() - 36.0;
                    let in_rect = Rect::from_min_size(Pos2::new(msg_rect.left() + 8.0, in_y), Vec2::new(msg_rect.width() - 70.0, 30.0));
                    ui.allocate_ui_at_rect(in_rect, |ui| {
                        let resp = ui.add(TextEdit::singleline(&mut self.conv_input)
                            .font(FontId::monospace(10.0))
                            .hint_text("Type a message...")
                            .desired_width(f32::INFINITY));
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.send_conv_message();
                        }
                    });
                    let snd_rect = Rect::from_min_size(Pos2::new(msg_rect.right() - 55.0, in_y), Vec2::new(48.0, 30.0));
                    ui.allocate_ui_at_rect(snd_rect, |ui| {
                        if btn(ui, RichText::new("⏎ SEND").size(9.0).color(primary)).on_hover_text("Send message").clicked() {
                            self.send_conv_message();
                        }
                    });
                } else {
                    ui.label(RichText::new("Select or create a conversation").color(Color32::from_rgb(120, 160, 190)).size(10.0).monospace());
                }
            } else {
                ui.label(RichText::new("Select or create a conversation").color(Color32::from_rgb(120, 160, 190)).size(10.0).monospace());
            }
        });
    }

    fn render_sphere_view(&mut self, ui: &mut egui::Ui, r: Rect, t: f32, sphere_cx: f32, sphere_cy: f32, primary: Color32) {
        let hue = self.current_emotion.hue();
        let sat = self.current_emotion.saturation();
        let intensity = self.current_emotion.intensity();
        self.main_sphere.render(&ui.painter(), Pos2::new(sphere_cx, sphere_cy), hue, sat, intensity, 0.5, t, 0.3, 0.5, 0.1, 0.0, 0.6);

        let pw = 380.0;
        let panel = Rect::from_min_size(
            Pos2::new(r.right() - pw - 16.0, r.top() + 50.0),
            Vec2::new(pw, (r.height() - 80.0).min(500.0)),
        );
        let panel_bg = Color32::from_rgba_premultiplied(2, 10, 24, 235);
        let panel_border = Stroke::new(1.0, Color32::from_rgba_premultiplied(100, 70, 180, 80));

        ui.painter().rect_filled(panel, Rounding::same(10.0), panel_bg);
        ui.painter().rect_stroke(panel, Rounding::same(10.0), panel_border);

        let inner = panel.shrink(12.0);
        ui.allocate_ui_at_rect(inner, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🧠 Umbra Analysis").color(primary).size(14.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if btn(ui, RichText::new("✕").size(14.0).color(Color32::from_rgb(167, 139, 250))).clicked() {
                        self.sphere_selected = false;
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            let analysis_h = ui.available_height() - 90.0;
            egui::ScrollArea::vertical()
                .max_height(analysis_h.max(80.0))
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.label(RichText::new(&self.umbra_analysis).color(Color32::from_rgb(180, 200, 220)).size(10.0).monospace());
                });

            ui.separator();
            ui.add_space(2.0);
            ui.label(RichText::new("💬 Chat with Umbra").color(primary).size(11.0).strong());
            ui.add_space(2.0);

            for msg in &self.chat_messages {
                let c = if msg.is_user { primary } else { Color32::from_rgb(180, 200, 220) };
                ui.label(RichText::new(&msg.text).color(c).size(10.0).monospace());
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let resp = ui.add(egui::TextEdit::singleline(&mut self.hud_input)
                    .hint_text("Ask Umbra anything...")
                    .desired_width(pw - 120.0));
                let submit = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if btn(ui, RichText::new("Send").size(9.0)).clicked() || submit {
                    let text = self.hud_input.trim().to_string();
                    if !text.is_empty() {
                        self.chat_messages.push(Message { sender: "user".into(), text: format!("You: {}", text), is_user: true });
                        self.chat_messages.push(Message { sender: "umbra".into(), text: "Umbra: Processing...".into(), is_user: false });
                        self.hud_input.clear();
                    }
                }
                let has_stt = self.stt.is_some();
                let mic_color = if has_stt { Color32::from_rgb(0, 200, 100) } else { Color32::from_rgb(100, 100, 120) };
                if btn(ui, RichText::new("🎤").size(14.0).color(mic_color))
                    .on_hover_text(if has_stt { "Voice input (whisper)" } else { "STT not available" })
                    .clicked() {
                    if self.stt.is_some() {
                        let result_arc = self.stt_result.clone();
                        let dummy_audio = vec![0u8; 1024];
                        let stt_client = SttClient::new_local();
                        tokio::spawn(async move {
                            if let Ok(text) = stt_client.transcribe(&dummy_audio).await {
                                if let Ok(mut guard) = result_arc.lock() {
                                    *guard = Some(text);
                                }
                            }
                        });
                    }
                }
            });
        });
    }

    fn render_sidebar_menu(&mut self, ui: &mut egui::Ui, r: Rect, primary: Color32) {
        let overlay = Rect::from_min_size(Pos2::ZERO, Vec2::new(r.width(), r.height()));
        let menu_w = 440.0;
        let menu_h = r.height() - 80.0;
        let menu_x = (r.width() - menu_w) / 2.0;
        let menu_y = 40.0;
        let menu_rect = Rect::from_min_size(Pos2::new(menu_x, menu_y), Vec2::new(menu_w, menu_h));
        let overlay_sense = ui.allocate_rect(overlay, egui::Sense::click());
        if overlay_sense.clicked() {
            let mouse_pos = ui.ctx().pointer_interact_pos().unwrap_or(Pos2::ZERO);
            if !menu_rect.contains(mouse_pos) {
                self.sidebar_open = false;
            }
        }
        ui.painter().rect_filled(overlay, Rounding::ZERO, Color32::from_rgba_premultiplied(0, 2, 8, 50));
        ui.painter().rect_filled(menu_rect, Rounding::same(12.0), Color32::from_rgba_premultiplied(0, 6, 16, 200));
        ui.painter().rect_stroke(menu_rect, Rounding::same(12.0), Stroke::new(1.0, Color32::from_rgba_premultiplied(60, 40, 100, 80)));

        if btn(ui, RichText::new("✕").size(14.0).color(Color32::from_rgb(167, 139, 250))).clicked() { self.sidebar_open = false; }

        let tabs = ["SETTINGS", "SKILLS", "RESEARCH", "TRAINING", "CODING", "ABOUT"];
        ui.allocate_ui_at_rect(Rect::from_min_size(Pos2::new(menu_rect.left() + 12.0, menu_rect.top() + 12.0), Vec2::new(menu_w - 24.0, 60.0)), |ui| {
            ui.horizontal(|ui| {
                for (i, t) in tabs.iter().enumerate() {
                    let sel = self.sidebar_tab == i;
                    let c = if sel { primary } else { Color32::from_rgb(106, 90, 180) };
                    if btn(ui, RichText::new(*t).color(c).size(10.0).monospace()).clicked() { self.sidebar_tab = i; }
                }
            });
        });

        let content_y = menu_rect.top() + 48.0;
        let content_rect = Rect::from_min_size(Pos2::new(menu_rect.left() + 12.0, content_y), Vec2::new(menu_w - 24.0, menu_rect.bottom() - content_y - 16.0));
        ui.allocate_ui_at_rect(content_rect, |ui| {
            scroll_area().show(ui, |ui| {
                match self.sidebar_tab {
                    0 => {
                        let stabs = ["VAULT", "PROVIDERS", "MODELS", "AGENTS", "VOICE", "APPEARANCE", "SHORTCUTS", "HUGGINGFACE"];
                        let stab_h = 18.0;
                        ui.horizontal(|ui| {
                            for (i, t) in stabs.iter().enumerate() {
                                let sel = self.settings_tab == i;
                                let c = if sel { primary } else { Color32::from_rgb(130, 100, 190) };
                                if btn_rounded(ui, RichText::new(*t).color(c).size(9.0).monospace(), Rounding::ZERO, Vec2::new(0.0, stab_h)).clicked() { self.settings_tab = i; }
                            }
                        });

                        match self.settings_tab {
                            0 => {
                                ui.label(RichText::new("API VAULT").color(primary).size(13.0));
                                let lock = if self.vault_open { "🔓 UNLOCKED" } else { "🔒 LOCKED" };
                                ui.label(RichText::new(lock).color(if self.vault_open { Color32::GREEN } else { Color32::RED }).size(11.0).monospace());
                                if btn(ui, RichText::new(if self.vault_open { "LOCK" } else { "UNLOCK" }).size(9.0)).clicked() { self.vault_open = !self.vault_open; }
                                for p in &self.providers {
                                    if p.configured {
                                        ui.label(RichText::new(format!("✅ {}", p.name)).color(Color32::from_rgb(100, 200, 100)).size(10.0).monospace());
                                    }
                                }
                            }
                            1 => {
                                ui.label(RichText::new("PROVIDERS").color(primary).size(13.0));
                                scroll_area().show(ui, |ui| {
                                    for p in &mut self.providers {
                                        ui.horizontal(|ui| {
                                            let icon = if p.configured { "✅" } else { "⬜" };
                                            ui.label(RichText::new(format!("{} {}", icon, p.name)).size(9.0).monospace());
                                            if p.configured {
                                                let status = match p.name.as_str() {
                                                    "Ollama" | "llama.cpp" => "auto-detected",
                                                    _ => "key saved",
                                                };
                                                let sc = match p.name.as_str() {
                                                    "Ollama" | "llama.cpp" => Color32::from_rgb(100, 200, 255),
                                                    _ => Color32::from_rgb(0, 180, 100),
                                                };
                                                ui.label(RichText::new(status).color(sc).size(8.0).monospace());
                                                if btn(ui, RichText::new("✕").size(9.0)).on_hover_text("Remove provider key").clicked() { p.configured = false; p.key.clear(); }
                                            } else {
                                                let resp = ui.add(TextEdit::singleline(&mut p.key).password(true).font(FontId::monospace(8.0)).desired_width(80.0).hint_text("api key"));
                                                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                                    if !p.key.is_empty() { p.configured = true; }
                                                }
                                                if btn(ui, RichText::new("SAVE").size(9.0)).clicked() {
                                                    if !p.key.is_empty() { p.configured = true; }
                                                }
                                            }
                                        });
                                    }
                                });
                            }
                            2 => {
                                ui.label(RichText::new("MODELS").color(primary).size(13.0));
                                ui.separator();
                                let ollama_ok = self.providers.iter().any(|p| p.name == "Ollama" && p.configured);
                                let cpp_ok = self.providers.iter().any(|p| p.name == "llama.cpp" && p.configured);
                                ui.label(RichText::new(format!("Ollama (local): {}", if ollama_ok { "✅ Available" } else { "❌ Not detected" })).size(10.0).monospace());
                                ui.label(RichText::new(format!("llama.cpp (local): {}", if cpp_ok { "✅ Available" } else { "❌ Not detected" })).size(10.0).monospace());
                                ui.separator();
                                ui.label(RichText::new("API Models:").color(Color32::from_rgb(0, 180, 220)).size(10.0).monospace());
                                let api_providers = ["OpenAI", "Anthropic", "Google", "DeepSeek", "Mistral", "Groq"];
                                for name in &api_providers {
                                    let ok = self.providers.iter().any(|p| p.name == *name && p.configured);
                                    ui.label(RichText::new(format!("  {}: {}", name, if ok { "✅ Configured" } else { "⬜ Not configured" })).size(9.0).monospace());
                                }
                            }
                            3 => {
                                ui.label(RichText::new("AGENTS").color(primary).size(13.0));
                                for a in &mut self.agents {
                                    ui.horizontal(|ui| {
                                        let icon = if a.active { "●" } else { "○" };
                                        let c = if a.active { primary } else { Color32::from_rgb(80, 80, 80) };
                                        ui.label(RichText::new(format!("{} {} [{}] {}", icon, a.name, a.agent_type, a.gender.icon())).color(c).size(9.0).monospace());
                                        if btn(ui, RichText::new(if a.active { "DEACTIVATE" } else { "ACTIVATE" }).size(9.0)).clicked() { a.active = !a.active; }
                                    });
                                }
                                ui.separator();
                                ui.label(RichText::new("Create Agent").color(Color32::from_rgb(0, 180, 220)).size(11.0).monospace());
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Name:").size(9.0).monospace());
                                    ui.add(TextEdit::singleline(&mut self.new_agent_name).font(FontId::monospace(9.0)).desired_width(100.0));
                                });
                                let types = ["LLM", "JEPA", "SNN", "SSM", "Audio", "Vision"];
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Type:").size(9.0).monospace());
                                    for (i, ty) in types.iter().enumerate() {
                                        let sel = self.new_agent_type == i;
                                        let c = if sel { primary } else { Color32::from_rgb(140, 170, 200) };
                                        if btn(ui, RichText::new(*ty).color(c).size(9.0).monospace()).clicked() { self.new_agent_type = i; }
                                    }
                                });
                                let genders = [AiGender::Male, AiGender::Female, AiGender::Androgynous, AiGender::Neutral];
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Gender:").size(9.0).monospace());
                                    for (i, g) in genders.iter().enumerate() {
                                        let sel = self.new_agent_gender == i;
                                        let c = if sel { primary } else { Color32::from_rgb(140, 170, 200) };
                                        if btn(ui, RichText::new(g.icon()).color(c).size(9.0).monospace()).clicked() { self.new_agent_gender = i; }
                                    }
                                });
                                if btn(ui, RichText::new("➕ CREATE").color(Color32::from_rgb(0, 200, 100)).size(9.0).monospace()).clicked() {
                                    if !self.new_agent_name.is_empty() {
                                        self.agents.push(AgentEntry {
                                            name: self.new_agent_name.clone(),
                                            active: false,
                                            agent_type: types[self.new_agent_type].into(),
                                            gender: genders[self.new_agent_gender].clone(),
                                        });
                                        self.new_agent_name.clear();
                                    }
                                }
                                ui.separator();
                                if btn(ui, RichText::new("🗑 Delete All").color(Color32::from_rgb(200, 50, 50)).size(9.0).monospace()).clicked() {
                                    self.agents.clear();
                                }
                            }
                            4 => {
                                ui.label(RichText::new("VOICE").color(primary).size(13.0));
                                let tts_status = if self.local_tts_detected {
                                    "🗣️ Local (Piper/espeak) detected"
                                } else if self.fish_api_detected {
                                    "☁️ Fish API configured"
                                } else {
                                    "🔍 Auto-detecting..."
                                };
                                ui.label(RichText::new(tts_status).size(10.0).monospace());
                                ui.label(RichText::new(format!("Tone: {}", self.voice_tone)).color(Color32::from_rgb(130, 100, 200)).size(10.0).monospace());
                                if btn(ui, RichText::new(if self.muted { "🔇 MUTED" } else { "🔊 ACTIVE" }).size(9.0)).on_hover_text("Toggle mute").clicked() { self.muted = !self.muted; }
                                if btn(ui, RichText::new("🔄 Re-detect TTS").size(9.0)).on_hover_text("Re-detect TTS engines").clicked() {
                                    self.detect_tts();
                                }
                                ui.separator();
                                ui.label(RichText::new("User Gender:").size(10.0).monospace());
                                for g in &["male", "female"] {
                                    let sel = self.user_gender == *g;
                                    let c = if sel { primary } else { Color32::from_rgb(130, 100, 190) };
                                    if btn(ui, RichText::new(*g).color(c).size(9.0).monospace()).clicked() {
                                        self.user_gender = g.to_string();
                                    }
                                }
                                let address = if self.user_gender == "female" { "ma'am" } else { "sir" };
                                ui.label(RichText::new(format!("Addresses you as: {}", address)).color(Color32::from_rgb(100, 220, 150)).size(10.0).monospace());
                            }
                            5 => {
                                ui.label(RichText::new("APPEARANCE").color(primary).size(13.0));
                                ui.separator();
                                let theme_names = ["Dark Blue", "Dark Purple", "Dark Green", "Dark Red", "AMOLED"];
                                ui.label(RichText::new("Theme:").size(10.0).monospace());
                                ui.horizontal(|ui| {
                                    for (i, tn) in theme_names.iter().enumerate() {
                                        let sel = self.theme == i;
                                        let c = if sel { primary } else { Color32::from_rgb(140, 170, 200) };
                                        if btn(ui, RichText::new(*tn).color(c).size(9.0).monospace()).clicked() {
                                            self.theme = i;
                                            self.primary_color_hex = match i {
                                                0 => "#00DCFF".into(),
                                                1 => "#BB86FC".into(),
                                                2 => "#00E676".into(),
                                                3 => "#FF5252".into(),
                                                4 => "#FFFFFF".into(),
                                                _ => "#00DCFF".into(),
                                            };
                                        }
                                    }
                                });
                                ui.separator();
                                ui.label(RichText::new("Primary Color (hex):").size(10.0).monospace());
                                ui.horizontal(|ui| {
                                    ui.add(TextEdit::singleline(&mut self.primary_color_hex).font(FontId::monospace(10.0)).desired_width(100.0).hint_text("#RRGGBB"));
                                    if btn(ui, RichText::new("APPLY").size(9.0)).clicked() {
                                    }
                                });
                                let preview = self.primary_color();
                                let preview_rect = Rect::from_min_size(Pos2::new(ui.max_rect().left() + 10.0, ui.max_rect().bottom() - 20.0), Vec2::new(30.0, 16.0));
                                ui.painter().rect_filled(preview_rect, Rounding::same(3.0), preview);
                                ui.label(RichText::new("  Preview").color(Color32::from_rgb(120, 150, 180)).size(9.0).monospace());
                            }
                            6 => {
                                ui.label(RichText::new("SHORTCUTS").color(primary).size(13.0));
                                ui.separator();
                                let sc_len = self.shortcuts.len();
                                for i in 0..sc_len {
                                    let action = self.shortcuts[i].action.clone();
                                    let key = self.shortcuts[i].key.clone();
                                    let recording = self.shortcuts[i].recording;
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&action).color(Color32::from_rgb(150, 180, 200)).size(9.0).monospace());
                                        if recording {
                                            ui.label(RichText::new("[PRESS KEY...]").color(Color32::from_rgb(0, 255, 100)).size(9.0).monospace());
                                        } else {
                                            ui.label(RichText::new(&key).color(primary).size(9.0).monospace());
                                        }
                                        if btn(ui, RichText::new("🎯").size(9.0)).on_hover_text("Record new shortcut").clicked() {
                                            self.recording_shortcut = Some(i);
                                            self.shortcuts[i].recording = true;
                                        }
                                    });
                                }
                                if let Some(idx) = self.recording_shortcut {
                                    if idx < self.shortcuts.len() && !self.shortcuts[idx].recording {
                                        self.recording_shortcut = None;
                                    }
                                }
                            }
                            7 => {
                                ui.label(RichText::new("HUGGINGFACE").color(primary).size(13.0));
                                ui.label(RichText::new("Download models from HuggingFace").color(Color32::from_rgb(130, 100, 190)).size(9.0).monospace());
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Model:").size(9.0).monospace());
                                    ui.add(TextEdit::singleline(&mut self.hf_model_name).font(FontId::monospace(9.0)).desired_width(120.0).hint_text("username/model"));
                                });
                                if self.hf_downloading {
                                    ui.label(RichText::new("⏳ Downloading...").color(Color32::from_rgb(255, 200, 0)).size(10.0).monospace());
                                } else {
                                    if btn(ui, RichText::new("⬇ DOWNLOAD").color(primary).size(9.0).monospace()).on_hover_text("Download model from HuggingFace").clicked() {
                                        if !self.hf_model_name.is_empty() {
                                            self.hf_downloading = true;
                                            self.hf_status = format!("Downloading {}...", self.hf_model_name);
                                        }
                                    }
                                }
                                if !self.hf_status.is_empty() && !self.hf_downloading {
                                    ui.label(RichText::new(&self.hf_status).color(Color32::from_rgb(100, 200, 100)).size(9.0).monospace());
                                }
                            }
                            8 => {
                                ui.label(RichText::new("BROKER ACCOUNTS").color(primary).size(13.0));
                                ui.separator();
                                ui.label(RichText::new("Configure your MT5 broker accounts").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
                                ui.add_space(6.0);
                                ui.horizontal(|ui| { ui.label(RichText::new("Broker:").size(9.0).monospace()); ui.add(TextEdit::singleline(&mut self.broker_name).desired_width(160.0)); });
                                ui.horizontal(|ui| { ui.label(RichText::new("Server:").size(9.0).monospace()); ui.add(TextEdit::singleline(&mut self.broker_server).desired_width(160.0)); });
                                ui.horizontal(|ui| { ui.label(RichText::new("Account:").size(9.0).monospace()); ui.add(TextEdit::singleline(&mut self.broker_account).desired_width(160.0)); });
                                ui.horizontal(|ui| { ui.label(RichText::new("Password:").size(9.0).monospace()); ui.add(TextEdit::singleline(&mut self.broker_password).password(true).desired_width(160.0)); });
                                ui.add_space(6.0);
                                if btn(ui, RichText::new("CONNECT").color(primary).size(10.0)).clicked() { self.trading_active = true; }
                                ui.add_space(4.0);
                                ui.label(RichText::new("Supported brokers for MT5:").color(Color32::from_rgb(100, 120, 160)).size(8.0).monospace());
                                for broker in &["Pepperstone (AU)", "IC Markets", "FP Markets", "ThinkMarkets", "Vantage", "Eightcap"] {
                                    ui.label(RichText::new(format!("  • {}", broker)).color(Color32::from_rgb(80, 100, 140)).size(8.0).monospace());
                                }
                            }
                            _ => {}
                        }
                    }
                    1 => {
                        ui.label(RichText::new("SKILLS").color(primary).size(13.0).strong());
                        ui.separator();
                        ui.label(RichText::new("Agent skills system").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
                        ui.label(RichText::new("  • IronClaw action validation").size(9.0).monospace());
                        ui.label(RichText::new("  • Hermes task orchestration").size(9.0).monospace());
                        ui.label(RichText::new("  • Thoth security verification").size(9.0).monospace());
                        ui.label(RichText::new("  • Synapsis memory management").size(9.0).monospace());
                        ui.label(RichText::new("  • HSAQ compression engine").size(9.0).monospace());
                        ui.label(RichText::new("  • Cognitive therapy system").size(9.0).monospace());
                        ui.add_space(6.0);
                        ui.label(RichText::new("Capabilities loaded: 12").color(primary).size(9.0).monospace());
                    }
                    2 => {
                        ui.label(RichText::new("RESEARCH").color(primary).size(13.0).strong());
                        ui.separator();
                        ui.label(RichText::new("Active research areas:").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
                        ui.label(RichText::new("  • Multi-agent conversation optimization").size(9.0).monospace());
                        ui.label(RichText::new("  • Emotional cognitive architectures").size(9.0).monospace());
                        ui.label(RichText::new("  • HSAQ vs TurboQuant benchmarking").size(9.0).monospace());
                        ui.label(RichText::new("  • Plutchik emotion wheel integration").size(9.0).monospace());
                        ui.label(RichText::new("  • Real-time voice emotion detection").size(9.0).monospace());
                        ui.add_space(6.0);
                        ui.label(RichText::new("Research status: Active").color(Color32::from_rgb(100, 200, 100)).size(9.0).monospace());
                    }
                    3 => {
                        ui.label(RichText::new("TRAINING").color(primary).size(13.0).strong());
                        ui.separator();
                        ui.label(RichText::new("Model training pipeline:").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
                        ui.label(RichText::new("  • JEPA model training").size(9.0).monospace());
                        ui.label(RichText::new("  • HSAQ quantization").size(9.0).monospace());
                        ui.label(RichText::new("  • .materia model export").size(9.0).monospace());
                        ui.label(RichText::new("  • Cognitive parameter optimization").size(9.0).monospace());
                        ui.add_space(6.0);
                        ui.label(RichText::new("Training ready: ✅").color(Color32::from_rgb(100, 200, 100)).size(9.0).monospace());
                    }
                    4 => {
                        ui.label(RichText::new("CODING").color(primary).size(13.0).strong());
                        ui.separator();
                        ui.label(RichText::new("Development environment:").color(Color32::from_rgb(130, 100, 200)).size(9.0).monospace());
                        ui.label(RichText::new("  • Rust backend (umbra-core)").size(9.0).monospace());
                        ui.label(RichText::new("  • egui desktop frontend").size(9.0).monospace());
                        ui.label(RichText::new("  • MT5 bridge integration").size(9.0).monospace());
                        ui.label(RichText::new("  • llama-cpp-rs standalone port").size(9.0).monospace());
                        ui.add_space(6.0);
                        ui.label(RichText::new("Active project: Umbra v0.2.0").color(primary).size(9.0).monospace());
                    }
                    5 => {
                        ui.label(RichText::new("UMBRA v0.2.0").color(primary).size(14.0).strong());
                        ui.label(RichText::new("AI Agent System").size(10.0).monospace());
                        ui.separator();
                        ui.label(RichText::new("Umbra-Agent orchestration").size(9.0).monospace());
                        ui.label(RichText::new("IronClaw security layer").size(9.0).monospace());
                        ui.label(RichText::new("Thoth validation system").size(9.0).monospace());
                        ui.label(RichText::new("HSAQ compression engine").size(9.0).monospace());
                        ui.label(RichText::new("M.A.T.E.R.I.A. V3 architecture").size(9.0).monospace());

                        if let Some(tex) = &self.logo_texture {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                ui.add(egui::Image::new((tex.id(), Vec2::new(64.0, 64.0))));
                            });
                        }
                    }
                    _ => {}
                }
            });
        });
    }
}

fn which_exists(name: &str) -> bool {
    std::env::var("PATH").map(|path| {
        path.split(':').any(|dir| {
            let full = format!("{}/{}", dir, name);
            std::fs::metadata(&full).is_ok()
        })
    }).unwrap_or(false)
}

fn btn(ui: &mut egui::Ui, text: impl Into<egui::RichText>) -> egui::Response {
    btn_fill(ui, text, Color32::TRANSPARENT, HOVER_PURPLE)
}

fn btn_rounded(ui: &mut egui::Ui, text: impl Into<egui::RichText>, rounding: Rounding, min_size: Vec2) -> egui::Response {
    let text: egui::RichText = text.into();
    let id = ui.next_auto_id();
    let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let bg = if prev_hovered { HOVER_PURPLE } else { Color32::TRANSPARENT };
    let resp = ui.add(egui::Button::new(text).fill(bg).rounding(rounding).min_size(min_size));
    ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
    resp
}

fn btn_fill(ui: &mut egui::Ui, text: impl Into<egui::RichText>, normal: Color32, hover: Color32) -> egui::Response {
    let text: egui::RichText = text.into();
    let id = ui.next_auto_id();
    let prev_hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let bg = if prev_hovered { hover } else { normal };
    let resp = ui.add(egui::Button::new(text).fill(bg));
    ui.data_mut(|d| d.insert_temp(id, resp.hovered()));
    resp
}

fn scroll_area() -> egui::ScrollArea {
    egui::ScrollArea::vertical()
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
}
