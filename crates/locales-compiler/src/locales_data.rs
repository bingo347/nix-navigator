use anyhow::Context as _;
use serde_yaml::{Mapping, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

#[derive(Debug, Default)]
pub struct LocaleData {
    pub rules: HashMap<String, Mapping>,
    pub sections: HashMap<String, Mapping>,
}

pub fn read(locales_path: impl AsRef<Path>) -> anyhow::Result<HashMap<String, LocaleData>> {
    let locales_path = locales_path.as_ref();

    info!("Reading locales directory: {}", locales_path.display());
    let locales_data = fs::read_dir(locales_path)
        .context("cannot read locales directory")?
        .filter_map(|entry| {
            macro_rules! extract_with_context {
                ($value:expr, $context:literal) => {
                    match $value.context($context) {
                        Ok(value) => value,
                        Err(err) => return Some(Err(err)),
                    }
                };
            }

            let entry = extract_with_context!(entry, "read dir entry");
            let file_name = entry.file_name();
            let ext = Path::new(&file_name).extension()?.to_str()?;
            let ("yml" | "yaml") = ext else {
                return None;
            };

            let locale = Path::new(&file_name)
                .file_prefix()?
                .to_string_lossy()
                .into_owned();
            info!("Found file for locale: {locale}");

            let path = entry.path();
            info!("Reading locale file: {}", path.display());
            let data = extract_with_context!(fs::read(&path), "cannot read locale file");

            info!("Parsing file for locale: {locale}");
            let mut data: Value =
                extract_with_context!(serde_yaml::from_slice(&data), "cannot parse locale file");
            extract_with_context!(data.apply_merge(), "invalid yaml references");

            Some(Ok((locale, path, data)))
        });

    let mut locales = HashMap::<String, LocaleData>::new();
    for locale_data in locales_data {
        let (locale, path, data) = locale_data?;
        let locale_entry = locales.entry(locale).or_default();
        let rules = data.get("$rules");
        let sections = data.get("$sections");

        if let Some(rules) = rules {
            let Some(rules) = rules.as_mapping() else {
                warn!("Invalid $rules in {}", path.display());
                continue;
            };
            locale_entry.rules.reserve(rules.len());
            for (rule_name, rule) in rules {
                let (Some(rule_name), Some(rule)) = (rule_name.as_str(), rule.as_mapping()) else {
                    warn!("Invalid rule in {}", path.display());
                    continue;
                };
                let has_previous = locale_entry
                    .rules
                    .insert(rule_name.to_string(), rule.clone())
                    .is_some();
                if has_previous {
                    warn!("Duplicate rule {rule_name} in {}", path.display());
                }
            }
        }

        if let Some(sections) = sections {
            let Some(sections) = sections.as_mapping() else {
                warn!("Invalid $sections in {}", path.display());
                continue;
            };
            locale_entry.sections.reserve(sections.len());
            for (section_name, section) in sections {
                let (Some(section_name), Some(section)) =
                    (section_name.as_str(), section.as_mapping())
                else {
                    warn!("Invalid section in {}", path.display());
                    continue;
                };
                let has_previous = locale_entry
                    .sections
                    .insert(section_name.to_string(), section.clone())
                    .is_some();
                if has_previous {
                    warn!("Duplicate section {section_name} in {}", path.display());
                }
            }
        }
    }

    check_missing_sections_and_keys(&locales);
    Ok(locales)
}

fn check_missing_sections_and_keys(locales: &HashMap<String, LocaleData>) {
    info!("Checking for missing sections and keys");

    let mut sections_with_keys = HashSet::<(&str, &str)>::new();
    for locale_data in locales.values() {
        for (section_name, section) in &locale_data.sections {
            for key_name in section.keys() {
                sections_with_keys.insert((section_name, key_name.as_str().unwrap_or_default()));
            }
        }
    }

    for (locale_name, locale_data) in locales {
        let mut missing_sections = HashSet::<&str>::new();
        for (section_name, key_name) in &sections_with_keys {
            if missing_sections.contains(*section_name) {
                continue;
            }
            let Some(section) = locale_data.sections.get(*section_name) else {
                missing_sections.insert(*section_name);
                warn!("Missing section {section_name} in locale {locale_name}");
                continue;
            };
            if !section.contains_key(*key_name) {
                warn!("Missing key {section_name}/{key_name} in locale {locale_name}");
            }
        }
    }
}
