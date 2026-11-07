use cosmic_text::{
    fontdb, Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, Weight,
};
use hayro::{
    hayro_interpret::{InterpreterSettings, RenderContent},
    hayro_syntax::Pdf,
    vello_cpu::color::palette::css::WHITE,
    PixmapSettings, RenderCache, RenderSettings,
};
use image::{ImageBuffer, Rgba};
use impulsor3000::app_config::{
    DEFAULT_LINE_SPACING_PERCENT, MAX_LINE_SPACING_PERCENT, MIN_LINE_SPACING_PERCENT,
};
use pdf_oxide::{editor::DocumentEditor, PdfDocument};
use std::{error::Error, fs::File, io::BufWriter, path::Path};

const REGULAR: &[u8] = include_bytes!("../fonts/LiberationSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../fonts/LiberationSans-Bold.ttf");

pub struct PdfEngine {
    arial: bool,
    line_spacing_percent: u16,
}

impl PdfEngine {
    pub fn new() -> Self {
        let system = FontSystem::new();
        Self {
            arial: has_arial(system.db()),
            line_spacing_percent: DEFAULT_LINE_SPACING_PERCENT,
        }
    }

    pub fn set_line_spacing_percent(&mut self, percent: u16) {
        self.line_spacing_percent =
            percent.clamp(MIN_LINE_SPACING_PERCENT, MAX_LINE_SPACING_PERCENT);
    }

    pub fn using_fallback(&self) -> bool {
        !self.arial
    }

