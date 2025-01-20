use pdfium_render::prelude::* ;
use iced::{ 
    alignment::Horizontal, executor, widget::{
        button, column, container, horizontal_rule, progress_bar, row, scrollable, svg, text, tooltip, vertical_space, Column, Tooltip
    }, window, Alignment, Element, Fill, Task, Theme 
};
use rust_embed::Embed;

use Impulsor_3000::choose_pdfium_by_os_arch;

use crate::impuls::{
    Impuls,
    ImpulsConvertingState,
    ImpulsModel
};

#[derive(Embed)]
#[folder = "imgs/svgs/"]
struct AssetImages;

pub enum PdfiumLibState {
    Ok(Pdfium),
    NotFound(String),
}

pub enum CurrentMode {
    Start,
    Default,
    Converting,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Exit,
    ConvertStart,
    ConvertNext,
    ConvertDone,
}

struct MainView {
    pdfs: Vec<ImpulsModel>,
    current_mode: CurrentMode,
    progress: usize,
    pdfium: PdfiumLibState,
}

impl MainView {
    fn process_file(&mut self) -> Task<Message> {
        if let PdfiumLibState::Ok(pdfium) = &self.pdfium {
            if self.progress < self.pdfs.len() {

                let impuls_model = &mut self.pdfs[self.progress];
                
                println!("Load Impuls: {}", &impuls_model.file_path);
                let impuls_loaded = Impuls::build_from_model(impuls_model, &pdfium);
                let impuls_loaded = impuls_loaded.unwrap();
    
                println!("Build HTML");
                impuls_model.state_html = match impuls_loaded.save_as_txt(impuls_model) {
                    Ok(_) => ImpulsConvertingState::Success,
                    Err(e) => ImpulsConvertingState::Failure(e.to_string()),
                };
    
                println!("Build Image");
                impuls_model.state_image = match impuls_loaded.save_as_jpg(impuls_model) {
                    Ok(_) => ImpulsConvertingState::Success,
                    Err(e) => ImpulsConvertingState::Failure(e.to_string()),
                };
    
                self.progress += 1;
                return Task::perform(async {()}, |_| Message::ConvertNext)
            }
            else {
                self.progress += 1;
                return Task::perform(async {()}, |_| Message::ConvertDone)
            }
        }
        else {
            return Task::perform(async {()}, |_| Message::ConvertDone)
        };
    }

