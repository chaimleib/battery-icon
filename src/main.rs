use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use clap::Parser;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

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
    fn input(&self) -> Result<Reader<BufReader<File>>, Box<dyn Error>> {
        let f = match File::open(&self.svg) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("SVG file {:?} should be readable: {:?}", &self.svg, e).into())
            }
        };
        let input = BufReader::new(f);
        let reader = Reader::from_reader(input);
        Ok(reader)
    }

    fn output(&self) -> Result<BufWriter<File>, Box<dyn Error>> {
        let f = match File::create(&self.output) {
            Ok(f) => f,
            Err(e) => {
                return Err(
                    format!("output file {:?} should be writable: {:?}", &self.output, e).into(),
                )
            }
        };
        let output = BufWriter::new(f);
        Ok(output)
    }
}

fn main() {
    let args = Args::parse();
    eprintln!("Args: {:?}", args);

    let mut reader = args.input().unwrap();
    let out_file = args.output().unwrap();
    let mut writer = Writer::new(out_file);
    let mut buf = Vec::new();

    loop {
        let event = reader.read_event_into(&mut buf);
        match event {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => {
                let new_tag = process_attributes(&e).expect("failed to process attributes");

                // Write the modified elem back into the document.
                assert!(writer.write_event(Event::Empty(new_tag)).is_ok())
            }
            Ok(e) => assert!(writer.write_event(e).is_ok()),
        }
        buf.clear();
    }
}

fn process_attributes<'a>(tag_in: &'a BytesStart) -> Result<BytesStart<'a>, Box<dyn Error>> {
    let name = str::from_utf8(tag_in.name().into_inner())?;
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

    match battery_fraction(&mut attr_map, 0.5) {
        Ok(_) => (),
        Err(e) => return Err(format!("failed to battery_fraction(rect#fraction): {:?}", e).into()),
    }

    // Write the modified attributes into the result.
    for (key, value) in attr_map {
        tag_out.push_attribute((key.as_str(), value.as_str()));
    }
    Ok(tag_out)
}

// battery_fraction adjusts a HashMap of attributes for a <rect /> tag.
// It scales its width from 100% to the percentage of the remaining charge.
// It also changes its color if the remaining charge is too low.
fn battery_fraction(
    attr_map: &mut HashMap<String, String>,
    charge: f64,
) -> Result<(), Box<dyn Error>> {
    // Check the width.
    let mut width: f64 = match attr_map.get("width") {
        Some(s) => match s.parse() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("failed to parse #fraction[width]: {:?}", e);
                return Err(e.into());
            }
        },
        None => return Err("#fraction had no [width]".into()),
    };
    eprintln!("old width = {:?}", width);
    width *= charge;
    attr_map.insert("width".to_string(), width.to_string());
    eprintln!("new width = {:?}", width);

    // Change the color if low battery.
    if charge < 0.3 {
        let style = match attr_map.get("style") {
            Some(s) => s,
            None => "",
        };
        let mut style_map: HashMap<String, String> = match parse_style_map(style) {
            Err(e) => {
                eprintln!("in #fraction: {}", e);
                HashMap::new()
            }
            Ok(m) => m,
        };
        let new_fill = if charge < 0.15 { "#ff0000" } else { "#ff8000" };
        let fill = match style_map.get("fill") {
            Some(s) => s,
            None => "",
        };
        eprintln!("changing fraction fill {:?} -> {:?}", fill, new_fill);
        style_map.insert("fill".to_string(), new_fill.to_string());
        let new_style = map_as_style(&style_map);
        attr_map.insert("style".to_string(), new_style);
    }
    Ok(())
}

// parse_style_map converts an SVG style attribute into a key-value map.
fn parse_style_map(style: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut map = HashMap::new();
    for kv in style.split(";") {
        let kv: Vec<&str> = kv.trim().splitn(2, ":").collect();
        if kv.len() != 2 {
            return Err(format!("failed to parse style kv: {:?}", kv).into());
        }
        let key = kv[0];
        let value = kv[1];
        map.insert(key.to_string(), value.to_string());
    }
    Ok(map)
}

// map_as_style converts a key-value map into an SVG style attribute.
fn map_as_style(map: &HashMap<String, String>) -> String {
    let mut style = String::new();
    for (k, v) in map {
        style.push(';');
        style.push_str(k);
        style.push(':');
        style.push_str(v);
    }
    if style.len() == 0 {
        return "".to_string();
    }
    style.trim_start_matches(';').to_string()
}
