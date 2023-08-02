use gtk::prelude::ApplicationExt;
use gtk_estate::gtk4 as gtk;

use gtk_estate::*;

//use gtk::Application;

use gtk_estate::adw::Application;

use std::{rc::*, any::Any};

use std::cell::{RefCell, Ref, RefMut};

//use corlib::rc_self_init_refcell_setup_returned_named;

use corlib::{NonOption, rc_self_setup}; //, rc_self_rfc_setup}; //rc_self_refcell_setup, //rc_self_init_refcell_setup_returned

use crate::window_state::*;

use tokio::runtime::{Runtime, Handle, Builder};

pub struct ApplicattionState
{

    app: Application,
    //weak_self: NonOption<Weak<RefCell<Self>>>,
    weak_self: RefCell<NonOption<Weak<Self>>>,
    tokio_rt: Runtime

}

impl ApplicattionState
{

    pub fn new(app: &Application) -> Rc<Self>  //Rc<RefCell<Self>> //
    {

        let tokio_rt = Builder::new_multi_thread().enable_all().build().expect("Tokio Runtime construction failed"); //.enable_io().build().expect("Tokio Runtime construction failed");

        //

        let this = Self
        {

            app: app.clone(),
            weak_self: RefCell::new(NonOption::invalid()),
            tokio_rt

        };

        //let res = rc_self_init_refcell_setup_returned!(this, weak_self);
        
        let rc_self = Rc::new(this); //Rc::new(RefCell::new(this));

        //rc_self_refcell_setup!(rc_self, weak_self);

        rc_self_setup!(rc_self, weak_self);

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

        rc_self.app.connect_activate(move |app|
        {

            //new window

            WindowState::new(app);

        });

        let sc = StateContainers::get();

        sc.adw().borrow_mut_applications().add(&rc_self); //_refcell

        //

        rc_self

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

}

/*
pub fn run()
{

    //let apst = ApplicattionState::new();

    //apst.

}
*/

impl_has_application!(app, ApplicattionState);

/*
impl HasObject<Application> for ApplicattionState
{

    fn get_object(&self) -> &Application
    {
        
        &self.app
        
    }

}
*/
