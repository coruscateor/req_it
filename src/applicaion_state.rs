use gtk::prelude::ApplicationExt;

use gtk_estate::corlib::impl_as_any;

use gtk_estate::{gtk4 as gtk, AdwApplcationWindowState, ApplicationAdapter, ApplicationStateContainer, StateContainers, StoredApplicationObject};

//use gtk_estate::*;

//use gtk::Application;

use gtk_estate::adw::Application;

use std::{rc::*, any::Any};

use std::cell::{RefCell, Ref, RefMut};

//use corlib::rc_self_init_refcell_setup_returned_named;

//use corlib::{NonOption, rc_self_setup}; //, rc_self_rfc_setup}; //rc_self_refcell_setup, //rc_self_init_refcell_setup_returned

use gtk_estate::corlib::as_any::AsAny;

use gtk::glib;

use gtk::glib::clone;

//use crate::window_state::*;

use tokio::runtime::{Runtime, Handle, Builder};

use crate::window_contents_state::WindowContentsState;

pub struct ApplicationState
{

    app: Application,
    //weak_self: NonOption<Weak<RefCell<Self>>>,
    //weak_self: RefCell<NonOption<Weak<Self>>>,
    //weak_self: Weak<Self>,
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
                //weak_self: weak_self.clone(), //RefCell::new(NonOption::invalid()),
                tokio_rt,
                app_ad: ApplicationAdapter::new(app, weak_self)


            }

        });

        //let res = rc_self_init_refcell_setup_returned!(this, weak_self);
        
        //let rc_self = Rc::new(this); //Rc::new(RefCell::new(this));

        //rc_self_refcell_setup!(rc_self, weak_self);

        //rc_self_setup!(rc_self, weak_self);

        //

        /*
        {

            let b_self = rc_self.borrow();

            /*
            b_self.app.connect_startup(|_|
            {
    
                if let Err(err) = adw::init()
                {

                    println!("Adwaita init errror: {}", err);

                }
    
            });
            */

            b_self.app.connect_activate(move |app|
            {

                //new window

                WindowState::new(app);

            });

        }
        */

        //let ws = this.weak_self.clone();

        //let ws = &this.weak_self;
        
        /* 
        this.app.connect_activate(clone!(@weak ws => move |_app|
        {

            if let Some(this) = ws.upgrade()
            {

                this.new_window();
                
            }*/

            //Default window

            //WindowState::new(app);

        //}));

        let ws = this.app_ad.weak_parent();

        this.app.connect_activate(move |_app|
        {

            if let Some(this) = ws.upgrade()
            {

                this.new_window();
                
            }

            //Default window

            //WindowState::new(app);

        });

        let sc = StateContainers::get();

        sc.set_application_state_or_panic(&this);

        //sc.adw().borrow_mut_applications().add(&rc_self); //_refcell

        //

        this

    }

    //get weak self

    //tokio_rt

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

        //WindowState::new(&self.app);

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

/*
pub fn run()
{

    //let apst = ApplicattionState::new();

    //apst.

}
*/

//impl_has_application!(app, ApplicattionState);

/*
impl HasObject<Application> for ApplicattionState
{

    fn get_object(&self) -> &Application
    {
        
        &self.app
        
    }

}
*/
