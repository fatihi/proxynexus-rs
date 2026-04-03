use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PrintLayoutInfoProps {
    pub on_close: EventHandler<()>,
    pub pos: (f64, f64, f64),
}

#[component]
pub fn PrintLayoutInfo(props: PrintLayoutInfoProps) -> Element {
    let (x, y, w) = props.pos;

    rsx! {
        div {
            class: "fixed inset-0 z-[2000]",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "absolute bg-white p-6 rounded-lg shadow-2xl border border-gray-200 w-80",
                style: "top: {y - 12.0}px; left: {x + w / 2.0}px; transform: translate(-50%, -100%);",
                onclick: move |evt| evt.stop_propagation(),

                button {
                    class: "absolute top-4 right-4 text-gray-400 hover:text-gray-600 focus:outline-none transition-colors",
                    onclick: move |_| props.on_close.call(()),
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M6 18L18 6M6 6l12 12" }
                    }
                }

                div { class: "flex flex-col gap-4 text-sm mt-2",
                    div {
                        h4 { class: "font-semibold", "Edge (Edge-to-Edge)" }
                        p { class: "text-gray-600", "Cards are arranged directly next to each other." }
                    }
                    div {
                        h4 { class: "font-semibold", "Gap" }
                        p { class: "text-gray-600", "Adds a 1/8\" (0.125\") gap between cards, while preserving their original size." }
                    }
                    div {
                        h4 { class: "font-semibold", "S Margin (Small Margin)" }
                        p { class: "text-gray-600", "Adds a 1mm white border around each card, scaling down the card proportionally." }
                    }
                    div {
                        h4 { class: "font-semibold", "L Margin (Large Margin)" }
                        p { class: "text-gray-600", "Adds a 2mm white border around each card, scaling down the card proportionally." }
                    }
                }
            }
        }
    }
}
