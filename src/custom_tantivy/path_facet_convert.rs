use std::path::{Path, PathBuf};
use tantivy::schema::*;

/// Converts to/from Facet/PathBuf
pub trait TantivyConvert {
    fn to_facet_value(&self) -> String;
    fn from_facet_value(facet_val: &Facet) -> PathBuf;
}

impl TantivyConvert for Path {
    #[cfg(target_os = "windows")]
    fn to_facet_value(&self) -> String {
        self.canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("\\", "/")
    }

    #[cfg(not(target_os = "windows"))]
    fn to_facet_value(&self) -> String {
        String::from(self.canonicalize().unwrap().to_str().unwrap())
    }

    #[cfg(target_os = "windows")]
    fn from_facet_value(facet_val: &Facet) -> PathBuf {
        Path::new(
            &facet_val
                .encoded_str()
                .replace(char::from(0), "/")
                .replacen("/?/", "", 1),
        )
        .to_path_buf()
    }

    #[cfg(not(target_os = "windows"))]
    fn from_facet_value(facet_val: &Facet) -> PathBuf {
        let mut location = String::from("/");
        location.push_str(&facet_val.encoded_str().replace(char::from(0), "/"));
        Path::new(&location).to_path_buf()
    }
}

mod test {
    #[test]
    fn test_path_facet_conversion() {
        use super::*;
        use std::env;
        use std::fs::File;

        let mut current_dir = env::current_dir().unwrap();
        current_dir.push("Cargo.toml");
        println!("{:?}", current_dir);

        let current_dir_facet_string = current_dir.to_facet_value();
        println!("{:?}", current_dir_facet_string);

        let facet = Facet::from_text(&current_dir_facet_string);
        println!("{:?}", facet);

        let dir_from_facet = Path::from_facet_value(&facet);
        println!("{:?}", dir_from_facet);

        File::open(dir_from_facet).unwrap();
    }
}
