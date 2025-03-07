# What is Scan Step?

Scan step is a value that determines how many pixels to skip when scanning for slice points. Think of it like "checking every Nth row" - if scan_step is 5, we check every 5th row instead of every single row. This is a performance vs precision tradeoff.

## How it Works in Code

```rust
// In SliceLocation::find_slice_locations
while row < last_row {
    let can_slice = self.check_row_for_slice(gray_img, row, threshold);
    
    if !can_slice {
        // If we can't slice here, skip ahead by scan_step
        row += self.scan_step as u32;
    }
}
```

The scan step is used in several places:
1. When searching for initial slice points
2. When looking for better slice points nearby
3. When moving up/down to find alternative slice locations

### Examples of Different Scan Steps:
- Scan Step 1: Check every row (slowest, most precise)
- Scan Step 5: Check every 5th row (good balance)
- Scan Step 10: Check every 10th row (fastest, least precise)

## Impact on Performance

### Time Complexity:
```rust
// With height H and scan_step S:
// Number of rows checked = H/S
// Lower scan_step = more rows checked = slower
```

For a 1000px height image:
- Scan Step 1: 1000 rows checked
- Scan Step 5: 200 rows checked
- Scan Step 10: 100 rows checked

## When Finding Better Slice Points

```rust
fn find_better_slice_point(&self, gray_img: &GrayImage, start_row: u32, 
    end_row: u32, target_height: u32, threshold: u8) -> Option<u32> {
    // ...
    for row in (search_start..search_end).step_by(self.scan_step as usize) {
        if self.check_row_for_slice(gray_img, row, threshold) {
            // Found a potential slice point
        }
    }
}
```

The scan step affects how thoroughly we search for alternative slice points:
1. Small scan step: More likely to find the optimal slice point
2. Large scan step: Might miss some good slice points but runs faster

## What Scan Step to Use?

### Small Scan Step (1-3):
- Best for best slice points
- When you need exact panel boundaries
- When processing time isn't a concern
- Good for images with thin panel borders

### Medium Scan Step (4-7):
- Good default choice
- Balances speed and accuracy
- Works well for most images
- Default is 5 for a reason!

### Large Scan Step (8+):
- When speed is priority
- For rough splits of large images
- When exact panel boundaries aren't crucial
- Good for batch processing many files

## Tips for Using Scan Step

1. **Start with Default (5)**:
   - It's a proven good balance by me uhm
   - Works well for most images
   - Fast enough for most uses

2. **Adjust Based on Results**:
   - Missing thin panel borders? → Lower scan step
   - Processing too slow? → Increase scan step
   - Getting uneven splits? → Try lower scan step

3. **Consider Your Content**:
   - For manhwa/manhua → Lower scan step (1-3)
   - For webcomics → Medium scan step (4-7)
   - For photos/art → Higher scan step (8+)

4. **Hardware Considerations**:
   - Faster CPU → Can use lower scan step
   - Slower CPU → Use higher scan step
   - Processing lots of files → Higher scan step

5. **Combine with Sensitivity**:
   - High precision: Low scan step + high sensitivity
   - Balanced: Medium scan step + medium sensitivity
   - Fast processing: High scan step + low sensitivity

## Common Issues and Solutions

### Problem: Missing Panel Borders
```
Scan Step 5:   ----X----X----
Actual Border: ----X---X-----
                      ^ Missed!
```
Solution: Lower scan step to catch all borders

### Problem: Slow Processing
```
Scan Step 1:   Every row = 1000 checks
Scan Step 5:   Every 5th row = 200 checks
Scan Step 10:  Every 10th row = 100 checks
```
Solution: Increase scan step if precision isn't critical

### Problem: Uneven Splits
```
Scan Step too high:
Actual Border:    ----X----
Available Steps:  ---X----X
                     ^ Off by several pixels
```
Solution: Lower scan step for more precise alignment

## Performance vs Quality Trade-offs

1. **Maximum Quality**:
   - Scan Step: 1
   - Pro: Catches every possible slice point
   - Con: Slowest processing speed

2. **Balanced (Recommended)**:
   - Scan Step: 5
   - Pro: Good accuracy, decent speed
   - Con: Might miss some perfect slice points

3. **Maximum Speed**:
   - Scan Step: 10+
   - Pro: Very fast processing
   - Con: Might miss good slice points 

## Note on Recent Updates

Please be aware that the `slicelogic` module has undergone significant updates. Ensure that you check the module for any changes that may affect how the scan step is applied during image processing. 