use pipebuilder_common::{api::models::PrintHeader, Result};
use std::fmt::{self, Display};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub(crate) fn print_record<T>(record: &T)
where
    T: Display,
{
    print!("{}", record)
}

pub(crate) fn print_records<T>(records: &[T])
where
    T: Display + PrintHeader,
{
    T::print_header();
    for record in records {
        print_record(record)
    }
}

pub(crate) fn print_utf8(buffer: Vec<u8>) -> Result<()> {
    let text = String::from_utf8(buffer)?;
    println!("{}", text);
    Ok(())
}
pub(crate) struct Printer {
    stderr: StandardStream,
}

impl Printer {
    pub fn new() -> Printer {
        Printer {
            stderr: StandardStream::stderr(ColorChoice::Auto),
        }
    }

    pub fn print(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        color: Color,
    ) -> Result<()> {
        self.stderr.reset()?;
        self.stderr
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))?;
        // write status
        write!(self.stderr, "{:>12}", status)?;
        // write message
        self.stderr.reset()?;
        match message {
            Some(message) => writeln!(self.stderr, " {}", message)?,
            None => write!(self.stderr, " ")?,
        }
        Ok(())
    }

    pub fn status<T: fmt::Display, U: fmt::Display>(
        &mut self,
        status: T,
        message: U,
    ) -> Result<()> {
        self.print(&status, Some(&message), Color::Green)
    }

    /*
    pub fn error<T: fmt::Display>(&mut self, message: T) -> Result<()> {
        self.print(&"Error", Some(&message), Color::Red)
    }

    pub fn warning<T: fmt::Display>(&mut self, message: T) -> Result<()> {
        self.print(&"Warning", Some(&message), Color::Yellow)
    }
    */
}
