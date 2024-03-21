
use std::cell::RefCell;

use std::rc::{Weak, Rc};

use std::time::Duration;

use gtk_estate::corlib::events::SenderEventFunc;

use gtk_estate::corlib::rc_default::RcDefault;

use gtk_estate::gtk4::prelude::{BoxExt, WidgetExt};

use gtk_estate::{StateContainers, StoredWidgetObject, WidgetAdapter, WidgetStateContainer}; //HasObject, impl_has_box, impl_has_object, //get_state_containers, 

use gtk_estate::gtk4::{self as gtk, Box, Orientation, Label, BaselinePosition, Align};

use gtk_estate::adw::{Application, ApplicationWindow, HeaderBar, WindowTitle, prelude::AdwApplicationWindowExt, gtk::prelude::ApplicationWindowExt, gtk::prelude::GtkWindowExt};

use gtk_estate::corlib::{impl_as_any, rc_self_setup, NonOption, as_any::AsAny}; //, rc_self_refcell_setup};

use gtk_estate::time_out::*;

use gtk_estate::adw::{TabBar, TabPage, TabView};

use crate::graphql_tab_state::GraphQLTabState;

use tokio::runtime::{Runtime, Handle, Builder};

use gtk_estate::gtk4::glib::object::Cast;

use crate::applicaion_state::ApplicationState;

use std::any::Any;

//use time::OffsetDateTime;

pub struct WindowContentsState
{

    //weak_self: RefCell<NonOption<Weak<Self>>>,
    adapted_contents_box: Rc<WidgetAdapter<Box, WindowContentsState>>,
    contents_box: Box,
    //app_window: ApplicationWindow,
    window_title: WindowTitle,
    hb: HeaderBar,
    //unix_time_label: Label,
    //internal_content: Box,
    //time_out: RcTimeOut
    tv: TabView,
    tb: TabBar,
    tokio_rt_handle: Handle

}

impl WindowContentsState
{

    pub fn new() -> Rc<Self> //(app_window: &ApplicationWindow) -> Rc<Self>
    {

        let contents_box = Box::new(Orientation::Vertical, 0);

        contents_box.set_vexpand(true);

        //cbox.set_baseline_position(BaselinePosition::Center);

        //cbox.set_valign(Align::Center);

        //Add Contents
        
        //HeaderBar

        let window_title = WindowTitle::new("Req It", "");

        let hb = HeaderBar::builder().title_widget(&window_title).build();

        contents_box.append(&hb);

        //internal_content

        //let internal_content = Box::new(Orientation::Vertical, 0);

        //Label

        /*
        let unix_time_label = Label::new(Some("unix_time_label"));

        internal_content.append(&unix_time_label);

        internal_content.set_vexpand(true);

        internal_content.set_valign(Align::Center);
        */

        //contents_box.append(&internal_content);

        let tv = TabView::new(); //TabView::builder().

        let tb = TabBar::builder().view(&tv).autohide(false).expand_tabs(false).build(); //::new();

        contents_box.append(&tb);

        contents_box.append(&tv);

        //let time_out = TimeOut::new(Duration::new(1, 0), true);

        let scs = StateContainers::get();

        let tokio_rt_handle;

        {

            //let application = app_window.application().unwrap();

            //let adw_application: Application = application.into();
    
            //let adw_application = application.downcast_ref::<Application>().unwrap(); //application as Application;
    
            //let applications = scs.adw().borrow_applications();

            //let app_state = applications.get(&adw_application).unwrap();
    
            //let app_state = scs.application_state();

            //let app_state_ref = app_state.downcast_ref::<ApplicattionState>().unwrap();

            //let app_state_ref = app_state.as_any().downcast_ref::<ApplicationState>().expect("Error: Not ApplicattionState!");

            let app_state = scs.application_state();

            let app_state_ref = app_state.as_any().downcast_ref::<ApplicationState>().expect("Error: Not ApplicattionState!");
            
            tokio_rt_handle = app_state_ref.clone_tokio_rt_handle();

        }

        let this = Rc::new_cyclic( move |weak_self|
        {

            Self
            {

                //weak_self: NonOption::invalid_rfc(), //invalid_refcell(),
                adapted_contents_box: WidgetAdapter::new(&contents_box, weak_self),
                contents_box,
                //app_window: app_window.clone(),
                window_title,
                hb,
                //unix_time_label,
                //internal_content,
                //time_out
                tv,
                tb,
                //Add refcell for mutable state
                tokio_rt_handle

            }

        });

        //let rc_self = Rc::new(this); //Rc::new(RefCell::new(this));

        //setup weak self reference

        //rc_self_refcell_setup!(rc_self, weak_self);

        //rc_self_setup!(rc_self, weak_self);

        //get the state containers singletion

        let scs = StateContainers::get(); //get_state_containers();

        //add this to the GTK boxes state

        //scs.get_gtk_state_ref().get_boxes_mut().add(&rc_self);

        //scs.get_gtk_ref().get_boxes_mut().add(&rc_self);

        //scs.gtk().borrow_mut_boxes().add(&rc_self); //add_refcell(&rc_self);

        scs.add(&this);

        //let weak_self = rc_self.weak_self.clone();

        this.tv.connect_close_page(|this, tp|
        {

            //Show message box if query is not saved
            
            //

            //let scs = StateContainers::get();

            //Remove state

            //scs.get_adw_ref().get_tab_pages_mut().remove(tp);

            //scs.adw().borrow_mut_tab_pages().remove(&tp);

            //finish

            this.close_page_finish(tp, true);                

            //When a tab closes it should automatically remove itself from the StateContainers.

            true

        });

        /*
        let ws = rc_self.weak_self.borrow().as_ref().clone();

        let on_timeout: Rc<SenderEventFunc<Rc<TimeOut>>> = Rc::new(move |_sender| {

            if let Some(this) = ws.upgrade()
            {

                let utc_now = OffsetDateTime::now_utc();

                let uts = utc_now.unix_timestamp();

                this.unix_time_label.set_label(&uts.to_string());

            }
            else
            {

                return false;

            }

            true

        });

        rc_self.time_out.on_time_out_subscribe(&on_timeout);

        {

            //add window contents

            //let rc_self_ref = rc_self.cbox;

            //rc_self_ref.window.set_child(Some(&rc_self_ref.contents));

            //rc_self.cbox.append(&hb);

            //rc_self_ref.window.show();

        }
        
        //contents.add_controller(controller)

        */

        //app_window.set_content(Some(&this.contents_box));

        //Append a tab

        let new_gql_ts = GraphQLTabState::new(&this.tv, &this.tokio_rt_handle);

        //Center the main contents paned widget

        //https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/prelude/trait.WidgetExt.html#method.connect_parent_notify

        new_gql_ts.set_contents_paned_position_halved(app_window.default_width());

        //{

            //let ql_ts_borrowed = new_gql_ts.borrow();

            //ql_ts_borrowed.set_contents_paned_position_halved(app_window.width()); //returns 0

            //ql_ts_borrowed.set_contents_paned_position_halved(app_window.default_width());

        //}     

        //rc_self.time_out.set_reoccurs(true);

        //rc_self.time_out.start();

        //done!

        this

    }

    /*
    pub fn set_contents_paned_position_halved(&self, value: i32)
    {

        self.contents_paned.set_position(value / 2);

    }
     */

}

impl_as_any!(WindowContentsState);

impl WidgetStateContainer for WindowContentsState
{

    fn dyn_adapter(&self) -> Rc<dyn StoredWidgetObject>
    {

        self.adapted_contents_box.clone()

    }

}

//impl_has_box!(contents_box, WindowContentsState);

