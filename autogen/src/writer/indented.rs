use std::io::Write;

pub struct IndentedWriter<W: Write> {
    writer: W,
    indentation: usize,
    line_broke: bool,
}

impl<W: Write> IndentedWriter<W> {
    pub fn new(writer: W) -> Self {
        IndentedWriter { writer, indentation: 0, line_broke: false }
    }

    pub fn indent(&mut self) {
        self.indentation += 2;
    }

    pub fn unindent(&mut self) {
        self.indentation -= 2;
    }
}

impl<W: std::io::Write> Write for IndentedWriter<W> {
    #[allow(clippy::same_item_push)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut new_buf = vec![];
        let mut extra = 0;

        for ch in buf {
            if self.line_broke && self.indentation > 0 {
                extra += self.indentation;
                for _ in 0..self.indentation {
                    new_buf.push(b' ');
                }
            }
            self.line_broke = false;

            new_buf.push(*ch);
            if ch == &b'\n' {
                self.line_broke = true;
            }
        }

        self.writer.write(&new_buf).map(|a| a - extra)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
