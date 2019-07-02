use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::OsStr;
use std::io::Cursor;

use std::time::{Duration, Instant};

use tract_core::ndarray;
use tract_core::prelude::*;

lazy_static! {
    static ref LABELS: Vec<&'static str> = {
        let labels: Vec<&str> = include_str!("../../models/imagenet_slim_labels.txt")
                        .lines()
                        .collect();
        labels
    };

    static ref MODEL: Model<TypedTensorInfo> = {
        let now = Instant::now();
        // load the model
        let model_bytes = include_bytes!("../../models/mobilenet_v2_1.4_224_frozen.pb");
        let mut model_bytes = Cursor::new(&model_bytes[..]);
        let mut model = tract_tensorflow::tensorflow().model_for_read(&mut model_bytes).unwrap();

        // specify input type and shape
        model.set_input_fact(0, TensorFact::dt_shape(f32::datum_type(), tvec!(1, 224, 224, 3))).unwrap();

        // optimize the model and get an execution plan
        let model = model.into_optimized().unwrap();
        info!("It took {} microseconds to load and optimize the model", now.elapsed().as_micros());
        model
    };
}

pub struct MobileNetV2Indexer;

impl Indexer for MobileNetV2Indexer {
    // https://github.com/image-rs/image#21-supported-image-formats
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("tif") ||
        extension == OsStr::new("tiff") ||
        extension == OsStr::new("jpg") ||
        extension == OsStr::new("jpeg") ||
        extension == OsStr::new("png") ||
        extension == OsStr::new("bmp") ||
        extension == OsStr::new("ico") ||
        extension == OsStr::new("gif")
    }

    // https://github.com/snipsco/tract/tree/master/examples/tensorflow-mobilenet-v2
    fn index_file(&self, path: &Path) -> DocumentSchema {
        let now = Instant::now();
        let t_model = MODEL.clone();
        let plan = SimplePlan::new(&t_model).unwrap();
        // println!("It took {} microseconds to clone and build the plan", now.elapsed().as_micros());

        // let now = Instant::now();
        // open image, resize it and make a Tensor out of it
        let image = image::open(path).unwrap().to_rgb();
        // println!("It took {} microseconds to load the image from disk", now.elapsed().as_micros());
        // let now = Instant::now();
        let resized = image::imageops::resize(&image, 224, 224, ::image::FilterType::Triangle);
        let image: Tensor = ndarray::Array4::from_shape_fn((1, 224, 224, 3), |(_, y, x, c)| {
            resized[(x as _, y as _)][c] as f32 / 255.0
        })
        .into();
        // println!("It took {} microseconds to pre-process the image", now.elapsed().as_micros());

        let now = Instant::now();
        // run the plan on the input
        let result = plan.run(tvec!(image)).unwrap();
        // println!("It took {} microseconds to run the image on the plan", now.elapsed().as_micros());

        // find and display the max value with its index
        let best = result[0]
            .to_array_view::<f32>().unwrap()
            .iter()
            .cloned()
            .zip(1..)
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        let mut body_res = "";

        if let Some((_score, index)) = best {
            body_res = LABELS.get(index as usize - 1).unwrap();
        }

        // dbg!(body_res);

        DocumentSchema {
            name: String::new(),
            body: body_res.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_indexing_mobile_net_v2_file() {
        let test_file_path = Path::new("./test_files/IMG_2551.jpeg");
        let indexed_document = MobileNetV2Indexer.index_file(test_file_path);

        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "eggnog");
    }

    #[test]
    fn test_supports_mobile_net_v2_extension() {
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("tif")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("tiff")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("jpg")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("jpeg")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("png")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("bmp")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("ico")));
        assert_eq!(true, MobileNetV2Indexer.supports_extension(OsStr::new("gif")));
        assert_eq!(false, MobileNetV2Indexer.supports_extension(OsStr::new("webp")));
    }
}