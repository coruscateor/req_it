use std::cell::{OnceCell, RefCell, RefMut};

use std::os::unix::process;
use std::rc::{Weak, Rc};

use std::any::Any;

//use std::sync::mpsc::TryRecvError;

use std::sync::OnceLock;

use std::time::Duration;

use corlib::text::AsStr;
use gtk_estate::adw::glib::clone::Upgrade;

//use gtk_estate::corlib::rfc::borrow_mut;

//use gtk_estate::corlib::upgrading::up_rc;

use corlib::upgrading::{up_rc, up_rc_pt};

use gtk_estate::gtk4::ListBox;

use gtk_estate::gtk4::{builders::ButtonBuilder, prelude::EditableExt};

use gtk_estate::{helpers::*, scs_add, RcSimpleTimeOut, SimpleTimeOut, StateContainers, StoredWidgetObject, WidgetAdapter, WidgetStateContainer};

use gtk_estate::gtk4::{self as gtk, Box, Orientation, TextView, Paned, Notebook, Label, CenterBox, DropDown, StringList, Text, Button, Viewport, ScrolledWindow, prelude::{BoxExt, TextViewExt, TextBufferExt, WidgetExt, EntryBufferExtManual, ButtonExt, ListModelExt}};

use gtk_estate::adw::{TabBar, TabPage, TabView};

//use gtk_estate::corlib::{impl_as_any, rc_self_setup, AsAny};

//use corlib::{impl_as_any, AsAny, rfc_borrow, rfc_borrow_mut};

use gtk_estate::corlib::{impl_as_any, AsAny};

//use corlib::upgrading::up_rc;

use corlib::rfc::borrow_mut;

use gtk_estate::helpers::{widget_ext::set_hvexpand_t, text_view::get_text_view_string, paned::set_paned_position_halved};

use tokio::runtime::Handle;

use widget_ext::set_margin_sides_and_bottom;

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Paned.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.ScrolledWindow.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Viewport.html

//https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/struct.Flap.html

//If gitlab.gnome.org goes down:

//https://web.archive.org/web/20221126181112/https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/index.html

use crate::actors::{WebSocketActor, WebSocketActorInputMessage, WebSocketActorState, WriteFrameProcessorActor, WriteFrameProcessorActorIOClient, WriteFrameProcessorActorState};

use crate::window_contents_state::WindowContentsState;

use act_rs::{enter, ActorFrontend};

//use act_rs::tokio::interactors::mpsc::SenderInteractor;

use tokio::sync::oneshot::{Sender, Receiver, channel};

use gtk::glib;

use gtk::glib::clone;

use tokio::sync::mpsc::error::TryRecvError;

use crate::actors::{ReadFrameProcessorActorOutputMessage, WebSocketActorOutputClientMessage};

use gtk_estate::gtk4::gio::ListModel;

//type OneshotTryRecvError = tokio::sync::oneshot::error::TryRecvError;

//static FORMAT_DROPDOWN_STRS: [&str] = ["JSON To CBOR", "Text"];

//static FORMAT_DROPDOWN_STRS: OnceLock<std::boxed::Box<[&str]>> = OnceLock::new(); //(|| { ["JSON To CBOR", "Text"] });

/*
static FORMAT_DROPDOWN_STRS: OnceLock<&'static [&str]> = OnceLock::new();

fn format_dropdown_strs() -> &'static [&'static str]
{

    FORMAT_DROPDOWN_STRS.get_or_init(||{ &["JSON To CBOR", "Text"] })

}
*/

//static FORMAT_DROPDOWN_STRS: [&'static str; 2] = ["JSON To CBOR", "Text"];

//static FORMAT_DROPDOWN_STRS: &'static [&str] = ["JSON To CBOR", "Text"];

static FORMAT_DROPDOWN_STRS: &[&str] = &["JSON To CBOR", "Text"];

static JSON_TO_CBOR: &str = "JSON To CBOR";

static TEXT: &str = "Text";

struct MutState
{

    //pub graphql_post_request_job: Option<Receiver<GraphQLRequestResult>>

}

impl MutState
{

