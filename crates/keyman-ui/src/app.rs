use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{
    ActiveTheme,
    button::*,
    table::*,
    v_flex, h_flex,
    Sizable, StyledExt, Disableable, IndexPath,
    checkbox::Checkbox,
};
use gpui_component::select::{Select, SelectState, SelectItem, SearchableVec, SelectEvent};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui::FocusHandle;
use keyman_core::config::AppConfig;
use keyman_core::engine::RemappingEngine;
use keyman_core::engine::SharedEngine;
use keyman_core::scheme::KeybindScheme;
use keyman_core::i18n;
use keyman_hook::key::VirtualKey;
use keyman_hook::hook::KeyboardHook;
use keyman_detect::GameDetectionService;

/// 方案选择项
#[derive(Clone, PartialEq)]
struct SchemeItem {
    id: String,
    name: String,
}

impl SelectItem for SchemeItem {
    type Value = String;

    fn title(&self) -> SharedString {
        SharedString::from(self.name.clone())
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }
}

pub struct KeymanApp {
    config: AppConfig,
    engine: SharedEngine,
    detection_service: GameDetectionService,
    capturing: Option<CaptureTarget>,
    error_msg: Option<String>,
    focus_handle: FocusHandle,
    editing_scheme_id: Option<String>,
    rename_input: Entity<InputState>,
    scheme_select: Entity<SelectState<SearchableVec<SchemeItem>>>,
    /// Pending new skill mapping: (source, target)
    pending_skill: (Option<VirtualKey>, Option<VirtualKey>),
}

#[derive(Clone, Copy, PartialEq)]
enum CaptureTarget {
    /// Re-capturing source key for existing mapping
    SkillSource { old_source: VirtualKey, old_target: VirtualKey },
    /// Re-capturing target key for existing mapping
    SkillTarget(VirtualKey),
    Inventory(usize),
    /// Capturing source/target for new pending row
    NewSkillSource,
    NewSkillTarget,
}

