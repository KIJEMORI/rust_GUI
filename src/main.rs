mod window;

use std::rc::Rc;

use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::button::Button;
use crate::window::component::interface::component_control::LabelControl;
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::label::Label;
use crate::window::component::layout::const_base_layout::{ConstBaseLayout, Direction};
use crate::window::component::layout::row_layout::RowLayout;
use crate::window::component::panel::Panel;
use crate::window::{
    app::App, component::interface::component_control::ComponentControl,
    component::interface::component_control::PanelControl,
};
fn main() {
    let mut app = App::new();
    let layout = RowLayout::new();
    app.set_layout(layout);

    let mut panel = Panel::default();

    let mut label1 = Label::from_str("Новая игра");

    label1.set_background(0xFF000000);
    label1.set_font_color(0xFF00FF00);

    panel.base.id = "LOL".to_string();
    panel.set_height(40);
    panel.set_width(400);
    panel.set_background(0xFF0000FF);

    let label1 = panel.add(label1);

    let panel_hov = app.add(panel);
    {
        let panel_setting = Rc::clone(&panel_hov);
        panel_hov
            .borrow_mut()
            .set_on_mouse_enter(UiCommand::ChangeColor(
                Some(panel_setting.clone()),
                0xFF00FFFF,
            ));
        let panel_setting = Rc::clone(&panel_hov);
        panel_hov
            .borrow_mut()
            .set_on_mouse_leave(UiCommand::ChangeColor(
                Some(panel_setting.clone()),
                0xFFFFFF00,
            ));
    }
    //let mut button = Label::from_str("FFFFFFFFF FFFFFFFFFFFFFFF");
    let btn_action = UiCommand::Batch(vec![
        UiCommand::ChangeColor(None, 0xFF00FFFF),
        UiCommand::SetText(None, "Успешно!".into()),
    ]);

    let mut button = Button::new("Продолжить игру", btn_action);
    // button.set_height(40);
    // button.set_width(500);

    button.set_background(0xFF00FFFF);
    button.set_font_color(0xFFFF00FF);

    // button.set_margin(Direction {
    //     up: 10,
    //     down: 10,
    //     right: 10,
    //     left: 10,
    // });

    //button.set_color(0xFF00FF00);
    let mut panel = Panel::default();

    panel.base.id = "LOL2".to_string();
    panel.set_height(40);
    panel.set_width(400);
    panel.set_margin(Direction {
        up: 10,
        down: 10,
        right: 10,
        left: 150,
    });
    panel.set_padding(Direction {
        up: 10,
        down: 10,
        right: 10,
        left: 10,
    });
    panel.set_background(0xFF000000);

    panel.add(button);

    let mut c = ConstBaseLayout::new();

    let panel = app.add(panel);

    // let mut panel = Panel::default();
    // panel.set_height(40);
    // panel.set_width(100);

    let mut label2 = Label::new("Настройки".to_string());
    label2.set_height(40);
    label2.set_width(400);
    label2.set_font_color(0xFF000000);
    label2.set_background(0xFFFF0000);

    app.add(label2);

    //app.add(panel);
    //

    let mut panel = Panel::default();

    panel.base.id = "pan pad".to_string();
    panel.set_height(40);
    panel.set_width(400);
    panel.set_background(0xFF0000FF);

    panel.set_padding(Direction {
        up: 10,
        down: 10,
        right: 10,
        left: 10,
    });

    let mut panel_ch = Panel::default();

    panel_ch.base.id = "pan ch".to_string();
    panel_ch.set_height(40);
    panel_ch.set_width(400);
    panel_ch.set_background(0xFFFF0000);

    panel.add(panel_ch);

    app.add(panel);

    app.run();

    print!("lol");
}
