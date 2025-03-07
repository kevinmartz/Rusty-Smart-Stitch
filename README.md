# Rusty Smart Stitch

<div align="center">
  <a href="https://github.com/kevinmartz/Rusty-Smart-Stitch">
    <img alt="RustySmartStitch.Logo" width="393" height="130" src="/assets/logo.png">
  </a>
  <h1>Rusty Smart Stitch</h1>
  <p>
    A very very small (5MB) tool for slicing and stitching webtoons/manhwa/manhua raws.
  </p>
  <p>
    A blazingly fast Rust implementation with parallel processing and advanced algorithms.
  </p>
  <a href="https://github.com/kevinmartz/Rusty-Smart-Stitch/releases/latest">
    <img src="https://img.shields.io/github/v/release/kevinmartz/Rusty-Smart-Stitch">
  </a>
  <a href="https://github.com/kevinmartz/Rusty-Smart-Stitch/commits/main">
    <img src="https://img.shields.io/github/last-commit/kevinmartz/Rusty-Smart-Stitch">
  </a>
  <a href="https://github.com/kevinmartz/Rusty-Smart-Stitch/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-AGPL--3.0-blue.svg">
  </a>
</div>

## Overview

A Rust implementation of SmartStitch with an even better algorithm and enhancements. Completely rewritten in Rust and has much more to offer!

For the build guide, check out [build guide](/docs/build_guide.md)!

## Screenshots

<div align="center">
<table>
<tr>
<td width="33%">
  <img src="/docs/screenshots/1.1.PNG" alt="Main Tab" width="250"/>
  <br/>
  <em>Main Interface</em>
</td>
<td width="33%">
  <img src="/docs/screenshots/1.2.PNG" alt="Main Tab with Files" width="250"/>
  <br/>
  <em>With Files Loaded</em>
</td>
<td width="33%">
  <img src="/docs/screenshots/2.PNG" alt="Advanced Tab" width="250"/>
  <br/>
  <em>Advanced Settings</em>
</td>
</tr>
<tr>
<td width="33%" colspan="3" align="center">
  <img src="/docs/screenshots/3.PNG" alt="Waifu2x Tab" width="250"/>
  <br/>
  <em>Waifu2x Integration</em>
</td>
</tr>
</table>
</div>

## Features

- 📦 Extremely small size (5MB)
- 🚀 Fast processing with parallel optimization
- 🎨 Multiple format support (PNG, JPG, JPEG, WebP, BMP, PSD)
- 📁 Folder and subfolder processing
- 💾 Profile system for saving settings
- 🖼️ Advanced image processing options
- 🔄 Integrated Waifu2x support (you still need to download the waifu2x-caffe)
- 📚 Documentation and examples
- 📂 Built-in update checker

## Quick Start

1. Download the latest release
2. Launch the application
3. Drag & drop images or use the file selector
4. Adjust settings (or use defaults)
5. Click "Process Images"
6. Check output directory for results

## Interface Guide

### Main Tab

#### File Input Methods
- 🖱️ Drag & drop files
- 📂 "Select Files" button
- 📁 "Select Folder" button (includes subfolders)

#### Core Settings
- **Height**: Target slice height
- **Sensitivity**: Slice detection precision (0-100)
  - 100: Very strict detection
  - 0: Direct slice at target height
  - [More about sensitivity](docs/sensitivity_explained.md)
- **Scan Step**: Processing precision (1-20)
  - Lower = More precise but slower
  - Default: 5 (recommended)
  - [More about scan step](docs/scan_step_explained.md)
- **Output Format**: JPG/PNG/WebP/BMP
- **Quality**: Output quality (for JPG/WebP)

### Advanced Tab

#### Advanced Settings
- **Custom Width**: Force specific output width
- **Edges**: Edge detection control
  - [More about edges](docs/edges_explained.md)
- **Upscale**: CPU-based upscaling
- **Resize**: Custom dimension control

#### Profile System
- 💾 Save custom configurations
- 📂 Load saved profiles
- 📤 Export profiles as JSON
- 📥 Import profiles from JSON

### Waifu2x Integration

Integrated [waifu2x-caffe](https://github.com/lltcggie/waifu2x-caffe) support with:
- Multiple conversion modes
- Noise reduction levels
- Custom magnification settings
- GPU acceleration support
- Advanced model options

## Performance Tips

### Optimal Settings for Different Content:

Take this with a grain of salt, but it's a good starting point.

#### Clean Manhwa/Webtoons
```
Sensitivity: [███████████] 70-100
Edges:       [█] 1-5
Scan Step:   [█] 1-5
```

#### Problematic Scans
```
Sensitivity: [███] 20-40
Edges:       [████████] 8+
Scan Step:   [████████] 8+
```

## Support & Contributing

### Getting Help
- 🐛 [Report bugs](https://github.com/kevinmartz/Rusty-Smart-Stitch/issues/new?template=bug_report.md)
- 💡 [Request features](https://github.com/kevinmartz/Rusty-Smart-Stitch/issues/new?template=feature_request.md)
- 📖 Check [documentation](docs/) for guides and explanations

### Contributing
Contributions! Please see [Contributing Guidelines](docs/CONTRIBUTING.md) for details on:
- 🤝 Code of Conduct
- 🔧 Development setup
- 📝 Coding standards
- 🔍 Pull request process
- 📋 Issue reporting

## TODO (in no particular order)
- ✅ Enhance the logic (kinda)
- [ ] Add a way to add a custom watermark
- [ ] other upscalers other than waifu2x (like chainner)
- [ ] enhacning my own upscaler

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0) - see [LICENSE](LICENSE) for details.

Key points:
- ✅ Open source and free to use
- ✅ All modifications must be open source
- ✅ Network use requires source distribution
- ✅ Must include original license and copyright
- ✅ Must state significant changes

## Acknowledgments

Special thanks to:
- [waifu2x-caffe](https://github.com/lltcggie/waifu2x-caffe) - Image upscaling
- [Manas](https://github.com/ManasHere) - Inspiration and ideas 
