#[macro_use]
extern crate clap;
use clap::{App, Arg};
use hela::{CsvDataReader, CsvWriterStdout, Fallible, InMemoryStore, PaymentEngine};

fn main() {
    match inner_main() {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
        _ => {}
    }
}
fn inner_main() -> Fallible<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    let input_fname = matches.value_of("INPUT").unwrap();

    let transactions = CsvDataReader::new(&input_fname)?;
    let store = InMemoryStore::new();
    let mut engine = PaymentEngine::new(Box::new(store));

    engine.execute_transactions(transactions)?;

    CsvWriterStdout::write(engine.accounts()?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_input() -> Fallible<()> {
        Ok(())
    }
}
