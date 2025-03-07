# RustySmartStitch: Step-by-Step Process

A detailed breakdown of how RustySmartStitch processes images, from loading to final output.

## 1. Initial Loading Process 📥

### PSD File Handling

The process for handling PSD files has been updated to improve efficiency and accuracy:

```rust
fn convert_psd_to_dynamic_image(&self, psd_path: &PathBuf) -> Result<DynamicImage>
```

The updated process converts PSD files through these steps:
```
[PSD File] -> [Convert to Dynamic Image] -> [Process Layers] -> [Return Image]
   📁            ⬇️                     🔄              📦
```
1. **Convert to Dynamic Image**: The PSD file is converted directly to a `DynamicImage` format.
2. **Process Layers**: Each layer is processed to ensure that transparency and color information are preserved.
3. **Return Image**: The final dynamic image is returned for further processing in the pipeline.

### Regular Image Loading
- Direct loading for standard formats (JPG, PNG, etc.)
- Immediate conversion to internal format

## 2. Image Merging Process 🤝

When multiple images are provided:
```rust
fn merge_images_from_memory(&self, png_data: &[Either<Vec<u8>, PathBuf>])
```

Process visualization:
```
Image 1: [==========]
Image 2: [==========]     →    [==========]
Image 3: [==========]          [==========]
                              [==========]
```

Process:
1. Vertical stacking of images
2. Format-specific handling:
   ```rust
   // Format-specific merging
   let result = if self.output_format == "png" || self.output_format == "webp" {
       self.merge_images_rgba(&images, max_width, total_height)?
   } else {
       self.merge_images_rgb(&images, max_width, total_height)?
   };
   ```
3. Parallel processing for large images:
   ```rust
   // Process chunks in parallel
   png_data.par_chunks(CHUNK_SIZE.min(png_data.len()))
   ```

## 3. Size Processing 📏

Pre-slice size adjustments:
```
Original:  [====== 1000px ======]
                    ↓
Custom:    [=== 800px ===]  (Aspect ratio preserved)
```

1. Custom width handling:
   ```rust
   if self.custom_width_enabled && self.custom_width > 0 {
       let aspect_ratio = width as f32 / height as f32;
       let new_height = (self.custom_width as f32 / aspect_ratio) as u32;
   }
   ```
2. Aspect ratio preservation
3. Resolution adjustments

## 4. Slice Point Detection 🔍

The `SliceLocation` struct analyzes the image using three key parameters:

### Scan Step Analysis
```
Scan Step = 5:
Row 1:  [----X----]  ✓ Check this row
Row 2:  [----X----]  ✗ Skip
Row 3:  [----X----]  ✗ Skip
Row 4:  [----X----]  ✗ Skip
Row 5:  [----X----]  ✓ Check this row
```

### Edge Processing
```
[XXXXX|==================|XXXXX]
      ↑                  ↑
   Ignored            Ignored
    Edge               Edge
```

### Sensitivity Implementation
```
Sensitivity Scale:
100% [Very Strict |------------------] 0% 
 90% [Strict      |---------------]
 70% [Balanced    |------------]
 40% [Flexible    |-------]
 20% [Very Flex.  |---]
 0%  [No Sensitivity |] Direct slice
```

## 5. Slicing Algorithm ✂️

The slicing process follows this sequence:
```
Target Height: 1000px
                   ↓
[==================] Original
         ↓
Attempt 1: Try exact height (1000px)
[========]
[========]  ← Perfect case

If can't find good point:
Attempt 2: Search 800px-1200px range (80-120%)
[==========]  ← Found at 1100px
[======]      ← Adjusted next slice

If still can't find:
Attempt 3: Force at 1300px (130%)
[============] ← Forced cut
[====]
```

Key code:
```rust
// Search range: 80% to 120% of target height
let search_start = start_row + (target_height as f32 * 0.8) as u32;
let search_end = (start_row + (target_height as f32 * 1.2) as u32).min(end_row);

// Force slice at 130% if no good point found
let forced_row = slice_locations[slice_locations.len() - 1] 
    + (target_height as f32 * 1.3) as u32;
```

