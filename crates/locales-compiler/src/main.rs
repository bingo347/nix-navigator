use std::{borrow::Cow, env, path::Path, process};

#[macro_use]
mod log;

mod compiler;
mod locales_data;

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
            let actions = rule_data
                .get("do")
                .and_then(|v| v.as_sequence())
                .expect("rule must have `do` sequence");
            for action in actions {
                let Some(predicate) = action.get("if").and_then(|v| v.as_str()) else {
                    continue;
                };
                let predicate =
                    compiler::parse_expression(predicate).expect("Failed to parse predicate");
                println!("Found predicate for {rule_name}: {predicate:#?}");
            }
        }
    }
}

fn print_usage_and_exit(exit_code: i32) -> ! {
    log!(Usage: "\n\t{}", "nxn-locales-compiler <locales dir>");
    process::exit(exit_code);
}
