//! Example plug-in: a "hello world" image procedure and a loader/exporter for
//! a minimal raw format (`.rawrgba`: the magic `RGBA`, width and height as
//! little-endian u32, followed by 8 bit RGBA pixels).

// GIMP runs plug-ins in the background, without a console window.
#![windows_subsystem = "windows"]

use gimp::{prelude::*, subclass::prelude::*};

const HELLO_PROC: &str = "plug-in-gimp-rs-hello";
const LOAD_PROC: &str = "file-gimp-rs-rawrgba-load";
const EXPORT_PROC: &str = "file-gimp-rs-rawrgba-export";

const MAGIC: &[u8; 4] = b"RGBA";
const HEADER_LEN: usize = 12;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ExamplePlugIn;

    #[glib::object_subclass]
    impl ObjectSubclass for ExamplePlugIn {
        const NAME: &'static str = "GimpRsExamplePlugIn";
        type Type = super::ExamplePlugIn;
        type ParentType = gimp::PlugIn;
    }

    impl ObjectImpl for ExamplePlugIn {}

    impl PlugInImpl for ExamplePlugIn {
        fn query_procedures(&self) -> Vec<String> {
            vec![HELLO_PROC.into(), LOAD_PROC.into(), EXPORT_PROC.into()]
        }

        fn create_procedure(&self, name: &str) -> Option<gimp::Procedure> {
            let plug_in = &*self.obj();
            let procedure: gimp::Procedure = match name {
                HELLO_PROC => {
                    let procedure = gimp::ImageProcedure::new(
                        plug_in,
                        name,
                        gimp::PDBProcType::Plugin,
                        |procedure, _run_mode, image, drawables, _config| {
                            gimp::message(&format!(
                                "Hello from Rust! The image is {}x{} with {} selected drawable(s).",
                                image.width(),
                                image.height(),
                                drawables.len()
                            ));
                            procedure.new_return_values(gimp::PDBStatusType::Success, None)
                        },
                    );
                    procedure.set_image_types("*");
                    procedure.set_menu_label("Hello from _Rust");
                    procedure.add_menu_path("<Image>/Filters/Development/Rust");
                    procedure.upcast()
                }
                LOAD_PROC => {
                    let procedure = gimp::LoadProcedure::new(
                        plug_in,
                        name,
                        gimp::PDBProcType::Plugin,
                        |procedure, _run_mode, file, _metadata, _flags, _config| {
                            procedure.return_result(load(file).and_then(|image| {
                                let values =
                                    procedure.new_return_values(gimp::PDBStatusType::Success, None);
                                values.set(1, &image).map_err(error)?;
                                Ok(values)
                            }))
                        },
                    );
                    procedure.set_menu_label("Raw RGBA image");
                    procedure.set_format_name("Raw RGBA");
                    procedure.set_extensions("rawrgba");
                    procedure.set_magics("0,string,RGBA");
                    procedure.upcast()
                }
                EXPORT_PROC => {
                    let procedure = gimp::ExportProcedure::new(
                        plug_in,
                        name,
                        gimp::PDBProcType::Plugin,
                        false,
                        |procedure, _run_mode, image, file, options, _metadata, _config| {
                            procedure.return_result(export(image, file, options).map(|()| {
                                procedure.new_return_values(gimp::PDBStatusType::Success, None)
                            }))
                        },
                    );
                    procedure.set_image_types("RGB*");
                    procedure.set_menu_label("Raw RGBA image");
                    procedure.set_format_name("Raw RGBA");
                    procedure.set_extensions("rawrgba");
                    procedure.set_capabilities(
                        gimp::ExportCapabilities::CAN_HANDLE_RGB
                            | gimp::ExportCapabilities::CAN_HANDLE_ALPHA,
                        None,
                    );
                    procedure.upcast()
                }
                _ => return None,
            };
            procedure.set_attribution("gimp-rs", "MIT", "2026");
            Some(procedure)
        }

        // The example ships no translations.
        fn set_i18n(&self, _procedure_name: &str) -> Option<I18n> {
            None
        }
    }
}

glib::wrapper! {
    pub struct ExamplePlugIn(ObjectSubclass<imp::ExamplePlugIn>) @extends gimp::PlugIn;
}

gimp::main!(ExamplePlugIn);

fn error(message: impl std::fmt::Display) -> glib::Error {
    glib::Error::new(gimp::PlugInError::Failed, &message.to_string())
}

fn load(file: &gio::File) -> Result<gimp::Image, glib::Error> {
    let (data, _etag) = file.load_contents(gio::Cancellable::NONE)?;
    if data.len() < HEADER_LEN || &data[..4] != MAGIC {
        return Err(error("Not a raw RGBA file"));
    }
    let width = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let height = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let pixels = &data[HEADER_LEN..];
    if pixels.len() != width as usize * height as usize * 4 {
        return Err(error(format!("Expected {width}x{height} RGBA pixels")));
    }

    let image = gimp::Image::with_precision(
        width as i32,
        height as i32,
        gimp::ImageBaseType::Rgb,
        gimp::Precision::U8NonLinear,
    );
    let name = file.basename().map(|p| p.display().to_string());
    let layer = gimp::Layer::new(
        &image,
        name.as_deref(),
        width as i32,
        height as i32,
        gimp::ImageType::RgbaImage,
        100.0,
        image.default_new_layer_mode(),
    );
    image.insert_layer(&layer, gimp::Layer::NONE, 0);

    let buffer = layer.buffer().ok_or_else(|| error("Layer has no buffer"))?;
    let format = gimp::babl::Format::format_with_space("R'G'B'A u8", &layer.format().space());
    let rect = gimp::gegl::Rectangle::new(0, 0, width, height);
    buffer.set_pixels(&rect, 0, &format, pixels, None);
    buffer.flush();

    Ok(image)
}

fn export(
    image: &gimp::Image,
    file: &gio::File,
    options: &gimp::ExportOptions,
) -> Result<(), glib::Error> {
    // Flattens, converts etc. according to the export capabilities.
    let image = options.image(image);
    let drawable = image
        .layers()
        .into_iter()
        .next()
        .ok_or_else(|| error("Image has no layers"))?;

    let (width, height) = (drawable.width() as u32, drawable.height() as u32);
    let buffer = drawable
        .buffer()
        .ok_or_else(|| error("Layer has no buffer"))?;
    let format = gimp::babl::Format::format_with_space("R'G'B'A u8", &drawable.format().space());
    let rect = gimp::gegl::Rectangle::new(0, 0, width, height);
    let pixels = buffer.get_pixels(&rect, 1.0, &format, gimp::gegl::AbyssPolicy::None);

    let mut data = Vec::with_capacity(HEADER_LEN + pixels.len());
    data.extend_from_slice(MAGIC);
    data.extend_from_slice(&width.to_le_bytes());
    data.extend_from_slice(&height.to_le_bytes());
    data.extend_from_slice(&pixels);
    file.replace_contents(
        &data,
        None,
        false,
        gio::FileCreateFlags::REPLACE_DESTINATION,
        gio::Cancellable::NONE,
    )?;
    Ok(())
}
