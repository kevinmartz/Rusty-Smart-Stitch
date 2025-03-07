mod image_merger;
mod image_saver;
mod processor;
mod psd_sup;
mod slice_location;

pub use processor::RustySmartStitch;
pub use slice_location::SliceLocation;

// Constants used across the module so change them here or go hardcode them in the code
pub(crate) const CHUNK_SIZE: usize = 256;

// The constant CHUNK_SIZE is used in:
// src/slicelogic/image_merger.rs:
// It is used in the merge_images_from_memory function to determine the chunk size for processing images in parallel.
// src/slicelogic/psd_sup.rs:
// in the context of determining an optimal chunk size for processing RGBA data from a PSD file.
// src/slicelogic/slice_location.rs:
// It is used to calculate an optimal chunk size based on the image width when checking rows for slices.
// src/slicelogic/processor.rs:
// through the image_merger module, which uses it in the merge_images_from_memory function.