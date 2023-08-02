use std::cell::RefCell;
use std::rc::{Weak, Rc};
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use gtk_estate::gtk4::{builders::ButtonBuilder, prelude::EditableExt};
use gtk_estate::gtk4::glib::clone::Downgrade;
use gtk_estate::{HasObject, StateContainers, impl_has_object, impl_has_tab_page, helpers::*, SimpleTimeOut, RcSimpleTimeOut}; //time_out::{TimeOut, RcTimeOut}};

use gtk_estate::gtk4::{self as gtk, Box, Orientation, TextView, Paned, Notebook, Label, CenterBox, DropDown, StringList, Text, Button, Viewport, ScrolledWindow, prelude::{BoxExt, TextViewExt, TextBufferExt, WidgetExt, EntryBufferExtManual, ButtonExt}};

use gtk_estate::adw::{TabBar, TabPage, TabView};

use gtk_estate::corlib::{NonOption, rc_self_setup}; //rc_self_refcell_setup};

use gtk_estate::helpers::{widget_ext::set_hvexpand_t, text_view::get_text_view_string, paned::set_paned_position_halved};

use tokio::runtime::Handle;

//use gtk4::prelude::EditableExt;

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Paned.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.ScrolledWindow.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Viewport.html

//https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/struct.Flap.html

//gitlab.gnome.org seemsd to have gone down:

//https://web.archive.org/web/20221126181112/https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/index.html

use crate::actors::graphql_actor::{GraphQLRuntimeActor, GraphQLActorState};

use crate::actors::graphql_actor_message::{GraphQLPostMessage, GraphQLRequestResult, GraphQLPostRequestParams};

//use act_rusty::ActorFrontend;

use act_rs::ActorFrontend;

//use act_rusty::tokio_interactors::{ mspc::SenderInteractor };

//use act_rusty::tokio::interactors::{ mspc::SenderInteractor };

use act_rs::tokio::interactors::{ mspc::SenderInteractor };

//use act_rusty::tokio_oneshot::{Sender, Receiver, channel};

//use act_rusty::tokio::oneshot::{Sender, Receiver, channel};

use tokio::sync::oneshot::{Sender, Receiver, channel};

//use corlib::impl_rfc_get_weak_self;

use gtk_estate::corlib::impl_rfc_get_weak_self;

type OneshotTryRecvError = tokio::sync::oneshot::error::TryRecvError;

pub struct GraphQLTabState
{

    weak_self: RefCell<NonOption<Weak<Self>>>, //<RefCell<Self>>>,
    contents_box: Box,
    tp: TabPage,

    //Header

    //verb_drop_down: DropDown,
    address_text: Text,
    run_button: Button,
    time_output_label: Label,

    //Tabpage

    //Top Left

    query_text: TextView,

    //Bottom Left - tabs

    query_variables: TextView,
    http_headers: TextView,

    //Top Right

    results_text: TextView,

    //Bottom Right

    tracing_text: TextView,

    //

    contents_paned: Paned,
    query_paned: Paned,
    results_paned: Paned,
    graphql_post_actor: GraphQLRuntimeActor,
    tokio_rt_handle: Handle,
    graphql_post_request_job: RefCell<Option<Receiver<GraphQLRequestResult>>>,
    graphql_post_request_job_timeout: RcSimpleTimeOut //RcTimeOut

}

impl GraphQLTabState
{

