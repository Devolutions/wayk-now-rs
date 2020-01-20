mod authentication;
mod config;

use crate::{
    authentication::AuthenticateSM,
    config::{configure_available_auth_types, configure_capabilities, configure_channels_to_open},
};
use config::Cli;
use std::{
    convert::TryFrom,
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    rc::Rc,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use structopt::StructOpt;
use wayk_proto::{
    channels_manager::ChannelsManager,
    header::AbstractNowHeader,
    message::{
        ClipboardFormatDef, NowChatTextMsg, NowClipboardControlRspMsg, NowClipboardFormatDataReqMsg,
        NowClipboardFormatDataRspMsgOwned, NowClipboardFormatListReqMsg, NowMessage, NowString256, NowString65535,
    },
    packet::{NowPacket, NowPacketAccumulator},
    serialization::Encode,
    sharee::{Sharee, ShareeCallbackTrait, ShareeResult},
    sm::{
        ChatChannelCallbackTrait, ChatChannelSM, ChatData, ChatDataRc, ClientConnectionSeqSM,
        ClipboardChannelCallbackTrait, ClipboardChannelSM, ClipboardData, ClipboardDataRc, DummyConnectionSeqCallback,
        VirtChannelSMResult,
    },
};

fn main() {
    // parse arguments
    let args = Cli::from_args();

    configure_logger(&args);

    log::trace!("{:?}", args);

    match TcpStream::connect(args.addr) {
        Ok(mut stream) => {
            log::info!("Connected to server at {}", stream.peer_addr().unwrap());

            let mut sharee = build_sharee(&args);
            let mut acc = NowPacketAccumulator::new();
            let mut buf = [0; 512];
            'main: loop {
                while sharee.waiting_for_packet() {
                    if let Some(packet) = acc.next_packet(&sharee.get_channels_ctx()) {
                        match packet {
                            Ok(packet) => {
                                log::debug!("Received {:?} packet.", packet.header.body_type());
                                handle_update_result(&mut stream, sharee.update_with_body(&packet.body));
                            }
                            Err(err) => log::error!("Invalid packet: {}", err),
                        }
                        acc.purge_old_packets();

                        if sharee.is_terminated() {
                            break 'main;
                        }
                    } else {
                        let n = stream.read(&mut buf).unwrap();
                        acc.accumulate(&buf[..n]);
                    }
                }

                while !sharee.waiting_for_packet() {
                    handle_update_result(&mut stream, sharee.update_without_body());

                    if sharee.is_terminated() {
                        break 'main;
                    }
                }
            }

            stream.shutdown(Shutdown::Both).unwrap();

            log::info!("Connection with server closed.");
        }
        Err(err) => log::error!("Couldn't connect to server: {}", err),
    }
}

fn configure_logger(args: &Cli) {
    use simplelog::*;
    use std::fs::File;
    CombinedLogger::init(vec![
        TermLogger::new(
            if args.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Warn
            },
            Config::default(),
            TerminalMode::Mixed,
        )
        .unwrap(),
        WriteLogger::new(
            if args.debug {
                LevelFilter::Trace
            } else {
                LevelFilter::Info
            },
            Config::default(),
            File::create("wayk_cli.log").unwrap(),
        ),
    ])
    .unwrap();
}

fn build_sharee(args: &Cli) -> Sharee<ClientConnectionSeqSM<DummyConnectionSeqCallback>, ShareeCallback> {
    // connection sequence
    let connection_seq = ClientConnectionSeqSM::builder(DummyConnectionSeqCallback)
        .available_auth_process(configure_available_auth_types())
        .capabilities(configure_capabilities())
        .channels_to_open(configure_channels_to_open())
        .authenticate_sm(AuthenticateSM::new(args.auth.clone()))
        .build();

    // chat channel
    let friendly_name = args
        .chat_config
        .clone()
        .map(|config| config.friendly_name)
        .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "Anonymous".into()));
    let status_text = args
        .chat_config
        .clone()
        .map(|config| config.status_text.unwrap_or_else(|| "".into()))
        .unwrap_or_else(|| "".into());

    let chat_data = ChatData::new()
        .friendly_name(friendly_name)
        .status_text(status_text)
        .into_rc();
    let chat_channel_sm = ChatChannelSM::new(
        Rc::clone(&chat_data),
        Box::new(get_current_timestamp),
        ChatCallback {
            chat_data,
            on_sync_message: args.on_sync_message.clone(),
        },
    );

    // clipboard channel
    let clipboard_data = ClipboardData::new().into_rc();
    let clipboard_channel_sm = ClipboardChannelSM::new(
        Rc::clone(&clipboard_data),
        ClipboardCallback {
            clipboard_data,
            on_ready_message: args.on_clipboard_ready.clone(),
        },
    );

    // channel manager
    let channels_manager = ChannelsManager::new()
        .with_sm(chat_channel_sm)
        .with_sm(clipboard_channel_sm);

    // finally, build the sharee
    Sharee::new(connection_seq, channels_manager, ShareeCallback)
}

