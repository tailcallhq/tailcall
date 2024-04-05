use crate::writer::IndentedWriter;
use crate::{Directives, InputTypeWriter};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::io::Write;

pub struct ServiceDocument<W: Write> {
    writer: IndentedWriter<W>,
    directive: Directives,
    input_types: InputTypeWriter,
}

impl<W: Write> ServiceDocument<W> {
    pub fn new(dest: W) -> Self {
        let writer = IndentedWriter::new(dest);
        let written_directives = HashSet::new();

        let directive = Directives::new(written_directives);
        let input_types = InputTypeWriter {};

        ServiceDocument { writer, directive, input_types }
    }

    pub fn print(&mut self) -> anyhow::Result<()> {
        self.write()?;
        Ok(())
    }

    fn write(&mut self) -> Result<()> {
        let mut extra_it = BTreeMap::new();
        self.directive.write(&mut extra_it)?;
        self.input_types.write(&mut self.writer, extra_it)?;
        Ok(())
    }
}
