use anyhow::Context as _;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    env, fs, io,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

#[macro_use]
mod log;

mod deserialize_wrapper;
mod parser;
mod thread_pool;

fn main() {
    let mut argv = env::args_os();
    argv.next().expect("must be exe name");

    let Some(locales_path) = argv.next() else {
        print_usage_and_exit(-2);
    };
    let locales_path = Path::new(&locales_path);

    if let Err(err) = run(locales_path) {
        error!("{err}");
        process::exit(1);
    }
}

fn run(locales_path: &Path) -> anyhow::Result<()> {
    let locales_data = load_locales(locales_path).context("Failed to load locales")?;
    let locales = parse_locales(locales_data)?;

    let all_sections_with_keys = Arc::new(
        locales
            .values()
            .flat_map(|locale| {
                locale.sections.iter().flat_map(|(section_name, section)| {
                    let section_name = Arc::<str>::from(section_name.as_str());
                    section
                        .keys()
                        .map(move |key| (section_name.clone(), Arc::<str>::from(key.as_str())))
                })
            })
            .fold(
                HashMap::<_, HashSet<_>>::new(),
                |mut acc, (section_name, key)| {
                    acc.entry(section_name).or_default().insert(key);
                    acc
                },
            ),
    );

    let mut handles = Vec::with_capacity(locales.len());
    for (locale_name, locale) in locales {
        let all_sections_with_keys = Arc::clone(&all_sections_with_keys);
        handles.push(thread_pool::spawn(move || {
            let _ = (locale_name, locale, all_sections_with_keys);
        }));
    }

    for handle in handles {
        handle.join();
    }

    Ok(())
}

struct LocaleFile {
    path: PathBuf,
    data: Vec<u8>,
}

fn load_locales(locales_path: &Path) -> io::Result<HashMap<String, Vec<LocaleFile>>> {
    let locales_path: Cow<_> = if locales_path.is_absolute() {
        locales_path.into()
    } else {
        let current_dir = env::current_dir()?;
        current_dir.join(locales_path).into()
    };
    let locales_path = locales_path.canonicalize()?;

    let mut locales = HashMap::<String, Vec<LocaleFile>>::new();

    info!("Reading locales directory: {}", locales_path.display());
    for entry in fs::read_dir(locales_path)? {
        let entry = entry?;

        let file_name = entry.file_name();
        let file_name = Path::new(&file_name);
        let Some("yml" | "yaml") = file_name.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };

        let locale = file_name
            .file_prefix()
            .map(|prefix| prefix.to_string_lossy().into_owned())
            .unwrap_or_default();
        if locale.is_empty() {
            continue;
        }

        let path = entry.path();
        info!("Found file '{}' for locale '{locale}'", path.display());

        let data = fs::read(&path)?;

        locales
            .entry(locale)
            .or_default()
            .push(LocaleFile { path, data });
    }

    Ok(locales)
}

fn parse_locales(
    locales_data: HashMap<String, Vec<LocaleFile>>,
) -> anyhow::Result<HashMap<String, parser::Locale>> {
    let mut locales = HashMap::with_capacity(locales_data.len());
    let mut handles = Vec::with_capacity(locales_data.len());

    for (locale_name, files) in locales_data {
        handles.push(thread_pool::spawn(move || {
            info!("Parsing locale: {locale_name}");
            let locale = files
                .into_iter()
                .map(|LocaleFile { path, data }| {
                    info!("Parsing file: {}", path.display());
                    let locale: parser::Locale =
                        serde_yaml::from_slice(&data).context("Failed to parse locale")?;
                    Ok::<_, anyhow::Error>(locale)
                })
                .try_fold(parser::Locale::default(), |mut acc, locale| {
                    let locale = locale?;
                    acc.rules.reserve(locale.rules.len());
                    acc.sections.reserve(locale.sections.len());

                    for (rule_name, rule) in locale.rules {
                        if acc.rules.contains_key(&rule_name) {
                            warn!("Duplicate rule: {rule_name}! Skipping");
                            continue;
                        }
                        acc.rules.insert(rule_name, rule);
                    }

                    for (section_name, section) in locale.sections {
                        if acc.sections.contains_key(&section_name) {
                            warn!("Duplicate section: {section_name}! Skipping");
                            continue;
                        }
                        acc.sections.insert(section_name, section);
                    }

                    Ok::<_, anyhow::Error>(acc)
                })?;

            Ok::<_, anyhow::Error>((locale_name, locale))
        }));
    }

    for result in handles {
        let (locale_name, locale) = result.join()?;
        locales.insert(locale_name, locale);
    }

    Ok(locales)
}

fn print_usage_and_exit(exit_code: i32) -> ! {
    log!(Usage: "\n\t{}", "nxn-locales-compiler <locales dir>");
    process::exit(exit_code);
}
