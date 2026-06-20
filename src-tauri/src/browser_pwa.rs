use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const PWA_SCHEME: &str = "wintrack-pwa://";
const MAX_SHORTCUT_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserKind {
    Chrome,
    Edge,
    Brave,
}

#[derive(Debug, Clone)]
pub struct BrowserPwa {
    pub display_name: String,
    pub stable_identifier: String,
}

#[derive(Debug, Clone)]
struct PwaCandidate {
    display_name: String,
    app_id: String,
    profile: Option<String>,
    icon_path: Option<PathBuf>,
    app_user_model_id: Option<String>,
}

impl BrowserKind {
    pub fn from_process_name(process_name: &str) -> Option<Self> {
        match process_name.to_lowercase().as_str() {
            "chrome.exe" => Some(Self::Chrome),
            "msedge.exe" => Some(Self::Edge),
            "brave.exe" => Some(Self::Brave),
            _ => None,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Edge => "edge",
            Self::Brave => "brave",
        }
    }

    fn process_name(self) -> &'static str {
        match self {
            Self::Chrome => "chrome.exe",
            Self::Edge => "msedge.exe",
            Self::Brave => "brave.exe",
        }
    }

    fn user_data_root(self) -> Option<PathBuf> {
        let local_app_data = std::env::var_os("LOCALAPPDATA")?;
        let root = match self {
            Self::Chrome => PathBuf::from(local_app_data).join(r"Google\Chrome\User Data"),
            Self::Edge => PathBuf::from(local_app_data).join(r"Microsoft\Edge\User Data"),
            Self::Brave => {
                PathBuf::from(local_app_data).join(r"BraveSoftware\Brave-Browser\User Data")
            }
        };
        Some(root)
    }

    fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "chrome" => Some(Self::Chrome),
            "edge" => Some(Self::Edge),
            "brave" => Some(Self::Brave),
            _ => None,
        }
    }
}

pub fn resolve_browser_pwa(
    browser: BrowserKind,
    browser_executable_path: &str,
    app_user_model_id: Option<&str>,
    window_title: &str,
) -> Option<BrowserPwa> {
    let candidates = collect_start_menu_candidates(browser);

    if let Some(app_user_model_id) = app_user_model_id {
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| {
                candidate
                    .app_user_model_id
                    .as_deref()
                    .map(|value| value.eq_ignore_ascii_case(app_user_model_id))
                    .unwrap_or(false)
            })
            .cloned()
        {
            return Some(finalize_candidate(
                browser,
                browser_executable_path,
                candidate,
            ));
        }

        if let Some(app_id) = extract_app_id(app_user_model_id) {
            if let Some(candidate) = candidates
                .iter()
                .find(|candidate| candidate.app_id.eq_ignore_ascii_case(&app_id))
                .cloned()
            {
                return Some(finalize_candidate(
                    browser,
                    browser_executable_path,
                    candidate,
                ));
            }

            if let Some(registration) =
                resolve_from_app_user_model_id(browser, Some(app_user_model_id))
            {
                return Some(finalize_candidate(
                    browser,
                    browser_executable_path,
                    registration,
                ));
            }
        }
    }

    if let Some(candidate) = candidates
        .iter()
        .find(|candidate| title_matches_pwa(window_title, &candidate.display_name))
        .cloned()
    {
        return Some(finalize_candidate(
            browser,
            browser_executable_path,
            candidate,
        ));
    }

    let mut seen_ids: HashSet<String> = candidates
        .iter()
        .map(|candidate| candidate.app_id.to_lowercase())
        .collect();
    for registration in collect_browser_registrations(browser) {
        if !seen_ids.insert(registration.app_id.to_lowercase()) {
            continue;
        }
        if title_matches_pwa(window_title, &registration.display_name) {
            return Some(finalize_candidate(
                browser,
                browser_executable_path,
                registration,
            ));
        }
    }

    None
}

pub fn is_pwa_identifier(identifier: &str) -> bool {
    parse_pwa_identifier(identifier).is_some()
}

