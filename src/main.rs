use clap::Parser;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::fs::File;
use std::io::{BufReader, Cursor};

/// Generates a battery icon with charging status.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the base SVG.
    svg: std::path::PathBuf,
    /// Path to the result image.
    output: std::path::PathBuf,
}

impl Args {
    fn input(&self) -> Result<Reader<BufReader<File>>, std::io::Error> {
        let f = File::open(&self.svg)
            .expect(format!("SVG file {:?} should be readable", &self.svg).as_str());
        let input = BufReader::new(f);
        let reader = Reader::from_reader(input);
        Ok(reader)
    }
}

fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);

    let mut reader = args.input().unwrap();
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let mut buf = Vec::new();

    loop {
        let event = reader.read_event_into(&mut buf);
        match event {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = str::from_utf8(e.name().into_inner()).unwrap();
                println!("Start(e): {:?}", name);

                // Copy the tag.
                let mut elem = BytesStart::new(name);
                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                // Add an attribute.
                elem.push_attribute(("my-key", "Start!"));

                // Write the modified elem back into the document.
                assert!(writer.write_event(Event::Start(elem)).is_ok())
            }
            Ok(Event::Empty(e)) => {
                let name = str::from_utf8(e.name().into_inner()).unwrap();
                println!("Empty(e): {:?}", name);

                // Copy the tag.
                let mut elem = BytesStart::new(name);
                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                // Add an attribute.
                elem.push_attribute(("my-key", "Empty!"));

                // Write the modified elem back into the document.
                assert!(writer.write_event(Event::Empty(elem)).is_ok())
            }
            Ok(Event::Text(e)) => {
                if e.trim_ascii().len() != 0 {
                    assert!(writer.write_event(Event::Text(e)).is_ok());
                }
            }
            Ok(e) => assert!(writer.write_event(e).is_ok()),
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    match String::from_utf8(result) {
        Ok(result) => println!("{}", result),
        Err(e) => panic!("Invalid UTF8 output: {}", e),
    }
}
