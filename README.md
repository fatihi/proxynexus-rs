# Proxy Nexus

Make high quality Netrunner proxies. 

Generate print-and-play PDFs or pre-formatted image files for ordering physical cards from MakePlayingCards.com.
Use a list of card names, a set name, or a netrunnerdb decklist URL.

The web app is hosted at https://proxynexus.net/. It has access to a complete collection of card images, 
both scans of real FFG cards and images extracted from [NSG](https://nullsignal.games/) print-and-play PDFs, not images from netrunnerdb.

It can also run locally as a desktop app or CLI for offline use. The CLI can also be used to create, 
manage and share your own card image collections. However, it seems local use is primarily for myself,
to simplify ongoing updates of the hosted image collection for the web app and for development purposes.

---

## Building & Running

### Prerequisites
- [Rust](https://rust-lang.org/learn/get-started/), 
- [dioxus-cli](https://dioxuslabs.com/learn/0.7/getting_started/) 
  provides the `dx` command for running the GUI


### Running the Web App Locally
```bash
dx serve --platform web
```
The web app fetches images from a Cloudflare R2 bucket, even when running locally, therefore it does not work offline. 
This is mostly for testing.


### Running the Desktop App Locally
```bash
dx serve
```
The desktop app runs locally, including its database and image file collections. You'll notice on first start that it won't
know of any card names or sets. **To make the Desktop app usable, you must first use the CLI to load a local card collection.**


### Building the CLI
```bash
cargo build -p proxynexus-cli --release
```
The built binary will be located at `./target/release/proxynexus-cli` (or `.\target\release\proxynexus-cli.exe` on Windows)

---

## Local Setup (CLI & Desktop)

When the CLI or desktop app runs for the first time, it synchronizes all card and set metadata from `netrunnerdb.com` and saves it locally.
The app then needs image files of cards, which are added from collection `.pnx` files. 
The CLI is able to create these collections from a folder of card scan image files, and manage them in the app.

### 1. Acquiring Images
You need a folder of correctly named card images. [Google Drive - Proxy Nexus Collections](https://drive.google.com/drive/folders/1d84k6Od5bSBK31-lQkJzRc71xGx6-zVS?usp=sharing). Here you'll find 3
folders of images that were used to create the collections for the web app.

If you want to make your own collection, the file names in the folder **must** follow the 
[image file naming conventions](#image-file-naming-convention).

### 2. Building a Collection `.pnx` File
```bash
proxynexus-cli collection build --images ./core_set_scans --output core_set.pnx
```
Creates a collection file `core_set.pnx` from all the images in the `core_set_scans` folder.


### 3. Adding the Collection
```bash
proxynexus-cli collection add core_set.pnx
```
Adds the new `core_set.pnx` collection. This updates the app's local database, and copies all the images to your
home drive under `~/.proxynexus/collections/`

The desktop app is now ready to use. You can also check which collections have been added using the command:

```bash
proxynexus-cli collection list
```

---

## CLI Commands

The `proxynexus-cli` supports the following subcommands. You can use `--help` on any command for more specific options.

**Generation:**
*   `generate pdf`: Generate a print-and-play PDF from a specific set, cardlist, or NetrunnerDB URL.
*   `generate mpc`: Generate a MakePlayingCards (MPC) formatted ZIP file.

**Collection Management:**
*   `collection build`: Create a new `.pnx` collection file from a directory of card scans.
*   `collection add`: Load a `.pnx` collection into your local app.
*   `collection list`: View all loaded collections.
*   `collection remove`: Delete a collection from your local app.

**Catalog Management:**
*   `catalog update`: Fetch the latest card and pack data from NetrunnerDB.
*   `catalog info`: View metadata about the local catalog.
*   `catalog import`: Import catalog data from local JSON files.

**Query & Export:**
*   `query`: Search the catalog and collections (e.g., `--list-sets` or `--set-name`).
*   `export`: Export the local database to an `init.sql` file. Required for the web app at `proxynexus-gui/public/init.sql`.

---

## Terminology

*   **Card:** Abstract representation of a card, uniquely identified by its title.
*   **Printing:** A specific physical representation of a card. A card can have multiple Printings, like a newer release of a card, or a version with alternate artwork.
*   **Variant:** A property of a Printing used to distinguish between multiple Printings of the same card.
*   **Part:** Some printings have more than one image. Most cards just have a "front" part, but double-sided cards have a "back" part as well.
*   **Collection:** A set of card image files and metadata. Can be packaged into a `.pnx` file by the CLI, and added to a local Proxy Nexus instance.
*   **Pack and Set:** Both mean the same thing and are used interchangeably. A retail expansion of cards. Netrunnerdb's API uses the term pack but their UI uses set.
*   **Card Request:** The user's intent when asking to generate a proxy. It specifies the card title and code and optional variant, collection, or pack overrides.

--- 

## Image File Naming Convention

Each image file represents a single printing and part. The collection builder relies solely on the file name to identify it.
The general syntax is:
`{card_code}_{variant}-{part}.{extension}`

Both the variant and part sections are optional, and default to "original" and "front" respectively if omitted.
Only PNG and JPEG files are supported.

#### File name scenarios:
*   **Standard Cards (No Variant, No Part):** The majority of card image files. (e.g., `01001.jpg` -> Code: 01001, Variant: original, Part: front).
*   **Variants (Alternate Art):** Contains an underscore `_` followed by a variant name. (e.g., `01001_alt1.jpg` -> Code: 01001, Variant: alt1, Part: front).
*   **Parts (Multiple Sides):** Contains a dash `-` followed by the part name. (e.g., `26066-back.jpg` -> Code: 26066, Variant: original, Part: back).
*   **Combined (Variants and Parts):** The variant must come before the part. (e.g., `09037_alt1-back.jpg` -> Code: 09037, Variant: alt1, Part: back).

**Strict Rules:**
*   **Numeric Card Codes Only:** The parser checks if the `{card_code}` section is numeric. If it doesn't start with numbers (e.g., `agenda_1.jpg`), it is ignored.
*   **Orphans:** If a part file doesn't have an associated front file, it is ignored.

---

## Card Requests and Variant Notation

#### Variant Notation in Card Lists

When generating from a card list, you can request a specific variant name, collection, or pack using the following notation.
`Quantity Card Name [variant:collection:pack_code]`

Examples:
*   **Requesting a specific variant:** `3x Sure Gamble [alt1]`
*   **Requesting a specific collection:** `3x Sure Gamble [:my_custom_scans]`
*   **Requesting a specific variant from a specific collection:** `3x Sure Gamble [promo:my_custom_scans]`
*   **Requesting a specific pack version:** `3x Sure Gamble [::core]`

The variant notation is optional.

**Discovering Available Variants:**

You can use the CLI's `query` command to see what's available.

The following lists the number of Printings per set, per collection:
```bash
./proxynexus-cli query --list-sets

Available Sets:

  - Draft                       [::draft]    # 9 in ffg-en
  - Core Set                    [::core]     # 104 in ffg-en, 41 in extras
  - What Lies Ahead             [::wla]      # 21 in ffg-en, 4 in extras
  - Trace Amount                [::ta]       # 21 in ffg-en
...
```

The following lists the variants and the collection they're in for each card in the set:
```bash
./proxynexus-cli query --set-name "Core Set"

Query Results:

1x Noise: Hacker Extraordinaire [original:ffg-en:core]  # also: [alt1:ffg-en:core], [alt1:extras:core]
2x Déjà Vu [original:ffg-en:core]                       # also: [alt1:ffg-en:core]
3x Demolition Run [original:ffg-en:core]
3x Stimhack [original:ffg-en:core]                      # also: [alt1:ffg-en:core]
...
```
The quantity comes from the pack's metadata from netrunnerdb. The output of this query is a valid card list.

#### Card Request Resolution

Whether you're using the notation above in a card list, or selecting a set name, or a netrunnerdb URL, 
the app converts this input into a list of **Card Requests**.

How card codes are determined:
*   **Cards Lists:** For each card name, get the **newest card code**.
*   **Sets and URLs:** Use the exact card code from the pack metadata or decklist URL.

Each Card Request in the list is then used to find the best available Printing, across all available collections, 
using the following priority hierarchy:
1.  Match the requested variant. If no variant is specified, default to the `"original"` variant.
2.  Match the exact collection or pack, if provided.
3.  Match the exact card code.
4.  Use the oldest chronological printing available.

---

## Updating the Web App's Collections

These steps aren't useful without access to the Cloudflare R2 bucket, but I'm including them here for posterity.

The web app is almost the same as the desktop app, but it doesn't include the collection management features,
making its database effectively read-only.


1.  Use the CLI to remove the old version of the collection (if replacing an existing one), and then add the new collection.
    ```bash
    proxynexus-cli collection remove <collection_name>
    proxynexus-cli collection add <new_collection.pnx>
    ```

2.  Sync the local `~/.proxynexus/collections` directory up to that bucket.
    ```bash
    rclone sync ~/.proxynexus/collections r2-bucket-name:proxynexus-collections --progress
    ```

3.  Export the local DB, containing the new collection metadata, as a new `init.sql` payload that the web app hydrates from.
    ```bash
    proxynexus-cli export --output proxynexus-gui/public/init.sql
    ```

4.  Run the web app locally (`dx serve --platform web`) to ensure the new `init.sql` loads correctly
    and the images are fetching from R2 as expected.

5.  Commit the updated `init.sql` file and merge it to `master`. GitHub will build and deploy the web app release files to Cloudflare Pages.

---

## Technical Notes

### Image Pre-Processing

#### Corner Infill
You might notice that real FFG cards have rounded corners, but the images used by Proxy Nexus are rectangular. 
This is because all images have been processed with the **Corner Infill Script**, located in `utils/corner_infill/`. 
This script uses OpenCV to detect the blank white corners of raw card scans, and fills them in using the Navier-Stokes
inpainting algorithm (`cv2.inpaint`). 

#### Page Slicer
To extract the raw image files from the NSG Print and Play PDFs, I use [pdfimager](https://github.com/sckott/pdfimager).
For some PDFs, each card is saved as a separate image. For others, only full-page 3x3 grid images are saved.
In order to "slice" these full page images, I used the **NSG Page Slicer Script**, located in  `utils/nsg_page_slicer/`. 
For more details on how this script works, please refer to the `utils/nsg_page_slicer/README.md`.

### MPC Processing

When generating images formatted for PDFs, images are used as-is from their collections.
However, when generating for MakePlayingCards.com, additional processing is done to each image on-the-fly.

#### Edge Replication
When printing physical proxies, because they require a print-safe bleed border, we need to extend the edges of the images. 
The old Proxy Nexus website used an entirely duplicate set of images, which were pre-processed with this bleed border.
That pre-processing relied on OpenCV, just like the corner infilling does. However, with this project's goal of 
supporting flexible collection management, and being written in Rust targeting WASM for the web app, 
OpenCV could not be used for its copyMakeBorder function. 

Instead, the `proxynexus-core` contains its own `add_bleed_border` function in `proxynexus-core/src/print_prep.rs`. 
It iteratively copies the outer edge pixels and rapidly blits them outward to create a seamless, print-ready bleed natively in Rust.
I benchmarked this function against a version that used the Rust bindings of OpenCV's copyMakeBorder, and while mine is 
slower, it's quite good enough for keeping the project as purely Rust as possible.

#### The Uniqueness Marker
Most orders on MPC will contain duplicates of the same image. It's also very convenient to use their
"place images for me" autofill feature. However, MPC's image upload will notice when duplicates of the same identical
image are uploaded, and skip them. This effectively breaks the autofill feature, meaning you'd need to manually
place the same image for the number of copies you want.

To bypass this, the MPC generation process applies a "Uniqueness Marker" (`apply_uniqueness_marker` in `print_prep.rs`).
It imperceptibly alters the RGB values of the top-left 2x2 pixels using a pseudo-random addition based on the number
of copies being made. These altered pixels get cut off anyways, because they're well in the bleed border.
This guarantees every file inside the generated `.zip` is technically unique as far as MPC is
concerned, and every file gets uploaded.

### Image Caching

When generating a large list of cards, it's likely that the app will be fetching and using the same image file more than once.
To save on network bandwidth and processing time, both the PDF and MPC generation processes make use of caching.

*   **PDF Generation:** Once the image bytes from the provider are obtained, it is only parsed into a `krilla::Image` structure once,
and stored in the cache. This cached copy is then used when adding additional copies of this same image to the PDF.
*   **MPC Generation:** This follows a similar process except it caches the image *after* the heavy bleed border is applied, 
but *before* the uniqueness marker is stamped. This ensures the expensive `add_bleed_border` function only runs once per file, 
while still allowing the fast uniqueness marker to stamp each individual copy just before it is written to the zip archive. 


## Rebuilt in Rust

This repo is a rebuild of [the previous project](https://github.com/axmccx/proxynexus/), and it aims to improve
all the flaws of that version.

Having a backend server generate each request was a huge flaw. The server was cheap to run but not free.
At the time, I felt clever building its caching system, but it would frequently run out of storage space, requiring
automated scripts to clear the cache and reboot the server. It sucked to see people online saying that the
website was down.

The database design wasn't great either. It relied on seed files to populate the DB, making it
super tedious to update the website with new cards. Admittedly, I never put much thought into the process of ongoing
card updates. I figured eventually everyone would just use NSG's print-and-play PDFs.

I built the old website with Node.js on the backend and vanilla JS on the frontend. It felt good to avoid
using a frontend framework and keeping things lightweight, but man, the backend was ugly and unpleasant to work with.
Lastly, I felt the use of Azure blob storage for the images and caching made it really hard for anyone else to stand up
the website on their end and make contributions.

Since last year, I've been learning Rust and decided that building a Proxy Nexus CLI in Rust would be a good learning exercise.
Since Rust is fun to use, I went looking for a UI framework and found Dioxus. Fascinated with its support for WASM web apps
as a compile target, it filled me with motivation to rebuild the website in Rust, just to see how well that could work.

In addition to supporting all the existing features, I had the following goals:

* Be able to run everything locally. All image processing and the database should be able to run entirely in the browser.
* Be free to host. Though the web version still needs a hosting service, luckily Cloudflare R2's free tier is good enough.
* Enable anyone to manage card images. This was a foundational change, but it would push me to set up the project 
in such a way that adding new cards would be as easy as possible. While I plan to keep the hosted web app updated, 
I would feel incredibly accomplished to see someone else create and share their own collection `.pnx` file!

This makes the website faster, more stable, free to host, a pleasure for me to work on,
and hopefully easier for anyone to dive into the codebase.