pub fn display_name_for_identifier(identifier: &str) -> Option<String> {
    let parsed = parse_pwa_identifier(identifier)?;
    find_pwa_metadata_for_profile(parsed.browser, &parsed.profile, &parsed.app_id)
        .map(|candidate| candidate.display_name)
        .filter(|name| !name.trim().is_empty())
}

pub fn icon_path_for_identifier(identifier: &str) -> Option<PathBuf> {
    let parsed = parse_pwa_identifier(identifier)?;
    find_pwa_metadata_for_profile(parsed.browser, &parsed.profile, &parsed.app_id)?.icon_path
}

pub fn encode_png_icon(path: &Path) -> Option<String> {
    let is_png = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("png"))
        .unwrap_or(false);
    if !is_png {
        return None;
    }
    fs::read(path)
        .ok()
        .map(|bytes| general_purpose::STANDARD.encode(bytes))
}

fn resolve_from_app_user_model_id(
    browser: BrowserKind,
    app_user_model_id: Option<&str>,
) -> Option<PwaCandidate> {
    let app_user_model_id = app_user_model_id?;
    let app_id = extract_app_id(app_user_model_id)?;
    let mut candidate =
        find_browser_registration(browser, &app_id).unwrap_or_else(|| PwaCandidate {
            display_name: app_id.clone(),
            app_id: app_id.clone(),
            profile: profile_from_app_user_model_id(app_user_model_id, &app_id),
            icon_path: None,
            app_user_model_id: Some(app_user_model_id.to_string()),
        });

    if candidate.profile.is_none() {
        candidate.profile = profile_from_app_user_model_id(app_user_model_id, &app_id);
    }
    candidate.app_user_model_id = Some(app_user_model_id.to_string());
    Some(candidate)
}

fn finalize_candidate(
    browser: BrowserKind,
    _browser_executable_path: &str,
    mut candidate: PwaCandidate,
) -> BrowserPwa {
    let profile = candidate
        .profile
        .take()
        .unwrap_or_else(|| "Default".to_string());
    BrowserPwa {
        display_name: clean_display_name(&candidate.display_name),
        stable_identifier: make_pwa_identifier(browser, &profile, &candidate.app_id),
    }
}

fn make_pwa_identifier(browser: BrowserKind, profile: &str, app_id: &str) -> String {
    format!(
        "{}{}/{}/{}",
        PWA_SCHEME,
        browser.slug(),
        urlencoding::encode(profile),
        app_id.to_lowercase()
    )
}

struct ParsedPwaIdentifier {
    browser: BrowserKind,
    profile: String,
    app_id: String,
}

fn parse_pwa_identifier(identifier: &str) -> Option<ParsedPwaIdentifier> {
    let rest = identifier.strip_prefix(PWA_SCHEME)?;
    let mut parts = rest.split('/');
    let browser = BrowserKind::from_slug(parts.next()?)?;
    let profile = urlencoding::decode(parts.next()?).ok()?.to_string();
    let app_id = parts.next()?.split('?').next()?.to_lowercase();
    if !is_chromium_app_id(&app_id) {
        return None;
    }
    Some(ParsedPwaIdentifier {
        browser,
        profile,
        app_id,
    })
}

fn collect_start_menu_candidates(browser: BrowserKind) -> Vec<PwaCandidate> {
    let mut candidates = Vec::new();
    let mut roots = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        roots.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    roots.push(PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs",
    ));

    for root in roots {
        collect_shortcuts_from_dir(browser, &root, &mut candidates, 0);
    }
    candidates
}

fn collect_shortcuts_from_dir(
    browser: BrowserKind,
    dir: &Path,
    candidates: &mut Vec<PwaCandidate>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_shortcuts_from_dir(browser, &path, candidates, depth + 1);
            continue;
        }

        let is_lnk = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("lnk"))
            .unwrap_or(false);
        if !is_lnk {
            continue;
        }

        if let Some(candidate) = parse_shortcut_candidate(browser, &path) {
            candidates.push(candidate);
        }
    }
}

