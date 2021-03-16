use common::error_adapter::log_and_return_error_string;
use contracts::file_to_process::FileToProcess;
use contracts::indexer::{DocumentSchema, Indexer};
use std::ffi::{OsStr, OsString};
use std::io::Cursor;

use common::anyhow::{Context, Error, Result};
use common::tracing::{span, Level};
use exif::{Rational, Tag, Value};
use reverse_geocoder::{Locations, Record, ReverseGeocoder};

lazy_static! {
    static ref LOCATIONS: Locations = Locations::from_memory();
    static ref GEOCODER: ReverseGeocoder<'static> = ReverseGeocoder::new(&LOCATIONS);
}

pub struct ExifIndexer;

impl Indexer for ExifIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("tif")
            || extension == OsStr::new("tiff")
            || extension == OsStr::new("jpg")
            || extension == OsStr::new("jpeg")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![
            OsString::from("tif"),
            OsString::from("tifd"),
            OsString::from("jpg"),
            OsString::from("jpeg"),
        ]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let path = file_to_process.path.to_str().unwrap();
        span!(Level::INFO, "exif_indexer: indexing image file", path).in_scope(|| {
            let reader = span!(Level::INFO, "exif_indexer: Loading exif data from image from memory").in_scope(|| {
                exif::Reader::new(&mut Cursor::new(&file_to_process.contents)).with_context(|| {
                    log_and_return_error_string(format!(
                        "exif_indexer: Failed to initialize exif reader for file at path: {:?}",
                        file_to_process.path
                    ))
                })
            })?;

            let mut lat = 0.0;
            let mut lon = 0.0;

            span!(Level::INFO, "exif_indexer: Processing exif fields").in_scope(|| {
                let mut lat_direction = 0_u8 as char;
                let mut lon_direction = 0_u8 as char;
                for f in reader.fields() {
                    match f.tag {
                        Tag::GPSLatitudeRef => {
                            if let Value::Ascii(val) = &f.value {
                                lat_direction = val[0][0] as char;
                            }
                        }
                        Tag::GPSLatitude => {
                            if let Value::Rational(val) = &f.value {
                                lat = value_to_deg(val);
                            }
                        }
                        Tag::GPSLongitudeRef => {
                            if let Value::Ascii(val) = &f.value {
                                lon_direction = val[0][0] as char;
                            }
                        }
                        Tag::GPSLongitude => {
                            if let Value::Rational(val) = &f.value {
                                lon = value_to_deg(val);
                            }
                        }
                        _ => {}
                    }
                }

                if lat_direction != 'N' {
                    lat *= -1.0;
                }

                if lon_direction != 'E' {
                    lon *= -1.0;
                }
            });

            let res = span!(Level::INFO, "exif_indexer: Look up the coordinates").in_scope(|| -> Result<&&Record, Error>{
                Ok(
                    GEOCODER.search(&[lat, lon])
                        .with_context(|| log_and_return_error_string(format!("exif_indexer: Failed to search for location in geocoder: lat = {:?} lon = {:?}", lat, lon)))?
                        .get(0)
                        .with_context(|| log_and_return_error_string(format!("exif_indexer: Failed to get first result from search in geocoder")))?
                        .1
                )
            })?;

            Ok(DocumentSchema {
                name: String::new(),
                body: format!("{} {} {} {}", res.name, res.admin1, res.admin2, res.admin3),
            })
        })
    }
}

fn value_to_deg(val: &[Rational]) -> f64 {
    def_to_dec_dec(val[0].to_f64(), val[1].to_f64(), val[2].to_f64())
}

fn def_to_dec_dec(deg: f64, min: f64, sec: f64) -> f64 {
    deg + min / 30.0 + sec / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::tokio;
    use contracts::file_to_process::new_file_to_process;
    use std::path::Path;

    #[tokio::test]
    async fn test_indexing_exif_file() {
        let test_file_path = Path::new("../../../test_files/IMG_2551.jpeg");
        let indexed_document = ExifIndexer
            .index_file(&new_file_to_process(test_file_path).await)
            .unwrap();

        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "Pacureti Prahova Comuna Pacureti RO");
    }

    #[test]
    fn test_supports_exif_extension() {
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("tif")));
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("tiff")));
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("jpg")));
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("jpeg")));
        assert_eq!(false, ExifIndexer.supports_extension(OsStr::new("png")));
    }
}
