use super::DocumentSchema;
use super::Indexer;
use crate::contracts::file_to_process::FileToProcess;
use crate::error_adapter::log_and_return_error_string;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::Cursor;
use std::time::Instant;

use anyhow::{Context, Result};
use image::ImageFormat;
use once_cell::sync::Lazy;
use tract_core::ndarray;
use tract_tensorflow::prelude::*;

static MODEL: Lazy<TypedModel> = Lazy::new(|| {
    let now = Instant::now();
    // load the model
    let model_bytes = include_bytes!("../../models/mobilenet_v2_1.4_224_frozen.pb");
    let mut model_bytes = Cursor::new(&model_bytes[..]);
    let mut model = tract_tensorflow::tensorflow()
        .model_for_read(&mut model_bytes)
        .unwrap();

    // .expect(&log_and_return_error_string(
    //     "mobile_net_v2_indexer: Failed to read model from bytes".to_string(),
    // ));

    // specify input type and shape
    model
        .set_input_fact(
            0,
            InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 224, 224, 3)),
        )
        .unwrap();

    // .expect(&log_and_return_error_string(
    //     "mobile_net_v2_indexer: Failed to specify input type and shape for model"
    //         .to_string(),
    // ));

    // optimize the model and get an execution plan
    let model = model.into_optimized().unwrap();

    // .expect("mobile_net_v2_indexer: Failed to optimize model");
    info!(
        "It took {} microseconds to load and optimize the model",
        now.elapsed().as_micros()
    );
    model
});

static LABELS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    let labels: Vec<&str> = include_str!("../../models/imagenet_slim_labels.txt")
        .lines()
        .collect();
    labels
});

static IMAGE_FORMATS: Lazy<HashMap<OsString, image::ImageFormat>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(OsString::from("tif"), ImageFormat::Tiff);
    map.insert(OsString::from("tiff"), ImageFormat::Tiff);
    map.insert(OsString::from("jpg"), ImageFormat::Jpeg);
    map.insert(OsString::from("jpeg"), ImageFormat::Jpeg);
    map.insert(OsString::from("png"), ImageFormat::Png);
    map.insert(OsString::from("bmp"), ImageFormat::Bmp);
    map.insert(OsString::from("ico"), ImageFormat::Ico);
    map.insert(OsString::from("gif"), ImageFormat::Gif);
    map
});

pub struct MobileNetV2Indexer;

impl Indexer for MobileNetV2Indexer {
    // https://github.com/image-rs/image#21-supported-image-formats
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("tif")
            || extension == OsStr::new("tiff")
            || extension == OsStr::new("jpg")
            || extension == OsStr::new("jpeg")
            || extension == OsStr::new("png")
            || extension == OsStr::new("bmp")
            || extension == OsStr::new("ico")
            || extension == OsStr::new("gif")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![
            OsString::from("tif"),
            OsString::from("tiff"),
            OsString::from("jpg"),
            OsString::from("jpeg"),
            OsString::from("png"),
            OsString::from("bmp"),
            OsString::from("ico"),
            OsString::from("gif"),
        ]
    }

    // https://github.com/snipsco/tract/tree/master/examples/tensorflow-mobilenet-v2
    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let now = Instant::now();
        let t_model: &TypedModel = &*MODEL;
        let plan = TypedSimplePlan::new(t_model).unwrap();

        // .expect(&log_and_return_error_string(format!(
        //     "mobile_net_v2_indexer: Failed to create plan for model"
        // )));

        info!(
            "It took {} microseconds to build the plan",
            now.elapsed().as_micros()
        );

        let now = Instant::now();
        let image_format = IMAGE_FORMATS
            .get(&file_to_process.path.extension().unwrap().to_os_string())
            .unwrap()
            .clone();
        // open image, resize it and make a Tensor out of it
        // image crate seems to be more tolerant to malformed image filies using the open function
        //let image = image::io::Reader::with_format(Cursor::new(&file_to_process.contents), image_format).decode()

        let image = image::open(&file_to_process.path).unwrap();
        // .with_context(|| {
        //     log_and_return_error_string(format!(
        //         "mobile_net_v2_indexer: Failed to load from memory image at path: {:?}",
        //         file_to_process.path
        //     ))
        // })?;

        info!(
            "It took {} microseconds to load the image from memory",
            now.elapsed().as_micros()
        );
        let now = Instant::now();
        let resized =
            image::imageops::resize(&image, 224, 224, image::imageops::FilterType::Triangle);
        let image: Tensor = ndarray::Array4::from_shape_fn((1, 224, 224, 3), |(_, y, x, c)| {
            f32::from(resized[(x as _, y as _)][c]) / 255.0
        })
        .into();
        info!(
            "It took {} microseconds to pre-process the image",
            now.elapsed().as_micros()
        );

        let now = Instant::now();
        // run the plan on the input
        let result = plan.run(tvec!(image)).unwrap();
        // .expect(&log_and_return_error_string(
        //     "mobile_net_v2_indexer: Failed to run the image through the model".to_string(),
        // ));
        info!(
            "It took {} microseconds to run the image on the plan",
            now.elapsed().as_micros()
        );

        // find and display the max value with its index
        let best = result[0]
            .to_array_view::<f32>()
            .unwrap()
            // .expect(&log_and_return_error_string(
            //     "mobile_net_v2_indexer: Failed to convert to array view".to_string(),
            // ))
            .iter()
            .cloned()
            .zip(1..)
            .max_by(|a, b| {
                a.0.partial_cmp(&b.0).unwrap()
                // .expect(&log_and_return_error_string(
                //     "mobile_net_v2_indexer: Failed to partial compare while sorting".to_string(),
                // ))
            });

        let mut body_res = "";

        if let Some((_score, index)) = best {
            body_res = LABELS.get(index as usize - 1).unwrap();
            // .with_context(|| {
            //     log_and_return_error_string(format!(
            //         "mobile_net_v2_indexer: Failed to get label associated with model result"
            //     ))
            // })?;
        }

        Ok(DocumentSchema {
            name: String::new(),
            body: body_res.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::file_to_process::newFileToProcess;
    use std::path::Path;

    #[cfg(not(target_os = "windows"))]
    #[tokio::test(core_threads = 1)]
    async fn test_indexing_mobile_net_v2_file() {
        let test_file_path = Path::new("./test_files/IMG_2551.jpeg");
        let indexed_document = MobileNetV2Indexer
            .index_file(&newFileToProcess(test_file_path).await)
            .unwrap();

        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "eggnog");
    }

    #[test]
    fn test_supports_mobile_net_v2_extension() {
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("tif"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("tiff"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("jpg"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("jpeg"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("png"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("bmp"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("ico"))
        );
        assert_eq!(
            true,
            MobileNetV2Indexer.supports_extension(OsStr::new("gif"))
        );
        assert_eq!(
            false,
            MobileNetV2Indexer.supports_extension(OsStr::new("webp"))
        );
    }
}
