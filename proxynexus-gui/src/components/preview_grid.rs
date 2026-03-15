use dioxus::prelude::*;
use proxynexus_core::models::Printing;

fn build_image_url(image_key: &str) -> String {
    #[cfg(feature = "desktop")]
    {
        format!("proxynexus://collections/{}", image_key)
    }

    #[cfg(feature = "web")]
    {
        format!("https://collections.proxynexus.net/{}", image_key)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PreviewGridProps {
    pub printings: Vec<Printing>,
}

#[component]
pub fn PreviewGrid(props: PreviewGridProps) -> Element {
    rsx! {
        div {
            class: "flex flex-wrap gap-4",
            for (index, printing) in props.printings.iter().enumerate() {
                div {
                    key: "{index}-front",
                    class: "w-[200px] overflow-hidden shadow-lg aspect-[2.5/3.5] bg-gray-400",
                    img {
                        src: "{build_image_url(&printing.image_key)}",
                        crossorigin: "anonymous",
                        class: "w-full h-full",
                        alt: "{printing.card_title}",
                    }
                }
                for (part_index, part) in printing.parts.iter().enumerate() {
                    div {
                        key: "{index}-{part_index}",
                        class: "w-[200px] overflow-hidden shadow-lg aspect-[2.5/3.5] bg-gray-400 opacity-90 border-2 border-dashed border-gray-400",
                        img {
                            src: "{build_image_url(&part.image_key)}",
                            crossorigin: "anonymous",
                            class: "w-full h-full",
                            alt: "{printing.card_title} ({part.name})",
                        }
                    }
                }
            }
        }
    }
}
