use std::cell::{OnceCell, RefCell, RefMut};

use std::fmt::Display;
use std::os::unix::process;
use std::rc::{Weak, Rc};

use std::any::Any;

use std::ops::Deref;

//use std::sync::mpsc::TryRecvError;

use std::sync::OnceLock;

use std::time::Duration;

use corlib::text::{AsStr, SendableText, SendableTextLog, SendableTextLogWithBuffer};
use gtk_estate::adw::gio::ListStore;
use gtk_estate::adw::glib::clone::Upgrade;

//use gtk_estate::corlib::rfc::borrow_mut;

//use gtk_estate::corlib::upgrading::up_rc;

use corlib::upgrading::{up_rc, up_rc_pt};

use gtk_estate::adw::prelude::{Cast, ObjectExt};

//use gtk_estate::gtk4::traits::ListBoxRowExt;

use gtk_estate::gtk4::{Align, BoxLayout, ListBox, ListBoxRow, StringObject, Widget};

use gtk_estate::gtk4::{builders::ButtonBuilder, prelude::EditableExt};

use gtk_estate::{helpers::*, scs_add, RcSimpleTimeOut, SimpleTimeOut, StateContainers, StoredWidgetObject, WidgetAdapter, WidgetStateContainer};

use gtk_estate::gtk4::{self as gtk, Box, Orientation, TextView, Paned, Notebook, Label, CenterBox, DropDown, StringList, Text, Button, Viewport, ScrolledWindow, prelude::{BoxExt, TextViewExt, TextBufferExt, WidgetExt, EntryBufferExtManual, ButtonExt, ListModelExt}};

use gtk_estate::adw::{TabBar, TabPage, TabView};

//use gtk_estate::corlib::{impl_as_any, rc_self_setup, AsAny};

//use corlib::{impl_as_any, AsAny, rfc_borrow, rfc_borrow_mut};

use gtk_estate::corlib::{impl_as_any, AsAny};

//use corlib::upgrading::up_rc;

use corlib::rfc::{borrow, borrow_mut, borrow_mut_param};

use gtk_estate::helpers::{widget_ext::set_hvexpand_t, text_view::get_text_view_string, paned::set_paned_position_halved};

use hyper::client::conn::http1::Connection;
use tokio::runtime::Handle;

use widget_ext::{set_margin_sides_and_bottom, set_margin_start_and_end, set_margin_top_and_bottom};

//use super::pipeline_message_counter::*;

use crate::actors::pipeline_message_counter::*;

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Paned.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.ScrolledWindow.html

//https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Viewport.html

//https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/struct.Flap.html

//If gitlab.gnome.org goes down:

//https://web.archive.org/web/20221126181112/https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/index.html

use crate::actors::{ProcessingFormat, WebSocketActor, WebSocketActorInputMessage, WebSocketActorState, WriteFrameProcessorActor, WriteFrameProcessorActorIOClient, WriteFrameProcessorActorInputMessage, WriteFrameProcessorActorState};

use crate::window_contents_state::WindowContentsState;

use act_rs::{enter, ActorFrontend};

//use act_rs::tokio::interactors::mpsc::SenderInteractor;

use tokio::sync::oneshot::{Sender, Receiver, channel};

use gtk::glib;

use gtk::glib::clone;

use tokio::sync::mpsc::error::TryRecvError;

use crate::actors::{ReadFrameProcessorActorOutputMessage, WebSocketActorOutputClientMessage};

use gtk_estate::gtk4::gio::ListModel;

use gtk_estate::gtk4::prelude::ListBoxRowExt;

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

//static FORMAT_DROPDOWN_STRS: &[&str] = &["JSON To CBOR", "Text"];

//static FORMAT_DROPDOWN_STRS: &[&str] = &["Text"];

//static JSON_TO_CBOR: &str = "JSON To CBOR";

static TEXT: &str = "Text";

static FORMAT_DROPDOWN_STRS: &[&str] = &[TEXT];

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
enum ConnectionStatus
{
    #[default]
    NotConnected,
    Connecting,
    //ReConnecting,
    //SwappingConnection,
    Connected,
    Disconnecting

}

impl ConnectionStatus
{

    pub fn is_not_connected(&self) -> bool
    {

        *self == Self::NotConnected

    }

    pub fn is_connecting(&self) -> bool
    {

        *self == Self::Connecting

    }

    /*
    pub fn is_re_connecting(&self) -> bool
    {

        *self == Self::ReConnecting

    }
    */

