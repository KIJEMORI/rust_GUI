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
use crate::window::component::block_3d::viewport::Viewport3D;
use crate::window::component::button::Button;
use crate::window::component::edit_label::EditLabel;
use crate::window::component::interface::component_control::{ComponentControlExt, LabelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::label::Label;
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
fn main_2() {
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

    let mut c = ConstBaseLayout::new();
    c.set_relative_width(90);
    c.set_relative_height(90);
    panel
        .as_layout_control_mut()
        .set_const_layout(Some(Box::new(c)));

    for i in 0..100 {
        for j in 0..100 {
            panel.add_model(Sphere::new(
                1.0,
                [
                    -100.0 / 2.0 + 1.5 * i as f32,
                    100.0 / 2.0 - 1.5 * j as f32,
                    0.0,
                ],
            ));
        }
    }

    //panel.add_model(Sphere::new(5.0, [0.0, 0.0, 0.0]));

    //panel.add_model(Cube::new(5.0, [0.0, 0.0, 0.0]));

    //panel.add_model(Tor::new(2.5, 0.5, [0.0, 1.5, 0.0]));

    app.add(panel);

    let mut label2 = Label::new("Настройки".to_string());
    label2
        .as_panel_control_mut()
        .set_height(40)
        .set_width(400)
        .set_background(0xFFFF0000);
    label2.set_font_color(0xFF000000);

    app.add(label2);

    app.run();

    print!("lol");
}

fn main() {
    //main_1();
    main_2();
}
