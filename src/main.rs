mod disk;
mod window;

#[cfg(feature = "3d_render")]
use std::cell::Cell;
use std::rc::Rc;

#[cfg(feature = "3d_render")]
use crate::window::component::base::ui_3d_command::Ui3DCommand;
use crate::window::component::base::ui_command::UiCommand;
#[cfg(feature = "3d_render")]
use crate::window::component::block_3d::model::cube::Cube;
#[cfg(feature = "3d_render")]
use crate::window::component::block_3d::model::sphere::Sphere;
#[cfg(feature = "3d_render")]
use crate::window::component::block_3d::model::tor::Tor;
#[cfg(feature = "3d_render")]
use crate::window::component::block_3d::type_union;
#[cfg(feature = "3d_render")]
use crate::window::component::block_3d::viewport::Viewport3D;
use crate::window::component::button::Button;
use crate::window::component::edit_label::EditLabel;
use crate::window::component::interface::component_control::{ComponentControlExt, LabelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::label::Label;
#[cfg(feature = "3d_render")]
use crate::window::component::layout::column_layout::ColumnLayout;
use crate::window::component::layout::const_base_layout::{ConstBaseLayout, Direction};
use crate::window::component::layout::row_layout::RowLayout;
use crate::window::component::panel::Panel;
use crate::window::component::scroll_panel::ScrollPanel;
use crate::window::{
    app::App, component::interface::component_control::ComponentControl,
    component::interface::component_control::PanelControl,
};

fn main_1() {
    let mut app = App::new();
    let layout = RowLayout::new();
    app.set_layout(layout);

    let mut panel = ScrollPanel::default();
    let layout = RowLayout::new();
    panel.set_layout(layout);

    panel
        .as_dragable_mut()
        .unwrap()
        .set_dragable(true)
        .set_in_drag(UiCommand::SetPosition(
            Cell::new(None),
            Cell::new(0.0),
            Cell::new(0.0),
        ));
    //.set_rails(window::component::managers::drag_manager::DragRails::Horizontal);

    let mut label1 = EditLabel::new("Новая игра");

    label1.as_panel_control_mut().set_background(0xFF000000);
    label1
        .as_label_control_mut()
        .unwrap()
        .set_font_color(0xFF00FF00);

    //panel.base.id = "LOL".to_string();
    panel
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFFFF0000);

    let mut c = ConstBaseLayout::new();
    c.set_relative_width(50);
    c.set_relative_height(50);
    panel
        .as_layout_control_mut()
        .set_const_layout(Some(Box::new(c)));

    panel.add(label1);

    for _ in 0..2 {
        let mut label = Label::from_str("LOL");
        label.as_panel_control_mut().set_background(0xFF00FF00);
        panel.add(label);
    }

    let mut panel_2 = ScrollPanel::default();
    let layout = RowLayout::new();
    panel_2.set_layout(layout);

    panel_2
        .as_panel_control_mut()
        .set_height(100)
        .set_width(400)
        .set_background(0xFF0000FF);

    //panel_2.base.id = "LOL2".to_string();

    for _ in 0..10 {
        let mut label = Label::from_str("LOL2");
        label.as_panel_control_mut().set_background(0xFFFF00FF);
        panel_2.add(label);
    }

    panel.add(panel_2);

    for _ in 0..2000 {
        let mut label = Label::from_str("LOL");
        label.as_panel_control_mut().set_background(0xFF00FFFF);
        panel.add(label);
    }

    let mut label = Label::from_str("LOL");
    label.as_panel_control_mut().set_background(0xFFFF00FF);
    panel.add(label);

    let panel_hov = app.add(panel);
    {
        let panel_setting = Rc::clone(&panel_hov);
        let id = panel_setting.borrow().as_base().id;
        if let Some(hovearable) = panel_hov.borrow_mut().as_hoverable_mut() {
            hovearable
                .set_on_mouse_enter(UiCommand::ChangeColor(Cell::new(Some(id)), 0xFFAA0AA0))
                .set_on_mouse_leave(UiCommand::ChangeColor(Cell::new(Some(id)), 0xFFFFFF00));
        }
    }

    //let mut button = Label::from_str("FFFFFFFFF FFFFFFFFFFFFFFF");
    let btn_action = UiCommand::Batch(vec![
        UiCommand::ChangeColor(Cell::new(None), 0xFF00FFFF),
        UiCommand::SetText(Cell::new(None), "Успешно!".into()),
    ]);

    let mut button = Button::new("Продолжить игру", btn_action);
    // button.set_height(40);
    // button.set_width(500);

    button.as_base_mut().settings.background_color = 0xFF00FFFF;
    button
        .as_label_control_mut()
        .unwrap()
        .set_font_color(0xFFFF00FF);

    // button.set_margin(Direction {
    //     up: 10,
    //     down: 10,
    //     right: 10,
    //     left: 10,
    // });

    //button.set_color(0xFF00FF00);
    let mut panel = Panel::default();

    panel
        .set_height(40)
        .set_width(400)
        .set_background(0xAA000000);

    let mut c = ConstBaseLayout::new();
    c.set_relative_width(100);
    c.set_relative_height(50);
    panel
        .as_layout_control_mut()
        .set_const_layout(Some(Box::new(c)))
        .set_padding(Direction {
            up: 10,
            down: 10,
            right: 10,
            left: 10,
        })
        .set_margin(Direction {
            up: 10,
            down: 10,
            right: 10,
            left: 50,
        });

    panel.add(button);

    app.add(panel);

    // let mut panel = Panel::default();
    // panel.set_height(40);
    // panel.set_width(100);

    let mut label2 = Label::new("Настройки".to_string());
    label2
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFFFF0000);
    label2.set_font_color(0xFF000000);

    app.add(label2);

    let mut panel = Panel::default();

    panel
        .set_height(40)
        .set_width(400)
        .set_background(0xFF0000FF);

    panel.as_layout_control_mut().set_padding(Direction {
        up: 10,
        down: 10,
        right: 10,
        left: 10,
    });

    let mut panel_ch = Panel::default();

    panel_ch
        .set_height(40)
        .set_width(400)
        .set_background(0xFFFF0000);

    panel.add(panel_ch);

    app.add(panel);

    app.run();

    print!("lol");
}

