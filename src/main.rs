use clap::Parser;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::collections::HashMap;
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
    eprintln!("Args: {:?}", args);

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
                eprintln!("Start(e): {:?}", name);

                // Copy the tag.
                let mut elem = BytesStart::new(name);
                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                // Add an attribute.
                elem.push_attribute(("my-key", "Start!"));

                // Write the modified elem back into the document.
                assert!(writer.write_event(Event::Start(elem)).is_ok())
            }
            Ok(Event::Empty(e)) => {
                let new_tag = match process_attributes(&e) {
                    Err(e) => panic!("failed to process_attributes: {:?}", e),
                    Ok(tag) => tag,
                };
                // Write the modified elem back into the document.
                assert!(writer.write_event(Event::Empty(new_tag)).is_ok())
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

fn process_attributes<'a>(tag_in: &'a BytesStart) -> Result<BytesStart<'a>, String> {
    let name = match str::from_utf8(tag_in.name().into_inner()) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };
    let mut tag_out = BytesStart::new(name);
    let attrs = tag_in.attributes().map(|attr| attr.unwrap());
    if name != "rect" {
        return Ok(tag_out.with_attributes(attrs));
    }

    let mut attr_map: HashMap<String, String> = HashMap::new();
    // Read the attributes out for modification.
    for attr in attrs {
        let key = str::from_utf8(attr.key.into_inner()).unwrap().to_string();
        let value = str::from_utf8(&attr.value).unwrap().to_string();
        attr_map.insert(key, value);
    }
    // Refresh attrs for later use.
    let attrs = tag_in.attributes().map(|attr| attr.unwrap());
    // Check the id.
    let id = match attr_map.get("id") {
        Some(val) => val,
        None => return Ok(tag_out.with_attributes(attrs)),
    };
    eprintln!("[id]={:?}", id);
    if id != &"fraction" {
        return Ok(tag_out.with_attributes(attrs));
    }

    match battery_fraction(&mut attr_map) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!(
                "failed to battery_fraction(rect#fraction): {:?}",
                e
            ))
        }
    }

    // Write the modified attributes into the result.
    for (key, value) in attr_map {
        tag_out.push_attribute((key.as_str(), value.as_str()));
    }
    Ok(tag_out)
}

// battery_fraction takes a HashMap of attributes for a <rect /> tag,
// and scales its width from 100% to the percentage of the remaining charge.
fn battery_fraction(attr_map: &mut HashMap<String, String>) -> Result<(), String> {
    // Check the width.
    let mut width: f64 = match attr_map.get("width") {
        Some(s) => match s.parse() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("failed to parse #fraction[width]: {:?}", e);
                return Err(e.to_string());
            }
        },
        None => return Err("#fraction had no [width]".to_string()),
    };
    let battery_fraction = 0.33;
    eprintln!("old width = {:?}", width);
    width *= battery_fraction;
    attr_map.insert("width".to_string(), width.to_string());
    eprintln!("new width = {:?}", width);
    Ok(())
}
