
use std::cell::RefCell;
use std::rc::{Weak, Rc};

use gtk_estate::gtk4::traits::{BoxExt, WidgetExt};
use gtk_estate::{HasObject, impl_has_application_window, impl_has_object, StateContainers}; //get_state_containers, 

use gtk_estate::gtk4::{self as gtk, Box, Orientation};

use gtk_estate::adw::{Application, ApplicationWindow, HeaderBar, WindowTitle, prelude::AdwApplicationWindowExt, gtk::prelude::ApplicationWindowExt, gtk::prelude::GtkWindowExt};

use gtk_estate::corlib::{NonOption, rc_self_rfc_setup}; //rc_self_refcell_setup};

use crate::window_contents_state::WindowContentsState;

pub struct WindowState
{

    weak_self: NonOption<Weak<RefCell<Self>>>, //RefCell<>,
    window: ApplicationWindow,
    //window_title: WindowTitle,
    //hb: HeaderBar,
    //contents: Box 

}

impl WindowState
{

    pub fn new(app: &Application) -> Rc<RefCell<Self>>
    {

        //setup GTK/Adw objects

        //let window_title = WindowTitle::new("Unix Time", "");

        //let hb = HeaderBar::builder().title_widget(&window_title).build();

        //let contents = Box::new(Orientation::Vertical, 0);

        let window = ApplicationWindow::builder()
                .application(app)
                .default_width(1000)
                .default_height(1000)
                //.title("Rustpad")
                //.show_menubar(true)
                //.content(&contents)
                .build();

        let this = Self
        {

            weak_self: NonOption::invalid(),
            window, //: ApplicationWindow::new(app),
            //window_title,
            //hb,
            //contents

        };

        let rc_self = Rc::new(RefCell::new(this));

        //setup weak self reference

        //rc_self_refcell_setup!(rc_self, weak_self);

        rc_self_rfc_setup!(rc_self, weak_self);
        
        //get the state containers singletion

        let scs = StateContainers::get(); //get_state_containers();

        //add this application window

        //scs.get_adw_state_ref().get_application_windows_mut().add_refcell(&rc_self);

        //scs.get_adw_ref().get_application_windows_mut().add_refcell(&rc_self);

        scs.adw().borrow_mut_application_windows().add_refcell(&rc_self);

        {

            //add window contents

            let rc_self_ref = rc_self.borrow();

            //rc_self_ref.window.set_child(Some(&rc_self_ref.contents));

            //rc_self_ref.contents.append(&rc_self_ref.hb);

            WindowContentsState::new(&rc_self_ref.window);

            rc_self_ref.window.show();

        }
        
        //contents.add_controller(controller)

        //done!

        rc_self

    }

}

impl_has_application_window!(window, WindowState);

/*
impl HasObject<ApplicationWindow> for WindowState
{

    fn get_object(&self) -> &ApplicationWindow
    {

        &self.window
        
    }

}
*/