#[cfg(feature = "3d_render")]
pub fn main_2() {
    let mut app = App::new();
    let layout = RowLayout::new();
    app.set_layout(layout);

    let mut panel = Viewport3D::new();
    let layout = RowLayout::new();
    panel.set_layout(layout);

    panel
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFFAAAAAA);

    panel
        .as_dragable_mut()
        .unwrap()
        .set_dragable(true)
        .set_in_drag(Ui3DCommand::rotate_camera());

    panel
        .as_scrollable_mut()
        .unwrap()
        .set_scrolable(true)
        .set_on_scroll(Ui3DCommand::change_distance_camera());

    panel
        .as_clickable_mut()
        .unwrap()
        .set_on_click(Ui3DCommand::paint());

    let mut c = ConstBaseLayout::new();
    c.set_relative_width(100);
    c.set_relative_height(90);
    panel
        .as_layout_control_mut()
        .set_const_layout(Some(Box::new(c)));

    panel.add_model(Sphere::new_with_color(2.0, [0.0, 0.0, 0.0], 0xFF55FF55));

    let panel = app.add(panel);

    let mut control_panel = Panel::default();
    control_panel.set_layout(ColumnLayout::new());
    let mut c = ConstBaseLayout::new();
    c.set_relative_width(100);
    c.set_relative_height(50);
    control_panel
        .as_layout_control_mut()
        .set_const_layout(Some(Box::new(c)));

    let mut union_button = Label::new("Объединение".to_string());
    union_button
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFF999999);
    union_button.set_font_color(0xFF000000);
    union_button.as_layout_control_mut().set_padding(Direction {
        up: 2,
        down: 2,
        right: 2,
        left: 2,
    });

    union_button
        .as_clickable_mut()
        .unwrap()
        .set_on_click(UiCommand::RefCommand(
            Rc::clone(&panel),
            Box::new(Ui3DCommand::change_union_pencil(type_union::UNION)),
        ));

    control_panel.add(union_button);

    let mut subtraction_button = Label::new("Вычитание".to_string());
    subtraction_button
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFF999999);
    subtraction_button.set_font_color(0xFF000000);
    subtraction_button
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 5,
        });

    subtraction_button
        .as_clickable_mut()
        .unwrap()
        .set_on_click(UiCommand::RefCommand(
            Rc::clone(&panel),
            Box::new(Ui3DCommand::change_union_pencil(type_union::SUBTRACTION)),
        ));

    control_panel.add(subtraction_button);

    let mut smooth_button = Label::new("Плавное объединение".to_string());
    smooth_button
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFF999999);
    smooth_button.set_font_color(0xFF000000);
    smooth_button
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 5,
        });

    smooth_button
        .as_clickable_mut()
        .unwrap()
        .set_on_click(UiCommand::RefCommand(
            Rc::clone(&panel),
            Box::new(Ui3DCommand::change_union_pencil(type_union::SMOOTH)),
        ));

    control_panel.add(smooth_button);

    let mut drawing_button = Label::new("Рисование цветом".to_string());
    drawing_button
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFF999999);
    drawing_button.set_font_color(0xFF000000);
    drawing_button
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 5,
        });

    drawing_button
        .as_clickable_mut()
        .unwrap()
        .set_on_click(UiCommand::RefCommand(
            Rc::clone(&panel),
            Box::new(Ui3DCommand::change_union_pencil(type_union::COLOR_DRAWING)),
        ));

    control_panel.add(drawing_button);

    app.add(control_panel);

    let mut color_panel = Panel::default();
    color_panel.set_layout(ColumnLayout::new());
    let mut c = ConstBaseLayout::new();
    c.set_relative_width(100);
    c.set_relative_height(100);
    color_panel
        .as_layout_control_mut()
        .set_const_layout(Some(Box::new(c)));

    let mut label = Label::new("R:".to_string());
    label
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0x00999999);
    label.set_font_color(0xFF000000);
    label
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 0,
        });

    color_panel.add(label);

    let mut r_color = EditLabel::new("85");
    r_color
        .as_panel_control_mut()
        .set_height(40)
        .set_width(50)
        .set_background(0xFF000000);
    r_color
        .as_label_control_mut()
        .unwrap()
        .set_font_color(0xFFFFFFFF)
        .fit_to_content(false);
    r_color
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 0,
        });

    let r_color = color_panel.add(r_color);

    let mut label = Label::new("G:".to_string());
    label
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0x00999999);
    label.set_font_color(0xFF000000);
    label
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 0,
        });

    color_panel.add(label);

    let mut g_color = EditLabel::new("85");
    g_color
        .as_panel_control_mut()
        .set_height(40)
        .set_width(50)
        .set_background(0xFF000000);
    g_color
        .as_label_control_mut()
        .unwrap()
        .set_font_color(0xFFFFFFFF)
        .fit_to_content(false);
    g_color
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 0,
        });

    let g_color = color_panel.add(g_color);

    let mut label = Label::new("B:".to_string());
    label
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0x00999999);
    label.set_font_color(0xFF000000);
    label
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 0,
        });

    color_panel.add(label);

    let mut b_color = EditLabel::new("85");
    b_color
        .as_panel_control_mut()
        .set_height(40)
        .set_width(50)
        .set_background(0xFF000000);
    b_color
        .as_label_control_mut()
        .unwrap()
        .set_font_color(0xFFFFFFFF)
        .fit_to_content(false);
    b_color
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 0,
        });

    let b_color = color_panel.add(b_color);

    let mut pallete = Panel::default();
    pallete
        .as_panel_control_mut()
        .set_height(40)
        .set_width(50)
        .set_background(0xFF555555);
    pallete
        .as_layout_control_mut()
        .set_padding(Direction {
            up: 2,
            down: 2,
            right: 2,
            left: 2,
        })
        .set_margin(Direction {
            up: 0,
            down: 0,
            right: 0,
            left: 10,
        });

    let pallete = color_panel.add(pallete);
    let pallete_rc_r = Rc::clone(&pallete);
    let pallete_rc_g = Rc::clone(&pallete);
    let pallete_rc_b = Rc::clone(&pallete);

    let viewport_1 = Rc::clone(&panel);
    let viewport_2 = Rc::clone(&panel);
    let viewport_3 = Rc::clone(&panel);

    r_color
        .borrow_mut()
        .as_keyboard_control_mut()
        .unwrap()
        .set_on_enter_press(UiCommand::Custom(
            Cell::new(None),
            Rc::new(move |sd| {
                let mut final_color = pallete_rc_r.borrow().as_panel_control().get_background();

                let color = sd
                    .borrow()
                    .as_label_control()
                    .unwrap()
                    .get_text()
                    .to_string();

                let a = (final_color >> 24) & 0xFF;
                let r = (final_color >> 16) & 0xFF;
                let g = (final_color >> 8) & 0xFF;
                let b = final_color & 0xFF;

                if let Ok(r) = color.parse::<i32>() {
                    let r = r.min(255).max(0) as u32;
                    final_color = (a << 24) | (r << 16) | (g << 8) | b;
                }

                pallete_rc_r
                    .borrow_mut()
                    .as_panel_control_mut()
                    .set_background(final_color);

                if let Some(viewport) = viewport_1.borrow_mut().as_paintful_mut() {
                    viewport.change_color_pencil(final_color);
                }
            }),
        ));

    g_color
        .borrow_mut()
        .as_keyboard_control_mut()
        .unwrap()
        .set_on_enter_press(UiCommand::Custom(
            Cell::new(None),
            Rc::new(move |sd| {
                let mut final_color = pallete_rc_g.borrow().as_panel_control().get_background();

                let color = sd
                    .borrow()
                    .as_label_control()
                    .unwrap()
                    .get_text()
                    .to_string();

                let a = (final_color >> 24) & 0xFF;
                let r = (final_color >> 16) & 0xFF;
                let g = (final_color >> 8) & 0xFF;
                let b = final_color & 0xFF;

                if let Ok(g) = color.parse::<i32>() {
                    let g = g.min(255).max(0) as u32;
                    final_color = (a << 24) | (r << 16) | (g << 8) | b;
                }

                pallete_rc_g
                    .borrow_mut()
                    .as_panel_control_mut()
                    .set_background(final_color);

                if let Some(viewport) = viewport_2.borrow_mut().as_paintful_mut() {
                    viewport.change_color_pencil(final_color);
                }
            }),
        ));

    b_color
        .borrow_mut()
        .as_keyboard_control_mut()
        .unwrap()
        .set_on_enter_press(UiCommand::Custom(
            Cell::new(None),
            Rc::new(move |sd| {
                let mut final_color = pallete_rc_b.borrow().as_panel_control().get_background();

                let color = sd
                    .borrow()
                    .as_label_control()
                    .unwrap()
                    .get_text()
                    .to_string();

                let a = (final_color >> 24) & 0xFF;
                let r = (final_color >> 16) & 0xFF;
                let g = (final_color >> 8) & 0xFF;
                let b = final_color & 0xFF;

                if let Ok(b) = color.parse::<i32>() {
                    let b = b.min(255).max(0) as u32;
                    final_color = (a << 24) | (r << 16) | (g << 8) | b;
                }

                pallete_rc_b
                    .borrow_mut()
                    .as_panel_control_mut()
                    .set_background(final_color);

                if let Some(viewport) = viewport_3.borrow_mut().as_paintful_mut() {
                    viewport.change_color_pencil(final_color);
                }
            }),
        ));

    app.add(color_panel);

    app.run();
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_os = "android"))]
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    //main_1();
    main_2();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_wasm() {
    // Перенаправляем паники Rust в консоль разработчика (F12)
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    // Запускаем асинхронный таск в браузере
    main_2();
}
