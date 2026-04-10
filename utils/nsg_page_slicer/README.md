# NSG Page Slicer

To extract image files from NSG Print and Play PDFs, I use [pdfimager](https://github.com/sckott/pdfimager).

The following command will save images from a PDF:
```bash
pdfimages -png name.pdf file-prefix
```

For some PDFs, each card is saved as a separate image. For others, only full-page 3x3 grid images are saved.

This Python script is for slicing these full-page images into individual card images. 
It works by first trimming off the outer white space of the page, based on the absolute size of the margins in pixels.
Then it divides the remaining image into 9 card images. 

The grids don't always divide evenly and some of the 1px wide seams between cards sometimes contain blended pixels 
from both adjacent cards. The script also allows you to configure exactly which of these seams to trim off. 
This configuration can be tuned, depending on the specific PDF.

## Configuration

To adjust the cut dimensions, modify the values at the top of `nsg_page_slicer.py`:
- `TOP`, `BOTTOM`, `LEFT`, `RIGHT`: Absolute pixel values to shave off the outer edges of the page.
- `ROW_TRIMS`, `COL_TRIMS`: Relative rules for dropping 1px seams between internal grid cells.

## How to Run

```bash
uv run nsg_page_slicer.py "/path/to/your/full_page_images/"
```

## Output & Debug Files

All sliced cards will be saved into a single `final_slices` folder in the input directory. 
The script also produces debug files to help review the result.

- **`debug_01_shave.png`**: Highlights the outer margin being removed in magenta. 
Look for whether card art was cut off or if there is still white space left between the highlighted border and the card.
- **`debug_02_blueprint.png`**: Highlights the pixels of the seams being deleted in red. 
Shows the effect of the `ROW_TRIMS` and `COL_TRIMS` values.
- **`debug_03_composite.png`**: Show all 9 final sliced cards stitched back together, separated by a 2px green gap. 
Look at the corner intersections of the card image, it can help spot any issues with settings.

Lastly, I use `cargo run -p proxynexus-cli -- generate bleed`, providing the final_slices folder path as the `--input-dir` argument,
to generate bleed borders on all the sliced images, so I can visually check that all images have been sliced accurately.