    pub fn new(tv: &TabView, tokio_rt_handle_ref: &Handle) -> Rc<Self> //Rc<RefCell<Self>>
    {

        //Setup GUI

        let contents_box = Box::new(Orientation::Vertical, 0);

        /*
        let query_text = TextView::new();

        contents_box.append(&query_text);

        query_text.buffer().set_text("test");

        query_text.set_hexpand(true);

        query_text.set_vexpand(true);

        query_text.set_margin_top(2);

        query_text.set_margin_start(2);

        query_text.set_margin_bottom(2);

        query_text.set_margin_end(2);
        */

        //Contains the query and results Paned widgets

        let header_box = Box::new(Orientation::Vertical, 2);

        //Address bar - First Row

        let address_box = Box::new(Orientation::Horizontal, 4);

        //let verb_label = Label::new(Some("Verb:"));

        //address_box.append(&verb_label);

        //let verb_list_model = StringList::new(&["POST"]);

        //let verb_drop_down = DropDown::new(Some(&verb_list_model), Option::<impl AsRef<Expression>>::None);

        //let verb_drop_down = DropDown::new(None, None);

        //let verb_drop_down = DropDown::from_strings(&["POST"]);


        //address_box.append(&verb_drop_down);

        let address_label = Label::new(Some("Address:"));

        address_box.append(&address_label);


        let address_text = Text::new();

        address_text.set_hexpand(true);

        address_text.buffer().set_text("http://localhost:8000");

        address_box.append(&address_text);


        header_box.append(&address_box);

        //Buttons and output labels - Second Row

        let tool_cbox = CenterBox::new(); //Box::new(Orientation::Horizontal, 0);

        //Left

        //Center

        let tool_center_box = Box::new(Orientation::Horizontal, 2);

        let run_button = Button::builder().label("Run").build(); //ButtonBuilder::new().label("Run").build();

        tool_center_box.append(&run_button);

        tool_cbox.set_center_widget(Some(&tool_center_box));

        //Right

        let tool_right_box = Box::new(Orientation::Horizontal, 2);

        let time_label = Label::new(Some("Time:"));

        tool_right_box.append(&time_label);

        let time_output_label = Label::new(Some("N/A"));

        tool_right_box.append(&time_output_label);

        tool_cbox.set_end_widget(Some(&tool_right_box));

        //

        header_box.append(&tool_cbox);

        contents_box.append(&header_box);

        //Horizontal container pane

        let contents_paned = Paned::new(Orientation::Horizontal);

        set_hvexpand_t(&contents_paned);

        //contents_box.append(&contents_paned);

        //contents_box.set_hexpand(true);

        //contents_box.set_vexpand(true);

        //set_hvexpand_t(&contents_box);

        //Vertical Panes

        //Left Side

        let query_paned = Paned::new(Orientation::Vertical);

        //query_paned.set_resize_start_child(true);  //false);

        //query_paned.set_shrink_start_child(false);

        //expand(

        //gtk_estate::helpers::

        //set_hvexpand_t(&query_paned);

        contents_paned.set_start_child(Some(&query_paned));

        //Right Side

        let results_paned = Paned::new(Orientation::Vertical);

        //set_hvexpand_t(&results_paned);

        contents_paned.set_end_child(Some(&results_paned));

        //query_paned children

        //start child

        //Top Left

        let query_text = TextView::new();

        //set_hvexpand_t(&query_text);
        
        //let query_text_viewport = Viewport::builder().child(&query_text).build(); //.scroll_to_focus(true)

        //query_text_viewport.set_scroll_to_focus(true);

        //set_hvexpand_t(&query_text_viewport);

        //query_paned.set_start_child(Some(&query_text_viewport));

        let query_text_scrolledwindow = ScrolledWindow::builder().child(&query_text).build();

        set_hvexpand_t(&query_text_scrolledwindow);

        query_paned.set_start_child(Some(&query_text_scrolledwindow));

        //query_text.buffer().set_text("query_text");

        //query_text.set_hexpand(true);

        //query_text.set_vexpand(true);

        //query_text.set_margin_top(2);

        //query_text.set_margin_start(2);

        //query_text.set_margin_bottom(2);

        //query_text.set_margin_end(2);

        //end child

        //Lower left pane

        let parameters_nb = Notebook::new();

        //set_hvexpand_t(&parameters_nb);

        query_paned.set_end_child(Some(&parameters_nb));

        //query variables - Notebook page



        let query_variables = TextView::new();

        //set_hvexpand_t(&query_variables);

        //query_variables.buffer().set_text("query_variables");

        //query_text.set_hexpand(true);

        //query_text.set_vexpand(true);

        //query_text.set_margin_top(2);

        //query_text.set_margin_start(2);

        //query_text.set_margin_bottom(2);

        //query_text.set_margin_end(2);


        let query_variables_label = Label::new(Some(&"Query Variables"));

        //ScrolledWindow

        let query_variables_scrolledwindow = ScrolledWindow::builder().child(&query_variables).build();

        set_hvexpand_t(&query_variables_scrolledwindow);

        parameters_nb.append_page(&query_variables_scrolledwindow, Some(&query_variables_label)); //&query_variables, Some(&query_variables_label));

        //HTTP Headers - Notebook page



        let http_headers = TextView::new();

        //set_hvexpand_t(&http_headers);

        //http_headers.buffer().set_text("http_headers");

        let http_headers_label = Label::new(Some(&"HTTP Headers"));

        //ScrolledWindow

        let http_headers_scrolledwindow = ScrolledWindow::builder().child(&http_headers).build();

        set_hvexpand_t(&http_headers_scrolledwindow);

        parameters_nb.append_page(&http_headers_scrolledwindow, Some(&http_headers_label)); //&http_headers, Some(&http_headers_label));

        //results_paned

        ////Stsrt Child - Upper right

        //Query Results - Upper right

        let results_text = TextView::new();

        let results_text_scrolledwindow = ScrolledWindow::builder().child(&results_text).build();

        set_hvexpand_t(&results_text_scrolledwindow);

        results_text.buffer().set_text("results_text");

        //set_hvexpand_t(&results_text);

        results_paned.set_start_child(Some(&results_text_scrolledwindow)); //results_text));

        //End Child - Lower right

        

        let tracing_text = TextView::new();

        let tracing_text_scrolledwindow = ScrolledWindow::builder().child(&tracing_text).build();

        set_hvexpand_t(&tracing_text_scrolledwindow);

        //tracing_text.buffer().set_text("tracing_text");

        //set_hvexpand_t(&tracing_text);

        results_paned.set_end_child(Some(&tracing_text_scrolledwindow)); //tracing_text));

        //Init new tab page - this tab page
        
        let tp = tv.append(&contents_box);

        tp.set_title("New Tab");

        //

        contents_box.append(&contents_paned);

        //let half_contents_box_width = contents_box.allocated_width() / 2;

        //let half_contents_box_width = contents_box.width() / 2;

        //contents_paned.set_position(half_contents_box_width);

        //Initaise reference type instance

        let actor_state = GraphQLActorState::new();

        let graphql_post_actor = GraphQLRuntimeActor::new(tokio_rt_handle_ref, actor_state);
  
        let this = Self
        {

            weak_self: NonOption::invalid_rfc(), //invalid_refcell(),
            contents_box,
            tp,
            
            //Header

            //verb_drop_down,
            address_text,
            run_button,
            time_output_label,

            //Tabpage

            //Top Left

            query_text,

            //Bottom Left - tabs

            query_variables,
            http_headers,

            //Top Right

            results_text,

            //Bottom Right

            tracing_text,

            //
            
            contents_paned,
            query_paned,
            results_paned,
            graphql_post_actor,
            tokio_rt_handle: tokio_rt_handle_ref.clone(),
            graphql_post_request_job: RefCell::new(None),
            graphql_post_request_job_timeout: SimpleTimeOut::new(Duration::new(1, 0)) //TimeOut::new(Duration::new(1, 0), true)

        };

        //Setup rc_self

        let rc_self = Rc::new(this); //(RefCell::new(this));

        //rc_self_refcell_setup!(rc_self, weak_self);

        rc_self_setup!(rc_self, weak_self);

        //

        //Add to StateContainers

        let scs = StateContainers::get();

        //scs.get_adw_ref().get_tab_pages_mut().add_refcell(&rc_self);

        //Add to application tab pages

        scs.adw().borrow_mut_tab_pages().add(&rc_self); //_refcell(

        //Run Button

        let weak_self = rc_self.downgrade(); //.weak_self.borrow().get_ref().clone();

        rc_self.run_button.connect_clicked(move |_btn|
        {

            if let Some(this) = weak_self.upgrade()
            {

                //Return if thed job is already active

                if this.graphql_post_request_job.borrow().is_some()
                {

                    return;

                }

                //Requsest

                //address_text

                //query_text

                //query_variables

                //http_headers

                let address = this.address_text.text().to_string(); //.buffer().text().to_string(); //.buffer().to_string();

                let query = get_text_view_string(&this.query_text); //.text().to_string(); //buffer().to_string();

                let query_variables = get_text_view_string(&this.query_variables); //.buffer().to_string();

                let http_headers = get_text_view_string(&this.http_headers); // .buffer().to_string();

                let message_params = GraphQLPostRequestParams::new(address, query, query_variables, http_headers);

                let (sender, reciver) = channel();

                let request_message = GraphQLPostMessage::Request(message_params, sender);

                let interactor = this.graphql_post_actor.get_interactor_ref();

                let send_res = interactor.get_sender_ref().blocking_send(Some(request_message));

                /*
                match send_res {

                    Ok(res) => ,
                    Err(err => todo!(),
                }
                */
                
                if let Err(err) = send_res
                {

                    this.results_text.buffer().set_text(err.to_string().as_str());

                }
                else
                {
                    
                    *this.graphql_post_request_job.borrow_mut() = Some(reciver);

                    this.graphql_post_request_job_timeout.start();

                }

                //Response

                //time_label

                //results_text

                //tracing_text

            }

        });

        //TimeOut

        let weak_self = rc_self.downgrade(); //.get_weak_self();

        rc_self.graphql_post_request_job_timeout.set_function(move |_sto| {

            if let Some(this) = weak_self.upgrade()
            {

                {

                    let mut graphql_post_request_job_mut = this.graphql_post_request_job.borrow_mut();

                    if let Some(rec) = graphql_post_request_job_mut.as_mut()
                    {

                        /*
                        if let Some(val) = rec.try_recv()
                        {

                            match val
                            {

                                Ok(res) => 
                                {

                                    //Job complete - set the result

                                    //let duration_text = res.get_duration().as_millis().to_string();

                                    let mut duration_millis = res.get_duration().as_millis().to_string();

                                    duration_millis.push_str(" ms");

                                    this.time_output_label.set_text(duration_millis.as_str());

                                    this.results_text.buffer().set_text(res.get_result_ref().as_str());

                                },
                                Err(err) =>
                                {

                                    //Error detected - Set the error

                                    this.time_output_label.set_text("N/A");

                                    this.results_text.buffer().set_text(err.to_string().as_str());

                                }

                            }

                        }
                        else
                        {

                            //Try again soon

                            return true;

                        }
                        */

                        match rec.try_recv()
                        {

                            Ok(res) => 
                            {

                                //Job complete - set the result

                                let mut duration_millis = res.get_duration().as_millis().to_string();

                                duration_millis.push_str(" ms");

                                this.time_output_label.set_text(duration_millis.as_str());

                                this.results_text.buffer().set_text(res.get_result_ref().as_str());

                            },
                            Err(err) =>
                            {

                                if let OneshotTryRecvError::Closed = err
                                {

                                    //Error detected - Set the error

                                    this.time_output_label.set_text("N/A");

                                    this.results_text.buffer().set_text(err.to_string().as_str());

                                    //Error - stop checking

                                    //Make sure the job has been dropped

                                    *graphql_post_request_job_mut = None;

                                    return false;

                                }

                                //Empty - Try again soon

                                return true;

                            }

                        }

                    }

                    //Drop the job once done 

                    *graphql_post_request_job_mut = None;

                }

                //true

                //false

            }
            /*
            else
            {

                //false

                true
            }
            */

            //Disable the Timeout

            false
            
            //Return Value: Continue?

        });

        rc_self

    }

    //Can be used to center the main contents paned widget
    
    pub fn set_contents_paned_position_halved(&self, value: i32)
    {

        //self.contents_paned.set_position(value / 2);

        set_paned_position_halved(&self.contents_paned, value)

    }

    impl_rfc_get_weak_self!();


}

impl_has_tab_page!(tp, GraphQLTabState);


//gtk_estate

/*
pub fn get_text_view_string(tv: &TextView) -> String
{

    let buffer = tv.buffer();

    let start = buffer.start_iter();

    let end = buffer.end_iter();

    buffer.text(&start, &end, false).to_string()

}

pub fn get_all_text_view_string(tv: &TextView) -> String
{

    let buffer = tv.buffer();

    let start = buffer.start_iter();

    let end = buffer.end_iter();

    buffer.text(&start, &end, true).to_string()

}
*/