fn parse_shortcut_candidate(browser: BrowserKind, path: &Path) -> Option<PwaCandidate> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > MAX_SHORTCUT_BYTES {
        return None;
    }

    let bytes = fs::read(path).ok()?;
    let strings = extract_shortcut_strings(&bytes);
    let combined = strings.join("\n");
    let lower = combined.to_lowercase();

    if !lower.contains(browser.process_name()) && !lower.contains(browser.slug()) {
        return None;
    }

    let app_id = extract_app_id(&combined)?;
    let profile = extract_flag_value(&combined, "--profile-directory=");
    let app_user_model_id = strings
        .iter()
        .find(|value| value.to_lowercase().contains("appusermodel"))
        .cloned()
        .or_else(|| {
            strings
                .iter()
                .find(|value| value.contains(&app_id) && value.to_lowercase().contains("userdata"))
                .cloned()
        });
    let icon_path = strings
        .iter()
        .filter_map(|value| icon_path_from_string(value))
        .find(|candidate| candidate.exists());

    Some(PwaCandidate {
        display_name: path.file_stem()?.to_string_lossy().to_string(),
        app_id,
        profile,
        icon_path,
        app_user_model_id,
    })
}

fn collect_browser_registrations(browser: BrowserKind) -> Vec<PwaCandidate> {
    let Some(root) = browser.user_data_root() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let profile_path = entry.path();
        if !profile_path.is_dir() || !looks_like_chromium_profile(&profile_path) {
            continue;
        }

        let profile = entry.file_name().to_string_lossy().to_string();
        let web_apps = profile_path.join("Web Applications");
        collect_manifest_candidates(&web_apps, &profile, &mut candidates, 0);
    }
    candidates
}

fn find_browser_registration(browser: BrowserKind, app_id: &str) -> Option<PwaCandidate> {
    collect_browser_registrations(browser)
        .into_iter()
        .find(|candidate| candidate.app_id.eq_ignore_ascii_case(app_id))
}

fn find_pwa_metadata_for_profile(
    browser: BrowserKind,
    profile: &str,
    app_id: &str,
) -> Option<PwaCandidate> {
    let profile_matches = |candidate: &PwaCandidate| {
        candidate
            .profile
            .as_deref()
            .map(|candidate_profile| candidate_profile.eq_ignore_ascii_case(profile))
            .unwrap_or(false)
    };

    let mut candidates = collect_start_menu_candidates(browser);
    if let Some(candidate) = candidates
        .iter()
        .find(|candidate| {
            candidate.app_id.eq_ignore_ascii_case(app_id) && profile_matches(candidate)
        })
        .cloned()
    {
        return Some(candidate);
    }
    if let Some(candidate) = candidates
        .drain(..)
        .find(|candidate| candidate.app_id.eq_ignore_ascii_case(app_id))
    {
        return Some(candidate);
    }

    let registrations = collect_browser_registrations(browser);
    if let Some(candidate) = registrations
        .iter()
        .find(|candidate| {
            candidate.app_id.eq_ignore_ascii_case(app_id) && profile_matches(candidate)
        })
        .cloned()
    {
        return Some(candidate);
    }
    registrations
        .into_iter()
        .find(|candidate| candidate.app_id.eq_ignore_ascii_case(app_id))
}

fn collect_manifest_candidates(
    dir: &Path,
    profile: &str,
    candidates: &mut Vec<PwaCandidate>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_manifest_candidates(&path, profile, candidates, depth + 1);
            continue;
        }

        let is_json = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }

        let Some(candidate) = parse_manifest_candidate(&path, profile) else {
            continue;
        };
        candidates.push(candidate);
    }
}

