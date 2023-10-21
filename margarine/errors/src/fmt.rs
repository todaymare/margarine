use std::fmt::Write;

use colourful::ColourBrush;
use common::{num_size, string_map::{StringMap, StringIndex}, source::{SourceRange, FileData}};
use display_plus::DisplayPlus;

pub struct ErrorFormatter<'me> {
    writer: &'me mut String,
    string_map: &'me StringMap<'me>,
    files: &'me [FileData],
}


impl<'me> ErrorFormatter<'me> {
    pub(crate) fn new(
        writer: &'me mut String, 
        string_map: &'me StringMap, 
        files: &'me [FileData]
    ) -> Self {
        
        Self {
            writer,
            string_map,
            files,
        }
    } 

    
    pub fn error<'fmt>(&'fmt mut self, msg: &str) -> CompilerError<'fmt, 'me> {
        CompilerError::new(self, msg)
    }


    pub fn string(&self, string_index: StringIndex) -> &str {
        self.string_map.get(string_index)
    }


    pub fn string_map(&self) -> &StringMap {
        self.string_map
    }
}


impl Write for ErrorFormatter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.writer.write_str(s)
    }
}


pub struct CompilerError<'me, 'fmt> {
    fmt: &'me mut ErrorFormatter<'fmt>,
}


impl<'me, 'fmt> CompilerError<'me, 'fmt> {
    fn new(f: &'me mut ErrorFormatter<'fmt>, msg: &str) -> Self {
        let _ = writeln!(f, "{}: {}", "error".red().bold(), msg.white().bold());
        Self {
            fmt: f,
        }
    }


    pub fn highlight(&mut self, source: SourceRange) {
        self.inner_highlight(source, None);
    }


    pub fn highlight_with_note(&mut self, source: SourceRange, note: &str) {
        self.inner_highlight(source, Some(note));
    }


    fn inner_highlight(&mut self, source: SourceRange, note: Option<&str>) {
        let file = source.file(self.fmt.files);

        let start_line = line_at(
            source.start() as usize, 
            file.read(),
            LineAt::ZERO
        ).unwrap();

        let end_line = line_at(
            source.end() as usize,
            file.read(),
            start_line,
        ).unwrap();

        let max_line_padding = num_size(end_line.line as u32 + 1) as usize;
        let ext = file.extension().read(self.fmt.string_map);
        let _ = writeln!(self.fmt, "{}{} {}{}{}:{}:{}",
            " ".repeated(max_line_padding),
            "-->".orange(),
            self.fmt.string_map.get(file.name()),
            if ext.is_empty() { "" } else { "." },
            ext,
            start_line.line + 1,
            characters_between(file.read(), start_line.offset, source.start() as usize)
        );

        let _ = writeln!(self.fmt, 
            "{} {} ",
            " ".repeated(max_line_padding), 
            "|".orange(), 
        );
        
        dbg!(source, start_line, end_line);

        {
            for line in file.read().lines().enumerate().take(end_line.line+1).skip(start_line.line) {
                if line.0 != start_line.line {
                    let _ = writeln!(self.fmt);
                }
                
                // The main line
                let _ = writeln!(self.fmt, 
                    "{:>max_line_padding$} {} {}", 
                    (line.0+1).orange(), "|".orange(), line.1
                );

                
                // The lil' arrows
                {                
                    let _ = write!(self.fmt, 
                        "{} {} ",
                        " ".repeated(max_line_padding),
                        "|".orange(), 
                    );

                    if line.0 == start_line.line && line.0 == end_line.line {                        
                        let _ = write!(
                            self.fmt, "{}{}",
                            " ".repeated(characters_between(
                                line.1, 0, 
                                source.start() as usize - start_line.offset
                            )),
                            
                            "^".repeated(characters_between(
                                line.1, 
                                source.start() as usize - start_line.offset, 
                                source.end() as usize - end_line.offset + 1, 
                            ).max(1)).red(),
                        );
                    } else if line.0 == start_line.line {
                        let _ = write!(
                            self.fmt, "{}{}",
                            " ".repeated(characters_between(
                                line.1, 0, 
                                source.start() as usize - start_line.offset
                            )),
                            
                            "^".repeated(characters_between(
                                line.1, source.start() as usize - start_line.offset, 
                                line.1.len()
                            ).max(1)).red(),
                        );
                    } else if line.0 == end_line.line {
                        let _ = write!(
                            self.fmt, "{}",
                            "^".repeated(characters_between(
                                line.1, 0, 
                                line.1.len() - (source.end() as usize - end_line.offset)
                            ).max(1)).red(),
                        );
                    } else {
                        let _ = write!(
                            self.fmt, "{}",
                            "^".repeated(line.1.len()).red(),
                        );
                    }
                }
            }
            if let Some(note) = note {
                let _ = write!(self.fmt, " {note}");
            }
            
            let _ = writeln!(self.fmt);

        }

        
        let _ = writeln!(self.fmt, 
            "{}{} ",
            " ".repeated(max_line_padding), 
            "---".orange(), 
        );
    }

}



#[derive(Clone, Copy, Debug)]
struct LineAt {
    offset: usize,
    line: usize,
}


impl LineAt {
    const ZERO: Self = Self {
        offset: 0,
        line: 0,
    };
}


fn line_at(offset: usize, data: &str, from: LineAt) -> Option<LineAt> {
    if offset < from.offset {
        return line_at(offset, data, LineAt::ZERO)
    }

    let mut current_offset = from.offset;
    if offset <= current_offset {
        return Some(LineAt {
            offset: current_offset,
            line: from.line,
        })
    }
    
    for line in data.lines().enumerate().skip(from.line) {
        println!("{line:?}");
        current_offset += line.1.len();
        current_offset += 1;
        
        if offset < current_offset {
            return Some(LineAt {
                offset: current_offset - line.1.len() - 1,
                line: line.0,
            })
        }
    }
    
    None
}


fn characters_between(data: &str, start: usize, end: usize) -> usize {
    let slice = &data[start..end];
    slice.len()
}


