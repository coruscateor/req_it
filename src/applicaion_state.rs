use gtk::prelude::ApplicationExt;

use gtk_estate::corlib::impl_as_any;

use gtk_estate::{gtk4 as gtk, AdwApplcationWindowState, ApplicationAdapter, ApplicationStateContainer, StateContainers, StoredApplicationObject};

use gtk_estate::adw::Application;

use std::{rc::*, any::Any};

use std::cell::{RefCell, Ref, RefMut};

use gtk_estate::corlib::as_any::AsAny;

use gtk::glib;

use gtk::glib::clone;

use tokio::runtime::{Runtime, Handle, Builder};

use crate::window_contents_state::WindowContentsState;

pub struct ApplicationState
{

    app: Application,
    tokio_rt: Runtime,
    app_ad: Rc<ApplicationAdapter<Application, ApplicationState>>

}

impl ApplicationState
{

    pub fn new(app: &Application) -> Rc<Self>
    {

        let tokio_rt = Builder::new_multi_thread().enable_all().build().expect("Tokio Runtime construction failed");

        let this = Rc::new_cyclic(|weak_self|
        {
                
            Self
            {

                app: app.clone(),
                tokio_rt,
                app_ad: ApplicationAdapter::new(app, weak_self)


            }

        });

        let ws = this.app_ad.weak_parent();

        this.app.connect_activate(move |_app|
        {

            if let Some(this) = ws.upgrade()
            {

                this.new_window();
                
            }

        });

        let sc = StateContainers::get();

        sc.set_application_state_or_panic(&this);

        this

    }

    pub fn get_tokio_rt_handle_ref(&self) -> &Handle
    {

        self.tokio_rt.handle()

    }

    pub fn clone_tokio_rt_handle(&self) -> Handle
    {

        self.tokio_rt.handle().clone()

    }

    pub fn new_window(&self)
    {

        let content = WindowContentsState::new();

        AdwApplcationWindowState::builder_with_content_visible(|builder| {

            builder.application(&self.app)
            .default_width(1000)
            .default_height(1000)
            .build()

        }, &content);

    }
    
}

impl_as_any!(ApplicationState);

impl ApplicationStateContainer for ApplicationState
{

    fn dyn_adapter(&self) -> Rc<dyn StoredApplicationObject>
    {

        self.app_ad.clone()

    }

}