impl KeymanApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = AppConfig::load_or_default();

        // 启动游戏检测服务 (before engine so we can share game state)
        let detection_service = GameDetectionService::start();
        let game_state = detection_service.state_handle();

        let mut engine = RemappingEngine::new(game_state);
        let schemes: Vec<_> = config.schemes.clone();
        let active_id = config.active_scheme_id.clone();
        engine.set_schemes(schemes, &active_id);
        let engine = std::sync::Arc::new(std::sync::Mutex::new(engine));

        // Install low-level keyboard hook
        {
            let engine = engine.clone();
            #[cfg(target_os = "windows")]
            let mut hook = keyman_hook::WindowsKeyboardHook::new();
            #[cfg(target_os = "macos")]
            let mut hook = keyman_hook::MacosKeyboardHook::new();
            #[cfg(target_os = "linux")]
            let mut hook = keyman_hook::LinuxKeyboardHook::new();
            if let Err(e) = hook.install(Box::new(move |event: &keyman_hook::event::RawKeyEvent| {
                let mut eng = engine.lock().unwrap();
                eng.process_event(event)
            })) {
                eprintln!("Failed to install keyboard hook: {}", e);
            }
            // Hook is installed; the HHOOK is stored inside the hook struct.
            // We must NOT drop it — but since low_level_keyboard_proc is a static
            // function registered with Windows, the hook stays alive as long as
            // the DLL/module is loaded. We intentionally leak the hook handle.
            std::mem::forget(hook);
        }

        // 创建方案选择器
        let scheme_items: SearchableVec<SchemeItem> = SearchableVec::new(
            config.schemes.iter().map(|s| SchemeItem {
                id: s.id.clone(),
                name: s.name.clone(),
            }).collect::<Vec<_>>()
        );
        let selected_idx = config.schemes.iter()
            .position(|s| s.id == config.active_scheme_id)
            .map(|i| IndexPath::default().row(i));

        let scheme_select = cx.new(|cx| {
            SelectState::new(scheme_items, selected_idx, window, cx)
        });

        // 订阅方案选择事件
        cx.subscribe_in(&scheme_select, window, |this, _select, event: &SelectEvent<SearchableVec<SchemeItem>>, _window, cx| {
            if let SelectEvent::Confirm(Some(scheme_id)) = event {
                this.config.switch_scheme(scheme_id);
                this.engine.lock().unwrap().set_active_scheme(&this.config.active_scheme_id);
                let _ = this.config.save();
                cx.notify();
            }
        }).detach();

        // Create rename input state
        let rename_input = cx.new(|cx| InputState::new(window, cx));

        // Subscribe to rename input events
        cx.subscribe_in(&rename_input, window, |this, _input, ev: &InputEvent, window, cx| {
            match ev {
                InputEvent::PressEnter { .. } => {
                    this.confirm_rename(window, cx);
                }
                InputEvent::Blur => {
                    if this.editing_scheme_id.is_some() {
                        this.confirm_rename(window, cx);
                    }
                }
                _ => {}
            }
        }).detach();

        Self {
            config,
            engine,
            detection_service,
            capturing: None,
            error_msg: None,
            focus_handle: cx.focus_handle(),
            editing_scheme_id: None,
            rename_input,
            scheme_select,
            pending_skill: (None, None),
        }
    }

    fn active_scheme(&self) -> Option<&KeybindScheme> {
        self.config.active_scheme()
    }

    fn format_key(key: &VirtualKey) -> String {
        match key {
            VirtualKey::A => "A".into(),
            VirtualKey::B => "B".into(),
            VirtualKey::C => "C".into(),
            VirtualKey::D => "D".into(),
            VirtualKey::E => "E".into(),
            VirtualKey::F => "F".into(),
            VirtualKey::G => "G".into(),
            VirtualKey::H => "H".into(),
            VirtualKey::I => "I".into(),
            VirtualKey::J => "J".into(),
            VirtualKey::K => "K".into(),
            VirtualKey::L => "L".into(),
            VirtualKey::M => "M".into(),
            VirtualKey::N => "N".into(),
            VirtualKey::O => "O".into(),
            VirtualKey::P => "P".into(),
            VirtualKey::Q => "Q".into(),
            VirtualKey::R => "R".into(),
            VirtualKey::S => "S".into(),
            VirtualKey::T => "T".into(),
            VirtualKey::U => "U".into(),
            VirtualKey::V => "V".into(),
            VirtualKey::W => "W".into(),
            VirtualKey::X => "X".into(),
            VirtualKey::Y => "Y".into(),
            VirtualKey::Z => "Z".into(),
            VirtualKey::Key0 => "0".into(),
            VirtualKey::Key1 => "1".into(),
            VirtualKey::Key2 => "2".into(),
            VirtualKey::Key3 => "3".into(),
            VirtualKey::Key4 => "4".into(),
            VirtualKey::Key5 => "5".into(),
            VirtualKey::Key6 => "6".into(),
            VirtualKey::Key7 => "7".into(),
            VirtualKey::Key8 => "8".into(),
            VirtualKey::Key9 => "9".into(),
            VirtualKey::Space => "Space".into(),
            VirtualKey::Enter => "Enter".into(),
            VirtualKey::Escape => "Esc".into(),
            VirtualKey::Tab => "Tab".into(),
            VirtualKey::ScrollLock => "ScrollLock".into(),
            VirtualKey::Pause => "Pause".into(),
            VirtualKey::Numpad0 => "Num0".into(),
            VirtualKey::Numpad1 => "Num1".into(),
            VirtualKey::Numpad2 => "Num2".into(),
            VirtualKey::Numpad3 => "Num3".into(),
            VirtualKey::Numpad4 => "Num4".into(),
            VirtualKey::Numpad5 => "Num5".into(),
            VirtualKey::Numpad6 => "Num6".into(),
            VirtualKey::Numpad7 => "Num7".into(),
            VirtualKey::Numpad8 => "Num8".into(),
            VirtualKey::Numpad9 => "Num9".into(),
            VirtualKey::Unknown(code) => format!("Unknown({})", code),
            _ => format!("{:?}", key),
        }
    }

    fn parse_key_from_string(key: &str) -> Option<VirtualKey> {
        match key.to_lowercase().as_str() {
            "a" => Some(VirtualKey::A),
            "b" => Some(VirtualKey::B),
            "c" => Some(VirtualKey::C),
            "d" => Some(VirtualKey::D),
            "e" => Some(VirtualKey::E),
            "f" => Some(VirtualKey::F),
            "g" => Some(VirtualKey::G),
            "h" => Some(VirtualKey::H),
            "i" => Some(VirtualKey::I),
            "j" => Some(VirtualKey::J),
            "k" => Some(VirtualKey::K),
            "l" => Some(VirtualKey::L),
            "m" => Some(VirtualKey::M),
            "n" => Some(VirtualKey::N),
            "o" => Some(VirtualKey::O),
            "p" => Some(VirtualKey::P),
            "q" => Some(VirtualKey::Q),
            "r" => Some(VirtualKey::R),
            "s" => Some(VirtualKey::S),
            "t" => Some(VirtualKey::T),
            "u" => Some(VirtualKey::U),
            "v" => Some(VirtualKey::V),
            "w" => Some(VirtualKey::W),
            "x" => Some(VirtualKey::X),
            "y" => Some(VirtualKey::Y),
            "z" => Some(VirtualKey::Z),
            "0" => Some(VirtualKey::Key0),
            "1" => Some(VirtualKey::Key1),
            "2" => Some(VirtualKey::Key2),
            "3" => Some(VirtualKey::Key3),
            "4" => Some(VirtualKey::Key4),
            "5" => Some(VirtualKey::Key5),
            "6" => Some(VirtualKey::Key6),
            "7" => Some(VirtualKey::Key7),
            "8" => Some(VirtualKey::Key8),
            "9" => Some(VirtualKey::Key9),
            "space" => Some(VirtualKey::Space),
            "enter" => Some(VirtualKey::Enter),
            "escape" => Some(VirtualKey::Escape),
            "tab" => Some(VirtualKey::Tab),
            _ => None,
        }
    }

    fn is_key_used(&self, new_key: VirtualKey, exclude_target: &CaptureTarget) -> bool {
        let Some(scheme) = self.active_scheme() else { return false };

        match exclude_target {
            CaptureTarget::SkillSource { old_source, .. } => {
                for (source, target) in &scheme.skill_mappings {
                    if *source != *old_source && (*source == new_key || *target == new_key) {
                        return true;
                    }
                }
                scheme.inventory_mappings.iter().any(|k| *k == Some(new_key))
            }
            CaptureTarget::SkillTarget(exclude_source) => {
                for (source, target) in &scheme.skill_mappings {
                    if *source != *exclude_source && (*source == new_key || *target == new_key) {
                        return true;
                    }
                }
                scheme.inventory_mappings.iter().any(|k| *k == Some(new_key))
            }
            CaptureTarget::Inventory(exclude_idx) => {
                for (source, target) in &scheme.skill_mappings {
                    if *source == new_key || *target == new_key {
                        return true;
                    }
                }
                scheme.inventory_mappings.iter().enumerate()
                    .any(|(idx, key)| idx != *exclude_idx && *key == Some(new_key))
            }
            CaptureTarget::NewSkillSource => {
                for (source, _) in &scheme.skill_mappings {
                    if *source == new_key {
                        return true;
                    }
                }
                scheme.inventory_mappings.iter().any(|k| *k == Some(new_key))
            }
            CaptureTarget::NewSkillTarget => false,
        }
    }

    fn start_capture(&mut self, target: CaptureTarget, window: &mut Window, cx: &mut Context<Self>) {
        self.capturing = Some(target);
        self.error_msg = None;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn cancel_capture(&mut self, cx: &mut Context<Self>) {
        self.capturing = None;
        self.error_msg = None;
        cx.notify();
    }

    fn apply_key(&mut self, new_key: VirtualKey, cx: &mut Context<Self>) {
        let Some(target) = self.capturing else { return };

        if self.is_key_used(new_key, &target) {
            self.error_msg = Some(i18n::t_key_used(&Self::format_key(&new_key)));
            cx.notify();
            return;
        }

        match target {
            CaptureTarget::SkillSource { old_source, old_target } => {
                if let Some(scheme) = self.config.active_scheme_mut() {
                    scheme.skill_mappings.remove(&old_source);
                    scheme.skill_mappings.insert(new_key, old_target);
                }
                self.save_and_sync_engine();
                self.capturing = None;
                self.error_msg = None;
            }
            CaptureTarget::SkillTarget(source_key) => {
                if let Some(scheme) = self.config.active_scheme_mut() {
                    scheme.skill_mappings.insert(source_key, new_key);
                }
                self.save_and_sync_engine();
                self.capturing = None;
                self.error_msg = None;
            }
            CaptureTarget::Inventory(idx) => {
                if let Some(scheme) = self.config.active_scheme_mut() {
                    if idx < 6 {
                        scheme.inventory_mappings[idx] = Some(new_key);
                    }
                }
                self.save_and_sync_engine();
                self.capturing = None;
                self.error_msg = None;
            }
            CaptureTarget::NewSkillSource => {
                self.pending_skill.0 = Some(new_key);
                self.capturing = None;
                self.error_msg = None;
                // Auto-add if both fields are set
                if self.pending_skill.1.is_some() {
                    self.confirm_add_skill(cx);
                    return;
                }
            }
            CaptureTarget::NewSkillTarget => {
                self.pending_skill.1 = Some(new_key);
                self.capturing = None;
                self.error_msg = None;
                // Auto-add if both fields are set
                if self.pending_skill.0.is_some() {
                    self.confirm_add_skill(cx);
                    return;
                }
            }
        }

        cx.notify();
    }

    fn save_and_sync_engine(&mut self) {
        let _ = self.config.save();
        let schemes: Vec<_> = self.config.schemes.clone();
        let active_id = self.config.active_scheme_id.clone();
        self.engine.lock().unwrap().set_schemes(schemes, &active_id);
    }

    fn remove_skill_mapping(&mut self, source: VirtualKey, cx: &mut Context<Self>) {
        if let Some(scheme) = self.config.active_scheme_mut() {
            scheme.skill_mappings.remove(&source);
        }
        self.save_and_sync_engine();
        cx.notify();
    }

    fn confirm_add_skill(&mut self, cx: &mut Context<Self>) {
        let (source, target) = self.pending_skill;
        if let (Some(s), Some(t)) = (source, target) {
            if let Some(scheme) = self.config.active_scheme_mut() {
                scheme.skill_mappings.insert(s, t);
            }
            self.save_and_sync_engine();
        }
        self.pending_skill = (None, None);
        cx.notify();
    }

    fn delete_scheme(&mut self, scheme_id: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.config.schemes.len() <= 1 {
            self.error_msg = Some(i18n::t_cannot_delete_last());
            cx.notify();
            return;
        }

        self.config.schemes.retain(|s| s.id != scheme_id);

        if self.config.active_scheme_id == scheme_id {
            if let Some(first) = self.config.schemes.first() {
                self.config.active_scheme_id = first.id.clone();
                self.engine.lock().unwrap().set_active_scheme(&self.config.active_scheme_id);
            }
        }

        let _ = self.config.save();
        self.refresh_scheme_select(window, cx);
        cx.notify();
    }

    fn start_rename(&mut self, scheme_id: String, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(scheme) = self.config.schemes.iter().find(|s| s.id == scheme_id) {
            self.rename_input.update(cx, |state, cx| {
                state.set_value(&scheme.name, window, cx);
            });
        }
        self.editing_scheme_id = Some(scheme_id);
        // Focus the input after a tick so the dialog is visible
        self.rename_input.update(cx, |state, cx| {
            state.focus(window, cx);
        });
        cx.notify();
    }

    fn cancel_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_scheme_id = None;
        self.rename_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        cx.notify();
    }

    fn confirm_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(scheme_id) = self.editing_scheme_id.take() {
            let new_name = self.rename_input.read(cx).value().to_string();
            let new_name = new_name.trim().to_string();
            if !new_name.is_empty() {
                // Check for duplicate name
                let duplicate = self.config.schemes.iter()
                    .any(|s| s.id != scheme_id && s.name == new_name);
                if duplicate {
                    self.error_msg = Some(i18n::t_scheme_exists(&new_name));
                    self.editing_scheme_id = Some(scheme_id);
                    cx.notify();
                    return;
                }
                if let Some(scheme) = self.config.schemes.iter_mut().find(|s| s.id == scheme_id) {
                    scheme.name = new_name;
                    let _ = self.config.save();
                }
            }
        }
        self.rename_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.refresh_scheme_select(window, cx);
        cx.notify();
    }

    fn refresh_scheme_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let scheme_items: SearchableVec<SchemeItem> = SearchableVec::new(
            self.config.schemes.iter().map(|s| SchemeItem {
                id: s.id.clone(),
                name: s.name.clone(),
            }).collect::<Vec<_>>()
        );
        let selected_idx = self.config.schemes.iter()
            .position(|s| s.id == self.config.active_scheme_id)
            .map(|i| IndexPath::default().row(i));

        self.scheme_select.update(cx, |state, cx| {
            state.set_items(scheme_items, window, cx);
            if let Some(idx) = selected_idx {
                state.set_selected_index(Some(idx), window, cx);
            }
        });
    }

    fn render_toolbar(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Sync if scheme was changed in-game (F11)
        {
            let engine = self.engine.lock().unwrap();
            if let Some(engine_id) = engine.active_scheme_id() {
                if engine_id != self.config.active_scheme_id {
                    self.config.active_scheme_id = engine_id.to_string();
                    let _ = self.config.save();
                    // Refresh select dropdown
                    let scheme_items: SearchableVec<SchemeItem> = SearchableVec::new(
                        self.config.schemes.iter().map(|s| SchemeItem {
                            id: s.id.clone(),
                            name: s.name.clone(),
                        }).collect::<Vec<_>>()
                    );
                    let selected_idx = self.config.schemes.iter()
                        .position(|s| s.id == self.config.active_scheme_id);
                    self.scheme_select.update(cx, |state, cx| {
                        state.set_items(scheme_items, _window, cx);
                        if let Some(idx) = selected_idx {
                            state.set_selected_index(Some(IndexPath::default().row(idx)), _window, cx);
                        }
                    });
                }
            }
        }

        let can_delete = self.config.schemes.len() > 1;
        let active_scheme_id = self.config.active_scheme_id.clone();

        h_flex()
            .w_full()
            .p_3()
            .gap_4()
            .items_center()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().text_sm().child(i18n::t("scheme")))
                    .child(Select::new(&self.scheme_select).w(px(150.)))
            )
            .child(
                Button::new("new-scheme")
                    .label(i18n::t("new"))
                    .compact()
                    .on_click(cx.listener(|this, _, window, cx| {
                        let id = format!("scheme-{}", this.config.schemes.len() + 1);
                        // Generate unique name
                        let mut idx = this.config.schemes.len() + 1;
                        let mut name = i18n::t_scheme_name(idx);
                        let existing_names: Vec<String> = this.config.schemes.iter().map(|s| s.name.clone()).collect();
                        while existing_names.contains(&name) {
                            idx += 1;
                            name = i18n::t_scheme_name(idx);
                        }
                        let scheme = KeybindScheme::default_dota();
                        let mut new_scheme = scheme;
                        new_scheme.id = id.clone();
                        new_scheme.name = name;
                        this.config.add_scheme(new_scheme);
                        this.config.active_scheme_id = id;
                        this.engine.lock().unwrap().set_active_scheme(&this.config.active_scheme_id);
                        let _ = this.config.save();
                        this.refresh_scheme_select(window, cx);
                        cx.notify();
                    }))
            )
            .child(
                Button::new("delete-scheme")
                    .label(i18n::t("delete"))
                    .compact()
                    .when(!can_delete, |b| b.disabled(true))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.delete_scheme(active_scheme_id.clone(), window, cx);
                    }))
            )
            .child(
                Button::new("rename-active")
                    .label(i18n::t("rename"))
                    .compact()
                    .on_click(cx.listener(|this, _, window, cx| {
                        if let Some(scheme) = this.active_scheme() {
                            this.start_rename(scheme.id.clone(), window, cx);
                        }
                    }))
            )
            // Language toggle — pushed to the right
            .child(
                div().ml_auto().child(
                    Button::new("lang-toggle")
                        .label(i18n::toggle_label())
                        .compact()
                        .on_click(cx.listener(|this, _, window, cx| {
                            i18n::toggle_lang();
                            // Update window title
                            window.set_window_title(&i18n::t_app_title());
                            cx.notify();
                        }))
                )
            )
    }

    fn render_rename_dialog(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editing = self.editing_scheme_id.clone();

        div()
            .when_some(editing, |this, _| {
                this.absolute()
                    .inset_0()
                    .bg(gpui::rgb(0x333333))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        v_flex()
                            .w_64()
                            .p_4()
                            .bg(cx.theme().background)
                            .rounded_lg()
                            .shadow_lg()
                            .gap_3()
                            .child(div().text_base().font_semibold().child(i18n::t("rename_scheme")))
                            .child(Input::new(&self.rename_input).cleanable(true))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        Button::new("cancel-rename")
                                            .label(i18n::t("cancel"))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.cancel_rename(window, cx);
                                            }))
                                    )
                                    .child(
                                        Button::new("confirm-rename")
                                            .label(i18n::t("confirm"))
                                            .primary()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.confirm_rename(window, cx);
                                            }))
                                    )
                            )
                    )
            })
    }

    fn render_mappings(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        const SLOT_LABELS: [&str; 6] = ["Num7", "Num8", "Num4", "Num5", "Num1", "Num2"];

        let Some(scheme) = self.active_scheme() else {
            return v_flex().w_full().p_4().child(i18n::t("no_active_scheme"));
        };

        let capturing = self.capturing;
        let error_msg = self.error_msg.clone();
        let pending = self.pending_skill;

        // Collect skill mappings into sorted vec for consistent rendering
        let skill_entries: Vec<(VirtualKey, VirtualKey)> = {
            let mut entries: Vec<_> = scheme.skill_mappings.iter()
                .map(|(k, v)| (*k, *v))
                .collect();
            entries.sort_by_key(|(k, _)| format!("{:?}", k));
            entries
        };

        v_flex()
            .w_full()
            .p_4()
            .gap_4()
            .when_some(error_msg, |this, err| {
                this.child(
                    div()
                        .w_full()
                        .p_2()
                        .rounded_md()
                        .bg(gpui::rgb(0xef4444))
                        .text_color(gpui::rgb(0xffffff))
                        .text_sm()
                        .child(err)
                )
            })
            .child(
                h_flex()
                    .w_full()
                    .gap_6()
                    .items_start()
                    // 技能键位
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_2()
                            .child(div().text_base().font_semibold().child(i18n::t("skill")))
                            .child(
                                Table::new()
                                    .small()
                                    .child(
                                        TableHeader::new().child(
                                            TableRow::new()
                                                .child(TableHead::new().child(i18n::t("key")))
                                                .child(TableHead::new().child(i18n::t("map_to")))
                                                .child(TableHead::new().child(""))
                                        )
                                    )
                                    .child(
                                        TableBody::new()
                                            // Existing mappings: both source and target are editable
                                            .children(
                                                skill_entries.iter().map(|(source_key, target_key)| {
                                                    let src = *source_key;
                                                    let tgt = *target_key;
                                                    let is_capturing_source = capturing == Some(CaptureTarget::SkillSource {
                                                        old_source: src,
                                                        old_target: tgt,
                                                    });
                                                    let is_capturing_target = capturing == Some(CaptureTarget::SkillTarget(src));

                                                    TableRow::new()
                                                        .child(
                                                            TableCell::new().child(
                                                                Button::new(SharedString::from(format!("skill-src-{}", Self::format_key(source_key))))
                                                                    .label(Self::format_key(source_key))
                                                                    .compact()
                                                                    .when(is_capturing_source, |b| b.primary())
                                                                    .on_click(cx.listener(move |this, _, window, cx| {
                                                                        this.start_capture(CaptureTarget::SkillSource {
                                                                            old_source: src,
                                                                            old_target: tgt,
                                                                        }, window, cx);
                                                                    }))
                                                            )
                                                        )
                                                        .child(
                                                            TableCell::new().child(
                                                                Button::new(SharedString::from(format!("skill-tgt-{}", Self::format_key(source_key))))
                                                                    .label(Self::format_key(target_key))
                                                                    .compact()
                                                                    .when(is_capturing_target, |b| b.primary())
                                                                    .on_click(cx.listener(move |this, _, window, cx| {
                                                                        this.start_capture(CaptureTarget::SkillTarget(src), window, cx);
                                                                    }))
                                                            )
                                                        )
                                                        .child(
                                                            TableCell::new().child(
                                                                Button::new(SharedString::from(format!("skill-del-{}", Self::format_key(source_key))))
                                                                    .label("×")
                                                                    .compact()
                                                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                                                        this.remove_skill_mapping(src, cx);
                                                                    }))
                                                            )
                                                        )
                                                })
                                            )
                                            // Pending new row
                                            .child(
                                                TableRow::new()
                                                    .child(
                                                        TableCell::new().child(
                                                            Button::new("new-src")
                                                                .label(match pending.0 {
                                                                    Some(k) => Self::format_key(&k),
                                                                    None => "-".into(),
                                                                })
                                                                .compact()
                                                                .when(capturing == Some(CaptureTarget::NewSkillSource), |b| b.primary())
                                                                .on_click(cx.listener(|this, _, window, cx| {
                                                                    this.start_capture(CaptureTarget::NewSkillSource, window, cx);
                                                                }))
                                                        )
                                                    )
                                                    .child(
                                                        TableCell::new().child(
                                                            Button::new("new-tgt")
                                                                .label(match pending.1 {
                                                                    Some(k) => Self::format_key(&k),
                                                                    None => "-".into(),
                                                                })
                                                                .compact()
                                                                .when(capturing == Some(CaptureTarget::NewSkillTarget), |b| b.primary())
                                                                .on_click(cx.listener(|this, _, window, cx| {
                                                                    this.start_capture(CaptureTarget::NewSkillTarget, window, cx);
                                                                }))
                                                        )
                                                    )
                                                    .child(TableCell::new())
                                            )
                                    )
                            )
                    )
                    // 物品栏键位
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_2()
                            .child(div().text_base().font_semibold().child(i18n::t("inventory")))
                            .child(
                                Table::new()
                                    .small()
                                    .child(
                                        TableHeader::new().child(
                                            TableRow::new()
                                                .child(TableHead::new().child(i18n::t("inv_slot")))
                                                .child(TableHead::new().child(i18n::t("key")))
                                        )
                                    )
                                    .child(
                                        TableBody::new().children(
                                            scheme.inventory_mappings.iter().enumerate().map(|(idx, key)| {
                                                let is_capturing_this = capturing == Some(CaptureTarget::Inventory(idx));
                                                let label = key.map(|k| Self::format_key(&k)).unwrap_or_else(|| "-".into());

                                                TableRow::new()
                                                    .child(TableCell::new().child(SLOT_LABELS[idx]))
                                                    .child(
                                                        TableCell::new().child(
                                                            Button::new(SharedString::from(format!("inv-{}", idx)))
                                                                .label(label)
                                                                .compact()
                                                                .when(is_capturing_this, |b| b.primary())
                                                                .on_click(cx.listener(move |this, _, window, cx| {
                                                                    this.start_capture(CaptureTarget::Inventory(idx), window, cx);
                                                                }))
                                                        )
                                                    )
                                            })
                                        )
                                    )
                            )
                    )
            )
            // 底部分隔线 + 屏蔽按键 + 快捷键说明
            .child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .px_4()
                    .pb_4()
                    .child(
                        div()
                            .w_full()
                            .h(px(1.))
                            .bg(cx.theme().border)
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_6()
                            .items_center()
                            .child(
                                Checkbox::new("block-win")
                                    .label(i18n::t("block_win"))
                                    .checked(scheme.blocked_keys.contains(&VirtualKey::LWin))
                                    .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                                        if let Some(scheme) = this.config.active_scheme_mut() {
                                            if *checked {
                                                if !scheme.blocked_keys.contains(&VirtualKey::LWin) {
                                                    scheme.blocked_keys.push(VirtualKey::LWin);
                                                }
                                                if !scheme.blocked_keys.contains(&VirtualKey::RWin) {
                                                    scheme.blocked_keys.push(VirtualKey::RWin);
                                                }
                                            } else {
                                                scheme.blocked_keys.retain(|k| *k != VirtualKey::LWin && *k != VirtualKey::RWin);
                                            }
                                        }
                                        this.save_and_sync_engine();
                                        let _ = this.config.save();
                                        cx.notify();
                                    }))
                            )
                            .child(
                                h_flex()
                                    .gap_4()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(i18n::t("f11_switch"))
                                    .child(i18n::t("f12_pause"))
                            )
                    )
            )
    }
}