    pub fn new() -> Self
    {

        Self
        {

            //graphql_post_request_job: None

        }

    }

}

pub struct WebSocketTabState
{

    adapted_contents_box: Rc<WidgetAdapter<Box, WebSocketTabState>>,
    tp: TabPage,

    //Header

    address_text: Text,
    connect_button: Button,
    disconnect_button: Button,
    time_output_label: Label,
    //["JSON To CBOR", "Text"]
    //format_dropdown: [&str],
    format_dropdown: DropDown,
    send_button: Button,

    //Tabpage

    //Top Left

    to_be_sent_text: TextView,

    //Bottom Left - tabs

    //query_variables: TextView,
    //http_headers: TextView,

    //Top Right

    //received_text: TextView,

    received_messages: ListBox,
    received_messages_child_observer: ListModel,

    //Bottom Right

    //tracing_text: TextView,

    //

    contents_paned: Paned,
    to_be_sent_paned: Paned,
    received_paned: Paned,
    //web_socket_actor: WebSocketActor,
    //write_frame_processor_actor: WriteFrameProcessorActor,
    write_frame_processor_actor_io_client: WriteFrameProcessorActorIOClient,
    tokio_rt_handle: Handle,

    web_socket_actor_poller: RcSimpleTimeOut<Weak<WebSocketTabState>>,
    mut_state: RefCell<MutState>

}

impl WebSocketTabState
{