    /*
    pub fn is_swapping_connection(&self) -> bool
    {

        *self == Self::SwappingConnection

    }
    */

    pub fn is_connected(&self) -> bool
    {

        *self == Self::Connected

    }

    pub fn is_disconnecting(&self) -> bool
    {

        *self == Self::Disconnecting

    }

    ///
    /// Active connections are in the Connecting, Connected or Disconnecting states.
    /// 
    pub fn is_active(&self) -> bool
    {

        self.is_connecting() | self.is_connected() | self.is_disconnecting()

    }

}

impl Display for ConnectionStatus
{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {

        let text;

        match self
        {

            ConnectionStatus::NotConnected => { text = "NotConnected" },
            ConnectionStatus::Connecting => { text = "Connecting" },
            //ConnectionStatus::ReConnecting => { text = "Reconnecting" },
            //ConnectionStatus::SwappingConnection => { text = "SwappingConnection" },
            ConnectionStatus::Connected => { text = "Connected" },
            ConnectionStatus::Disconnecting => { text = "Disconnecting" }

        }
        
        write!(f, "{}", text)
        
    }

}

//thread 'main' has overflowed its stack
//fatal runtime error: stack overflow

/*
impl Display for ConnectionStatus
{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        
        write!(f, "{}", self)
        
    }

}
*/

struct MutState
{

    pub connection_status: ConnectionStatus,
    pub processing_format: ProcessingFormat,
    //pub received_log: SendableTextLog,
    //pub opt_received_log_cache: Option<String>
    pub received_log: SendableTextLogWithBuffer
}

impl MutState
{