fn parse_manifest_candidate(path: &Path, profile: &str) -> Option<PwaCandidate> {
    let text = fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&text).ok()?;
    let app_id = json
        .get("app_id")
        .and_then(Value::as_str)
        .or_else(|| json.get("id").and_then(Value::as_str))
        .and_then(extract_app_id)
        .or_else(|| app_id_from_path(path))?;

    let display_name = json
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| json.get("short_name").and_then(Value::as_str))
        .map(clean_display_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| app_id.clone());

    let icon_path = best_manifest_icon_path(path, &json).or_else(|| icon_path_near_manifest(path));

    Some(PwaCandidate {
        display_name,
        app_id,
        profile: Some(profile.to_string()),
        icon_path,
        app_user_model_id: None,
    })
}

fn best_manifest_icon_path(manifest_path: &Path, json: &Value) -> Option<PathBuf> {
    let icons = json.get("icons")?.as_array()?;
    let manifest_dir = manifest_path.parent()?;

    icons
        .iter()
        .filter_map(|icon| {
            let src = icon.get("src")?.as_str()?;
            let path = manifest_dir.join(src.replace('/', "\\"));
            if !path.exists() {
                return None;
            }
            let size = icon
                .get("sizes")
                .and_then(Value::as_str)
                .and_then(largest_size_hint)
                .unwrap_or(0);
            Some((size, path))
        })
        .max_by_key(|(size, _)| *size)
        .map(|(_, path)| path)
}

fn icon_path_near_manifest(manifest_path: &Path) -> Option<PathBuf> {
    let root = manifest_path.parent()?;
    let mut best: Option<(u64, PathBuf)> = None;
    collect_icon_files(root, &mut best, 0);
    best.map(|(_, path)| path)
}

fn collect_icon_files(dir: &Path, best: &mut Option<(u64, PathBuf)>, depth: usize) {
    if depth > 4 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_icon_files(&path, best, depth + 1);
            continue;
        }

        let is_icon = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "png" | "ico"))
            .unwrap_or(false);
        if !is_icon {
            continue;
        }

        let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if best
            .as_ref()
            .map(|(best_size, _)| size > *best_size)
            .unwrap_or(true)
        {
            *best = Some((size, path));
        }
    }
}

fn looks_like_chromium_profile(path: &Path) -> bool {
    path.join("Preferences").exists()
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "Default" || name.starts_with("Profile "))
            .unwrap_or(false)
}

fn extract_shortcut_strings(bytes: &[u8]) -> Vec<String> {
    let mut strings = extract_ascii_strings(bytes);
    strings.extend(extract_utf16le_strings(bytes));
    strings
}

fn extract_ascii_strings(bytes: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = Vec::new();

    for byte in bytes {
        if (0x20..=0x7e).contains(byte) {
            current.push(*byte);
        } else if current.len() >= 4 {
            if let Ok(value) = String::from_utf8(current.clone()) {
                result.push(value);
            }
            current.clear();
        } else {
            current.clear();
        }
    }

    if current.len() >= 4 {
        if let Ok(value) = String::from_utf8(current) {
            result.push(value);
        }
    }

    result
}

fn extract_utf16le_strings(bytes: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = Vec::new();

    for chunk in bytes.chunks_exact(2) {
        let value = u16::from_le_bytes([chunk[0], chunk[1]]);
        if (0x20..=0x7e).contains(&value) {
            current.push(value);
        } else if current.len() >= 4 {
            result.push(String::from_utf16_lossy(&current));
            current.clear();
        } else {
            current.clear();
        }
    }

    if current.len() >= 4 {
        result.push(String::from_utf16_lossy(&current));
    }

    result
}

fn extract_app_id(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    if let Some(value) = extract_flag_value(&lower, "--app-id=") {
        return Some(value);
    }

    for token in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
        if is_chromium_app_id(token) {
            return Some(token.to_string());
        }
    }

    None
}

