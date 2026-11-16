use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use mae_adapter_rust_slint::RustSlintTargetAdapter;
use mae_domain::{
    ArtifactRevision, ComponentNode, EffectBinding, InteractionBinding, LayoutConstraint,
    MigrationStrategy, MigrationTask, PropDefinition, RouteDefinition, SemanticUiModel, StateField,
    StyleToken, TargetAdapterPort, TargetPlatform, TaskId,
};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("mae_seam4_slint_{prefix}_{pid}_{id}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn sample_windows_semantic_model() -> SemanticUiModel {
    let mut model = SemanticUiModel::new(TargetPlatform::WindowsDesktop);
    model.components.push(
        ComponentNode::new("App", "App", "src/App.tsx")
            .with_child("Header")
            .with_child("TaskCard"),
    );
    model.components.push(
        ComponentNode::new("Header", "Header", "src/components/Header.tsx")
            .with_prop(PropDefinition::new("title", "string", true)),
    );
    model.components.push(
        ComponentNode::new("TaskCard", "TaskCard", "src/components/TaskCard.tsx")
            .with_prop(PropDefinition::new("label", "string", true)),
    );

    model
        .state_fields
        .push(StateField::new("completedCount", "number", "0", "useState"));
    model
        .state_fields
        .push(StateField::new("filterText", "string", "all", "useState"));

    model.interactions.push(
        InteractionBinding::new("TaskCard", "onClick", "incrementCompleted")
            .with_transition("setCompletedCount"),
    );

    model
        .effects
        .push(EffectBinding::new("loadInitialTasks", "mount", true));

    model
        .routes
        .push(RouteDefinition::new("/", "DashboardView"));
    model
        .routes
        .push(RouteDefinition::new("/settings", "SettingsView"));

    model.layout_constraints.push(
        LayoutConstraint::new("column", 12.0, 16.0)
            .with_responsive_rule("@media (min-width: 1024px)"),
    );

    model
        .style_tokens
        .push(StyleToken::new("--color-primary", "#2563eb"));
    model
        .style_tokens
        .push(StyleToken::new("--surface-bg", "#0f172a"));

    model
}

