use chrono;
use druid::widget::{
    Button, Controller, CrossAxisAlignment, Either, FillStrat, Flex, FlexParams, Image, Label,
    Padding, TextBox, ZStack,
};
use druid::{
    lens, piet::InterpolationMode, Code, Data, Env, Event, EventCtx, FileDialogOptions, FileSpec,
    ImageBuf, Lens, Point, UnitPoint, Widget, WidgetExt,
};

use screenshots::Screen;
use std::ops::Index;
use std::time::{Duration, Instant, SystemTime};

use druid_widget_nursery::DropdownSelect;

use crate::data::{Format, Screenshot, self};
use image::*;
// use crate::saver::Saver;

//albero
pub fn ui_builder() -> impl Widget<Screenshot> {

    let mut col = Flex::column().with_child(Flex::row().with_child(
        Button::new("SCREEN 📷").on_click(|_ctx, data: &mut Screenshot, _env| {
            data.do_screen(_ctx);
        }),
    ).with_child(Button::new("Capture Area 🖱️")));

    let mut row = Flex::row();

    let button_modifica = Either::new(
        |data: &Screenshot, _: &Env| data.screen_fatto,
        Button::new("Modifica nome").on_click(|ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            data.name = data.new_name.clone();
            data.new_name = "".to_string();
            Screenshot::toggle_textbox_state(data);
            ctx.request_update();
        }),
        Label::new(""),
    );

    let gestisci_screen = Either::new(
        |data: &Screenshot, _: &Env| data.screen_fatto,
        Button::new("Gestisci screen").on_click(|ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            data.screen_window(ctx);
            ctx.request_update();
        }),
        Label::new(""),
    );

    // Creiamo un widget Either che può essere o una Label o una TextBox in base allo stato.
    let screen_name = Either::new(
        |data: &Screenshot, _: &Env| data.editing_name,
        TextBox::new()
            .lens(Screenshot::new_name)
            .controller(Enter {}),
        Label::new(|data: &Screenshot, _: &Env| {
            format!("{}{}", data.name, data.format.to_string())
        }),
    );

    let dropdown = DropdownSelect::new(vec![
        ("MainFormat", Format::MainFormat),
        ("Png", Format::Png),
        ("Jpg", Format::Jpg),
        ("Gif", Format::Gif),
    ])
    .lens(Screenshot::format)
    .disabled_if(|data: &Screenshot, _: &Env| data.name == "")
    .align_right();

    let button_save = Button::new("SAVE")
        .disabled_if(|data: &Screenshot, _: &Env| data.name == "")
        .on_click(move |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
            let rs = FileSpec::new("gif", &["gif"]);
            let txt = FileSpec::new("png", &["png"]);
            let other = FileSpec::new("Bogus file", &["foo", "bar", "baz"]);
            let save_dialog_options = FileDialogOptions::new()
                .allowed_types(vec![rs, txt, other])
                .default_type(txt)
                .default_name(data.name.clone())
                .name_label("Target")
                .title("Choose a target for this lovely file")
                .button_text("Export");

            ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(save_dialog_options.clone()))
        })
        .align_right();

    row.add_child(screen_name);
    row.add_child(button_modifica);
    row.add_child(gestisci_screen);

    let mut row2 = Flex::row();
    row2.add_child(dropdown);
    row2.add_child(button_save);

    col.add_default_spacer();

    // // col.add_child(row);
    // col.add_child(row2);
    // col

    ZStack::new(col.with_flex_child(row, FlexParams::new(1.0, CrossAxisAlignment::Start)))
        .with_aligned_child(Padding::new(5., row2), UnitPoint::BOTTOM_RIGHT)

    // .with_child(TextBox::new()
    //     .with_placeholder("file_name")
    //     // .with_placeholder(|data: &Screenshot, _: &Env| format!("{}{}", data.name, data.format.to_string()))
    //     .lens(Screenshot::name))
    //         .with_child(Label::new(|data: &Screenshot, _: &Env| {
    //             format!("{}{}", data.name, data.format.to_string())
    //         }))

    //         .with_child(Button::new("Modifca").on_click(|ctx: &mut EventCtx, data: &mut Screenshot, _env|{
    //             data.editing_name = true;
    //             ctx.request_update();
    //         })
    //         )
    //         .with_child(
    //             TextBox::new()
    //                 .lens(Screenshot::new_name)
    //                 // .expand_width()
    //                 // .padding(8.0),

    //                 // .event(ctx, event, data, env)
    //         )
    //         .with_flex_spacer(0.1)
    //         .with_child(
    //             DropdownSelect::new(vec![
    //                 ("MainFormat", Format::MainFormat),
    //                 ("Png", Format::Png),
    //                 ("Jpg", Format::Jpg),
    //                 ("Gif", Format::Gif),
    //             ])
    //             .lens(Screenshot::format)
    //             .disabled_if(|data: &Screenshot, _: &Env| data.name == ""),
    //         )
    //         .with_child(
    //             Button::new("SAVE")
    //                 .disabled_if(|data: &Screenshot, _: &Env| data.name == "")
    //                 .on_click(move |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
    //                     let rs = FileSpec::new("gif", &["gif"]);
    //                     let txt = FileSpec::new("png", &["png"]);
    //                     let other = FileSpec::new("Bogus file", &["foo", "bar", "baz"]);
    //                     let save_dialog_options = FileDialogOptions::new()
    //                         .allowed_types(vec![rs, txt, other])
    //                         .default_type(txt)
    //                         .default_name(data.name.clone())
    //                         .name_label("Target")
    //                         .title("Choose a target for this lovely file")
    //                         .button_text("Export");

    //                     ctx.submit_command(
    //                         druid::commands::SHOW_SAVE_PANEL.with(save_dialog_options.clone()),
    //                     )
    //                 }),
    //         ),
    // )

    // let todos = List::new(|| {
    //     let bg = Color::rgba8(0, 0, 0, 50);

    // Flex::row()
    //     .with_child(Label::new(|data: &Screenshot, _: &Env| data.name.clone()))
    //     .with_default_spacer()
    //     // .with_child(Checkbox::new("png").lens(TodoItem::checked))
    //     .with_flex_spacer(0.1)
    //     .with_child(DropdownSelect::new(vec![
    //         ("Png", Screenshot::new_with_values("daniel".to_string(), Format::Png)),
    //         ("Jpg",Screenshot::new_with_values("luca".to_string(), Format::Jpg)),
    //         ("Gif", Screenshot::new_with_values("deraj".to_string(), Format::Gif)),
    //     ]))
    // .with_child(Button::new("png").on_click(
    //     |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
    //         let data_clone = data.clone();
    //         // let mouse_position = ctx.window().get_position();
    //         let menu: Menu<TodoState> =
    //             Menu::empty().entry(MenuItem::new("Remove").on_activate(
    //                 move |_, main_data: &mut TodoState, _| {
    //                     let location = main_data.todos.iter().position( |n| n == &data_clone).unwrap();
    //                     main_data.todos.remove(location);
    //                 },
    //             ));
    //         ctx.show_context_menu(menu, Point::new(0., 0.))
    //     },
    // ))
    // .with_child(Button::new("jpg").on_click(
    //     |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
    //         let data_clone = data.clone();
    //         let menu: Menu<TodoState> =
    //             Menu::empty().entry(MenuItem::new("Remove").on_activate(
    //                 move |_, main_data: &mut TodoState, _| {
    //                     let location = main_data.todos.iter().position( |n| n == &data_clone).unwrap();
    //                     main_data.todos.remove(location);
    //                 },
    //             ));
    //         ctx.show_context_menu(menu, Point::new(0., 0.))
    //     },
    // ))
    // .with_child(Button::new("gif").on_click(
    //     |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
    //         let data_clone = data.clone();
    //         let menu: Menu<TodoState> =
    //             Menu::empty().entry(MenuItem::new("Remove").on_activate(
    //                 move |_, main_data: &mut TodoState, _| {
    //                     let location = main_data.todos.iter().position( |n| n == &data_clone).unwrap();
    //                     main_data.todos.remove(location);
    //                 },
    //             ));
    //         ctx.show_context_menu(menu, Point::new(0., 0.))
    //     },
    // ))
    // .with_child(Button::new("raw").on_click(
    //     |ctx: &mut EventCtx, data: &mut Screenshot, _env| {
    //         let data_clone = data.clone();
    //         let menu: Menu<TodoState> =
    //             Menu::empty().entry(MenuItem::new("Remove").on_activate(
    //                 move |_, main_data: &mut TodoState, _| {
    //                     let location = main_data.todos.iter().position( |n| n == &data_clone).unwrap();
    //                     main_data.todos.remove(location);
    //                 },
    //             ));
    //         ctx.show_context_menu(menu, Point::new(0., 0.))
    //     },
    // ))
    //         .background(bg)
    // })
    // .lens(TodoState::todos)
    // .scroll()
    // .vertical();

    // let clear_complete = Button::new("Clear Completed");
    //     // .on_click(|_, data: &mut TodoState, _| data.todos.retain(|item| !item.checked));

    // ZStack::new(Flex::column().with_child(header).with_flex_child(screen, 1.))
    //     .with_aligned_child(Padding::new(5., clear_complete), UnitPoint::BOTTOM_RIGHT)
}

struct Enter;

impl<W: Widget<Screenshot>> Controller<Screenshot, W> for Enter {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut Screenshot,
        env: &Env,
    ) {
        if let Event::KeyUp(key) = event {
            if key.code == Code::Enter {
                if data.new_name.trim() != "" {
                    data.name = data.new_name.clone();
                    data.new_name = "".to_string();
                    Screenshot::toggle_textbox_state(data);
                }
            }
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &Screenshot,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &Screenshot,
        data: &Screenshot,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}
