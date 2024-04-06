use crate::{Directives, InputTypeWriter};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::io::Write;

pub struct ServiceDocument<W: Write> {
    writer: W,
    directive: Directives,
    input_types: InputTypeWriter,
}

impl<W: Write> ServiceDocument<W> {
    pub fn new(writer: W) -> Self {
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
        let directive_str = self.directive.write(&mut extra_it)?;
        let input_types_str = self.input_types.write(extra_it)?;
        write!(self.writer, "{directive_str}")?;
        write!(self.writer, "{input_types_str}")?;
        Ok(())
    }
}
