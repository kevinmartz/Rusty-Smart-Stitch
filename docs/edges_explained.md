# What is Edges?

Edges is a value that determines how many pixels to ignore at the left and right sides of each row when looking for slice points. This is important because manhwa pages often have messy or inconsistent edges due to artifacts, page borders, or binding marks.

## How it Works in Code

```rust
fn check_row_for_slice(&self, gray_img: &GrayImage, row: u32, threshold: u8) -> bool {
    let width = gray_img.width() as i32;
    let ignorable = self.edges;
    
    // Skip some pixels at the edges cuz they're usually messy
    let mut x = (ignorable + 1) as u32;
    while x < (width - ignorable) as u32 {
        // Check pixels...
    }
}
```

The edges value is used to:
1. Skip the first N pixels from the left edge
2. Skip the last N pixels from the right edge
3. Only check the "clean" middle section of each row

### Visual Example:
```
With edges = 5:

[XXXXX|==================|XXXXX]
      ^                  ^
      |                  |
   Ignored           Ignored
   (5px)             (5px)
```

## Impact on Slice Detection

### Without Edges Setting:
```
Full row check:
[NOISE]====================[NOISE]
^                              ^
These messy edges could prevent finding good slice points
```

### With Edges Setting:
```
Ignoring edges:
[XXXX]===================[XXXX]
     ^                   ^
     Clean section only
```

## What Edges Value to Use?

### Small Edges (1-3):
- For clean scans with minimal edge noise
- When pages are well-aligned
- Basically for manhwa/webtoons
- When you need to check near the edges

### Medium Edges (4-7):
- Default setting (5) works for most cases
- Good for typical most cases
- Balances edge noise vs content
- Safe choice for mixed content

### Large Edges (8+):
- For messy scans with lots of edge artifacts
- When pages have binding shadows
- For batch processing old manga or manhwa basically those oldass scans/raws
- When edges are consistently problematic

## Tips for Using Edges

1. **Start with Default (5)**:
   - Works well for most cases
   - Handles typical artifacts
   - Safe default choice

2. **Adjust Based on Your Scans**:
   - Lots of binding shadows → Increase edges
   - Clean digital content → Decrease edges
   - Missing content near edges → Decrease edges
   - False positives at edges → Increase edges

3. **Consider Your Source Raw**:
   - For manhwa → Lower edges (1-5)
   - For manga → Medium edges (5-8)
   - For old/messy scans → Higher edges (8+)

4. **Watch for These Issues**:
   - Text/content near page edges
   - Binding shadows or marks
   - Artifacts
   - Page alignment issues

## Common Problems and Solutions

### Problem: Missing Edge Content
```
Edges too high:
[XXXXXX|Text=========|XXXXXX]
       ^ Important text ignored
```
Solution: Lower edges value

### Problem: False Edge Detection
```
Edges too low:
[Noise|===========|Noise]
^                     ^
Binding marks causing issues
```
Solution: Increase edges value

### Problem: Asymmetric Pages
```
Left binding:
[NOISE===|===========|===]
Consider different left/right edges
```
Solution: Use medium/high edges value

## Combining with Other Settings

1. **With Sensitivity**:
   - High edges + high sensitivity = Good for messy scans
   - Low edges + high sensitivity = Good for digital content aka manhwa
   - Medium edges + medium sensitivity = Good general setup

2. **With Scan Step**:
   - The edges setting doesn't affect performance
   - Can freely adjust based on your needs
   - Focus on image quality when choosing edges value

## Best Practices

1. **For Digital Content**:
   - Edges: 1-5
   - Clean edges mean less need to ignore

2. **For Scanned Manga**:
   - Edges: 5-8
   - Handles typical scanning artifacts

3. **For Problematic Scans**:
   - Edges: 8+
   - Better to ignore more than get false positives

4. **Special Cases**:
   - Content very close to edges: Use 1-3
   - Heavy binding shadows: Use 8+
   - Inconsistent scans: Stick to 5-8

## Technical Details

The edges parameter affects:
```rust
// Effective width checked = total_width - (2 * edges)
// Example with 1000px width and edges=5:
// Checked region = 1000 - (2 * 5) = 990px
```

This means:
1. Higher edges = smaller checked region
2. Lower edges = larger checked region
3. No impact on processing speed
4. Only affects what parts of each row are checked 