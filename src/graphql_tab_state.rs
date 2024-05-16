use std::cell::RefCell;

use std::rc::{Weak, Rc};

use std::any::Any;

use std::sync::mpsc::TryRecvError;

use std::time::Duration;

use gtk_estate::gtk4::{builders::ButtonBuilder, prelude::EditableExt};

use gtk_estate::{helpers::*, RcSimpleTimeOut, SimpleTimeOut, StateContainers, StoredWidgetObject, WidgetAdapter, WidgetStateContainer};

use gtk_estate::gtk4::{self as gtk, Box, Orientation, TextView, Paned, Notebook, Label, CenterBox, DropDown, StringList, Text, Button, Viewport, ScrolledWindow, prelude::{BoxExt, TextViewExt, TextBufferExt, WidgetExt, EntryBufferExtManual, ButtonExt}};

use gtk_estate::adw::{TabBar, TabPage, TabView};

use gtk_estate::corlib::{impl_as_any, rc_self_setup, AsAny};

use gtk_estate::helpers::{widget_ext::set_hvexpand_t, text_view::get_text_view_string, paned::set_paned_position_halved};

use tokio::runtime::Handle;

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Paned.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.ScrolledWindow.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Viewport.html

//https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/struct.Flap.html

//If gitlab.gnome.org goes down:

//https://web.archive.org/web/20221126181112/https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/index.html

use crate::actors::graphql_actor::{GraphQLRuntimeActor, GraphQLActorState};

use crate::actors::graphql_actor_message::{GraphQLPostMessage, GraphQLRequestResult, GraphQLPostRequestParams};

use crate::window_contents_state::WindowContentsState;

use act_rs::ActorFrontend;

use act_rs::tokio::interactors::mpsc::SenderInteractor;

use tokio::sync::oneshot::{Sender, Receiver, channel};

use gtk::glib;

use gtk::glib::clone;

type OneshotTryRecvError = tokio::sync::oneshot::error::TryRecvError;

pub struct GraphQLTabState
{

    adapted_contents_box: Rc<WidgetAdapter<Box, GraphQLTabState>>,
    tp: TabPage,

    //Header

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
    graphql_post_request_job_timeout: RcSimpleTimeOut

}

impl GraphQLTabState
{

