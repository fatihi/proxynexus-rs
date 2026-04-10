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
The app then needs images files of cards, which are added from collection `.pnx` files. 
The CLI is able to create these collections from a folder of card scans image files, and manage them in the app.

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
proxynexus-cli collection info
```

For more details about the CLI, check the "CLI Section" below.
[//]: # (TODO add a comprehensive list of CLI commands)

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
