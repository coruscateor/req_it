mod applicaion_state;

use gtk_estate::{adw::{prelude::*, Application}, StateContainers};

use crate::applicaion_state::ApplicationState;

mod window_contents_state;

mod graphql_tab_state;

mod actors;

fn main()
{
    
    let app = Application::builder().application_id("org.req_it_gui").build();
    
    //Initaslise the StateContainers object.

    //StateContainers::init();

    ApplicationState::new(&app);

    app.run();

}
