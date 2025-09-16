use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use clap::Parser;
use quick_xml::events::attributes::Attribute;
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
        let f = File::open(&self.svg)
            .map_err(|e| format!("SVG file {:?} should be readable: {e}", &self.svg))?;
        let input = BufReader::new(f);
        let reader = Reader::from_reader(input);
        Ok(reader)
    }

    fn output(&self) -> Result<BufWriter<File>, Box<dyn Error>> {
        let f = File::create(&self.output)
            .map_err(|e| format!("output file {:?} should be writable: {e}", &self.output))?;
        let output = BufWriter::new(f);
        Ok(output)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    eprintln!("Args: {:?}", args);

    let mut reader = args.input()?;
    let out_file = args.output()?;
    let mut writer = Writer::new(out_file);
    let mut buf = Vec::new();

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| format!("error at position {}: {e}", reader.error_position()))?;
        match event {
            Event::Eof => break,

            Event::Empty(e) => {
                let new_tag = process_attributes(&e).map_err(|e| {
                    format!("failed to process attributes of self-closing tag: {e}")
                })?;

                // Write the modified elem back into the document.
                writer
                    .write_event(Event::Empty(new_tag))
                    .map_err(|e| format!("failed to write self-closing tag: {e}"))?;
            }

            e => writer
                .write_event(e)
                .map_err(|e| format!("failed to write other element: {e}"))?,
        }
        buf.clear();
    }
    Ok(())
}

fn process_attributes<'a>(tag_in: &'a BytesStart) -> Result<BytesStart<'a>, Box<dyn Error>> {
    let name = str::from_utf8(tag_in.name().into_inner())?;
    let mut tag_out = BytesStart::new(name);
    let attrs: Vec<Attribute> = tag_in
        .attributes()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to collect attributes of {name}: {e}"))?;
    if name != "rect" {
        return Ok(tag_out.with_attributes(attrs));
    }

    let mut attr_map: HashMap<String, String> = HashMap::new();
    // Read the attributes out for modification.
    for attr in &attrs {
        let key = str::from_utf8(attr.key.into_inner())
            .map_err(|e| format!("failed to decode {name} attr key as utf8: {e}"))?
            .to_string();
        let value = str::from_utf8(&attr.value)
            .map_err(|e| format!("failed to decode {name}[{key}] value as utf8: {e}"))?
            .to_string();
        attr_map.insert(key, value);
    }
    // Check the id. Return input unmodified if not present.
    let id = match attr_map.get("id") {
        Some(val) => val.to_string(),
        None => return Ok(tag_out.with_attributes(attrs)),
    };
    eprintln!("[id]={:?}", id);
    if id != "fraction" {
        return Ok(tag_out.with_attributes(attrs));
    }

    battery_fraction(&mut attr_map, 0.5)
        .map_err(|e| format!("failed to battery_fraction({name}#{id}): {e}"))?;

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
    let mut width: f64 = attr_map
        .get("width")
        .ok_or_else(|| "#fraction had no [width]")?
        .parse()
        .map_err(|e| format!("failed to parse #fraction[width]: {e}"))?;
    eprintln!("old width = {:?}", width);
    width *= charge;
    attr_map.insert("width".to_string(), width.to_string());
    eprintln!("new width = {:?}", width);

    // Change the color if low battery.
    if charge < 0.3 {
        let style = attr_map.get("style").map_or("", String::as_str);
        let mut style_map: HashMap<String, String> =
            parse_style_map(style).map_err(|e| format!("in #fraction: {e}"))?;

        let fill = style_map.get("fill").map_or("", String::as_str);
        let new_fill = if charge < 0.15 { "#ff0000" } else { "#ff8000" };
        eprintln!("changing fraction fill {fill:?} -> {new_fill:?}");
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
            return Err(format!("failed to parse style kv: {kv:?}").into());
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