    pub fn render(&self, path: &Path, output: &Path) -> Result<Vec<String>, Box<dyn Error>> {
        let doc = PdfDocument::open(path)?;
        let rotation = doc.get_page_rotation(0)?;
        if rotation != 0 {
            return Err(
                "Gedrehte PDF-Seiten werden für die Bildausgabe noch nicht unterstützt.".into(),
            );
        }
        if doc.get_annotations(0)?.iter().any(|annotation| {
            annotation.subtype_enum != pdf_oxide::annotation_types::AnnotationSubtype::Widget
        }) {
            return Err("Die PDF enthält zusätzliche Anmerkungen, die nicht sicher als Bild dargestellt werden können.".into());
        }
        let (x0, y0, x1, y1) = doc.get_page_visible_box(0)?;
        let (width, height) = (x1 - x0, y1 - y0);
        if width <= 0.0 || height <= 0.0 {
            return Err("Ungültige PDF-Seitengröße.".into());
        }
        let landscape = width > height;
        let (max_w, max_h) = if landscape {
            (2000, 1600)
        } else {
            (1600, 2000)
        };
        let pdf = Pdf::new(std::fs::read(path)?)
            .map_err(|error| format!("PDF konnte nicht gerendert werden: {error:?}"))?;
        let page = pdf.pages().first().ok_or("Die PDF enthält keine Seite.")?;
        let scale = (max_w as f32 / width).min(max_h as f32 / height);
        let settings = InterpreterSettings {
            render_content: RenderContent::PAGE,
            ..Default::default()
        };
        let pixmap = hayro::render(
            page,
            &RenderCache::new(),
            &settings,
            &RenderSettings::default(),
            &PixmapSettings {
                x_scale: scale,
                y_scale: scale,
                bg_color: WHITE,
            },
        );
        let rendered_width = u32::from(pixmap.width());
        let rendered_height = u32::from(pixmap.height());
        let mut image = ImageBuffer::<Rgba<u8>, _>::from_raw(
            rendered_width,
            rendered_height,
            pixmap.data_as_u8_slice().to_vec(),
        )
        .ok_or("Ungültige Bilddaten vom PDF-Renderer.")?;
        let sx = rendered_width as f32 / width;
        let sy = rendered_height as f32 / height;
        let mut editor = DocumentEditor::open(path)?;
        let mut fonts = FontSystem::new();
        let family = if self.arial {
            "Arial"
        } else {
            fonts.db_mut().load_font_data(REGULAR.to_vec());
            fonts.db_mut().load_font_data(BOLD.to_vec());
            "Liberation Sans"
        };
        let mut cache = SwashCache::new();
        let mut warnings = Vec::new();
        for field in editor.get_form_fields()? {
            if field.page_index() != 0 {
                continue;
            }
            let Some(pdf_oxide::extractors::forms::FieldType::Text) = field.field_type() else {
                return Err(
                    format!("Feld '{}' ist kein unterstütztes Textfeld.", field.name()).into(),
                );
            };
            if field.get_alignment().unwrap_or(0) != 0 {
                return Err(format!(
                    "Die Ausrichtung von Feld '{}' wird nicht unterstützt.",
                    field.name()
                )
                .into());
            }
            let pdf_oxide::editor::FormFieldValue::Text(value) = field.value() else {
                continue;
            };
            if value.is_empty() {
                continue;
            }
            let Some(bounds) = field.bounds() else {
                return Err(format!("Position von Feld '{}' fehlt.", field.name()).into());
            };
            // In pdf_oxide 0.3.78, these two members contain the upper-right
            // coordinates of /Rect, despite their width/height names.
            let (left, bottom, right, top) = (bounds.x, bounds.y, bounds.width, bounds.height);
            if right <= left || top <= bottom {
                return Err(format!("Ungültige Position von Feld '{}'.", field.name()).into());
            }
            let px = ((left - x0) * sx).round() as i32;
            let py = ((y1 - top) * sy).round() as i32;
            let box_w = (right - left) * sx;
            let box_h = (top - bottom) * sy;
            let da = field.get_default_appearance().unwrap_or("");
            let tokens: Vec<_> = da.split_whitespace().collect();
            let font_size = tokens
                .windows(3)
                .find_map(|t| (t[2] == "Tf").then(|| t[1].parse::<f32>().ok()).flatten())
                .filter(|v| *v > 0.0)
                .unwrap_or(23.0)
                * sy;
            let bold = da.to_ascii_lowercase().contains("bold");
            let value = if field.flags().unwrap_or(0) & (1 << 12) == 0 {
                value.replace(['\r', '\n'], " ")
            } else {
                value.replace("\r\n", "\n").replace('\r', "\n")
            };
            let line_height = font_size * f32::from(self.line_spacing_percent) / 100.0;
            let mut buffer = Buffer::new(&mut fonts, Metrics::new(font_size, line_height));
            buffer.set_size(&mut fonts, Some((box_w - 4.0 * sx).max(1.0)), None);
            buffer.set_text(
                &mut fonts,
                &value,
                Attrs::new().family(Family::Name(family)).weight(if bold {
                    Weight::BOLD
                } else {
                    Weight::NORMAL
                }),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut fonts, true);
            let lines = buffer.layout_runs().count();
            if lines as f32 * line_height > box_h + font_size * 0.25 {
                warnings.push(format!(
                    "Der Text im Feld '{}' ist zu lang und wird im Bild abgeschnitten.",
                    field.name()
                ));
            }
            buffer.draw(
                &mut fonts,
                &mut cache,
                Color::rgb(0, 0, 0),
                |gx, gy, _, _, color| {
                    let x = px + (2.0 * sx).round() as i32 + gx;
                    let y = py + (2.0 * sy).round() as i32 + gy;
                    if x < px
                        || y < py
                        || x >= px + box_w as i32
                        || y >= py + box_h as i32
                        || x < 0
                        || y < 0
                        || x >= image.width() as i32
                        || y >= image.height() as i32
                    {
                        return;
                    }
                    let pixel = image.get_pixel_mut(x as u32, y as u32);
                    let alpha = color.a() as u32;
                    for (c, fg) in [color.r(), color.g(), color.b()].iter().enumerate() {
                        pixel.0[c] =
                            ((u32::from(*fg) * alpha + u32::from(pixel.0[c]) * (255 - alpha) + 127)
                                / 255) as u8;
                    }
                    pixel.0[3] = 255;
                },
            );
        }
        let image = if landscape {
            image::imageops::rotate90(&image)
        } else {
            image
        };
        let mut out = BufWriter::new(File::create(output)?);
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90).encode_image(&image)?;
        Ok(warnings)
    }
}