fn send_packet<'packet, W: Write>(writer: &mut W, packet: NowPacket<'packet>) {
    writer.write_all(&packet.encode().unwrap()).unwrap();
    log::debug!("Sent {:?} packet.", packet.header.body_type());
}

fn handle_update_result<'packet, W: Write>(writer: &mut W, update_result: ShareeResult<'packet>) {
    match update_result {
        Ok(response) => {
            if let Some(response) = response {
                send_packet(writer, response)
            }
        }
        Err(err) => log::error!("Sharee update error: {}", err),
    }
}

struct ShareeCallback;
impl ShareeCallbackTrait for ShareeCallback {
    fn on_unprocessed_message<'msg: 'a, 'a>(&mut self, message: &'a NowMessage<'msg>) -> ShareeResult<'msg> {
        log::info!("Received {:?} message", message.get_type());
        Ok(None)
    }
}

struct ClipboardCallback {
    clipboard_data: ClipboardDataRc,
    on_ready_message: Option<String>,
}

impl ClipboardChannelCallbackTrait for ClipboardCallback {
    fn on_control_rsp<'msg>(&mut self, _: &NowClipboardControlRspMsg) -> VirtChannelSMResult<'msg> {
        Ok(Some(
            NowClipboardFormatListReqMsg::new_with_formats(
                self.clipboard_data.borrow_mut().next_sequence_id(),
                vec![ClipboardFormatDef::new(
                    0,
                    NowString256::from_str("UTF8_STRING").unwrap(),
                )],
            )
            .into(),
        ))
    }

    fn on_format_data_req<'msg>(&mut self, _: &NowClipboardFormatDataReqMsg) -> VirtChannelSMResult<'msg> {
        if let Some(data) = &self.on_ready_message {
            if self.clipboard_data.borrow().is_owner() {
                Ok(Some(
                    NowClipboardFormatDataRspMsgOwned::new_with_format_data(
                        self.clipboard_data.borrow_mut().next_sequence_id(),
                        0,
                        data.as_bytes().to_vec(),
                    )
                    .into(),
                ))
            } else {
                log::warn!("couldn't take clipboard ownership");
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

struct ChatCallback {
    chat_data: ChatDataRc,
    on_sync_message: Option<String>,
}

impl ChatChannelCallbackTrait for ChatCallback {
    fn on_message<'msg>(&mut self, text_msg: &NowChatTextMsg) -> VirtChannelSMResult<'msg> {
        println!(
            "|Chat| Message from {}: {}",
            self.chat_data.borrow().distant_friendly_name,
            text_msg.text.as_str()
        );
        Ok(None)
    }

    fn on_synced<'msg>(&mut self) -> VirtChannelSMResult<'msg> {
        let borrowed_config = self.chat_data.borrow();
        println!(
            "|Chat| Synced with {}. Their status text is `{}`",
            borrowed_config.distant_friendly_name, borrowed_config.distant_status_text
        );
        drop(borrowed_config);

        if let Some(sync_msg) = self.on_sync_message.take() {
            Ok(Some(
                NowChatTextMsg::new(get_current_timestamp(), 0, NowString65535::try_from(sync_msg)?).into(),
            ))
        } else {
            Ok(None)
        }
    }
}

fn get_current_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as u32
}
