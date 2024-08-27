mod applicaion_state;

use gtk_estate::{adw::{prelude::*, Application}, StateContainers};

use crate::applicaion_state::ApplicationState;

mod window_contents_state;

//mod graphql_tab_state;

mod actors;

mod web_socket_tab_state;

use web_socket_tab_state::*;

fn main()
{
    
    let app = Application::builder().application_id("org.req_it_gui").build();
    
    //Initaslise the StateContainers object.

    //StateContainers::init();

    ApplicationState::new(&app);

    app.run();

}

/*

error[E0599]: no method named `forget_permits` found for struct `Semaphore` in the current scope
  --> /run/media/paul/Main Stuff/SoftwareProjects/Rust/libsync/src/tokio_helpers/semaphore_controller.rs:84:17
   |
84 |        self.sem.forget_permits(1)
   |                 ^^^^^^^^^^^^^^ method not found in `Semaphore`

 */