    pub fn new(wcs: &Rc<WindowContentsState>) -> Rc<Self>
    {

        //Setup the GUI

        let contents_box = Box::new(Orientation::Vertical, 0);

        //Contains the query and received Paned widgets

        let header_box = Box::new(Orientation::Vertical, 2);

        contents_box.append(&header_box);

        //Address bar - First Row

        let address_box = Box::new(Orientation::Horizontal, 4);

        let address_label = Label::new(Some("Address:"));

        address_box.append(&address_label);

        let address_text = Text::new();

        address_text.set_hexpand(true);

        address_text.buffer().set_text("http://localhost:3000");

        address_box.append(&address_text);

        header_box.append(&address_box);

        //Buttons and output labels - Second Row

        //CenterBox - Level 1

        let tool_cbox = CenterBox::new();

        //tool_cbox.set_margin_start(5);

        //tool_cbox.set_margin_end(5);

        //tool_cbox.set_margin_bottom(5);

        set_margin_sides_and_bottom(&tool_cbox, 5);

        //Left

        let tool_left_box = Box::new(Orientation::Horizontal, 20);

        //Format DropDown 

        let format_dropdown = DropDown::from_strings(FORMAT_DROPDOWN_STRS);

        tool_left_box.append(&format_dropdown);

        //Send Button

        let send_button = Button::builder().label("Send").build();

        tool_left_box.append(&send_button);

        //

        tool_cbox.set_start_widget(Some(&tool_left_box));

        //Center
        
        let tool_center_box = Box::new(Orientation::Horizontal, 2);

        let connect_button = Button::builder().label("Connect").build();

        tool_center_box.append(&connect_button);

        let disconnect_button = Button::builder().label("Disconnect").visible(false).build();

        tool_center_box.append(&disconnect_button);

        tool_cbox.set_center_widget(Some(&tool_center_box));

        //Right

        let tool_right_box = Box::new(Orientation::Horizontal, 2);

        //Binary to BSON or JSON, JSON Only DropDown

        let time_label = Label::new(Some("Time:"));

        tool_right_box.append(&time_label);

        let time_output_label = Label::new(Some("N/A"));

        tool_right_box.append(&time_output_label);

        tool_cbox.set_end_widget(Some(&tool_right_box));

        //

        header_box.append(&tool_cbox);

        //CenterBox - Level 2

        /*
        let tool_cbox_l2 = CenterBox::new();

        set_margin_sides_and_bottom(&tool_cbox_l2, 5);

        //Left

        let tool_left_box = Box::new(Orientation::Horizontal, 2);

        let send_button = Button::builder().label("Send").build();

        tool_left_box.append(&send_button);

        tool_cbox_l2.set_start_widget(Some(&tool_left_box));

        //

        header_box.append(&tool_cbox_l2);
                */

        //

        //Horizontal container pane

        let contents_paned = Paned::new(Orientation::Horizontal);

        set_hvexpand_t(&contents_paned);

        //Vertical Panes

        //Left Side

        let to_be_sent_paned = Paned::new(Orientation::Vertical);

        contents_paned.set_start_child(Some(&to_be_sent_paned));

        //Right Side

        let received_paned = Paned::new(Orientation::Vertical);

        contents_paned.set_end_child(Some(&received_paned));

        //query_paned children

        //start child

        //Top Left

        let to_be_sent_text = TextView::new();

        let to_be_sent_text_scrolledwindow = ScrolledWindow::builder().child(&to_be_sent_text).build();

        set_hvexpand_t(&to_be_sent_text_scrolledwindow);

        to_be_sent_paned.set_start_child(Some(&to_be_sent_text_scrolledwindow));

        //end child

        //Lower left pane

        /*
        let parameters_nb = Notebook::new();

        to_be_sent_paned.set_end_child(Some(&parameters_nb));

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
        */

        //received_paned

        //Start Child - Upper right

        //Received - Upper right

        //let received_text = TextView::new();

        let received_messages = ListBox::new();

        let received_messages_child_observer = received_messages.observe_children();

        let recived_messages_scrolled_window = ScrolledWindow::builder().child(&received_messages).build();

        set_hvexpand_t(&recived_messages_scrolled_window);

        //received_messages.buffer().set_text("received_text");

        received_paned.set_start_child(Some(&recived_messages_scrolled_window));

        //End Child - Lower right

        /*
        let tracing_text = TextView::new();

        let tracing_text_scrolledwindow = ScrolledWindow::builder().child(&tracing_text).build();

        set_hvexpand_t(&tracing_text_scrolledwindow);

        received_paned.set_end_child(Some(&tracing_text_scrolledwindow));
        */

        //Init new tab page - this tab page
        
        let tp = wcs.tab_view().append(&contents_box);

        tp.set_title("Websocket Tab");

        //

        contents_box.append(&contents_paned);

        //Initialise reference type instance

        //let actor_state = WriteFrameProcessorActorState::new(); //WebSocketActorState::new();

        //let web_socket_actor; //= GraphQLActor::new(actor_state); //GraphQLRuntimeActor::new(wcs.tokio_rt_handle(), actor_state);
  
        //Try entering the runtime here instead of using a runtime actor. 

        /*{

            let _entered_rt = wcs.tokio_rt_handle().enter();

            web_socket_actor = WebSocketActor::new(actor_state);

        }*/

        let tokio_rt_handle = wcs.tokio_rt_handle();

        //let web_socket_actor 
        
        let write_frame_processor_actor_io_client = enter!(tokio_rt_handle, || {

            //WebSocketActor::new(actor_state)

            //WriteFrameProcessorActor::new(actor_state)

            WriteFrameProcessorActorState::spawn()

        });

        let this =  Rc::new_cyclic( move |weak_self|
        {

            Self
            {

                adapted_contents_box: WidgetAdapter::new(&contents_box, weak_self),
                tp,
                
                //Header

                address_text,
                connect_button,
                disconnect_button,
                time_output_label,
                format_dropdown,
                send_button,

                //Tabpage

                //Top Left

                to_be_sent_text,

                //Bottom Left - tabs

                //query_variables,
                //http_headers,

                //Top Right

                //received_text,

                //Bottom Right

                //tracing_text,
                received_messages,
                received_messages_child_observer,

                //
                
                contents_paned,
                to_be_sent_paned,
                received_paned,
                //web_socket_actor,
                //write_frame_processor_actor,
                write_frame_processor_actor_io_client,
                tokio_rt_handle: wcs.tokio_rt_handle().clone(),
                web_socket_actor_poller: SimpleTimeOut::with_state_ref(Duration::new(1, 0), weak_self), //new(Duration::new(1, 0)),
                mut_state: RefCell::new(MutState::new())

            }

        });

        //Get the StateContainers

        //let scs = StateContainers::get();

        //Add the GraphQLTabState to the the StateContainers

        //scs.add(&this);

        //Add this WebSocketTabState object to the StateContainers Rc instance.

        scs_add!(this);

        //Connect Button

        let weak_self = this.adapted_contents_box.weak_parent();

        this.connect_button.connect_clicked(move |btn|
        {

            up_rc(&weak_self, |this|
            {

                btn.set_sensitive(false);

                borrow_mut(&this.mut_state, |_mut_mut_state|
                {

                    //Return if we're already connected.

                    if this.web_socket_actor_poller.is_active()
                    {

                        return;

                    }

                    let address = this.address_text.text().to_string();

                    /*
                        web_socket_actor_message::WebSocketActorInputMessage` doesn't implement `std::fmt::Debug`
                        the trait `std::fmt::Debug` is not implemented for `web_socket_actor_message::WebSocketActorInputMessage`, which is required by `tokio::sync::mpsc::error::TrySendError<web_socket_actor_message::WebSocketActorInputMessage>: std::fmt::Debug`
                        add `#[derive(Debug)]` to `web_socket_actor_message::WebSocketActorInputMessage` or manually `impl std::fmt::Debug for web_socket_actor_message::WebSocketActorInputMessage`
                        the trait `std::fmt::Debug` is implemented for `tokio::sync::mpsc::error::TrySendError<T>`
                        required for `tokio::sync::mpsc::error::TrySendError<web_socket_actor_message::WebSocketActorInputMessage>` to implement `std::fmt::Debug`rustcClick for full compiler diagnostic
                        result.rs(1073, 12): required by a bound in `Result::<T, E>::unwrap`
                        web_socket_actor_message.rs(31, 1): consider annotating `web_socket_actor_message::WebSocketActorInputMessage` with `#[derive(Debug)]`: `#[derive(Debug)]
                        `
                        web_socket_tab_state.rs(498, 70): consider removing this method call, as the receiver has type `&tokio::sync::mpsc::Sender<web_socket_actor_message::WebSocketActorInputMessage>` and `&tokio::sync::mpsc::Sender<web_socket_actor_message::WebSocketActorInputMessage>: std::fmt::Debug` trivially holds
                     */

                    if let Err(_err) = this.write_frame_processor_actor_io_client.web_socket_input_sender().try_send(WebSocketActorInputMessage::ConnectTo(address)) //.web_socket_actor_io_client().input_sender().try_send(WebSocketActorInputMessage::ConnectTo(address)) //web_socket_actor.interactor().input_sender().try_send(WebSocketActorInputMessage::ConnectTo(address))
                    {

                        //Error: Counld not contact web_socket_actor.

                        panic!("Error: Counld not contact web_socket_actor.");

                    }

                    this.web_socket_actor_poller.start();

                });

            });
            
        });

        let weak_self = this.adapted_contents_box.weak_parent();

        this.disconnect_button.connect_clicked(move |btn|
        {

            //Disconnect from the server.

            //The assumption is that the prior connection was successful.
            
            up_rc(&weak_self, |this|
            {

                btn.set_visible(false);

                if let Err(_err) = this.write_frame_processor_actor_io_client.web_socket_input_sender().try_send(WebSocketActorInputMessage::Disconnect) //.web_socket_actor_io_client().input_sender().try_send(WebSocketActorInputMessage::Disconnect)
                {

                    //Error: Could not contact web_socket_actor.

                    panic!("Error: Could not contact web_socket_actor.");

                }
                /*
                else
                {

                    let connect_button = &this.connect_button;
                    
                    connect_button.set_sensitive(true);
                    
                    connect_button.set_visible(true);

                }
                */

            });

        });

        //TimeOut

        this.web_socket_actor_poller.set_on_time_out_fn(move |sto|
        {

            up_rc_pt(sto.state(), |this|
            {

                let mut receiver = this.write_frame_processor_actor_io_client.read_frame_actor_io_client().output_receiver_lock().expect("Error: read_frame_actor_io_client().output_receiver_lock() panicked");

                match receiver.try_recv()
                {

                    Ok(res) =>
                    {

                        match res
                        {

                            ReadFrameProcessorActorOutputMessage::Processed(processed_text) =>
                            {

                                //TextView should be buffered...

                                /*
                                let tv = TextView::builder().editable(false).build();

                                tv.buffer().set_text(&processed_text);

                                this.received_messages.prepend(&tv);

                                this.regulate_received_messages();
                                */

                                this.output_message(&processed_text);

                            },
                            ReadFrameProcessorActorOutputMessage::ClientMessage(message) =>
                            {

                                match message
                                {

                                    WebSocketActorOutputClientMessage::ConnectionSucceed(message) =>
                                    {

                                        //Post connect_button.connect_clicked

                                        this.output_message(message.as_str());

                                        this.connect_button.set_visible(false);

                                        this.disconnect_button.set_visible(true);

                                    },
                                    WebSocketActorOutputClientMessage::ConnectionFailed(message) =>
                                    {

                                        this.output_message(message.as_str());

                                        if this.disconnect_button.is_visible()
                                        {

                                            this.disconnect_button.set_visible(false);

                                        }

                                        let connect_button = &this.connect_button;
                    
                                        connect_button.set_sensitive(true);
                                        
                                        connect_button.set_visible(true);

                                    },
                                    WebSocketActorOutputClientMessage::Disconnected(message) =>
                                    {

                                        //Post disconnect_button.connect_clicked

                                        this.output_message(message.as_str());

                                        if this.disconnect_button.is_visible()
                                        {

                                            this.disconnect_button.set_visible(false);

                                        }

                                        let connect_button = &this.connect_button;
                    
                                        connect_button.set_sensitive(true);
                                        
                                        connect_button.set_visible(true);

                                    },
                                    WebSocketActorOutputClientMessage::NotConnected(message) =>
                                    {

                                        this.output_message(message.as_str());

                                        if this.disconnect_button.is_visible()
                                        {

                                            this.disconnect_button.set_visible(false);

                                        }

                                        let connect_button = &this.connect_button;
                    
                                        if !connect_button.is_sensitive()
                                        {

                                            connect_button.set_sensitive(true);

                                        }

                                        if !connect_button.is_visible()
                                        {
                                        
                                            connect_button.set_visible(true);

                                        }

                                    }

                                }

                            }

                        }

                    },
                    Err(err) =>
                    {

                        match err
                        {

                            TryRecvError::Empty =>
                            {

                                //Are read frames being processed?

                                if this.write_frame_processor_actor_io_client.is_processing_read_frames()
                                {

                                    return true;

                                }

                                //Is disconnected?

                                if this.connect_button.is_visible()
                                {

                                    return false;

                                }

                            },
                            TryRecvError::Disconnected =>
                            {

                                panic!("Error: The Read Frame Actor is non-functional");

                            }

                        }

                    }

                }

                true

                /*
                borrow_mut(&this.mut_state,|mut mut_state|
                {

                    /*
                    if let Some(rec) = mut_state.graphql_post_request_job.as_mut()
                    {


                    }
                    */

                    //Stop the Timeout

                    false

                })
                */
                
            })

            //Stop the Timeout as weak_self is not upgradeable.

            //false

        });

        this

    }

    //Used, in this case, to center the contents_paned widget.
    
    pub fn set_contents_paned_position_halved(&self, value: i32)
    {

        set_paned_position_halved(&self.contents_paned, value)

    }

    fn regulate_received_messages(&self)
    {

        let limit = 20; //500;

        if limit < self.received_messages_child_observer.n_items()
        {

            let received_messages = &self.received_messages;

            if let Some(last_child) = received_messages.last_child()
            {

                received_messages.remove(&last_child);

                //Cache last_child...

            }

        }

    }

    fn output_message(&self, message: &str)
    {

        let tv = TextView::builder().editable(false).build();

        tv.buffer().set_text(message);

        self.received_messages.prepend(&tv);

        self.regulate_received_messages();

    }

}

impl_as_any!(WebSocketTabState);

impl WidgetStateContainer for WebSocketTabState
{

    fn dyn_adapter(&self) -> Rc<dyn StoredWidgetObject>
    {

        self.adapted_contents_box.clone()

    }

}