fn has_arial(database: &fontdb::Database) -> bool {
    let has_face = |bold| {
        database.faces().any(|face| {
            face.families
                .iter()
                .any(|(family, _)| family.eq_ignore_ascii_case("Arial"))
                && face.style == fontdb::Style::Normal
                && (face.weight.0 >= 600) == bold
        })
    };
    has_face(false) && has_face(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_arial_uses_the_bundled_fallback() {
        assert!(!has_arial(&fontdb::Database::new()));
        let mut engine = PdfEngine {
            arial: false,
            line_spacing_percent: DEFAULT_LINE_SPACING_PERCENT,
        };
        assert!(engine.using_fallback());
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf/form.pdf");
        let output =
            std::env::temp_dir().join(format!("impulsor-pdf-fallback-{}.jpg", std::process::id()));
        assert!(engine.render(&input, &output).unwrap().is_empty());
        let image = image::open(&output).unwrap().into_rgb8();
        assert_eq!(image.dimensions(), (1600, 2000));
        // The Losung field should contain glyphs, not a blank rectangle.
        let dark = image
            .enumerate_pixels()
            .filter(|(x, y, p)| {
                *x >= 50 && *x < 740 && *y >= 160 && *y < 500 && p.0.iter().all(|v| *v < 100)
            })
            .count();
        assert!(dark > 100, "the form text was not rendered");
        // A saved value with an unchanged appearance stream must still change the image.
        use pdf_oxide::editor::{EditableDocument, FormFieldValue};
        let modified_pdf =
            std::env::temp_dir().join(format!("impulsor-pdf-value-{}.pdf", std::process::id()));
        let modified_image =
            std::env::temp_dir().join(format!("impulsor-pdf-value-{}.jpg", std::process::id()));
        engine.set_line_spacing_percent(200);
        assert!(engine.render(&input, &modified_image).unwrap().is_empty());
        let spaced = image::open(&modified_image).unwrap().into_rgb8();
        let spacing_changes = image
            .enumerate_pixels()
            .filter(|(x, y, pixel)| {
                *x >= 50 && *x < 740 && *y >= 160 && *y < 800 && *pixel != spaced.get_pixel(*x, *y)
            })
            .count();
        assert!(
            spacing_changes > 100,
            "line spacing did not affect the rendered text"
        );
        engine.set_line_spacing_percent(DEFAULT_LINE_SPACING_PERCENT);
        let mut editor = DocumentEditor::open(&input).unwrap();
        editor
            .set_form_field_value(
                "Losung",
                FormFieldValue::Text("Neuer Inhalt mit Umlaut: öäü\n\nNoch eine Zeile".into()),
            )
            .unwrap();
        editor.save(&modified_pdf).unwrap();
        assert!(engine
            .render(&modified_pdf, &modified_image)
            .unwrap()
            .is_empty());
        let updated = image::open(&modified_image).unwrap().into_rgb8();
        let background = updated
            .enumerate_pixels()
            .filter(|(x, y, p)| {
                *x >= 50 && *x < 600 && *y >= 1100 && *y < 1210 && p.0.iter().all(|v| *v < 100)
            })
            .count();
        assert!(background > 20, "the static page content disappeared");
        let changed = image
            .enumerate_pixels()
            .filter(|(x, y, p)| {
                *x >= 50 && *x < 740 && *y >= 160 && *y < 500 && *p != updated.get_pixel(*x, *y)
            })
            .count();
        assert!(
            changed > 100,
            "changed field value did not update the image"
        );

        let mut editor = DocumentEditor::open(&input).unwrap();
        for field in ["Losung", "Text Tagesimpuls"] {
            editor
                .set_form_field_value(
                    field,
                    FormFieldValue::Text("Ein langer Text mit Umlauten. ".repeat(200)),
                )
                .unwrap();
        }
        editor.save(&modified_pdf).unwrap();
        let warnings = engine.render(&modified_pdf, &modified_image).unwrap();
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|warning| warning.contains("Losung")));
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("Text Tagesimpuls")));
        assert_eq!(
            image::open(&modified_image)
                .unwrap()
                .into_rgb8()
                .dimensions(),
            (1600, 2000)
        );
        std::fs::remove_file(modified_pdf).unwrap();
        std::fs::remove_file(modified_image).unwrap();
        std::fs::remove_file(output).unwrap();
    }
}
