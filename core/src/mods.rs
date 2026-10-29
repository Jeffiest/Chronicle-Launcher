//! MoonMod (MoonFactory) mods: each a folder named after its id, holding `mod.json` and its Lua entry script. The
//! launcher finds them where the game does, checks their manifests by MoonFactory's rules, and keeps the player's
//! choices in the same `<save>/moonmod/profile.json` the in-game Mod Manager reads and writes, so the two agree.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// Where the game looks for mods (MoonMod `port/include/moonmod/mods_directory.hpp`, `ModsDirectory`): `mods/` at the
/// root of a checkout (the folder with `kazarov.conf`), when there is one; otherwise `mods/` beside the executable,
/// when there is one; otherwise `<save>/mods`.
pub fn mods_dir(executable_dir: &Path, save: &Path) -> PathBuf {
    for dir in executable_dir.ancestors() {
        if dir.join("kazarov.conf").is_file() {
            if dir.join("mods").is_dir() {
                return dir.join("mods");
            }
            break;
        }
    }
    if executable_dir.join("mods").is_dir() {
        return executable_dir.join("mods");
    }
    save.join("mods")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub default: Value,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NativeExtension {
    pub library: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: i64,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub framework_api: i64,
    pub games: Vec<String>,
    pub entry: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub optional_dependencies: Vec<String>,
    #[serde(default)]
    pub load_after: Vec<String>,
    #[serde(default)]
    pub load_before: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub native_extension: Option<NativeExtension>,
    #[serde(default)]
    pub settings: Vec<Setting>,
}

const PERMISSIONS: [&str; 8] = [
    "storage.read",
    "storage.write",
    "game.read",
    "game.write",
    "content.register",
    "scripts.override",
    "debug.inspect",
    "native.extension",
];

fn package_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (3..=64).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b".-_".contains(b))
}

fn setting_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b".-_".contains(b))
}

