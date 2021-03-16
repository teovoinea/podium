use contracts::file_to_process::FileToProcess;
use contracts::indexer::{DocumentSchema, Indexer};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::Cursor;

use common::anyhow;
use common::anyhow::{Error, Result};
use common::tracing::{span, Level};
use image::ImageFormat;
use once_cell::sync::Lazy;
use tract_core::ndarray;
use tract_tensorflow::prelude::*;

static MODEL: Lazy<TypedModel> = Lazy::new(|| {
    span!(Level::INFO, "mobile_net_v2_indexer: Preparing typed model").in_scope(|| {
        let mut model = span!(Level::INFO, "mobile_net_v2_indexer: Loading model").in_scope(|| {
            // load the model
            let model_bytes = include_bytes!("../../../../models/mobilenet_v2_1.4_224_frozen.pb");
            let mut model_bytes = Cursor::new(&model_bytes[..]);
            tract_tensorflow::tensorflow()
                .model_for_read(&mut model_bytes)
                .unwrap()
            // .expect(&log_and_return_error_string(
            //     "mobile_net_v2_indexer: Failed to read model from bytes".to_string(),
            // ));
        });

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
        let model = span!(Level::INFO, "mobile_net_v2_indexer: Optimize model")
            .in_scope(|| model.into_optimized().unwrap());

        // .expect("mobile_net_v2_indexer: Failed to optimize model");
        model
    })
});

static LABELS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    let labels: Vec<&str> = include_str!("../../../../models/imagenet_slim_labels.txt")
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
        let path = file_to_process.path.to_str().unwrap();
        span!(Level::INFO, "mobile_net_v2_indexer: indexing image file", path).in_scope(|| {
            let t_model: &TypedModel = &*MODEL;
            let plan = span!(Level::INFO, "mobile_net_v2_indexer: Creating plan").in_scope(|| {
                match TypedSimplePlan::new(t_model) {
                    Ok(plan) => Ok(plan),
                    Err(e) => Err(anyhow::anyhow!(format!(
                        "mobile_net_v2_indexer: Failed to create plan for model with additional error info {:?}",
                        e
                    )))
                }
            })?;

            let image = span!(Level::INFO, "mobile_net_v2_indexer: Load image").in_scope(|| {
                let image_format = match IMAGE_FORMATS.get(&file_to_process.path.extension().unwrap().to_os_string()) {
                    Some(image_format) => Ok(image_format),
                    None => Err(anyhow::anyhow!(format!(
                        "mobile_net_v2_indexer: Failed to recognize image format",
                    )))
                }?.clone();

                // open image, resize it and make a Tensor out of it
                match image::io::Reader::with_format(Cursor::new(&file_to_process.contents), image_format).decode() {
                    Ok(image) => Ok(image),
                    Err(e) => Err(anyhow::anyhow!(format!(
                        "mobile_net_v2_indexer: Failed to load image with format with additional error info {:?}",
                        e
                    )))
                }
                // image crate seems to be more tolerant to malformed image filies using the open function
            })?;

            let image: Tensor = span!(Level::INFO, "mobile_net_v2_indexer: Pre-process image").in_scope(|| {
                let resized =
                    image::imageops::resize(&image, 224, 224, image::imageops::FilterType::Triangle);

                ndarray::Array4::from_shape_fn((1, 224, 224, 3), |(_, y, x, c)| {
                    f32::from(resized[(x as _, y as _)][c]) / 255.0
                })
                .into()
            });

            // run the plan on the input
            let result = span!(Level::INFO, "mobile_net_v2_indexer: Run image through model").in_scope(||{
                match plan.run(tvec!(image)) {
                    Ok(result) => Ok(result),
                    Err(e) => Err(anyhow::anyhow!(format!(
                        "mobile_net_v2_indexer: Failed to run the image through the model with additional error info {:?}",
                        e
                    )))
                }
            })?;

            let body_res = span!(Level::INFO, "mobile_net_v2_indexer: Map model output").in_scope(|| -> Result<&&str, Error> {
                // find and display the max value with its index
                let best = match result[0].to_array_view::<f32>() {
                    Ok(arr) => Ok(arr),
                    Err(e) => Err(anyhow::anyhow!(format!(
                        "mobile_net_v2_indexer: Failed to convert to array view with additional error info {:?}",
                        e
                    )))
                }?
                .iter()
                .cloned()
                .zip(1..)
                .max_by(|a, b| {
                    a.0.partial_cmp(&b.0).unwrap()
                    // .expect(&log_and_return_error_string(
                    //     "mobile_net_v2_indexer: Failed to partial compare while sorting".to_string(),
                    // ))
                });

                if let Some((_score, index)) = best {
                    return Ok(LABELS.get(index as usize - 1).unwrap());
                    // .with_context(|| {
                    //     log_and_return_error_string(format!(
                    //         "mobile_net_v2_indexer: Failed to get label associated with model result"
                    //     ))
                    // })?;
                }

                Ok(&"")
            })?;

            Ok(DocumentSchema {
                name: file_to_process.path(),
                body: body_res.to_string(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::tokio;
    use contracts::file_to_process::new_file_to_process;
    use std::path::Path;

    #[tokio::test]
    async fn test_indexing_mobile_net_v2_file() {
        let test_file_path = Path::new("../../../test_files/IMG_2551.jpeg");
        let indexed_document = MobileNetV2Indexer
            .index_file(&new_file_to_process(test_file_path).await)
            .unwrap();

        assert_eq!(indexed_document.name, "../../../test_files/IMG_2551.jpeg");
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