    pub fn new() -> Self
    {

        Self
        {

            connection_status: ConnectionStatus::NotConnected,
            processing_format: ProcessingFormat::Text,
            //received_log: SendableTextLog::with_capacity(20),
            //opt_received_log_cache: Some(String::with_capacity(10000))
            received_log: SendableTextLogWithBuffer::with_capacity_and_buffer(20, String::with_capacity(10000))

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
    //time_output_label: Label,
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

    //received_messages: Box, //ListBox,
    //received_messages_child_observer: ListModel,

    //received_messages_presenter: ListBox,
    //received_messages: ListStore,

    //Bottom Right

    //tracing_text: TextView,

    //

    received_log: TextView,

    contents_paned: Paned,
    to_be_sent_paned: Paned,
    received_paned: Paned,
    //web_socket_actor: WebSocketActor,
    //write_frame_processor_actor: WriteFrameProcessorActor,
    write_frame_processor_actor_io_client: WriteFrameProcessorActorIOClient,
    tokio_rt_handle: Handle,

    web_socket_actor_poller: RcSimpleTimeOut<Weak<WebSocketTabState>>,
    send_ping_button: Button,
    connected_address_text: Text,
    connection_status_text: Text,
    mut_state: RefCell<MutState>,
    pipeline_message_count_decrementor: Decrementor

}

impl WebSocketTabState
{

    pub fn new(wcs: &Rc<WindowContentsState>) -> Rc<Self>
    {

        //Setup the GUI

        let contents_box = Box::new(Orientation::Vertical, 0);

        set_margin_start_and_end(&contents_box, 5);

        contents_box.set_margin_bottom(10);

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

        header_box.set_margin_top(10);

        //set_margin_start_and_end(&header_box, 10);

        header_box.set_margin_bottom(5);

        //set_margin_top_and_bottom(&header_box, 5);
        
        //Buttons and output labels - Second Row

        //CenterBox - Level 1

        let tool_cbox = CenterBox::new();

        //tool_cbox.set_margin_start(5);

        //tool_cbox.set_margin_end(5);

        //tool_cbox.set_margin_bottom(5);

        //set_margin_sides_and_bottom(&tool_cbox, 5);

        //Left

        let tool_left_box = Box::new(Orientation::Horizontal, 40);

        tool_left_box.set_margin_end(10);

        //tool_left_box.set_hexpand(true);

        //tool_left_box.set_hexpand_set(true);

        //Format DropDown 

        let format_dropdown = DropDown::from_strings(FORMAT_DROPDOWN_STRS);

        tool_left_box.append(&format_dropdown);

        //Send Button

        let send_button = Button::builder().label("Send").build();

        send_button.set_halign(Align::Center);

        send_button.set_hexpand(true);

        send_button.set_sensitive(false);

        tool_left_box.append(&send_button);

        //

        tool_cbox.set_start_widget(Some(&tool_left_box));

        //Center
        
        let tool_center_box = Box::new(Orientation::Horizontal, 2);

        let connect_button = Button::builder().label("Connect").build();

        tool_center_box.append(&connect_button);

        let disconnect_button = Button::builder().label("Disconnect").build(); //.visible(false)

        disconnect_button.set_sensitive(false);

        tool_center_box.append(&disconnect_button);

        tool_cbox.set_center_widget(Some(&tool_center_box));

        //Right

        let tool_right_box = Box::new(Orientation::Horizontal, 0); //2);

        //tool_right_box.set_margin_start(10);

        //Binary to BSON or JSON, JSON Only DropDown

        /*
        let time_label = Label::new(Some("Time:"));

        tool_right_box.append(&time_label);

        let time_output_label = Label::new(Some("N/A"));

        tool_right_box.append(&time_output_label);
        */

        //The current connected address

        let connected_address_text = Text::new();

        connected_address_text.set_sensitive(false);

        //connected_address_text.set_hexpand(true);

        //connected_address_text.buffer().set_text("http://localhost:3000");

        //connected_address_text.set_halign(Align::End); //.set_alignment(xalign)

        tool_right_box.append(&connected_address_text);

        //The current connection status

        let connection_status_text = Text::new();

        connection_status_text.set_sensitive(false);

        //connection_status_text.set_hexpand(true);

        let default_cn = ConnectionStatus::NotConnected;

        connection_status_text.buffer().set_text(default_cn.to_string()); //"NotConnected");

        tool_right_box.append(&connection_status_text);

        //The ping button

        let send_ping_button = Button::builder().label("Ping").build();

        send_ping_button.set_sensitive(false);

        tool_right_box.append(&send_ping_button);

        //

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

        //let received_messages = Box::new(Orientation::Vertical, 10); //ListBox::new();

        /*
        let received_messages_presenter = ListBox::new();

        received_messages_presenter.set_vexpand(true);

        received_messages_presenter.set_margin_top(20);

        //received_messages_pesenter.set_valign(Align::Start);

        //let received_messages_child_observer = received_messages.observe_children();

        let received_messages = ListStore::new::<ListBoxRow>(); //<Text>();

        received_messages_presenter.bind_model(Some(&received_messages), |obj|
        {

            //Not being called

            let row = ListBoxRow::new();

            match obj.downcast_ref::<Widget>()
            {

                Some(val) =>
                {

                    row.set_child(Some(val)); //.clone()

                }
                None =>
                {

                    let txt = Text::builder().editable(false).text("Error: Could not convert to Widget.").build(); //.upcast()

                    row.set_child(Some(&txt));

                }

            }

            row.into()

            /*
            match obj.downcast_ref::<Widget>()
            {

                Some(val) =>
                {

                    val.clone()

                }
                None =>
                {

                    Text::builder().editable(false).text("Error: Could not convert to Widget.").build().upcast()

                }

            }
            */

        });
        */

        //let recived_messages_scrolled_window = ScrolledWindow::builder().child(&received_messages_presenter).build();

        let received_log = TextView::builder().editable(false).build(); //sensitive(false).build();

        let recived_messages_scrolled_window = ScrolledWindow::builder().child(&received_log).build();

        set_hvexpand_t(&recived_messages_scrolled_window);

        //received_messages.buffer().set_text("received_text");

        received_paned.set_start_child(Some(&recived_messages_scrolled_window));

        //let box_layout = BoxLayout::builder().orientation(Orientation::Vertical).spacing(10).build();

        //received_paned.set_layout_manager(Some(box_layout));
        
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

        //For counting messages in the read side of the pipeline.

        let (pipeline_message_count_incrementor, pipeline_message_count_decrementor) = inc_dec();

        let write_frame_processor_actor_io_client = enter!(tokio_rt_handle, WriteFrameProcessorActorState::spawn(pipeline_message_count_incrementor));

        //let web_socket_actor 
        
        /*
        let write_frame_processor_actor_io_client = enter!(tokio_rt_handle, || {

            //WebSocketActor::new(actor_state)

            //WriteFrameProcessorActor::new(actor_state)

            WriteFrameProcessorActorState::spawn()

        });
        */

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
                //time_output_label,
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
                //received_messages_presenter,
                //received_messages,
                //received_messages_child_observer,

                //

                received_log,
                
                contents_paned,
                to_be_sent_paned,
                received_paned,
                //web_socket_actor,
                //write_frame_processor_actor,
                write_frame_processor_actor_io_client,
                tokio_rt_handle: wcs.tokio_rt_handle().clone(),
                web_socket_actor_poller: SimpleTimeOut::with_state_ref(Duration::new(1, 0), weak_self), //new(Duration::new(1, 0)),
                send_ping_button,
                connected_address_text,
                connection_status_text,
                mut_state: RefCell::new(MutState::new()),
                pipeline_message_count_decrementor

            }

        });

        //Add this WebSocketTabState object to the StateContainers Rc instance.

        scs_add!(this);

        //Connect Button

        let weak_self = this.adapted_contents_box.weak_parent();

        this.connect_button.connect_clicked(move |_btn|
        {

            up_rc(&weak_self, |this|
            {

                let address_text_buffer = this.address_text.buffer();

                if address_text_buffer.length() == 0
                {

                    //display error

                    //this.output_message("Error: no address provided.");

                    this.update_received_output(SendableText::Str("Error: no address provided."));

                    return;

                }

                let address = this.address_text.text().to_string();

                if let Err(_err) = this.write_frame_processor_actor_io_client.web_socket_input_sender().try_send(WebSocketActorInputMessage::ConnectTo(address)) //.web_socket_actor_io_client().input_sender().try_send(WebSocketActorInputMessage::ConnectTo(address)) //web_socket_actor.interactor().input_sender().try_send(WebSocketActorInputMessage::ConnectTo(address))
                {

                    //Error: Counld not contact web_socket_actor.

                    panic!("Error: Counld not contact web_socket_actor.");

                }

                /*
                let res = borrow(&this.mut_state,|mut_state_ref|
                {

                    mut_state_ref.connection_status

                });
                */

                this.set_status(ConnectionStatus::Connecting);

                /*
                if res == ConnectionStatus::Connected
                {

                    //Reconnecting?

                    //this.set_status(ConnectionStatus::ReConnecting);

                    this.set_status(ConnectionStatus::SwappingConnection);

                }
                else
                {

                    //Not currently connected
                    
                    this.set_status(ConnectionStatus::Connecting);

                }
                */

            });
            
        });

        let weak_self = this.adapted_contents_box.weak_parent();

        //The disconnect button

        this.disconnect_button.connect_clicked(move |_btn|
        {

            //Disconnect from the server.

            //The assumption is that the prior connection was successful.
            
            up_rc(&weak_self, |this|
            {

                //let address_text_buffer = this.address_text.buffer();

                let res = borrow(&this.mut_state, |mut_state|
                {

                    mut_state.connection_status

                    /* 
                    match mut_state.connection_status
                    {

                        //Are we anything other than connected?

                        ConnectionStatus::NotConnected | ConnectionStatus::Connecting | ConnectionStatus::Disconnecting => //ConnectionStatus::SwappingConnection |
                        {

                            return false;

                        }
                        ConnectionStatus::Connected => return true

                    }
                    */

                });

                if res.is_active()
                {

                    if let Err(_err) = this.write_frame_processor_actor_io_client.web_socket_input_sender().try_send(WebSocketActorInputMessage::Disconnect) //WebSocketActorInputMessage::ConnectTo(address_text_buffer.text().into()))
                    {
    
                        //Error: Could not contact web_socket_actor.
    
                        //panic!("Error: Could not contact web_socket_actor.");
    
                        //this.output_message("Error: Could not contact WriteFrameProcessorActor.");

                        this.update_received_output(SendableText::Str("Error: Could not contact WriteFrameProcessorActor."));

                        return;

                    }
    
                    this.set_status(ConnectionStatus::Disconnecting);

                }

            });

        });

        let weak_self = this.adapted_contents_box.weak_parent();

        this.send_button.connect_clicked(move |_btn|
        {

            up_rc(&weak_self, |this|
            {

                let format = borrow(&this.mut_state, |mut_state|
                {

                    mut_state.processing_format

                });

                let tv_string = get_text_view_string(&this.to_be_sent_text);

                if let Err(err) = this.write_frame_processor_actor_io_client.sender().try_send(WriteFrameProcessorActorInputMessage::Process(tv_string, format))
                {

                    panic!("{}", err)

                }

            });

        });

        //Select the format of the message payload.

        let weak_self = this.adapted_contents_box.weak_parent();

        this.format_dropdown.connect_selected_item_notify(move |dd|
        {

            up_rc(&weak_self, |this|
            {

                if let Some(item) = dd.selected_item()
                {

                    if let Some(so_item) = item.downcast_ref::<StringObject>()
                    {

                        //let text = TEXT;

                        match so_item.string().as_str()
                        {

                            "Text" => //TEXT => //text =>
                            {

                                //Change output format to text.

                                borrow_mut(&this.mut_state, |mut mut_state|
                                {

                                    mut_state.processing_format = ProcessingFormat::Text;

                                })

                            }
                            _ =>
                            {
                            }

                        }

                    }

                    //Else error...

                }

            });


        });

        let weak_self = this.adapted_contents_box.weak_parent();

        this.send_ping_button.connect_clicked(move |_btn|
        {

            up_rc(&weak_self, |this|
            {
    
                if let Err(err) = this.write_frame_processor_actor_io_client.web_socket_input_sender().try_send(WebSocketActorInputMessage::SendPing)
                {

                    panic!("{}", err)

                }


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

                                //this.pipeline_message_count_decrementor.dec();

                                //this.output_message(&processed_text);

                                this.update_received_output(SendableText::String(processed_text));

                            },
                            ReadFrameProcessorActorOutputMessage::ClientMessage(message) =>
                            {

                                match message
                                {

                                    WebSocketActorOutputClientMessage::ConnectionSucceed(message) =>
                                    {

                                        //mut_state.connection_status = ConnectionStatus::Connected;

                                        //Post connect_button.connect_clicked

                                        //this.output_message(&message); //.as_str());

                                        //this.connect_button.set_visible(false);

                                        //this.disconnect_button.set_visible(true);

                                        this.update_received_output(message);

                                        this.set_status(ConnectionStatus::Connected);

                                    },
                                    WebSocketActorOutputClientMessage::ConnectionFailed(message) =>
                                    {

                                        //mut_state.connection_status = ConnectionStatus::NotConnected;

                                        //this.output_message(&message);

                                        this.update_received_output(message);

                                        this.set_status(ConnectionStatus::NotConnected);

                                    },
                                    WebSocketActorOutputClientMessage::Disconnected(message) =>
                                    {

                                        //mut_state.connection_status = ConnectionStatus::NotConnected;

                                        //Post disconnect_button.connect_clicked

                                        //this.output_message(&message); //.as_str());

                                        this.update_received_output(message);

                                        this.set_status(ConnectionStatus::NotConnected);

                                    },
                                    WebSocketActorOutputClientMessage::NotConnected(message) =>
                                    {

                                        //mut_state.connection_status = ConnectionStatus::NotConnected;

                                        //this.output_message(&message);

                                        this.update_received_output(message);

                                        this.set_status(ConnectionStatus::NotConnected);

                                    }
                                    WebSocketActorOutputClientMessage::Disconnecting(message) =>
                                    {

                                        //mut_state.connection_status = ConnectionStatus::Disconnecting;

                                        //this.output_message(&message);

                                        this.update_received_output(message);

                                        this.set_status(ConnectionStatus::Disconnecting);

                                    }
                                    WebSocketActorOutputClientMessage::PingFrameReceived(message) | WebSocketActorOutputClientMessage::PongFrameReceived(message) | WebSocketActorOutputClientMessage::CloseFrameReceived(message) =>
                                    {

                                        //this.output_message(&message);

                                        this.update_received_output(message);

                                    }

                                }

                            }

                        }

                        true

                    }
                    Err(err) =>
                    {

                        //panic!("{}", err);

                        match err
                        {

                            TryRecvError::Empty =>
                            {

                                let connection_status = borrow(&this.mut_state, |mut_state|
                                {

                                    mut_state.connection_status

                                });

                                //Are read frames being processed or is the connection status currently still in an active state?

                                //if connection_status.is_active() || this.pipeline_message_count_decrementor.has_messages() //this.write_frame_processor_actor_io_client.is_processing_read_frames() || 

                                let is_active = connection_status.is_active();

                                let has_messages = this.pipeline_message_count_decrementor.has_messages();

                                if is_active || has_messages
                                {

                                    return true;

                                }

                                //Is disconnected?

                                /*
                                if this.connect_button.is_visible()
                                {

                                    return false;

                                }
                                */

                            },
                            TryRecvError::Disconnected =>
                            {

                                panic!("Error: The Read Frame Actor is non-functional");

                            }

                        }

                        false

                    }

                }

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

    /*
    fn regulate_received_messages(&self)
    {

        let limit = 20; //500;

        let n_items = self.received_messages.n_items();

        if n_items > limit
        {

            self.received_messages.remove(n_items - 1);

        }

        /*
        if self.received_messages_child_observer.n_items() == limit
        {

            let received_messages = &self.received_messages;

            if let Some(last_child) = received_messages.last_child()
            {

                received_messages.remove(&last_child);

                //Cache last_child...

            }

        }
        */

    }
    */

    fn update_received_output(&self, message: SendableText)
    {

        /*
        borrow_mut_param(&self.mut_state, (self, message), |mut mut_state|
        {

            mut_state.received_log.push(message);

        });
        */

        let mut mut_state = self.mut_state.borrow_mut();

        mut_state.received_log.push_only(SendableText::Str("\n\n"));

        mut_state.received_log.push(message);

        self.received_log.buffer().set_text(&mut_state.received_log.buffer());

        /*
        match mut_state.opt_received_log_cache.take()
        {

            Some(mut res) =>
            {

                mut_state.received_log.overwrite(&mut res);

                self.received_log.buffer().set_text(&res);

                mut_state.opt_received_log_cache = Some(res);

            }
            None =>
            {

                let mut new_string = String::with_capacity(1000);

                mut_state.received_log.overwrite(&mut new_string);

                self.received_log.buffer().set_text(&new_string);

                mut_state.opt_received_log_cache = Some(new_string);

            }

        }

        self.pipeline_message_count_decrementor.dec();
        */

    }

    /*
    fn output_message(&self, message: &str)
    {

        //let tv = TextView::builder().editable(false).build();

        //tv.buffer().set_text(message);

        /*
        let n_items = self.received_messages.n_items();

        let position;

        if n_items == 0
        {

            position = n_items;

        }
        else
        {

            position = n_items - 1;

        }
        */

        //self.received_messages.insert(0, &tv);

        //let row = ListBoxRow::new();

        //row.set_child(Some(&tv));

        //self.received_messages.insert(0, &row);

        //row.activate();

        //row.set_visible(true); //.show();

        //self.received_messages.append(&tv);

        //self.received_messages.append(&row);

        //self.received_messages.prepend(&tv);

        self.pipeline_message_count_decrementor.dec();

        //self.regulate_received_messages();

        //self.received_messages.queue_draw(); //.activate();

        //self.received_messages.bind_property(source_property, target, target_property)

        //self.received_messages.realize() //.queue_resize();

        //self.received_messages.activate();

    }
    */

    fn set_status(&self, status: ConnectionStatus)
    {

        borrow_mut(&self.mut_state, |mut mut_state|
        {

            if mut_state.connection_status == status
            {

                return;

            }

            match status
            {

                ConnectionStatus::NotConnected =>
                {

                    //self.disconnect_button.set_visible(false);

                    //self.format_dropdown.set_sensitive(true);

                    self.send_button.set_sensitive(false);

                    self.connect_button.set_sensitive(true);

                    self.disconnect_button.set_sensitive(false);

                    //let connect_button = &self.connect_button;
                    
                    //connect_button.set_visible(true);

                    self.connected_address_text.buffer().set_text("");

                    //self.connection_status_text

                    self.send_ping_button.set_sensitive(false);

                }
                ConnectionStatus::Connecting =>
                {

                    //self.format_dropdown.set_sensitive(false);

                    self.connect_button.set_sensitive(false);

                    self.disconnect_button.set_sensitive(false);

                    //self.connect_button.set_sensitive(false);

                    self.connected_address_text.buffer().set_text(self.connected_address_text.buffer().text());

                    //

                    self.web_socket_actor_poller.start();

                }
                /*
                ConnectionStatus::SwappingConnection =>
                {

                    self.send_button.set_sensitive(false);

                    self.connect_button.set_sensitive(false);

                    self.connected_address_text.buffer().set_text(self.connected_address_text.buffer().text());

                    self.send_ping_button.set_sensitive(false);

                }
                */
                ConnectionStatus::Connected =>
                {

                    //self.connect_button.set_visible(false);

                    //self.disconnect_button.set_visible(true);

                    self.send_button.set_sensitive(true);

                    self.connect_button.set_sensitive(true);

                    self.disconnect_button.set_sensitive(true);

                    self.send_ping_button.set_sensitive(true);

                }
                ConnectionStatus::Disconnecting =>
                {

                    self.send_button.set_sensitive(false);

                    self.connect_button.set_sensitive(false);

                    self.disconnect_button.set_sensitive(false);

                    self.send_ping_button.set_sensitive(false);

                }

            }

            mut_state.connection_status = status;

            self.connection_status_text.buffer().set_text(status.to_string());

        })

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
