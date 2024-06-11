//! Stub plugin for XMP Metadata streams, to be expanded later

use crate::OffsetDateTime;
use lopdf;

use utils::random_character_string_32;
use PdfConformance;

/// Initial struct for Xmp metatdata. This should be expanded later for XML handling, etc.
/// Right now it just fills out the necessary fields
#[derive(Debug, Clone)]
pub struct XmpMetadata {
    /// Document ID
    pub document_id: String,
    /// Instance ID
    pub instance_id: Option<String>,
    /// Web-viewable or "default" or to be left empty. Usually "default".
    pub rendition_class: Option<String>,
    /// Document version
    pub document_version: u32,
}

impl XmpMetadata {
    /// Creates a new XmpMetadata object
    pub fn new(rendition_class: Option<String>, document_version: u32) -> Self {
        let document_id: String = random_character_string_32();
        Self {
            document_id: document_id,
            instance_id: None,
            rendition_class: rendition_class,
            document_version: document_version,
        }
    }

    /// Consumes the XmpMetadata and turns it into a PDF Object.
    /// This is similar to the
    pub(in types) fn into_obj<S>(
        self,
        conformance: PdfConformance,
        trapping: bool,
        creation_date: OffsetDateTime,
        modification_date: OffsetDateTime,
        metadata_date: OffsetDateTime,
        document_title: S,
    ) -> lopdf::Object
    where
        S: Into<String> + ::std::fmt::Display,
    {
        use lopdf::Object::*;
        use lopdf::{Dictionary as LoDictionary, Stream as LoStream};
        use std::iter::FromIterator;

        // Shared between XmpMetadata and DocumentInfo
        let trapping = if trapping { "True" } else { "False" };

        // let xmp_instance_id = "2898d852-f86f-4479-955b-804d81046b19";
        let instance_id = self
            .instance_id
            .unwrap_or_else(|| random_character_string_32());
        let create_date = to_pdf_xmp_date(creation_date);
        let modification_date = to_pdf_xmp_date(modification_date);
        let metadata_date = to_pdf_xmp_date(metadata_date);

        let pdf_x_version = conformance.get_identifier_string();
        let document_version = self.document_version.to_string();
        let document_id = self.document_id.to_string();

        let rendition_class = match self.rendition_class {
            Some(class) => class,
            None => "".to_string(),
        };

        let xmp_metadata = format!(
            include_str!("../../../templates/catalog_xmp_metadata.txt"),
            create_date,
            modification_date,
            metadata_date,
            document_title,
            document_id,
            instance_id,
            rendition_class,
            document_version,
            pdf_x_version,
            trapping
        );

        Stream(LoStream::new(
            LoDictionary::from_iter(vec![("Type", "Metadata".into()), ("Subtype", "XML".into())]),
            xmp_metadata.as_bytes().to_vec(),
        ))
    }
}

// D:2018-09-19T10:05:05+00'00'
fn to_pdf_xmp_date(date: OffsetDateTime) -> String {
    // Since the time is in UTC, we know that the time zone
    // difference to UTC is 0 min, 0 sec, hence the 00'00
    format!(
        "D:{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00'00'",
        date.year(),
        date.month(),
        date.day(),
        date.hour(),
        date.minute(),
        date.second(),
    )
}