fn semver(version: &str) -> bool {
    let core = version.split(['+', '-']).next().unwrap_or("");
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// A path inside the package: relative, no `..`, no root.
fn inside(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Whether `value` is one this setting accepts (MoonFactory `Runtime::Impl::SettingValue`).
pub fn accepts(setting: &Setting, value: &Value) -> bool {
    let min = setting.min.unwrap_or(f64::MIN);
    let max = setting.max.unwrap_or(f64::MAX);
    match setting.kind.as_str() {
        "boolean" => value.is_boolean(),
        "integer" => value
            .as_i64()
            .is_some_and(|n| (n as f64) >= min && (n as f64) <= max),
        "number" => value
            .as_f64()
            .is_some_and(|n| n.is_finite() && n >= min && n <= max),
        "string" => value.as_str().is_some_and(|s| s.len() <= 256),
        "choice" => value
            .as_str()
            .is_some_and(|s| setting.choices.iter().any(|c| c == s)),
        _ => false,
    }
}

/// MoonFactory's checks (MoonMod `src/manifest.cpp`, `ReadManifest`), in its words, for a package at `root`.
pub fn check(manifest: &Manifest, root: &Path) -> Result<(), String> {
    if manifest.schema_version != 1 {
        return Err("schema_version must be 1".into());
    }
    if !package_id(&manifest.id) {
        return Err("id must be 3-64 lowercase letters, digits, dots, underscores, or hyphens".into());
    }
    if manifest.name.is_empty() || manifest.name.len() > 96 {
        return Err("field 'name' must be 1 to 96 bytes".into());
    }
    if !semver(&manifest.version) {
        return Err("version must use semantic version form MAJOR.MINOR.PATCH".into());
    }
    if !inside(&manifest.entry) || !manifest.entry.ends_with(".lua") {
        return Err("entry must be a relative .lua file inside the package".into());
    }
    if !root.join(&manifest.entry).is_file() {
        return Err(format!("its entry script {} is missing", manifest.entry));
    }
    if manifest.framework_api < 1 {
        return Err("framework_api must be a positive integer".into());
    }
    if manifest.games.is_empty() || manifest.games.len() > 2 {
        return Err("games must contain at least one supported game id".into());
    }
    if manifest
        .games
        .iter()
        .any(|g| g != "chronicle" && g != "chronicle-two")
    {
        return Err("games may contain only 'chronicle' and 'chronicle-two'".into());
    }
    for list in [
        &manifest.dependencies,
        &manifest.optional_dependencies,
        &manifest.load_after,
        &manifest.load_before,
    ] {
        if list.len() > 64 || list.iter().any(|id| !package_id(id)) {
            return Err("a dependency or load order entry is not a mod id".into());
        }
    }
    if let Some(unknown) = manifest
        .permissions
        .iter()
        .find(|p| !PERMISSIONS.contains(&p.as_str()))
    {
        return Err(format!(
            "unsupported permission '{unknown}' for framework API major 1"
        ));
    }
    let wants_native = manifest.permissions.iter().any(|p| p == "native.extension");
    match &manifest.native_extension {
        Some(native) => {
            let library = native.library.to_ascii_lowercase();
            if !inside(&native.library) || ![".dll", ".so", ".dylib"].iter().any(|e| library.ends_with(e)) {
                return Err(
                    "native_extension.library must be a safe relative .dll, .so, or .dylib path".into(),
                );
            }
            if !wants_native {
                return Err("native_extension requires the native.extension manifest permission".into());
            }
        }
        None if wants_native => {
            return Err("native.extension permission requires a native_extension library declaration".into());
        }
        None => {}
    }
    if manifest.settings.len() > 128 {
        return Err("field 'settings' must be an array with at most 128 entries".into());
    }
    let mut seen = std::collections::HashSet::new();
    for setting in &manifest.settings {
        if !setting_id(&setting.id) || !seen.insert(setting.id.as_str()) {
            return Err("setting ids must be unique lowercase ids beginning with a letter".into());
        }
        if !["boolean", "integer", "number", "string", "choice"].contains(&setting.kind.as_str()) {
            return Err(format!("setting '{}' has an unknown type", setting.id));
        }
        if setting.kind == "choice" && setting.choices.is_empty() {
            return Err(format!(
                "choice setting '{}' must have at least one choice",
                setting.id
            ));
        }
        if setting.min.zip(setting.max).is_some_and(|(min, max)| min > max)
            || setting.step.is_some_and(|s| s <= 0.0)
        {
            return Err(format!(
                "setting '{}' has bounds that do not fit together",
                setting.id
            ));
        }
        if !accepts(setting, &setting.default) {
            return Err(format!(
                "setting '{}' has a default that does not match its type or bounds",
                setting.id
            ));
        }
    }
    Ok(())
}

/// Reads and checks `<root>/mod.json`.
pub fn read_manifest(root: &Path) -> Result<Manifest, String> {
    let file = root.join("mod.json");
    let size = std::fs::metadata(&file)
        .map_err(|_| "mod.json is missing".to_string())?
        .len();
    if size == 0 || size > 256 * 1024 {
        return Err("mod.json is empty, unreadable, or larger than 256 KiB".into());
    }
    let bytes = std::fs::read(&file).map_err(|_| "mod.json could not be opened".to_string())?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|e| format!("mod.json could not be read: {e}"))?;
    check(&manifest, root)?;
    Ok(manifest)
}

/// The player's choices: `<save>/moonmod/profile.json`. Keys the launcher does not know are kept.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile(Map<String, Value>);

fn profile_file(save: &Path) -> PathBuf {
    save.join("moonmod").join("profile.json")
}

impl Profile {
    pub fn load(save: &Path) -> Result<Profile, String> {
        let mut profile = Map::new();
        profile.insert("format_version".into(), 1.into());
        let file = profile_file(save);
        match std::fs::read(&file) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("the mod profile could not be read: {e}")),
            Ok(bytes) => {
                let loaded: Value = serde_json::from_slice(&bytes).map_err(|e| {
                    format!(
                        "the mod profile is not valid JSON ({e}); fix or remove {}",
                        file.display()
                    )
                })?;
                let Value::Object(loaded) = loaded else {
                    return Err("the mod profile is not a JSON object".into());
                };
                if loaded.get("format_version") != Some(&Value::from(1)) {
                    return Err("the mod profile has a format this launcher does not know".into());
                }
                profile = loaded;
            }
        }
        for key in ["enabled", "settings", "native_trusted"] {
            if !profile.get(key).is_some_and(Value::is_object) {
                profile.insert(key.into(), Value::Object(Map::new()));
            }
        }
        Ok(Profile(profile))
    }

    pub fn save(&self, save: &Path) -> io::Result<()> {
        let file = profile_file(save);
        std::fs::create_dir_all(file.parent().unwrap())?;
        let temporary = file.with_extension("json.launcher-tmp");
        std::fs::write(
            &temporary,
            serde_json::to_vec_pretty(&Value::Object(self.0.clone()))?,
        )?;
        std::fs::rename(temporary, file)
    }

    fn section(&self, key: &str) -> &Map<String, Value> {
        self.0[key]
            .as_object()
            .expect("load makes every section an object")
    }

    fn section_mut(&mut self, key: &str) -> &mut Map<String, Value> {
        self.0
            .get_mut(key)
            .and_then(Value::as_object_mut)
            .expect("load makes every section an object")
    }

    pub fn enabled(&self, id: &str) -> bool {
        self.section("enabled")
            .get(id)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn set_enabled(&mut self, id: &str, on: bool) {
        self.section_mut("enabled").insert(id.into(), on.into());
    }

    /// A native mod counts as trusted when the game recorded a trust for it; the game itself checks that the trust
    /// matches the library's fingerprint. The launcher never writes this.
    pub fn native_trusted(&self, id: &str) -> bool {
        self.section("native_trusted")
            .get(id)
            .is_some_and(|v| !v.is_null() && v != &Value::Bool(false))
    }

    pub fn setting(&self, id: &str, setting: &Setting) -> Value {
        self.section("settings")
            .get(id)
            .and_then(|s| s.get(&setting.id))
            .filter(|v| accepts(setting, v))
            .cloned()
            .unwrap_or_else(|| setting.default.clone())
    }

    pub fn set_setting(&mut self, id: &str, setting: &str, value: Option<Value>) {
        let settings = self.section_mut("settings");
        let entry = settings
            .entry(id.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        let map = entry.as_object_mut().unwrap();
        match value {
            Some(value) => map.insert(setting.into(), value),
            None => map.remove(setting),
        };
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SettingView {
    #[serde(flatten)]
    pub setting: Setting,
    pub value: Value,
}

/// One folder in the mods folder, as the launcher shows it.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Mod {
    pub folder: String,
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub dependencies: Vec<String>,
    pub settings: Vec<SettingView>,
    pub enabled: bool,
    pub native: bool,
    pub native_trusted: bool,
    /// Why the game will not load it: a broken manifest, another game, a duplicate id. `None` when it can.
    pub problem: Option<String>,
}

/// Every mod in the folder, by name. A mod for the other game only is listed with that as its problem.
pub fn list(mods: &Path, save: &Path, game_id: &str) -> Result<Vec<Mod>, String> {
    let profile = Profile::load(save)?;
    let Ok(entries) = std::fs::read_dir(mods) else {
        return Ok(Vec::new());
    };
    let mut folders: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    folders.sort();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for root in folders {
        let folder = root.file_name().unwrap_or_default().to_string_lossy().to_string();
        let mut item = Mod {
            id: folder.clone(),
            name: folder.clone(),
            folder,
            version: None,
            author: None,
            description: None,
            permissions: Vec::new(),
            dependencies: Vec::new(),
            settings: Vec::new(),
            enabled: false,
            native: false,
            native_trusted: false,
            problem: None,
        };
        match read_manifest(&root) {
            Err(problem) => item.problem = Some(problem),
            Ok(manifest) => {
                item.id = manifest.id.clone();
                item.name = manifest.name.clone();
                item.version = Some(manifest.version.clone());
                item.author = manifest.author.clone();
                item.description = manifest.description.clone();
                item.permissions = manifest.permissions.clone();
                item.dependencies = manifest.dependencies.clone();
                item.enabled = profile.enabled(&manifest.id);
                item.native = manifest.native_extension.is_some();
                item.native_trusted = item.native && profile.native_trusted(&manifest.id);
                item.settings = manifest
                    .settings
                    .iter()
                    .map(|s| SettingView {
                        setting: s.clone(),
                        value: profile.setting(&manifest.id, s),
                    })
                    .collect();
                if !manifest.games.iter().any(|g| g == game_id) {
                    item.problem = Some("made for the other game".into());
                } else if !seen.insert(manifest.id.clone()) {
                    item.problem = Some("another folder has the same mod id, and comes first".into());
                }
            }
        }
        out.push(item);
    }
    out.sort_by_key(|m| (m.problem.is_some(), m.name.to_lowercase()));
    Ok(out)
}

fn find(mods: &Path, save: &Path, game_id: &str, id: &str) -> Result<Mod, String> {
    list(mods, save, game_id)?
        .into_iter()
        .find(|m| m.id == id && m.problem.is_none())
        .ok_or_else(|| format!("No mod called {id} that this game can load"))
}

pub fn set_enabled(mods: &Path, save: &Path, game_id: &str, id: &str, on: bool) -> Result<(), String> {
    find(mods, save, game_id, id)?;
    let mut profile = Profile::load(save)?;
    profile.set_enabled(id, on);
    profile.save(save).map_err(|e| e.to_string())
}

/// Sets a setting, or with `None` puts it back to its default. A value the mod does not accept is refused.
pub fn set_setting(
    mods: &Path,
    save: &Path,
    game_id: &str,
    id: &str,
    setting: &str,
    value: Option<Value>,
) -> Result<(), String> {
    let found = find(mods, save, game_id, id)?;
    let definition = found
        .settings
        .iter()
        .find(|s| s.setting.id == setting)
        .ok_or_else(|| format!("{} has no setting called {setting}", found.name))?;
    if let Some(value) = &value {
        if !accepts(&definition.setting, value) {
            return Err(format!("{} does not take that value", definition.setting.name));
        }
    }
    let mut profile = Profile::load(save)?;
    profile.set_setting(id, setting, value);
    profile.save(save).map_err(|e| e.to_string())
}

/// The package in a folder: the folder itself, or the one folder in it, as a zip usually wraps a mod.
fn package_root(dir: &Path) -> Option<PathBuf> {
    if dir.join("mod.json").is_file() {
        return Some(dir.to_path_buf());
    }
    let mut inner = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir());
    let only = inner.next()?;
    (inner.next().is_none() && only.join("mod.json").is_file()).then_some(only)
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_dir(&entry.path(), &to.join(entry.file_name()))?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}

const ZIP_LIMIT: u64 = 512 << 20;

/// Unpacks a zip into `into`, refusing entries that would land outside it, links, and more than 512 MiB in all.
fn unzip(file: &Path, into: &Path) -> Result<(), String> {
    let reader = std::fs::File::open(file).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(reader).map_err(|_| "not a zip file".to_string())?;
    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(name) = entry.enclosed_name() else {
            return Err("the zip has a file that would land outside its folder".into());
        };
        if entry.is_symlink() {
            return Err("the zip holds a link, which a mod does not need".into());
        }
        let target = into.join(name);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        total += entry.size();
        if total > ZIP_LIMIT {
            return Err("the zip unpacks to more than 512 MiB".into());
        }
        std::fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
        let copied = io::copy(&mut (&mut entry).take(ZIP_LIMIT), &mut out).map_err(|e| e.to_string())?;
        if copied != entry.size() {
            return Err("the zip says one size for a file and holds another".into());
        }
    }
    Ok(())
}

