use super::DocumentSchema;
use super::Indexer;
use exif::{Rational, Tag, Value};
use std::ffi::OsStr;
use std::path::Path;

use reverse_geocoder::{Locations, ReverseGeocoder};

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

    fn index_file(&self, path: &Path) -> DocumentSchema {
        let file = std::fs::File::open(path).unwrap();
        let reader = exif::Reader::new(&mut std::io::BufReader::new(&file)).unwrap();
        let mut lat_direction = 0_u8 as char;
        let mut lat = 0.0;
        let mut lon_direction = 0_u8 as char;
        let mut lon = 0.0;
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

        let res = GEOCODER.search(&[lat, lon]).unwrap().get(0).unwrap().1;
        DocumentSchema {
            name: String::new(),
            body: format!("{} {} {} {}", res.name, res.admin1, res.admin2, res.admin3),
        }
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

    #[test]
    fn test_indexing_text_file() {
        let test_file_path = Path::new("./test_files/IMG_2551.jpeg");
        let indexed_document = ExifIndexer.index_file(test_file_path);

        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "Pacureti Prahova Comuna Pacureti RO");
    }

    #[test]
    fn test_supports_text_extension() {
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("tif")));
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("tiff")));
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("jpg")));
        assert_eq!(true, ExifIndexer.supports_extension(OsStr::new("jpeg")));
        assert_eq!(false, ExifIndexer.supports_extension(OsStr::new("png")));
    }
}
