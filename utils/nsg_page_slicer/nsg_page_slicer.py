import cv2
import os
import numpy as np
import argparse
import glob

# Outer shave values for newer PDFs
TOP = 76
BOTTOM = 77
LEFT = 76
RIGHT = 77

# # Outer shave values used for downfall and uprising
# TOP = 153
# BOTTOM = 153
# LEFT = 153
# RIGHT = 153

# Internal trims for deleting seem pixels.
# Format for ROW_TRIMS: { row_index: (drop_top, drop_bottom) }
# Format for COL_TRIMS: { col_index: (drop_left, drop_right) }
ROW_TRIMS = {
    1: (1, 0),
    # 2: (1, 0),  # only for downfall and uprising only
}

COL_TRIMS = {
    1: (1, 0),
    2: (1, 0),
}


def main():
    parser = argparse.ArgumentParser(description="Slice and debug image grids.")
    parser.add_argument("directory", help="Directory containing PNG images to process.")
    args = parser.parse_args()

    if not os.path.isdir(args.directory):
        print(f"Error: Directory '{args.directory}' does not exist.")
        return

    search_pattern = os.path.join(args.directory, "*.png")
    image_paths = sorted(glob.glob(search_pattern))

    if not image_paths:
        print(f"No PNG files found in '{args.directory}'.")
        return

    print(f"Found {len(image_paths)} PNG file(s) to process.\n")

    for img_path in image_paths:
        base_name = os.path.splitext(os.path.basename(img_path))[0]
        output_dir = os.path.join(args.directory, base_name)

        print(f"--- Processing: {base_name} ---")

        image = cv2.imread(img_path)
        if image is None:
            print(f"  Error: Could not load image '{img_path}'. Skipping.")
            continue

        os.makedirs(output_dir, exist_ok=True)
        final_slices_dir = os.path.join(args.directory, "final_slices")
        os.makedirs(final_slices_dir, exist_ok=True)

        orig_h, orig_w = image.shape[:2]

        # ==========================================
        # DEBUG 1: Shave Preview
        # ==========================================
        debug_shave = image.copy()
        # Draw magenta over the shaved areas
        debug_shave[0:TOP, :] = (255, 0, 255)  # Top
        debug_shave[orig_h - BOTTOM : orig_h, :] = (255, 0, 255)  # Bottom
        debug_shave[:, 0:LEFT] = (255, 0, 255)  # Left
        debug_shave[:, orig_w - RIGHT : orig_w] = (255, 0, 255)  # Right

        shave_out = os.path.join(output_dir, "debug_01_shave.png")
        cv2.imwrite(shave_out, debug_shave)
        print(f"  Saved '{shave_out}'")

        # Shave the outer edges
        shaved_image = image[TOP:-BOTTOM, LEFT:-RIGHT]
        h, w = shaved_image.shape[:2]

        # Calculate the base size of a grid cell
        base_cell_w = w // 3
        base_cell_h = h // 3

        print(f"  Shaved canvas: {w}x{h}")
        print(f"  Base perfect cell size: {base_cell_w}x{base_cell_h}")

        # ==========================================
        # DEBUG 2: Blueprint Preview Setup
        # ==========================================
        debug_blueprint = shaved_image.copy()

        count = 0
        composite_cards = []

        for row in range(3):
            row_cards = []
            for col in range(3):
                base_start_x = col * base_cell_w
                base_end_x = (col + 1) * base_cell_w

                base_start_y = row * base_cell_h
                base_end_y = (row + 1) * base_cell_h

                # Apply the trims for this specific row and column
                drop_top, drop_bottom = ROW_TRIMS.get(row, (0, 0))
                drop_left, drop_right = COL_TRIMS.get(col, (0, 0))

                final_start_x = base_start_x + drop_left
                final_end_x = base_end_x - drop_right

                final_start_y = base_start_y + drop_top
                final_end_y = base_end_y - drop_bottom

                if drop_top > 0:
                    debug_blueprint[
                        base_start_y : base_start_y + drop_top, base_start_x:base_end_x
                    ] = (0, 0, 255)
                if drop_bottom > 0:
                    debug_blueprint[
                        base_end_y - drop_bottom : base_end_y, base_start_x:base_end_x
                    ] = (0, 0, 255)
                if drop_left > 0:
                    debug_blueprint[
                        base_start_y:base_end_y, base_start_x : base_start_x + drop_left
                    ] = (0, 0, 255)
                if drop_right > 0:
                    debug_blueprint[
                        base_start_y:base_end_y, base_end_x - drop_right : base_end_x
                    ] = (0, 0, 255)

                card = shaved_image[
                    final_start_y:final_end_y, final_start_x:final_end_x
                ]

                card_out = os.path.join(
                    final_slices_dir, f"{base_name}_card_{count:03d}.png"
                )
                cv2.imwrite(card_out, card)

                row_cards.append(card)
                count += 1

            composite_cards.append(row_cards)

        blueprint_out = os.path.join(output_dir, "debug_02_blueprint.png")
        cv2.imwrite(blueprint_out, debug_blueprint)
        print(f"  Saved '{blueprint_out}'")

        # ==========================================
        # DEBUG 3: Composite Sheet
        # ==========================================
        GAP = 2
        GAP_COLOR = (0, 255, 0)

        # Pad cards in each row to the same height before hstacking
        row_images = []
        for row_cards in composite_cards:
            max_h = max(c.shape[0] for c in row_cards)
            padded = [
                cv2.copyMakeBorder(
                    c, 0, max_h - c.shape[0], 0, 0, cv2.BORDER_CONSTANT, value=GAP_COLOR
                )
                for c in row_cards
            ]
            separator = np.full((max_h, GAP, 3), GAP_COLOR, dtype=np.uint8)
            row_images.append(
                np.hstack([x for card in padded for x in (card, separator)][:-1])
            )

        # Pad rows to the same width before vstacking
        max_w = max(r.shape[1] for r in row_images)
        padded_rows = [
            cv2.copyMakeBorder(
                r, 0, 0, 0, max_w - r.shape[1], cv2.BORDER_CONSTANT, value=GAP_COLOR
            )
            for r in row_images
        ]
        row_separator = np.full((GAP, max_w, 3), GAP_COLOR, dtype=np.uint8)
        debug_sheet = np.vstack(
            [x for row in padded_rows for x in (row, row_separator)][:-1]
        )

        composite_out = os.path.join(output_dir, "debug_03_composite.png")
        cv2.imwrite(composite_out, debug_sheet)
        print(f"  Saved '{composite_out}'")

        print(f"  Finished processing '{base_name}'.\n")


if __name__ == "__main__":
    main()