fn extract_flag_value(text: &str, flag: &str) -> Option<String> {
    let start = text.find(flag)? + flag.len();
    let rest = &text[start..];
    let value = if let Some(stripped) = rest.strip_prefix('"') {
        stripped.split('"').next()?
    } else {
        rest.split(|c: char| c.is_whitespace() || c == '\0' || c == '"' || c == '\'')
            .next()?
    };
    let cleaned = value.trim_matches(|c| c == '"' || c == '\'' || c == '\0');
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn profile_from_app_user_model_id(app_user_model_id: &str, app_id: &str) -> Option<String> {
    let lower = app_user_model_id.to_lowercase();
    let app_pos = lower.find(&app_id.to_lowercase())?;
    let before = &app_user_model_id[..app_pos].trim_end_matches(['.', '_']);
    let marker = ".UserData.";
    if let Some(marker_pos) = before.find(marker) {
        let profile = before[marker_pos + marker.len()..].trim_matches('.');
        if !profile.is_empty() {
            return Some(profile.to_string());
        }
    }
    None
}

fn app_id_from_path(path: &Path) -> Option<String> {
    for component in path.components().rev() {
        let text = component.as_os_str().to_string_lossy().to_lowercase();
        let text = text.strip_prefix("_crx_").unwrap_or(&text);
        if is_chromium_app_id(text) {
            return Some(text.to_string());
        }
    }
    None
}

fn icon_path_from_string(value: &str) -> Option<PathBuf> {
    let lower = value.to_lowercase();
    let index = lower.find(".ico").or_else(|| lower.find(".png"))?;
    let end = index + 4;
    let before = &value[..end];
    let start = before.rfind(['"', '\0']).map(|pos| pos + 1).unwrap_or(0);
    let candidate = before[start..].trim();
    if candidate.is_empty() {
        None
    } else {
        Some(PathBuf::from(candidate))
    }
}

fn largest_size_hint(value: &str) -> Option<u32> {
    value
        .split_whitespace()
        .filter_map(|size| size.split_once('x'))
        .filter_map(|(w, h)| Some(w.parse::<u32>().ok()?.max(h.parse::<u32>().ok()?)))
        .max()
}

fn is_chromium_app_id(value: &str) -> bool {
    value.len() == 32 && value.chars().all(|c| matches!(c, 'a'..='p'))
}

fn title_matches_pwa(window_title: &str, display_name: &str) -> bool {
    let title = normalize_title(window_title);
    let display = normalize_title(display_name);

    if title.is_empty() || display.len() < 3 {
        return false;
    }

    title == display
        || title.starts_with(&(display.clone() + " - "))
        || title.ends_with(&(" - ".to_string() + &display))
}

fn normalize_title(value: &str) -> String {
    let mut cleaned = value.trim().to_lowercase();
    for suffix in [
        " - google chrome",
        " - microsoft edge",
        " - brave",
        " - brave browser",
    ] {
        if let Some(stripped) = cleaned.strip_suffix(suffix) {
            cleaned = stripped.trim().to_string();
        }
    }
    cleaned
}

fn clean_display_name(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".lnk")
        .trim_end_matches(".exe")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_chromium_app_id_from_arguments() {
        let text = r#""chrome.exe" --profile-directory="Profile 1" --app-id=abcdefghijklmnopabcdefghijklmnop"#;
        assert_eq!(
            extract_app_id(text).as_deref(),
            Some("abcdefghijklmnopabcdefghijklmnop")
        );
        assert_eq!(
            extract_flag_value(text, "--profile-directory=").as_deref(),
            Some("Profile 1")
        );
    }

    #[test]
    fn parses_stable_identifier() {
        let id = make_pwa_identifier(
            BrowserKind::Edge,
            "Profile 1",
            "abcdefghijklmnopabcdefghijklmnop",
        );
        let parsed = parse_pwa_identifier(&id).unwrap();
        assert_eq!(parsed.browser, BrowserKind::Edge);
        assert_eq!(parsed.app_id, "abcdefghijklmnopabcdefghijklmnop");
    }

    #[test]
    fn title_heuristic_requires_close_match() {
        assert!(title_matches_pwa("YouTube", "YouTube"));
        assert!(title_matches_pwa("YouTube - Google Chrome", "YouTube"));
        assert!(!title_matches_pwa(
            "Inbox - Gmail - Google Chrome",
            "YouTube"
        ));
    }
}
