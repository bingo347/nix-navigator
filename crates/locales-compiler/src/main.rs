use std::{borrow::Cow, env, path::Path, process};

#[macro_use]
mod log;

mod deserialize_wrapper;
mod locales_data;
mod parser;

fn main() {
    let mut argv = env::args_os();
    argv.next().expect("must be exe name");

    let Some(locales_path) = argv.next() else {
        print_usage_and_exit(-1);
    };
    let locales_path = Path::new(&locales_path);
    let locales_path: Cow<_> = if locales_path.is_absolute() {
        locales_path.into()
    } else {
        let current_dir = env::current_dir().expect("Cannot get current directory");
        current_dir.join(locales_path).into()
    };
    let locales_path = locales_path
        .canonicalize()
        .expect("Cannot canonicalize locales dir path");

    let locales = locales_data::read(&locales_path).expect("Failed to read locales");

    for (locale_name, locale_data) in &locales {
        info!("Compiling rules for locale: {locale_name}");
        for (rule_name, rule_data) in &locale_data.rules {
            let rule: parser::Rule =
                serde_yaml::from_value(rule_data.clone().into()).expect("Failed to parse rule");
            println!("Found rule '{rule_name}': {rule:#?}");
        }

        info!("Compiling sections for locale: {locale_name}");
        for (section_name, section_data) in &locale_data.sections {
            for (key_name, key_data) in section_data {
                let key_name = key_name.as_str().unwrap_or_default();
                let template: parser::FullTemplate =
                    serde_yaml::from_value(key_data.clone()).expect("Failed to parse key");
                println!("Found key '{section_name}/{key_name}': {template:#?}");
            }
        }
    }
}

fn print_usage_and_exit(exit_code: i32) -> ! {
    log!(Usage: "\n\t{}", "nxn-locales-compiler <locales dir>");
    process::exit(exit_code);
}
