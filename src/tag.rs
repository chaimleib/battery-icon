use std::error::Error;

use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesEnd, BytesStart};
use quick_xml::name::QName;

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub id: String,
}

impl Tag {
    pub fn new(b: &dyn TagBytes) -> Result<Tag, Box<dyn Error>> {
        let name = str::from_utf8(b.name().into_inner())?.to_string();
        let id_attr = b.attributes().filter_map(|attr| attr.ok()).find(|attr| {
            let key = str::from_utf8(attr.key.into_inner()).unwrap_or("");
            key == "id"
        });
        let id = if let Some(attr) = id_attr {
            str::from_utf8(&attr.value).unwrap_or("").to_string()
        } else {
            "".to_string()
        };
        let result = Tag { name, id };
        Ok(result)
    }
}

pub trait TagBytes {
    fn name(&self) -> QName<'_>;
    fn attributes(&self) -> Attributes<'_>;
}

impl TagBytes for BytesStart<'_> {
    fn name(&self) -> QName<'_> {
        self.name()
    }

    fn attributes(&self) -> Attributes<'_> {
        self.attributes()
    }
}

impl TagBytes for BytesEnd<'_> {
    fn name(&self) -> QName<'_> {
        self.name()
    }

    fn attributes(&self) -> Attributes<'_> {
        Attributes::new("", 0)
    }
}
