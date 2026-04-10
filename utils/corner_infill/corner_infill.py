import os
import cv2
import argparse
import numpy as np

WHITE_CUTOFF = 253      # Pixel intensity to consider as "blank scanner background"
KERNEL_SIZE = (5, 5)    # Kernel size for dilating the mask to catch fuzzy edges
INPAINT_RADIUS = 3      # Radius for the Navier-Stokes inpainting algorithm

def process_corners(img):
    height, width, _ = img.shape
    
    # The physical corner radius is ~0.125 inches on a 2.48 inch wide card.
    # Therefore, the blank scanner background will exist within the outermost ~4.8% of the image.
    corner_depth = int(min(height, width) * 0.048)

    mask = np.zeros((height, width), dtype=np.uint8)

    # Identify the pixels in the four corners that are pure white
    for x in range(0, corner_depth):
        for y in range(0, corner_depth):
            # Top-left, Bottom-left, Top-right, Bottom-right
            corners = [
                (y, x), 
                (height - 1 - y, x), 
                (y, width - 1 - x), 
                (height - 1 - y, width - 1 - x)
            ]
            
            for cy, cx in corners:
                if np.any(img[cy, cx] > WHITE_CUTOFF):
                    mask[cy, cx] = 255

    kernel = np.ones(KERNEL_SIZE, np.uint8)
    mask = cv2.dilate(mask, kernel, iterations=3)

    inpainted_img = cv2.inpaint(img, mask, INPAINT_RADIUS, cv2.INPAINT_NS)
    
    return inpainted_img

def main():
    parser = argparse.ArgumentParser(description="Fill in the blank corners of physical card scans.")
    parser.add_argument("input", help="Path to a single image or directory of images.")
    parser.add_argument("-o", "--output", help="Optional: Output directory to save processed images. Defaults to '<input>-infilled'.")
    
    args = parser.parse_args()

    input_path = os.path.abspath(args.input)
    if args.output:
        output_dir = args.output
    else:
        clean_input_path = input_path.rstrip(os.sep)
        output_dir = f"{clean_input_path}-infilled"

    os.makedirs(output_dir, exist_ok=True)

    if os.path.isfile(args.input):
        files = [args.input]
    elif os.path.isdir(args.input):
        files = [os.path.join(args.input, f) for f in os.listdir(args.input) 
                 if f.lower().endswith(('.png', '.jpg', '.jpeg'))]
    else:
        print("Error: Input path is invalid.")
        return

    print(f"Processing {len(files)} image(s)...")

    for file_path in files:
        img = cv2.imread(file_path, cv2.IMREAD_COLOR)
        if img is None:
            print(f"Skipping {file_path} (could not read file).")
            continue

        processed_img = process_corners(img)

        filename = os.path.basename(file_path)
        out_path = os.path.join(output_dir, filename)
        
        cv2.imwrite(out_path, processed_img)
        print(f"Saved: {out_path}")

    print("Corner infill complete.")

if __name__ == '__main__':
    main()
