use std::fs::File;

use crate::{Account, Fallible, HelaError, Transaction};

/// CSV Data Reader
pub struct CsvDataReader(csv::Reader<File>);

impl CsvDataReader {
    /// Create new reader from a path
    pub fn new(fname: &str) -> Fallible<CsvDataReader> {
        let rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(fname)
            .map_err(HelaError::CsvError)?;
        Ok(CsvDataReader(rdr))
    }
}

impl Iterator for CsvDataReader {
    type Item = Fallible<Transaction>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rec = csv::StringRecord::new();
        match self.0.read_record(&mut rec) {
            Err(e) => Some(Err(HelaError::CsvError(e))),
            Ok(rec_read) => {
                if rec_read {
                    Some(rec.deserialize(None).map_err(HelaError::CsvError))
                } else {
                    None
                }
            }
        }
    }
}

/// CSV Data to Stdout Writer
pub struct CsvWriterStdout;

impl CsvWriterStdout {
    #[cfg(not(debug_assertions))]
    /// Write accounts to stdout
    pub fn write<W: std::io::Write>(
        accounts: Box<dyn Iterator<Item = Account> + '_>,
        wtr: Option<W>,
    ) -> Fallible<()> {
        let mut writer = if let Some(w) = wtr {
            csv::Writer::from_writer(w)
        } else {
            csv::Writer::from_writer(std::io::stdout())
        };
        for acc in accounts {
            writer.serialize(acc).map_err(HelaError::CsvError)?;
        }
        writer.flush()?;
        Ok(())
    }

    #[cfg(debug_assertions)]
    /// Write accounts to stdout
    pub fn write<W: std::io::Write>(
        accounts: Box<dyn Iterator<Item = Account> + '_>,
        wtr: Option<W>,
    ) -> Fallible<()> {
        let mut accounts: Vec<_> = accounts.collect();
        accounts.sort_by_key(|acc| acc.client);
        if let Some(w) = wtr {
            let mut writer = csv::Writer::from_writer(w);
            for acc in accounts {
                writer.serialize(acc).map_err(HelaError::CsvError)?;
            }
            writer.flush()?;
        } else {
            let mut writer = csv::Writer::from_writer(std::io::stdout());
            for acc in accounts {
                writer.serialize(acc).map_err(HelaError::CsvError)?;
            }
            writer.flush()?;
        };
        Ok(())
    }
}
