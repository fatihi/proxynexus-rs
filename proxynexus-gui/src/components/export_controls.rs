use dioxus::prelude::*;
use proxynexus_core::pdf::{CutLines, PageSize, PdfOptions, PrintLayout};

#[derive(Clone, PartialEq, Debug)]
pub enum ExportConfig {
    Pdf(PdfOptions),
    Mpc,
}

#[derive(Props, Clone, PartialEq)]
pub struct ExportControlsProps {
    pub progress: Signal<Option<f32>>,
    pub is_disabled: bool,
    pub on_generate: EventHandler<ExportConfig>,
}

#[derive(Clone, PartialEq, Debug)]
struct PageSizeValidation {
    result: Option<PageSize>,
    width_invalid: bool,
    height_invalid: bool,
}

#[component]
pub fn ExportControls(props: ExportControlsProps) -> Element {
    let mut export_format = use_signal(|| "pdf".to_string());
    let mut page_size_preset = use_signal(|| "Letter".to_string());
    let mut cut_lines = use_signal(CutLines::default);
    let mut print_layout = use_signal(PrintLayout::default);

    let mut custom_width = use_signal(|| "".to_string());
    let mut custom_height = use_signal(|| "".to_string());
    let mut custom_unit = use_signal(|| "in".to_string());

    let page_size_validation = use_memo(move || -> PageSizeValidation {
        match page_size_preset().as_str() {
            "A4" => PageSizeValidation {
                result: Some(PageSize::A4),
                width_invalid: false,
                height_invalid: false,
            },
            "Custom" => {
                let w_text = custom_width();
                let h_text = custom_height();

                let w = w_text.parse::<f32>();
                let h = h_text.parse::<f32>();

                let factor = if custom_unit() == "cm" {
                    1.0 / 2.54
                } else {
                    1.0
                };

                let w_valid = matches!(w, Ok(v) if (v * factor) > 0.0 && (v * factor) <= 60.0);
                let h_valid = matches!(h, Ok(v) if (v * factor) > 0.0 && (v * factor) <= 60.0);

                let result = match (w, h) {
                    (Ok(w_val), Ok(h_val)) if w_valid && h_valid => {
                        Some(PageSize::Custom(w_val * factor, h_val * factor))
                    }
                    _ => None,
                };

                PageSizeValidation {
                    result,
                    width_invalid: !w_valid && !w_text.is_empty(),
                    height_invalid: !h_valid && !h_text.is_empty(),
                }
            }
            _ => PageSizeValidation {
                result: Some(PageSize::Letter),
                width_invalid: false,
                height_invalid: false,
            },
        }
    });

    rsx! {
        div {
            class: "p-4 border-t border-gray-200 bg-gray-50 flex flex-col gap-4",
            h2 { class: "text-lg font-bold text-gray-800", "Export" }

            div { class: "flex flex-col gap-2",
                label { class: "text-sm font-medium text-gray-700", "Format" }
                select {
                    disabled: (props.progress)().is_some(),
                    class: "w-full p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm",
                    value: "{export_format()}",
                    onchange: move |evt| export_format.set(evt.value().clone()),
                    option { value: "pdf", "PDF" }
                    option { value: "mpc", "MakePlayingCards.com" }
                }
            }

            if export_format() == "pdf" {
                div { class: "flex flex-col gap-2",
                    label { class: "text-sm font-medium text-gray-700", "Page Size" }
                    select {
                        disabled: (props.progress)().is_some(),
                        class: "w-full p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm",
                        value: "{page_size_preset()}",
                        onchange: move |evt| page_size_preset.set(evt.value().clone()),
                        option { value: "Letter", "Letter" }
                        option { value: "A4", "A4" }
                        option { value: "Custom", "Custom" }
                    }
                }

                if page_size_preset() == "Custom" {
                    div { class: "flex gap-2 items-start pt-2",
                        div { class: "flex flex-col w-full gap-1",
                            input {
                                disabled: (props.progress)().is_some(),
                                class: if page_size_validation().width_invalid {
                                    "w-full p-2 border border-red-500 rounded-md outline-none focus:ring-2 focus:ring-red-400 bg-red-50 text-sm"
                                } else {
                                    "w-full p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm"
                                },
                                type: "text",
                                placeholder: "Width",
                                value: "{custom_width()}",
                                oninput: move |evt| custom_width.set(evt.value().clone())
                            }
                            if page_size_validation().width_invalid {
                                span { class: "text-xs text-red-500 font-medium", "Invalid" }
                            }
                        }
                        div { class: "flex flex-col w-full gap-1",
                            input {
                                disabled: (props.progress)().is_some(),
                                class: if page_size_validation().height_invalid {
                                    "w-full p-2 border border-red-500 rounded-md outline-none focus:ring-2 focus:ring-red-400 bg-red-50 text-sm"
                                } else {
                                    "w-full p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm"
                                },
                                type: "text",
                                placeholder: "Height",
                                value: "{custom_height()}",
                                oninput: move |evt| custom_height.set(evt.value().clone())
                            }
                            if page_size_validation().height_invalid {
                                span { class: "text-xs text-red-500 font-medium", "Invalid" }
                            }
                        }
                        select {
                            disabled: (props.progress)().is_some(),
                            class: "p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm h-[38px]",
                            value: "{custom_unit()}",
                            onchange: move |evt| custom_unit.set(evt.value().clone()),
                            option { value: "in", "in" }
                            option { value: "cm", "cm" }
                        }
                    }
                }

                div { class: "flex flex-col gap-2",
                    label { class: "text-sm font-medium text-gray-700", "Cut Lines" }
                    select {
                        disabled: (props.progress)().is_some(),
                        class: "w-full p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm",
                        value: match cut_lines() {
                            CutLines::None => "None",
                            CutLines::Margins => "Margins",
                            CutLines::FullPage => "FullPage",
                        },
                        onchange: move |evt| {
                            let selected = match evt.value().as_str() {
                                "None" => CutLines::None,
                                "FullPage" => CutLines::FullPage,
                                _ => CutLines::Margins,
                            };
                            cut_lines.set(selected);
                        },
                        option { value: "None", "None" }
                        option { value: "Margins", "Margins" }
                        option { value: "FullPage", "Full Page" }
                    }
                }

                div { class: "flex flex-col gap-2",
                    label { class: "text-sm font-medium text-gray-700", "Print Style" }
                    select {
                        disabled: (props.progress)().is_some(),
                        class: "w-full p-2 border border-gray-300 rounded-md outline-none focus:ring-2 focus:ring-blue-400 bg-white text-sm",
                        value: match print_layout() {
                            PrintLayout::EdgeToEdge => "EdgeToEdge",
                            PrintLayout::SmallMargin => "SmallMargin",
                            PrintLayout::LargeMargin => "LargeMargin",
                            PrintLayout::NarrowGap => "NarrowGap",
                            PrintLayout::WideGap => "WideGap",
                        },
                        onchange: move |evt| {
                            let selected = match evt.value().as_str() {
                                "EdgeToEdge" => PrintLayout::EdgeToEdge,
                                "SmallMargin" => PrintLayout::SmallMargin,
                                "LargeMargin" => PrintLayout::LargeMargin,
                                "NarrowGap" => PrintLayout::NarrowGap,
                                "WideGap" => PrintLayout::WideGap,
                                _ => PrintLayout::EdgeToEdge,
                            };
                            print_layout.set(selected);
                        },
                        option { value: "EdgeToEdge", "Edge-to-Edge" }
                        option { value: "SmallMargin", "Small Margin" }
                        option { value: "LargeMargin", "Large Margin" }
                        option { value: "NarrowGap", "Narrow Gap" }
                        option { value: "WideGap", "Wide Gap" }
                    }
                }
            }

            if let Some(p) = (props.progress)() {
                div { class: "flex flex-col gap-2 mt-2",
                    div { class: "w-full bg-gray-200 rounded-full h-4 overflow-hidden",
                        div {
                            class: "bg-blue-600 h-full transition-all duration-75",
                            style: "width: {p * 100.0}%",
                        }
                    }
                    div { class: "text-xs text-center text-gray-500 font-medium",
                        "{ (p * 100.0) as u32 }%"
                    }
                }
            } else {
                button {
                    class: if props.is_disabled {
                        "w-full py-2 px-4 bg-gray-300 text-gray-500 font-semibold rounded-md shadow-sm mt-2 cursor-not-allowed"
                    } else {
                        "w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-md shadow-sm transition-colors mt-2"
                    },
                    disabled: props.is_disabled,
                    onclick: move |_| {
                        if props.is_disabled { return; }
                        let config = match export_format().as_str() {
                            "mpc" => ExportConfig::Mpc,
                            _ => match page_size_validation().result {
                                Some(page_size) => ExportConfig::Pdf(PdfOptions {
                                    page_size,
                                    cut_lines: cut_lines(),
                                    print_layout: print_layout(),
                                }),
                                None => return,
                            }
                        };
                        props.on_generate.call(config);
                    },
                    "Generate"
                }
            }
        }
    }
}
