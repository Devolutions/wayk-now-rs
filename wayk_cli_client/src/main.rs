mod authentication;
mod config;

use crate::authentication::AuthenticateSM;
use crate::config::{configure_available_auth_types, configure_capabilities, configure_channels_to_open};
use config::Cli;
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use wayk_proto::channels_manager::ChannelsManager;
use wayk_proto::header::AbstractNowHeader;
use wayk_proto::message::{
    ClipboardFormatDef, NowChatTextMsg, NowClipboardControlRspMsg, NowClipboardFormatDataReqMsg,
    NowClipboardFormatDataRspMsgOwned, NowClipboardFormatListReqMsg, NowString256, NowString65535,
};
use wayk_proto::packet::{NowPacket, NowPacketAccumulator};
use wayk_proto::serialization::Encode;
use wayk_proto::sharee::Sharee;
use wayk_proto::sm::{
    ChannelResponses, ChatChannelCallbackTrait, ChatChannelSM, ChatData, ClientConnectionSeqSM,
    ClipboardChannelCallbackTrait, ClipboardChannelSM, ClipboardData, SMData, SMEvent,
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
                                handle_events(&mut stream, sharee.update_with_body(&packet.body));
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
                    handle_events(&mut stream, sharee.update_without_body());

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
        ),
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

fn build_sharee(args: &Cli) -> Sharee<ClientConnectionSeqSM> {
    // connection sequence
    let connection_seq = ClientConnectionSeqSM::new(AuthenticateSM::new(args.auth.clone()));

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

    let chat_data = ChatData::new().friendly_name(friendly_name).status_text(status_text);
    let chat_channel_sm = ChatChannelSM::new(
        chat_data,
        Box::new(get_current_timestamp),
        ChatCallback {
            on_sync_message: args.on_sync_message.clone(),
        },
    );

    // clipboard channel
    let clipboard_data = ClipboardData::new();
    let clipboard_channel_sm = ClipboardChannelSM::new(
        clipboard_data,
        ClipboardCallback {
            on_ready_message: args.on_clipboard_ready.clone(),
        },
    );

    // channel manager
    let channels_manager = ChannelsManager::new()
        .with_sm(chat_channel_sm)
        .with_sm(clipboard_channel_sm);

    // finally, build the sharee
    Sharee::builder(connection_seq)
        .supported_auths(configure_available_auth_types())
        .capabilities(configure_capabilities())
        .channels_to_open(configure_channels_to_open())
        .channels_manager(channels_manager)
        .build()
}

fn send_packet<W: Write>(writer: &mut W, packet: NowPacket<'_>) {
    writer.write_all(&packet.encode().unwrap()).unwrap();
    log::debug!("Sent {:?} packet.", packet.header.body_type());
}

fn handle_events<W: Write>(writer: &mut W, events: Vec<SMEvent<'_>>) {
    for ev in events {
        match ev {
            SMEvent::StateTransition(s) => log::info!("State transition: {:?}", s),
            SMEvent::PacketToSend(rsp) => send_packet(writer, rsp),
            SMEvent::Data(e) => log::info!("Proto data: {:?}", e),
            SMEvent::Warn(e) => log::warn!("Sharee warning: {}", e),
            SMEvent::Error(e) => log::error!("Sharee error: {}", e),
            SMEvent::Fatal(e) => {
                log::error!("Sharee FATAL error: {}", e);
                panic!("Fatal error: {}", e);
            },
        }
    }
}

struct ClipboardCallback {
    on_ready_message: Option<String>,
}

impl ClipboardChannelCallbackTrait for ClipboardCallback {
    fn on_control_rsp<'msg>(
        &mut self,
        clipboard_data: &mut ClipboardData,
        _: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        _: &NowClipboardControlRspMsg,
    ) {
        to_send.push(NowClipboardFormatListReqMsg::new_with_formats(
            clipboard_data.next_sequence_id(),
            vec![ClipboardFormatDef::new(
                0,
                NowString256::from_str("UTF8_STRING").unwrap(),
            )],
        ));
    }

    fn on_format_data_req<'msg>(
        &mut self,
        clipboard_data: &mut ClipboardData,
        _: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        _: &NowClipboardFormatDataReqMsg,
    ) {
        if let Some(data) = &self.on_ready_message {
            if clipboard_data.is_owner() {
                to_send.push(NowClipboardFormatDataRspMsgOwned::new_with_format_data(
                    clipboard_data.next_sequence_id(),
                    0,
                    data.as_bytes().to_vec(),
                ))
            } else {
                log::warn!("couldn't take clipboard ownership");
            }
        }
    }
}

struct ChatCallback {
    on_sync_message: Option<String>,
}

impl ChatChannelCallbackTrait for ChatCallback {
    fn on_message(&mut self, chat_data: &mut ChatData, _: &mut ChannelResponses<'_>, text_msg: &NowChatTextMsg) {
        println!(
            "|Chat| Message from {}: {}",
            chat_data.distant_friendly_name,
            text_msg.text.as_str()
        );
    }

    fn on_synced<'msg>(&mut self, chat_data: &mut ChatData, to_send: &mut ChannelResponses<'_>) {
        println!(
            "|Chat| Synced with {}. Their status text is `{}`",
            chat_data.distant_friendly_name, chat_data.distant_status_text
        );

        if let Some(sync_msg) = self.on_sync_message.take() {
            match NowString65535::try_from(sync_msg) {
                Ok(msg) => to_send.push(NowChatTextMsg::new(get_current_timestamp(), 0, msg)),
                Err(e) => log::warn!("{}", e),
            }
        }
    }
}

fn get_current_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as u32
}
