use euclid::{Point2D, Size2D};
//use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use lyon_path::builder::{FlatPathBuilder, SvgPathBuilder};
use lyon_path::default::Path;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

//use image::{DynamicImage, RgbaImage, ImageBuffer};

#[derive(Copy, Clone, Debug)]
struct MyVertex { position: [f32; 2], normal: [f32; 2] }

pub fn load_font() {
    let font = SystemSource::new().select_best_match(&[FamilyName::SansSerif],
                                                     &Properties::new())
        .unwrap()
        .load()
        .unwrap();

    println!("loaded {} font, glyphs {}", font.full_name(), font.glyph_count());


    let glyph_id = font.glyph_for_char('Š').unwrap();

    let mut builder = Path::builder();
    font.outline(glyph_id, HintingOptions::None, &mut builder).expect("outline failed");
    let path = builder.build_and_reset();


    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<MyVertex, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    {
        // Compute the tessellation.
        tessellator.tessellate_path(
            path.path_iter(),
            &FillOptions::default().with_tolerance(100.0),
            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                MyVertex {
                    position: vertex.position.to_array(),
                    normal: vertex.normal.to_array(),
                }
            }),
        ).unwrap();
    }

    // The tessellated geometry is ready to be uploaded to the GPU.
    println!(" -- {} vertices {} indices",
             geometry.vertices.len(),
             geometry.indices.len()
    );

    println!(" -- {:#?} vertices {:#?} indices",
             geometry.vertices,
             geometry.indices
    );


//    let mut canvas = Canvas::new(&Size2D::new(64, 64), Format::Rgba32);
//    font.rasterize_glyph(&mut canvas,
//                         glyph_id,
//                         64.0,
//                         &Point2D::new(1.0, 1.0),
//                         HintingOptions::None,
//                         RasterizationOptions::GrayscaleAa)
//        .unwrap();
//
//    let image = DynamicImage::ImageRgba8(
//        RgbaImage::from(
//            ImageBuffer::from_vec(64, 64, canvas.pixels)
//                .expect("failed to create buffer from vec")
//        )
//    );
//
//    image.save("test-a.png").unwrap();
//
    panic!("loaded");
}