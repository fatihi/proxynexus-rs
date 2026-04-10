# Corner Infill Utility

Physical cards have rounded corners. When scanned we'll have the white scanner background showing
in the corners of the rectangular images.

This script uses OpenCV to detect those white corners, and fills them in using the Navier-Stokes 
inpainting algorithm (`cv2.inpaint`).

The following command process a folder of raw scanned card images, 
and save their infilled version to `./raw_scans-infilled/`):

```bash
uv run corner_infill.py ./raw_scans/
```