    pub fn new(wcs: &Rc<WindowContentsState>) -> Rc<Self>
    {

        //Setup the GUI

        let contents_box = Box::new(Orientation::Vertical, 0);

        //Contains the query and results Paned widgets

        let header_box = Box::new(Orientation::Vertical, 2);

        //Address bar - First Row

        let address_box = Box::new(Orientation::Horizontal, 4);

        let address_label = Label::new(Some("Address:"));

        address_box.append(&address_label);

        let address_text = Text::new();

        address_text.set_hexpand(true);

        address_text.buffer().set_text("http://localhost:8000");

        address_box.append(&address_text);


        header_box.append(&address_box);

        //Buttons and output labels - Second Row

        let tool_cbox = CenterBox::new();

        //Left

        //Center
        
        let tool_center_box = Box::new(Orientation::Horizontal, 2);

        let run_button = Button::builder().label("Run").build();

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

        //Vertical Panes

        //Left Side

        let query_paned = Paned::new(Orientation::Vertical);

        contents_paned.set_start_child(Some(&query_paned));

        //Right Side

        let results_paned = Paned::new(Orientation::Vertical);

        

        contents_paned.set_end_child(Some(&results_paned));

        //query_paned children

        //start child

        //Top Left

        let query_text = TextView::new();

        let query_text_scrolledwindow = ScrolledWindow::builder().child(&query_text).build();

        set_hvexpand_t(&query_text_scrolledwindow);

        query_paned.set_start_child(Some(&query_text_scrolledwindow));

        //end child

        //Lower left pane

        let parameters_nb = Notebook::new();

        query_paned.set_end_child(Some(&parameters_nb));

        let query_variables = TextView::new();

        let query_variables_label = Label::new(Some(&"Query Variables"));

        //ScrolledWindow

        let query_variables_scrolledwindow = ScrolledWindow::builder().child(&query_variables).build();

        set_hvexpand_t(&query_variables_scrolledwindow);

        parameters_nb.append_page(&query_variables_scrolledwindow, Some(&query_variables_label));

        //HTTP Headers - Notebook page

        let http_headers = TextView::new();

        let http_headers_label = Label::new(Some(&"HTTP Headers"));

        //ScrolledWindow

        let http_headers_scrolledwindow = ScrolledWindow::builder().child(&http_headers).build();

        set_hvexpand_t(&http_headers_scrolledwindow);

        parameters_nb.append_page(&http_headers_scrolledwindow, Some(&http_headers_label));

        //results_paned

        //Start Child - Upper right

        //Query Results - Upper right

        let results_text = TextView::new();

        let results_text_scrolledwindow = ScrolledWindow::builder().child(&results_text).build();

        set_hvexpand_t(&results_text_scrolledwindow);

        results_text.buffer().set_text("results_text");

        results_paned.set_start_child(Some(&results_text_scrolledwindow));

        //End Child - Lower right

        let tracing_text = TextView::new();

        let tracing_text_scrolledwindow = ScrolledWindow::builder().child(&tracing_text).build();

        set_hvexpand_t(&tracing_text_scrolledwindow);

        results_paned.set_end_child(Some(&tracing_text_scrolledwindow));

        //Init new tab page - this tab page
        
        let tp = wcs.tab_view().append(&contents_box);

        tp.set_title("New Tab");

        //

        contents_box.append(&contents_paned);

        //Initialise reference type instance

        let actor_state = GraphQLActorState::new();

        let graphql_post_actor = GraphQLRuntimeActor::new(wcs.tokio_rt_handle(), actor_state);
  
        let this =  Rc::new_cyclic( move |weak_self|
        {

            Self
            {

                adapted_contents_box: WidgetAdapter::new(&contents_box, weak_self),
                tp,
                
                //Header

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
                tokio_rt_handle: wcs.tokio_rt_handle().clone(),
                graphql_post_request_job: RefCell::new(None),
                graphql_post_request_job_timeout: SimpleTimeOut::new(Duration::new(1, 0))

            }

        });

        //Get the StateContainers

        let scs = StateContainers::get();

        //Add the GraphQLTabState to the the StateContainers

        scs.add(&this);

        //Run Button

        let weak_self = this.adapted_contents_box.weak_parent();

        let weak_self_mv = weak_self.clone();

        this.run_button.connect_clicked(move |_btn|
        {

            if let Some(this) = weak_self_mv.upgrade()
            {

                //Return if the job is already active

                if this.graphql_post_request_job.borrow().is_some()
                {

                    return;

                }

                //Request

                let address = this.address_text.text().to_string();

                let query = get_text_view_string(&this.query_text);

                let query_variables = get_text_view_string(&this.query_variables);

                let http_headers = get_text_view_string(&this.http_headers);

                let message_params = GraphQLPostRequestParams::new(address, query, query_variables, http_headers);

                let (sender, reciver) = channel();

                let request_message = GraphQLPostMessage::Request(message_params, sender);

                let interactor = this.graphql_post_actor.interactor();

                let send_res = interactor.sender().blocking_send(Some(request_message));
                
                if let Err(err) = send_res
                {

                    this.results_text.buffer().set_text(err.to_string().as_str());

                }
                else
                {
                    
                    *this.graphql_post_request_job.borrow_mut() = Some(reciver);

                    this.graphql_post_request_job_timeout.start();

                }

            }

        });

        //TimeOut

        this.graphql_post_request_job_timeout.set_function(move |_sto| {

            if let Some(this) = weak_self.upgrade()
            {

                {

                    let mut graphql_post_request_job_mut = this.graphql_post_request_job.borrow_mut();

                    if let Some(rec) = graphql_post_request_job_mut.as_mut()
                    {

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

                                    //Make sure the job has been dropped

                                    *graphql_post_request_job_mut = None;

                                    //Stop the reoccurring Timeout

                                    return false;

                                }

                                //The Recevier is empty, try again soon.

                                return true;

                            }

                        }

                    }

                    //Drop the job if it's done. 

                    *graphql_post_request_job_mut = None;

                }

            }

            //Stop the Timeout as weak_self is not upgradeable.

            false

        });

        this

    }

    //Used, in this case, to center the contents_paned widget.
    
    pub fn set_contents_paned_position_halved(&self, value: i32)
    {

        set_paned_position_halved(&self.contents_paned, value)

    }

}

impl_as_any!(GraphQLTabState);

impl WidgetStateContainer for GraphQLTabState
{

    fn dyn_adapter(&self) -> Rc<dyn StoredWidgetObject>
    {

        self.adapted_contents_box.clone()

    }

}
