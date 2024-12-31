# What is Sensitivity?

Sensitivity is a value between 0-100 that controls how picky the program is when looking for places to slice your image. Think of it like adjusting how "strict" the program should be when deciding what counts as a good slice point, in this case, if there is bubbles/sfx it will try to avoid them.

## How it Works in Code

```rust
// In SliceLocation::run
let threshold = {
    let mut cache = self.threshold_cache.borrow_mut();
    if let Some(&t) = cache.get(&self.sensitivity) {
        t
    } else {
        // This is where sensitivity gets converted to a threshold
        let t = ((255.0 * (1.0 - (self.sensitivity as f32 / 100.0))) as u32) as u8;
        cache.insert(self.sensitivity, t);
        t
    }
};
```

The sensitivity value gets converted into a threshold that's used to compare pixels. Here's how:

1. Your sensitivity (0-100) gets divided by 100 to make it a percentage
2. That percentage is subtracted from 1.0 to flip it (higher sensitivity = lower threshold)
3. The result is multiplied by 255 (max pixel value) to get the actual threshold

### Examples:
- Sensitivity 100 → Threshold 0 (super strict)
- Sensitivity 80 → Threshold 51
- Sensitivity 50 → Threshold 127
- Sensitivity 20 → Threshold 204
- Sensitivity 0 → Threshold 255 (Direct slice)

## When Checking Rows

The threshold is used in `check_row_for_slice` to decide if a row is good for slicing:

```rust
fn check_row_for_slice(&self, gray_img: &GrayImage, row: u32, threshold: u8) -> bool {
    // ...
    if !((prev_pixel <= 9 && next_pixel <= 9) || 
        next_pixel.abs_diff(prev_pixel) <= threshold) {
        return false;
    }
    // ...
}
```

A row is considered good for slicing if either:
1. Both pixels are nearly black/white (<=9), OR
2. The difference between adjacent pixels is less than the threshold

## What Sensitivity Level to Use?

### High Sensitivity (70-100):
- Best for manga/manhwa with clear panel borders
- Good when you want slices exactly at panel boundaries
- More likely to fail finding slice points if images aren't clean

### Medium Sensitivity (40-70):
- Good general-purpose setting
- Works well for mixed content
- Balances between finding good splits and being flexible

### Low Sensitivity (0-40):
- Best for messy images or when exact splits don't matter
- More likely to find slice points but might not be at perfect spots
- Good when you just want to split into roughly equal chunks 
  aka Direct slice it will use the exact height you given

## Tips for Using Sensitivity

1. **Start at 50**: It's a good middle ground for most images

2. **Adjust Based on Results**:
   - If it's missing obvious panel borders → Increase sensitivity
   - If it's not finding any slice points → Decrease sensitivity
   - If slices are slightly off from panel borders → Try increasing sensitivity

3. **Consider Your Content**:
   - Clean manhwa scans → Higher sensitivity (70-100)
   - Mixed content → Medium sensitivity (40-70)
   - Messy scans or photos → Lower sensitivity (0-40)

4. **Combine with Scan Step**:
   - High sensitivity + low scan_step = Most precise (but slowest)
   - Low sensitivity + high scan_step = Fastest (but less precise) 