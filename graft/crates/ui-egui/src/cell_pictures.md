# crates/ui-egui/src/cell_pictures.rs

- MAX_ENCODED · constant · L10-L10 — const MAX_ENCODED: usize = 16 * 1024 * 1024;
- MAX_PIXELS · constant · L11-L11 — const MAX_PIXELS: u64 = 16_000_000;
- MAX_CACHE_BYTES · constant · L12-L12 — const MAX_CACHE_BYTES: usize = 64 * 1024 * 1024;
- MAX_CACHE_ITEMS · constant · L14-L14 — const MAX_CACHE_ITEMS: usize = 4096;
- Cache · struct · L17-L20 — struct Cache
- Entry · struct · L22-L30 — struct Entry
- texture · function · L33-L70 — fn texture(&mut self, ctx: &egui::Context, picture: &Arc<CellPicture>) -> Option<TextureHandle>
- dimensions · function · L73-L91 — fn dimensions(picture: &CellPicture) -> Option<(image::ImageFormat, (u32, u32))>
- decode · function · L94-L97 — fn decode(picture: &CellPicture) -> Option<egui::ColorImage>
- decode_format · function · L99-L108 — fn decode_format(picture: &CellPicture, format: image::ImageFormat) -> Option<egui::ColorImage>
- fitted_rect · function · L110-L117 — fn fitted_rect(cell: Rect, size: [usize; 2]) -> Option<Rect>
- paint · function · L119-L135 — pub(crate) fn paint(painter: &Painter, cell: Rect, picture: &Arc<CellPicture>)
- tests · module · L138-L183 — mod tests
- picture · function · L141-L146 — fn picture(color: [u8; 4]) -> Result<Arc<CellPicture>, image::ImageError>
- aspect_fit_and_bounded_content_cache · function · L149-L182 — fn aspect_fit_and_bounded_content_cache() -> Result<(), Box<dyn std::error::Error>>