## 6. Format-Specific Processing 🔄

### JPG Processing
```rust
// Handle transparency
if rgba_pixel[3] < 128 {
    *pixel = image::Rgb([255, 255, 255]); // Transparent to white
} else {
    *pixel = image::Rgb([rgba_pixel[0], rgba_pixel[1], rgba_pixel[2]]);
}

// Quality settings
let encoder = if self.output_quality >= 100 {
    JpegEncoder::new_with_quality(buf_writer, 100)
} else {
    JpegEncoder::new_with_quality(buf_writer, self.output_quality.clamp(1, 99) as u8)
};
```

### PNG Processing
```rust
[RGBA Data] → [PNG Compression] → [Final PNG]
   🎨            🔄               📦
- Keeps transparency
- Lossless quality
// Direct save with transparency
slice_img.write_to(&mut BufWriter::new(buf_writer), image::ImageFormat::Png)?;
```

### WebP Processing
```rust
if self.output_quality >= 100 {
    // Lossless mode
    encoder.encode_lossless()
} else {
    // Lossy with quality
    encoder.encode(quality)
}
```

## 7. Progress Tracking 📊

```rust
// Progress phases
0-30%:  File loading
30-40%: Image merging
40-50%: Slice detection
50-100%: Slice processing

// Update progress
progress_callback(slice_progress);
```

## Performance Optimizations 🚀

### Parallel Processing
```rust
const PARALLEL_THRESHOLD: usize = 1000;
if img.height() > PARALLEL_THRESHOLD as u32 {
    (0..img.height()).into_par_iter()...
}
```

### Memory Management
```rust
const CHUNK_SIZE: usize = 4096;
png_data.par_chunks(CHUNK_SIZE)
```

## Parameter Optimization Guide 🎯

### Clean Manhwa/Webtoons
```
Sensitivity: [███████████] 70-100 (strict detection)
Edges:       [█] 1-3 (minimal edge ignore)
Scan Step:   [█] 1-3 (precise scanning)
```

### Standard Manga
```
Sensitivity: [██████] 40-70 (balanced)
Edges:       [████] 4-7 (moderate)
Scan Step:   [████] 4-7 (balanced)
```

### Problematic Scans
```
Sensitivity: [███] 20-40 (flexible)
Edges:       [████████] 8+ (heavy edge ignore)
Scan Step:   [████████] 8+ (faster scanning)
```

## Error Handling 🚨

```rust
// Slice point fallback system
if let Some(better_row) = find_better_slice_point(...) {
    slice_locations.push(better_row);
} else {
    // Force slice at 130% height
    let forced_row = slice_locations[slice_locations.len() - 1] 
        + (target_height as f32 * 1.3) as u32;
    slice_locations.push(forced_row);
}

// Format conversion safety
match self.output_format.as_str() {
    "jpg" => handle_jpg_conversion(),
    "png" => handle_png_conversion(),
    "webp" => handle_webp_conversion(),
    _ => handle_jpg_fallback()
}
```

## 3. RustySmartStitch Struct Updates

The `RustySmartStitch` struct has been updated with new parameters:
- **custom_width**: Allows users to specify a custom width for output images.
- **upscale_enabled**: A boolean flag to enable or disable upscaling of images.
- **upscale_factor**: Specifies the factor by which to upscale images.
- **resize_enabled**: A boolean flag to enable or disable resizing of images.
- **resize_width** and **resize_height**: Specify dimensions for resizing images.

## 4. Enhanced Image Processing

The `process` method has been improved to handle PSD files more effectively and includes parallel processing for slice detection:
- **PSD Handling**: The method now converts PSD files to dynamic images, allowing for better integration with the processing pipeline.
- **Parallel Processing**: Slices are processed in parallel, improving performance when handling large images.

## 5. New Image Saving Options

New formats and quality settings have been introduced for saving images:
- **WEBP Support**: Users can now save images in WEBP format with adjustable quality settings.
- **Quality Control**: The saving functions allow for specifying quality levels for JPEG and WEBP formats.