use pipebuilder_common::Result;
use std::fmt::Display;

pub(crate) fn print_record<T>(record: &T)
where
    T: Display,
{
    print!("{}", record)
}

pub(crate) fn print_records<T>(records: &[T])
where
    T: Display,
{
    for record in records {
        print_record(record)
    }
}

pub(crate) fn print_utf8(buffer: Vec<u8>) -> Result<()> {
    let text = String::from_utf8(buffer)?;
    println!("{}", text);
    Ok(())
}