
use std::cell::RefCell;

use std::rc::{Weak, Rc};

use std::time::Duration;

use gtk_estate::adw::ffi::AdwBreakpointBin;

use gtk_estate::adw::glib::clone;

use gtk_estate::adw::glib::object::ObjectExt;

use gtk_estate::corlib::events::SenderEventFunc;

use gtk_estate::corlib::rc_default::RcDefault;

use gtk_estate::gtk4::prelude::{BoxExt, WidgetExt};

use gtk_estate::{StateContainers, StoredWidgetObject, WidgetAdapter, WidgetStateContainer};

use gtk_estate::gtk4::{self as gtk, Box, Orientation, Label, BaselinePosition, Align};

use gtk_estate::adw::{Application, ApplicationWindow, HeaderBar, WindowTitle, prelude::AdwApplicationWindowExt, gtk::prelude::ApplicationWindowExt, gtk::prelude::GtkWindowExt};

use gtk_estate::corlib::{impl_as_any, as_any::AsAny};

//use gtk_estate::{RcTimeOut, TimeOut}; //time_out::*;

use gtk_estate::adw::{TabBar, TabPage, TabView};

//use crate::graphql_tab_state::GraphQLTabState;

use crate::WebSocketTabState;

use tokio::runtime::{Runtime, Handle, Builder};

use gtk_estate::gtk4::glib::object::Cast;

use crate::applicaion_state::ApplicationState;

use std::any::Any;

use gtk_estate::helpers::widget_ext::find_parent;

pub struct WindowContentsState
{

    adapted_contents_box: Rc<WidgetAdapter<Box, WindowContentsState>>,
    window_title: WindowTitle,
    hb: HeaderBar,
    tv: TabView,
    tb: TabBar,
    tokio_rt_handle: Handle

}

impl WindowContentsState
{

    pub fn new() -> Rc<Self>
    {

        let contents_box = Box::new(Orientation::Vertical, 0);

        contents_box.set_vexpand(true);

        //Add Contents
        
        //HeaderBar

        let window_title = WindowTitle::new("Req It", "");

        let hb = HeaderBar::builder().title_widget(&window_title).build();

        contents_box.append(&hb);

        let tv = TabView::new();

        let tb = TabBar::builder().view(&tv).autohide(false).expand_tabs(false).build();

        contents_box.append(&tb);

        contents_box.append(&tv);

        let scs = StateContainers::get();

        let tokio_rt_handle;

        {

            let app_state = scs.application_state();

            let app_state_ref = app_state.as_any().downcast_ref::<ApplicationState>().expect("Error: Not ApplicattionState!");
            
            tokio_rt_handle = app_state_ref.clone_tokio_rt_handle();

        }

        let this = Rc::new_cyclic( move |weak_self|
        {

            Self
            {

                adapted_contents_box: WidgetAdapter::new(&contents_box, weak_self),
                window_title,
                hb,
                tv,
                tb,
                tokio_rt_handle

            }

        });

        //get the state containers singletion

        let scs = StateContainers::get();

        scs.add(&this);

        this.tv.connect_close_page(|this, tp|
        {

            //Todo: Show message box if query is not saved.

            this.close_page_finish(tp, true);                

            //When a tab closes it should automatically be removed from the StateContainers.

            true

        });

        this.adapted_contents_box.widget().connect_parent_notify(clone!(@strong this => move |root|
        {

            //If the TabView has any pages do nothing.

            if this.tv.n_pages() > 0
            {

                return;

            }
            
            //if let Some(parent) = root.parent()
            //{

            /*
            let parent = root.parent().expect("Must have parent!");

            //Comments kept for "parental debugging".

            println!("Direct parent: {}", parent.type_().name()); //Direct parent: AdwBreakpointBin

            //let bp_parent = parent.downcast_ref::<AdwBreakpointBin>().expect("Error: Must be an AdwBreakpointBin");

            let pp_widget = parent.parent().expect("Error: AdwBreakpointBin - No Parent");

            println!("This parent: {}", pp_widget.type_().name()); //Next parent: AdwApplicationWindow

            let pp2_widget = pp_widget.parent().expect("Must have parent!");

            println!("This parent 2: {}", pp2_widget.type_().name()); 

            //let aw_parent = pp2_widget.downcast_ref::<ApplicationWindow>().expect("Error: Must be an ApplicationWindow.");

            let pp3_widget = pp2_widget.parent().expect("Must have parent!");

            println!("This parent 3: {}", pp3_widget.type_().name()); 

            let aw_parent = pp3_widget.downcast_ref::<ApplicationWindow>().expect("Error: Must be an ApplicationWindow.");
            */

            let aw_parent = find_parent::<ApplicationWindow>(root); //find_parent::<ApplicationWindow, _>(root);

            let new_ws_ts = WebSocketTabState::new(&this);

            new_ws_ts.set_contents_paned_position_halved(aw_parent.default_width());
            
            //let new_gql_ts = GraphQLTabState::new(&this);

            //let sr = parent.size_request();

            //new_gql_ts.set_contents_paned_position_halved(aw_parent.default_width());
                
            //}
        


        }));

        //done!

        this

    }

    pub fn tab_view(&self) -> &TabView
    {

        &self.tv

    }

    pub fn tokio_rt_handle(&self) -> &Handle
    {

        &self.tokio_rt_handle

    }

}

impl_as_any!(WindowContentsState);

impl WidgetStateContainer for WindowContentsState
{

    fn dyn_adapter(&self) -> Rc<dyn StoredWidgetObject>
    {

        self.adapted_contents_box.clone()

    }

}