    fn new() -> (MainView, Task<Message>) {

        let pdfium_lib_state = match choose_pdfium_by_os_arch() {
            Ok(pdfium_path) => {
                match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&pdfium_path)) {
                    Ok(pdfium) => {
                        let pdfium = Pdfium::new(pdfium);
                        PdfiumLibState::Ok(pdfium)
                    },
                    Err(e) => {
                        panic!("Pdfium library not found at {pdfium_path} \n Error: {e:?}")
                    },
                }
            },
            Err(e) => PdfiumLibState::NotFound(e),
        };       

        (
            Self {
                pdfs: vec![],
                current_mode: CurrentMode::Start,
                progress: 0,
                pdfium: pdfium_lib_state,
            }, Task::none()
        )
    }

    fn title(&self) -> String {
        String::from("Impulsor 3000")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConvertStart => {
                self.current_mode = CurrentMode::Converting;

                let picked_files = rfd::FileDialog::new()
                    .set_title("Impuls PDF-Datei(en) auswählen")
                    .add_filter("Impuls.pdf", &["pdf"])
                    //.set_directory("~")
                    .pick_files();

                self.pdfs = vec![];
                for file in &picked_files.unwrap() {
                    
                    let impuls = ImpulsModel::build_from_path_buf(
                        &file
                    );
                    self.pdfs.push(impuls);
                }

                self.progress = 0;
                return Task::perform(async {()}, |_| Message::ConvertNext)
            },
            Message::ConvertNext => {
                return self.process_file();
            },
            Message::ConvertDone => {
                self.current_mode = CurrentMode::Default;
            },
            Message::Exit => {
                return window::get_latest().and_then(window::close)
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {

        let title = container(
            svg(
                iced::widget::svg::Handle::from_memory(AssetImages::get("banner.svg").unwrap().data)
            )
            .width(Fill)
        )
            .max_height(150)
            .center_x(Fill);

        match self.current_mode {
            CurrentMode::Start => {
                let content = match &self.pdfium {
                    PdfiumLibState::Ok(_) => {
                        container(
                            column![
                                vertical_space().height(100),
                                button(container(text("Impuls-PDF-Datei(en) auswählen")).center_x(Fill).center_y(Fill))
                                    .on_press(Message::ConvertStart)
                                    .height(100).width(500),
                                // container(text("Oder Dateien in das Fenster ziehen"))
                                //     .height(100).width(500)
                                //     .center_x().center_y(),
                            ]
                        )
                    },
                    PdfiumLibState::NotFound(error_msg) => {
                        container(text(error_msg))
                    },
                };

                column![
                    title,
                    container(content)
                        .width(Fill)
                        .height(Fill)
                        .align_x(Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                ].align_x(Alignment::Center)
                .into()
            },

            CurrentMode::Default | CurrentMode::Converting => {
    
                let mut content = Column::new().align_x(Alignment::Center);

                fn build_icon_default() -> Tooltip<'static, Message> {
                    let handle_image_success = iced::widget::svg::Handle::from_memory(AssetImages::get("default.svg").unwrap().data);
                    let svg_image_success = svg(handle_image_success).height(20).width(20);
                    
                    let tooltip_message = container(text("Verarbeitung ausstehend")).style(container::bordered_box);
                    let image_success = tooltip(svg_image_success, tooltip_message, tooltip::Position::Left);
                    image_success
                }

                fn build_icon_image_success() -> Tooltip<'static, Message> {
                    let handle_image_success = iced::widget::svg::Handle::from_memory(AssetImages::get("image-success.svg").unwrap().data);
                    let svg_image_success = svg(handle_image_success).height(20).width(20);
                    
                    let tooltip_message = container(text("Bild erfolgreich erstellt")).style(container::bordered_box);
                    let image_success = tooltip(svg_image_success, tooltip_message, tooltip::Position::Left);
                    image_success
                }

                fn build_icon_image_error(error_msg: &str) -> Tooltip<'static, Message> {
                    let tooltip_message = format!("Bildumwandlung fehlerhaft! Fehler:\n{error_msg}");

                    let handle_image_error = iced::widget::svg::Handle::from_memory(AssetImages::get("image-error.svg").unwrap().data);
                    let svg_image_error = svg(handle_image_error).height(20).width(20);
                    
                    let tooltip_message = container(text(tooltip_message)).style(container::bordered_box);
                    let image_error = tooltip(svg_image_error, tooltip_message, tooltip::Position::Left);
                    image_error
                }

                fn build_icon_html_success() -> Tooltip<'static, Message> {
                    let handle_html_success = iced::widget::svg::Handle::from_memory(AssetImages::get("html-success.svg").unwrap().data);
                    let svg_html_success = svg(handle_html_success).height(20).width(20);
                    
                    let tooltip_message: container::Container<Message, _, iced::Renderer> = container(text("Wordpress-Text erfolgreich erstellt")).style(container::bordered_box);
                    let html_success = tooltip(svg_html_success, tooltip_message, tooltip::Position::Left);
                    html_success
                }

                fn build_icon_html_error(error_msg: &str) -> Tooltip<'static, Message> {
                    let tooltip_message = format!("Wordpress-Text konnte nicht erstellt werden! Fehler:\n{error_msg}");

                    let handle_html_error = iced::widget::svg::Handle::from_memory(AssetImages::get("html-error.svg").unwrap().data);
                    let svg_html_error = svg(handle_html_error).height(20).width(20);
                    
                    let tooltip_message: container::Container<Message, _, iced::Renderer> = container(text(tooltip_message)).style(container::bordered_box);
                    let html_error: Tooltip<Message, _, iced::Renderer> =  tooltip(svg_html_error, tooltip_message, tooltip::Position::Left);
                    html_error
                }

                for i in &self.pdfs {
                    let impuls_name = text(&i.file_name).align_x(Horizontal::Left).width(Fill);
                    let impuls_tip = tooltip(impuls_name, &*i.file_path, tooltip::Position::FollowCursor);

                    let impuls_state_html = match &i.state_html {
                        ImpulsConvertingState::Default => build_icon_default(),
                        ImpulsConvertingState::Success => build_icon_html_success(),
                        ImpulsConvertingState::Failure(msg) => build_icon_html_error(msg),
                    };

                    let impuls_state_image = match &i.state_image {
                        ImpulsConvertingState::Default => build_icon_default(),
                        ImpulsConvertingState::Success => build_icon_image_success(),
                        ImpulsConvertingState::Failure(msg) => build_icon_image_error(msg),
                    };

                    let impuls_state_html: container::Container<_, _, _> = container(impuls_state_html).align_x(Horizontal::Center);
                    let impuls_state_img = container(impuls_state_image).align_x(Horizontal::Right);

                    let r_i = row![impuls_tip, impuls_state_html, impuls_state_img].spacing(20);
                    
                    content = content.push(r_i);
                }
        
                //control-row
                let control_row = 
                    if matches!(self.current_mode, CurrentMode::Converting) {
                        row![
                            progress_bar(0.0..=self.pdfs.len() as f32, self.progress as f32).width(Fill)
                        ].spacing(20).padding(20)
                    } 
                    else { 
                        row![
                            container(
                                button("Neu umwandeln")
                                    .on_press(Message::ConvertStart)
                                    .style(button::secondary)
                            ).align_x(Horizontal::Left).width(Fill),
                            container(
                                button("Beenden")
                                    .on_press(Message::Exit)
                            ).align_x(Horizontal::Right).style(container::bordered_box)
                        ].spacing(20).padding(20)
                    };
                
                column!(
                    container(title).padding(20),
                    horizontal_rule(1),
                    container(scrollable(content)).padding(20).width(Fill),
                    vertical_space(),
                    horizontal_rule(1),
                    control_row,
                ).into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }
}

pub fn main() -> iced::Result {
    iced::application(MainView::title, MainView::update, MainView::view)
        .theme(MainView::theme)
        .run_with(MainView::new)
}