impl Render for KeymanApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let capturing = self.capturing;
        let focus_handle = self.focus_handle.clone();

        v_flex()
            .size_full()
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
                // Skip key handling if rename dialog is open (TextInput handles it)
                if this.editing_scheme_id.is_some() {
                    return;
                }

                let Some(target) = capturing else { return };

                let key = event.keystroke.key.to_lowercase();

                if key == "escape" {
                    this.cancel_capture(cx);
                    return;
                }

                // Delete/Backspace clears the mapping
                if key == "delete" || key == "backspace" {
                    match target {
                        CaptureTarget::SkillSource { old_source, .. } |
                        CaptureTarget::SkillTarget(old_source) => {
                            this.remove_skill_mapping(old_source, cx);
                        }
                        CaptureTarget::Inventory(idx) => {
                            if let Some(scheme) = this.config.active_scheme_mut() {
                                scheme.inventory_mappings[idx] = None;
                            }
                            this.save_and_sync_engine();
                            cx.notify();
                        }
                        _ => {}
                    }
                    this.capturing = None;
                    cx.notify();
                    return;
                }

                let Some(vk) = Self::parse_key_from_string(&key) else {
                    this.error_msg = Some(i18n::t_unsupported_key(&key));
                    cx.notify();
                    return;
                };

                this.apply_key(vk, cx);
            }))
            .child(self.render_toolbar(window, cx))
            .child(
                div()
                    .id("scroll-container")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scroll()
                    .child(self.render_mappings(window, cx))
            )
            .child(self.render_rename_dialog(window, cx))
    }
}