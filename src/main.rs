use clap::Parser;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
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

    let mut buf = Vec::new();

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        let item = reader.read_event_into(&mut buf);
        match item {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => {
                let inner = str::from_utf8(e);
                println!("Start(e): {:?}", inner.unwrap());
                for attr in e.attributes() {
                    let attr = match attr {
                        Ok(attr) => attr,
                        Err(ref e) => {
                            println!("Invalid attr {:?}: {:?}", &attr, e);
                            continue;
                        }
                    };
                    let key = str::from_utf8(attr.key.into_inner());
                    let value = str::from_utf8(&attr.value);
                    println!("  [{}]={:?}", key.unwrap(), value.unwrap());
                }
                assert!(writer.write_event(item.unwrap()).is_ok())
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

fn process_attributes<'a>(
    attrs: &mut Attributes,
    buf: &'a mut String,
) -> Result<Attributes<'a>, String> {
    buf.clear();

    for ref attr in attrs {
        let attr = match attr {
            Err(e) => return Err(format!("Invalid attr {:?}: {:?}", &attr.clone(), e)),
            Ok(attr) => attr,
        };

        let key = match str::from_utf8(attr.key.into_inner()) {
            Err(e) => return Err(e.to_string()),
            Ok(key) => key,
        };
        let value = match str::from_utf8(&attr.value) {
            Err(e) => return Err(e.to_string()),
            Ok(value) => value,
        };
        println!("  [{}]={:?}", &key, &value);
        buf.push(' ');
        buf.push_str(&key);
        buf.push('=');
        buf.push_str(&value);
    }

    Ok(Attributes::new(buf.as_str(), 0))
}
