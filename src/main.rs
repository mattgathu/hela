#[macro_use]
extern crate clap;
use clap::{App, Arg};
use hela::{CsvDataReader, CsvWriterStdout, Fallible, InMemoryStore, PaymentEngine};

fn main() {
    match inner_main() {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
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

    CsvWriterStdout::write(engine.accounts()?, Some(std::io::stdout()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    macro_rules! tst {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() -> Fallible<()> {
                let mut input_file = NamedTempFile::new()?;
                input_file.write_all($input.as_bytes())?;
                let txns = CsvDataReader::new(input_file.path().to_str().unwrap())?;
                let store = InMemoryStore::new();
                let mut engine = PaymentEngine::new(Box::new(store));
                engine.execute_transactions(txns)?;
                let mut output = vec![];
                CsvWriterStdout::write(engine.accounts()?, Some(&mut output))?;
                let data = String::from_utf8(output)?;
                assert_eq!(data, $expected);
                Ok(())
            }
        };
    }

    tst!(
        test_dispute,
        "type,client,tx,amount\ndeposit,2,12,1.77\ndispute,2,12\ndeposit,2,13, 1.77\ndeposit,2,14, 1.77", 
        "client,available,held,total,locked\n2,3.54,1.77,5.31,false\n"
    );

    tst!(
        test_chargeback,
        "type,client,tx,amount\ndeposit,1,1,100.1\nchargeback,1,1\ndispute,1,1\nchargeback,1,1",
        "client,available,held,total,locked\n1,0.00,0.00,0.00,true\n"
    );

    tst!(
       test_resolution,
       "type,client,tx,amount\ndeposit,1,1,100.1\ndispute,1,1\ndeposit,2,12,1.77\ndispute,2,12\nresolve,2,12\nresolve,2,12\nresolve,2,12\nresolve,2,12",
       "client,available,held,total,locked\n1,0.00,100.10,100.10,false\n2,1.77,0.00,1.77,false\n"
   );

    tst!(
        test_scenario_1,
        "type,client,tx,amount
      deposit,1,1,1.0
      deposit,1,3,2.0
      withdrawal,1,5,1.5
      dispute,1,3
      deposit,2,2,2.0
      withdrawal,2,4,3.0",
        "client,available,held,total,locked\n1,-0.50,2.00,1.50,false\n2,2.00,0.00,2.00,false\n"
    );
}