#[test]
fn scaffold_target_generates_canonical_windows_rust_slint_layout() {
    let target_dir = unique_temp_dir("scaffold");
    let model = sample_windows_semantic_model();
    let adapter = RustSlintTargetAdapter::with_crate_name("DesktopTaskTracker");

    let summary = adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target");

    assert_eq!(summary.target_crate_name, "desktoptasktracker");
    assert!(summary.has_windows_subsystem);
    assert!(summary.files_generated.len() >= 12);

    assert!(target_dir.join("Cargo.toml").exists());
    assert!(target_dir.join("build.rs").exists());
    assert!(target_dir.join("src/main.rs").exists());
    assert!(target_dir.join("src/app/mod.rs").exists());
    assert!(target_dir.join("src/app/state.rs").exists());
    assert!(target_dir.join("src/app/events.rs").exists());
    assert!(target_dir.join("src/app/reducer.rs").exists());
    assert!(target_dir.join("src/app/effects.rs").exists());
    assert!(target_dir.join("src/app/navigation.rs").exists());
    assert!(target_dir.join("ui/app.slint").exists());
    assert!(target_dir.join("ui/theme/tokens.slint").exists());
    assert!(target_dir.join("ui/components/header.slint").exists());
    assert!(target_dir.join("ui/components/taskcard.slint").exists());
    assert!(target_dir.join("tests/target_behavior_tests.rs").exists());

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn scaffold_generates_exact_avatar_radius_and_bottom_pinned_chat_stream() {
    let target_dir = unique_temp_dir("chat_primitives");
    let mut model = sample_windows_semantic_model();
    model
        .components
        .push(ComponentNode::new("Avatar", "Avatar", "src/Avatar.tsx"));
    model.components.push(ComponentNode::new(
        "ChatMessageList",
        "ChatMessageList",
        "src/ChatMessageList.tsx",
    ));
    model.components.push(ComponentNode::new(
        "VideoAttachment",
        "VideoAttachment",
        "src/VideoAttachment.tsx",
    ));
    RustSlintTargetAdapter::default()
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target");

    let avatar = fs::read_to_string(target_dir.join("ui/components/avatar.slint")).unwrap();
    assert!(avatar.contains("border-radius: self.height / 2;"));
    let chat = fs::read_to_string(target_dir.join("ui/components/chatmessagelist.slint")).unwrap();
    assert!(chat.contains("Flickable {"));
    assert!(chat.contains("changed content-height => { self.content-y = min(0px, self.height - self.content-height); }"));
    let video = fs::read_to_string(target_dir.join("ui/components/videoattachment.slint")).unwrap();
    assert!(video.contains("interval: 33ms;"));
    assert!(video.contains("in property <[image]> frames;"));
    assert!(video.contains("root.frames[root.frame-index]"));
    assert!(!video.contains("callback frame-requested(int);"));
    assert!(video.contains("callback launch-external-player();"));
    let media = fs::read_to_string(target_dir.join("src/app/media.rs")).unwrap();
    assert!(media.contains("pub fn extract_video_frames"));
    assert!(media.contains("fps=30"));
    assert!(media.contains("pub fn open_external_player"));
    let output = std::process::Command::new("cargo")
        .args(["test", "--no-run"])
        .current_dir(&target_dir)
        .output()
        .expect("compile generated video target");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn modular_monolith_topology_scaffolds_a_testable_cargo_workspace() {
    let target_dir = unique_temp_dir("modular_monolith");
    let adapter = RustSlintTargetAdapter::default()
        .with_topology(mae_adapter_rust_slint::WorkspaceTopology::ModularMonolith);
    adapter
        .scaffold_target(
            &sample_windows_semantic_model(),
            target_dir.to_str().unwrap(),
        )
        .unwrap();
    let appearance = fs::read_to_string(target_dir.join("appearance.toml")).unwrap();
    assert!(appearance.contains("bubble_radius_px = 16"));
    assert!(appearance.contains("primary_color = \"#2563eb\""));
    for setting in [
        "control_radius_px = 12",
        "icon_size_px = 20",
        "font_size_px = 14",
        "motion_duration_ms = 180",
        "app_title_en = \"Workspace\"",
        "app_title_ru = \"Рабочее пространство\"",
    ] {
        assert!(
            appearance.contains(setting),
            "missing config setting {setting}"
        );
    }
    let generated_main =
        fs::read_to_string(target_dir.join("crates/migrated-slint-desktop-app/src/main.rs"))
            .unwrap();
    assert!(generated_main.contains("MAE_CONFIG"));
    assert!(generated_main.contains("AppearanceSettings::from_toml"));
    assert!(generated_main.contains("MAE_VISUAL_ROUTE"));
    assert!(generated_main.contains("MAE_VISUAL_OUTPUT"));
    assert!(generated_main.contains("take_snapshot"));
    let app_source =
        fs::read_to_string(target_dir.join("crates/migrated-slint-application/src/lib.rs"))
            .unwrap();
    assert!(app_source.contains("pub enum AppEvent"));
    assert!(app_source.contains("pub fn reduce"));
    assert!(app_source.contains("pub fn seed_mock_workspace"));
    let desktop_manifest =
        fs::read_to_string(target_dir.join("crates/migrated-slint-desktop-app/Cargo.toml"))
            .unwrap();
    assert!(desktop_manifest.contains("slint-build"));
    assert!(desktop_manifest.contains("headless-test"));
    assert!(target_dir
        .join("crates/migrated-slint-desktop-app/build.rs")
        .exists());
    let clean_lint = std::process::Command::new("bash")
        .arg("scripts/check-slint-tokens.sh")
        .current_dir(&target_dir)
        .output()
        .expect("run generated Slint token linter");
    assert!(
        clean_lint.status.success(),
        "{}",
        String::from_utf8_lossy(&clean_lint.stderr)
    );
    let violation =
        target_dir.join("crates/migrated-slint-desktop-app/ui/components/ui/pill.slint");
    fs::write(
        &violation,
        "export component Pill inherits Rectangle { border-radius: 999px; }\n",
    )
    .unwrap();
    let bad_lint = std::process::Command::new("bash")
        .arg("scripts/check-slint-tokens.sh")
        .current_dir(&target_dir)
        .output()
        .expect("run linter with forbidden pill radius");
    assert!(
        !bad_lint.status.success(),
        "the token linter accepted a hard-coded pill radius"
    );
    fs::remove_file(violation).unwrap();

    let output = std::process::Command::new("cargo")
        .args(["test", "--workspace", "--no-default-features"])
        .current_dir(&target_dir)
        .output()
        .expect("run generated workspace tests");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let headless = std::process::Command::new("cargo")
        .args([
            "test",
            "-p",
            "migrated-slint-desktop-app",
            "--no-default-features",
            "--features",
            "headless-test",
        ])
        .current_dir(&target_dir)
        .output()
        .expect("run generated desktop headless tests");
    assert!(
        headless.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&headless.stdout),
        String::from_utf8_lossy(&headless.stderr)
    );
    let native_image = target_dir.join(".mae-visual/native/settings.ppm");
    fs::create_dir_all(native_image.parent().unwrap()).unwrap();
    let manifest = target_dir.join("crates/migrated-slint-desktop-app/Cargo.toml");
    let capture = std::process::Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--no-default-features",
            "--features",
            "headless-test",
        ])
        .current_dir(&target_dir)
        .env("MAE_VISUAL_ROUTE", "/settings")
        .env("MAE_VISUAL_OUTPUT", &native_image)
        .output()
        .expect("capture a generated native Slint route");
    assert!(
        capture.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&capture.stdout),
        String::from_utf8_lossy(&capture.stderr)
    );
    assert!(fs::read(&native_image)
        .unwrap()
        .starts_with(b"P6\n1280 800\n255\n"));
    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn scaffold_target_enforces_windows_gui_subsystem_attribute_in_main_rs() {
    let target_dir = unique_temp_dir("win_subsys");
    let model = sample_windows_semantic_model();
    let adapter = RustSlintTargetAdapter::default();

    adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target");

    let main_rs = fs::read_to_string(target_dir.join("src/main.rs")).expect("read main.rs");
    assert!(
        main_rs.contains("#![cfg_attr(target_os = \"windows\", windows_subsystem = \"windows\")]"),
        "src/main.rs must include Windows GUI subsystem attribute for WindowsDesktop target"
    );

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn scaffold_target_enforces_logic_ui_separation_between_slint_and_rust_reducer() {
    let target_dir = unique_temp_dir("logic_ui_sep");
    let model = sample_windows_semantic_model();
    let adapter = RustSlintTargetAdapter::default();

    adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target");

    let app_slint = fs::read_to_string(target_dir.join("ui/app.slint")).expect("read app.slint");
    let reducer_rs =
        fs::read_to_string(target_dir.join("src/app/reducer.rs")).expect("read reducer.rs");
    let events_rs =
        fs::read_to_string(target_dir.join("src/app/events.rs")).expect("read events.rs");

    // Declarative Slint entrypoint exposes properties and callbacks
    assert!(app_slint.contains("export component MainWindow inherits Window"));
    assert!(app_slint.contains("in-out property <string> state-completedCount;"));
    assert!(app_slint.contains("callback incrementCompleted();"));

    // Business state mutation lives in pure Rust reducer & events
    assert!(events_rs.contains("IncrementCompleted"));
    assert!(reducer_rs.contains("pub fn reduce(state: &mut AppState, event: AppEvent)"));
    assert!(reducer_rs.contains("AppEvent::SubmitItem"));
    assert!(reducer_rs.contains("AppEvent::Navigate"));

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn emit_target_code_advances_artifact_revision_monotonically() {
    let target_dir = unique_temp_dir("revision");
    let model = sample_windows_semantic_model();
    let adapter = RustSlintTargetAdapter::default();

    adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold");

    let task = MigrationTask::new(
        TaskId::new("task-1"),
        "Migrate TaskCard component and state",
        MigrationStrategy::DirectNativeMapping,
    );

    let rev2 = adapter
        .emit_target_code(&task, &model, target_dir.to_str().unwrap())
        .expect("emit 1");
    let rev3 = adapter
        .emit_target_code(&task, &model, target_dir.to_str().unwrap())
        .expect("emit 2");

    assert_eq!(rev2, ArtifactRevision(2));
    assert_eq!(rev3, ArtifactRevision(3));

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn scaffold_target_generates_ubuntu_desktop_entry_and_wayland_x11_backend() {
    let target_dir = unique_temp_dir("ubuntu_scaffold");
    let mut model = sample_windows_semantic_model();
    model.platform = TargetPlatform::UbuntuDesktop;

    let adapter = RustSlintTargetAdapter::with_crate_name("ubuntu-task-tracker");
    let summary = adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold ubuntu target");

    assert_eq!(summary.target_crate_name, "ubuntu-task-tracker");
    assert!(!summary.has_windows_subsystem);

    let cargo_toml = fs::read_to_string(target_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo_toml.contains("target_platform = \"UBUNTU_DESKTOP\""));
    assert!(cargo_toml.contains("window_backend = \"wayland-x11-winit\""));
    assert!(cargo_toml.contains("desktop_entry = \"packaging/ubuntu-task-tracker.desktop\""));

    let desktop_path = target_dir.join("packaging/ubuntu-task-tracker.desktop");
    assert!(desktop_path.exists());
    let desktop_entry = fs::read_to_string(&desktop_path).expect("read .desktop");
    assert!(desktop_entry.contains("[Desktop Entry]"));
    assert!(desktop_entry.contains("Type=Application"));
    assert!(desktop_entry.contains("Exec=ubuntu-task-tracker"));

    assert!(target_dir.join("packaging/ubuntu-deps.sh").exists());

    let main_rs = fs::read_to_string(target_dir.join("src/main.rs")).expect("read main.rs");
    assert!(main_rs.contains("LINUX_WINDOW_BACKEND"));

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn scaffold_target_preserves_roundness_font_size_scale_and_child_component_composition() {
    let target_dir = unique_temp_dir("roundness_scale");
    let mut model = sample_windows_semantic_model();
    model.layout_constraints[0] = LayoutConstraint::new("column", 12.0, 16.0)
        .with_border_radius(8.0)
        .with_font_size(14.0)
        .with_scale_factor(1.0);

    let adapter = RustSlintTargetAdapter::default();
    adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target");

    let tokens_slint =
        fs::read_to_string(target_dir.join("ui/theme/tokens.slint")).expect("read tokens.slint");
    assert!(tokens_slint.contains("out property <length> border-radius: 8px;"));
    assert!(tokens_slint.contains("out property <length> font-size: 14px;"));
    assert!(tokens_slint.contains("out property <float> scale-factor: 1.00;"));

    let header_slint = fs::read_to_string(target_dir.join("ui/components/header.slint"))
        .expect("read header.slint");
    assert!(header_slint.contains("border-radius: ThemeTokens.border-radius;"));
    assert!(header_slint.contains("font-size: ThemeTokens.font-size;"));

    let app_slint = fs::read_to_string(target_dir.join("ui/app.slint")).expect("read app.slint");
    assert!(app_slint.contains("default-font-size: ThemeTokens.font-size;"));
    assert!(app_slint.contains("border-radius: ThemeTokens.border-radius;"));
    assert!(app_slint.contains("import { Header } from \"components/header.slint\";"));
    assert!(app_slint.contains("import { TaskCard } from \"components/taskcard.slint\";"));

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn scaffold_target_generates_pure_rust_api_client_for_shared_mock_backend() {
    let target_dir = unique_temp_dir("api_client_gen");
    let mut model = sample_windows_semantic_model();
    model.add_api_endpoint(mae_domain::ApiEndpointBinding::new(
        "GET",
        "http://127.0.0.1:4010/api/state",
        "src/App.tsx",
    ));
    model.add_api_endpoint(mae_domain::ApiEndpointBinding::new(
        "POST",
        "http://127.0.0.1:4010/api/tasks",
        "src/App.tsx",
    ));

    let adapter = RustSlintTargetAdapter::default();
    adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target with api_client");

    let api_client_rs = fs::read_to_string(target_dir.join("src/app/api_client.rs"))
        .expect("read src/app/api_client.rs");
    assert!(api_client_rs.contains("pub const BACKEND_ADDR: &str = \"127.0.0.1:4010\";"));
    assert!(api_client_rs.contains("(\"GET\", \"http://127.0.0.1:4010/api/state\")"));
    assert!(api_client_rs.contains("(\"POST\", \"http://127.0.0.1:4010/api/tasks\")"));
    assert!(api_client_rs.contains("pub fn fetch_backend_state()"));
    assert!(api_client_rs.contains("pub fn post_create_task_with_priority("));
    assert!(api_client_rs.contains("pub fn post_complete_all_tasks()"));
    assert!(api_client_rs.contains("pub fn post_move_kanban_card("));
    assert!(api_client_rs.contains("pub fn post_update_time_range("));
    assert!(api_client_rs.contains("pub fn post_update_theme_mode("));

    let tokens_slint =
        fs::read_to_string(target_dir.join("ui/theme/tokens.slint")).expect("read tokens.slint");
    assert!(tokens_slint.contains("out property <bool> responsive-wrap: true;"));
    assert!(tokens_slint.contains("out property <length> max-content-width: 1240px;"));

    let effects_rs =
        fs::read_to_string(target_dir.join("src/app/effects.rs")).expect("read src/app/effects.rs");
    assert!(effects_rs.contains("pub fn sync_from_mock_backend()"));

    let _ = fs::remove_dir_all(&target_dir);
}

#[test]
fn test_scaffold_target_generates_responsive_controls_options_and_callbacks() {
    let target_dir = unique_temp_dir("responsive_controls");
    let mut model = sample_windows_semantic_model();

    let mut comp =
        mae_domain::ComponentNode::new("SettingsView", "SettingsView", "src/SettingsView.tsx");
    comp.elements.push(
        mae_domain::UiElementNode::new(mae_domain::UiElementKind::Button, "Complete All Visible")
            .with_variant("complete-all"),
    );
    comp.elements.push(
        mae_domain::UiElementNode::new(mae_domain::UiElementKind::SelectDropdown, "Theme Select")
            .with_variant("theme-selector")
            .with_options(vec!["dark-slate".into(), "light-fluent".into()]),
    );
    comp.elements.push(mae_domain::UiElementNode::new(
        mae_domain::UiElementKind::Checkbox,
        "Compact Mode",
    ));
    model.add_component(comp);

    let adapter = RustSlintTargetAdapter::default();
    adapter
        .scaffold_target(&model, target_dir.to_str().unwrap())
        .expect("scaffold target");

    let app_slint = fs::read_to_string(target_dir.join("ui/app.slint")).expect("read app.slint");
    assert!(app_slint.contains("min-width: 640px;"));
    assert!(app_slint.contains("callback complete-all-items();"));
    assert!(app_slint.contains("callback set-time-range(string);"));
    assert!(app_slint.contains("callback set-theme-mode(string);"));
    assert!(
        app_slint.contains("in-out property <bool> responsive-wrap: ThemeTokens.responsive-wrap;")
    );

    let settings_slint = fs::read_to_string(target_dir.join("ui/components/settingsview.slint"))
        .expect("read settingsview.slint");
    assert!(settings_slint.contains("#4f46e5")); // complete-all button background
    assert!(settings_slint.contains("root.complete-all-triggered();"));
    assert!(settings_slint.contains("dark-slate"));
    assert!(settings_slint.contains("light-fluent"));
    assert!(settings_slint.contains("alignment: start;")); // start-aligned checkbox

    let _ = fs::remove_dir_all(&target_dir);
}