/// What installing would add, shown before anything is written.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Preview {
    pub manifest: Manifest,
    /// Whether a mod with this id is there already (installing replaces it, keeping its settings).
    pub replaces: bool,
    pub for_this_game: bool,
}

/// A staged package: unpacked or copied beside the mods folder, checked, waiting for `install` or `discard`.
#[derive(Debug)]
pub struct Staged {
    dir: PathBuf,
    root: PathBuf,
    pub preview: Preview,
}

/// Unpacks or copies `source` (a `.zip` or a folder) next to the mods folder and checks it.
pub fn stage(source: &Path, mods: &Path, game_id: &str) -> Result<Staged, String> {
    let parent = mods.parent().ok_or("the mods folder has no parent")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let dir = parent.join(format!(".mod-install-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let result = (|| {
        if source.is_dir() {
            copy_dir(source, &dir.join("package")).map_err(|e| e.to_string())?;
        } else {
            unzip(source, &dir)?;
        }
        let root = package_root(&dir)
            .or_else(|| package_root(&dir.join("package")))
            .ok_or("no mod.json in it, at its top or in its one folder")?;
        let manifest = read_manifest(&root)?;
        let preview = Preview {
            replaces: mods.join(&manifest.id).exists(),
            for_this_game: manifest.games.iter().any(|g| g == game_id),
            manifest,
        };
        Ok(Staged {
            dir: dir.clone(),
            root,
            preview,
        })
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    result
}

impl Staged {
    /// Moves the package into `<mods>/<id>`. A mod already there is replaced; its settings stay in the profile.
    pub fn install(self, mods: &Path) -> Result<String, String> {
        let id = self.preview.manifest.id.clone();
        std::fs::create_dir_all(mods).map_err(|e| e.to_string())?;
        let target = mods.join(&id);
        let old = self.dir.join("replaced");
        if target.exists() {
            std::fs::rename(&target, &old).map_err(|e| e.to_string())?;
        }
        if let Err(e) = std::fs::rename(&self.root, &target) {
            if old.exists() {
                let _ = std::fs::rename(&old, &target);
            }
            return Err(e.to_string());
        }
        let _ = std::fs::remove_dir_all(&self.dir);
        Ok(id)
    }

    pub fn discard(self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Deletes a mod's folder. Its choices stay in the profile, so putting it back brings them back.
pub fn remove(mods: &Path, folder: &str) -> Result<(), String> {
    if !inside(folder) || folder.contains(['/', '\\']) {
        return Err("no such mod folder".into());
    }
    std::fs::remove_dir_all(mods.join(folder)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hello(id: &str, games: &[&str]) -> Value {
        json!({
            "schema_version": 1, "id": id, "name": "Hello World", "version": "1.0.0", "author": "MoonFactory Example",
            "framework_api": 1, "games": games, "entry": "main.lua", "permissions": ["storage.read"],
            "settings": [
                {"id": "greeting", "name": "Show greeting", "type": "boolean", "default": true},
                {"id": "count", "name": "Count", "type": "integer", "default": 3, "min": 1, "max": 5},
                {"id": "mood", "name": "Mood", "type": "choice", "default": "calm", "choices": ["calm", "bold"]}
            ]
        })
    }

    fn put(mods: &Path, folder: &str, manifest: &Value) {
        let root = mods.join(folder);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("mod.json"), serde_json::to_vec(manifest).unwrap()).unwrap();
        std::fs::write(root.join("main.lua"), "return {}").unwrap();
    }

    #[test]
    fn finds_the_mods_folder_where_the_game_does() {
        let dir = tempfile::tempdir().unwrap();
        let (checkout, save) = (dir.path().join("Chronicle"), dir.path().join("save"));
        let exe = checkout.join("port/build/windows-x64-release");
        std::fs::create_dir_all(&exe).unwrap();
        assert_eq!(mods_dir(&exe, &save), save.join("mods"));
        std::fs::create_dir_all(exe.join("mods")).unwrap();
        assert_eq!(mods_dir(&exe, &save), exe.join("mods"));
        std::fs::write(checkout.join("kazarov.conf"), "").unwrap();
        assert_eq!(
            mods_dir(&exe, &save),
            exe.join("mods"),
            "a checkout without mods/ falls through"
        );
        std::fs::create_dir_all(checkout.join("mods")).unwrap();
        assert_eq!(mods_dir(&exe, &save), checkout.join("mods"));
    }

    #[test]
    fn reads_moonmods_own_examples() {
        let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mods");
        for entry in std::fs::read_dir(&examples).unwrap() {
            let root = entry.unwrap().path();
            read_manifest(&root).unwrap_or_else(|e| panic!("{}: {e}", root.display()));
        }
    }

    /// Every bundled mod and example in a MoonMod checkout: `MOONMOD_CHECKOUT=<path> cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn reads_every_mod_in_a_moonmod_checkout() {
        let checkout = PathBuf::from(std::env::var("MOONMOD_CHECKOUT").expect("set MOONMOD_CHECKOUT"));
        let mut read = 0;
        for folder in ["mods", "examples"] {
            for entry in std::fs::read_dir(checkout.join(folder)).unwrap() {
                let root = entry.unwrap().path();
                read_manifest(&root).unwrap_or_else(|e| panic!("{}: {e}", root.display()));
                read += 1;
            }
        }
        assert!(read > 0);
    }

    #[test]
    fn says_why_a_manifest_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let cases = [
            (json!({"id": "X"}), "missing field"),
            (json!({"extra": 1}), "unknown field"),
        ];
        for (patch, why) in cases {
            let mut manifest = hello("a.mod", &["chronicle"]);
            for (k, v) in patch.as_object().unwrap() {
                manifest[k] = v.clone();
            }
            if why == "missing field" {
                manifest.as_object_mut().unwrap().remove("entry");
            }
            put(dir.path(), "a", &manifest);
            let err = read_manifest(&dir.path().join("a")).unwrap_err();
            assert!(err.contains(why), "{err}");
        }
        let mut manifest = hello("Bad Id", &["chronicle"]);
        put(dir.path(), "b", &manifest);
        assert!(read_manifest(&dir.path().join("b"))
            .unwrap_err()
            .contains("id must be"));
        manifest = hello("a.mod", &["chronicle"]);
        manifest["permissions"] = json!(["native.extension"]);
        put(dir.path(), "c", &manifest);
        assert!(read_manifest(&dir.path().join("c"))
            .unwrap_err()
            .contains("requires a native_extension"));
        manifest = hello("a.mod", &["chronicle"]);
        manifest["settings"][1]["default"] = json!(9);
        put(dir.path(), "d", &manifest);
        assert!(read_manifest(&dir.path().join("d"))
            .unwrap_err()
            .contains("default"));
    }

    #[test]
    fn the_schema_has_no_field_the_launcher_does_not_know() {
        let schema: Value =
            serde_json::from_str(include_str!("../../vendor/moonmod/manifest.schema.json")).unwrap();
        let known = [
            "schema_version",
            "id",
            "name",
            "version",
            "author",
            "description",
            "framework_api",
            "games",
            "entry",
            "dependencies",
            "optional_dependencies",
            "load_after",
            "load_before",
            "permissions",
            "native_extension",
            "settings",
        ];
        for key in schema["properties"].as_object().unwrap().keys() {
            assert!(
                known.contains(&key.as_str()),
                "the manifest schema has {key}; teach mods.rs about it"
            );
        }
        let permissions = &schema["properties"]["permissions"]["items"]["enum"];
        let permissions: Vec<&str> = permissions
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap())
            .collect();
        assert_eq!(permissions, PERMISSIONS);
    }

    #[test]
    fn lists_switches_and_sets_settings_through_the_profile() {
        let dir = tempfile::tempdir().unwrap();
        let (mods, save) = (dir.path().join("mods"), dir.path().join("save"));
        put(
            &mods,
            "example.hello",
            &hello("example.hello", &["chronicle", "chronicle-two"]),
        );
        put(
            &mods,
            "example.other",
            &hello("example.other", &["chronicle-two"]),
        );
        std::fs::create_dir_all(mods.join("broken")).unwrap();
        std::fs::create_dir_all(save.join("moonmod")).unwrap();
        std::fs::write(
            save.join("moonmod/profile.json"),
            r#"{"format_version":1,"enabled":{},"future":{"x":1}}"#,
        )
        .unwrap();

        let listed = list(&mods, &save, "chronicle").unwrap();
        let names: Vec<(&str, Option<&str>)> = listed
            .iter()
            .map(|m| (m.id.as_str(), m.problem.as_deref()))
            .collect();
        assert_eq!(names[0], ("example.hello", None));
        assert!(names.contains(&("example.other", Some("made for the other game"))));
        assert!(names
            .iter()
            .any(|(id, p)| *id == "broken" && p.unwrap().contains("mod.json is missing")));

        set_enabled(&mods, &save, "chronicle", "example.hello", true).unwrap();
        assert!(set_enabled(&mods, &save, "chronicle", "example.other", true).is_err());
        set_setting(
            &mods,
            &save,
            "chronicle",
            "example.hello",
            "count",
            Some(json!(5)),
        )
        .unwrap();
        assert!(set_setting(
            &mods,
            &save,
            "chronicle",
            "example.hello",
            "count",
            Some(json!(6))
        )
        .is_err());
        assert!(set_setting(
            &mods,
            &save,
            "chronicle",
            "example.hello",
            "mood",
            Some(json!("loud"))
        )
        .is_err());
        set_setting(
            &mods,
            &save,
            "chronicle",
            "example.hello",
            "mood",
            Some(json!("bold")),
        )
        .unwrap();

        let hello = &list(&mods, &save, "chronicle").unwrap()[0];
        assert!(hello.enabled);
        let values: Vec<&Value> = hello.settings.iter().map(|s| &s.value).collect();
        assert_eq!(values, [&json!(true), &json!(5), &json!("bold")]);

        set_setting(&mods, &save, "chronicle", "example.hello", "count", None).unwrap();
        assert_eq!(
            list(&mods, &save, "chronicle").unwrap()[0].settings[1].value,
            json!(3)
        );

        let profile: Value =
            serde_json::from_slice(&std::fs::read(save.join("moonmod/profile.json")).unwrap()).unwrap();
        assert_eq!(
            profile["future"]["x"], 1,
            "keys the launcher does not know are kept"
        );
        assert_eq!(profile["enabled"]["example.hello"], true);
        assert_eq!(profile["settings"]["example.hello"]["mood"], "bold");
    }

    #[test]
    fn a_broken_profile_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let (mods, save) = (dir.path().join("mods"), dir.path().join("save"));
        put(&mods, "example.hello", &hello("example.hello", &["chronicle"]));
        std::fs::create_dir_all(save.join("moonmod")).unwrap();
        std::fs::write(save.join("moonmod/profile.json"), "{").unwrap();
        assert!(set_enabled(&mods, &save, "chronicle", "example.hello", true).is_err());
        assert_eq!(
            std::fs::read_to_string(save.join("moonmod/profile.json")).unwrap(),
            "{"
        );
    }

    fn zip_of(path: &Path, files: &[(&str, &[u8])]) {
        let mut writer = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        for (name, data) in files {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            std::io::Write::write_all(&mut writer, data).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn installs_from_a_zip_with_a_preview_first() {
        let dir = tempfile::tempdir().unwrap();
        let mods = dir.path().join("game/mods");
        let manifest = serde_json::to_vec(&hello("author.zipped", &["chronicle"])).unwrap();
        let file = dir.path().join("zipped.zip");
        zip_of(
            &file,
            &[
                ("zipped-1.0.0/mod.json", &manifest),
                ("zipped-1.0.0/main.lua", b"return {}"),
            ],
        );

        let staged = stage(&file, &mods, "chronicle").unwrap();
        assert_eq!(staged.preview.manifest.id, "author.zipped");
        assert!(!staged.preview.replaces && staged.preview.for_this_game);
        assert!(
            !mods.join("author.zipped").exists(),
            "nothing is installed by a preview"
        );
        assert_eq!(staged.install(&mods).unwrap(), "author.zipped");
        assert!(mods.join("author.zipped/main.lua").is_file());

        let again = stage(&file, &mods, "chronicle").unwrap();
        assert!(again.preview.replaces);
        again.discard();
        let leftovers: Vec<_> = std::fs::read_dir(dir.path().join("game"))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(leftovers.len(), 1, "only mods/ is left: {leftovers:?}");
    }

    #[test]
    fn refuses_zips_that_reach_outside() {
        let dir = tempfile::tempdir().unwrap();
        let mods = dir.path().join("game/mods");
        let file = dir.path().join("evil.zip");
        zip_of(&file, &[("../escape.txt", b"x")]);
        assert!(stage(&file, &mods, "chronicle").unwrap_err().contains("outside"));
        assert!(!dir.path().join("game/escape.txt").exists() && !dir.path().join("escape.txt").exists());
        let file = dir.path().join("empty.zip");
        zip_of(&file, &[("readme.txt", b"no mod here")]);
        assert!(stage(&file, &mods, "chronicle")
            .unwrap_err()
            .contains("no mod.json"));
    }

    #[test]
    fn installs_from_a_folder_and_removes() {
        let dir = tempfile::tempdir().unwrap();
        let (source, mods) = (dir.path().join("src"), dir.path().join("game/mods"));
        put(&source, "my-mod", &hello("author.folder", &["chronicle"]));
        let id = stage(&source.join("my-mod"), &mods, "chronicle")
            .unwrap()
            .install(&mods)
            .unwrap();
        assert!(mods.join(&id).join("mod.json").is_file());
        assert!(
            source.join("my-mod/mod.json").is_file(),
            "the source folder is kept"
        );
        assert!(remove(&mods, "../game").is_err());
        remove(&mods, &id).unwrap();
        assert!(!mods.join(&id).exists());
    }
}
