use alloc::fmt;
use alloc::str::FromStr;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Errors when parsing a callsign-SSID into an `Address`
#[derive(Debug)]
pub enum AddressParseError {
    InvalidFormat,
    InvalidSsid { source: core::num::ParseIntError },
    SsidOutOfRange,
}

#[cfg(feature = "std")]
impl std::error::Error for AddressParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidSsid { source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for AddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(
                f,
                "Address must be a callsign, '-', and a numeric SSID. Example: VK7NTK-0"
            ),
            Self::InvalidSsid { source } => write!(f, "Could not parse SSID: {}", source),
            Self::SsidOutOfRange => write!(f, "SSID must be between 0 and 15 inclusive"),
        }
    }
}

/// Errors when parsing a byte buffer into an `Ax25Frame`
#[derive(Debug)]
pub enum FrameParseError {
    OnlyNullBytes,
    NoEndToAddressField,
    AddressFieldTooShort {
        start: usize,
        end: usize,
    },
    FrameTooShort {
        len: usize,
    },
    AddressInvalidUtf8 {
        source: alloc::string::FromUtf8Error,
    },
    ContentZeroLength,
    MissingPidField,
    UnrecognisedSFieldType,
    UnrecognisedUFieldType,
    WrongSizeFrmrInfo,
    Xid,
}

#[cfg(feature = "std")]
impl std::error::Error for FrameParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AddressInvalidUtf8 { source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for FrameParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OnlyNullBytes => write!(f, "Supplied frame only contains null bytes"),
            Self::NoEndToAddressField => write!(f, "Unable to locate end of address field"),
            Self::AddressFieldTooShort { start, end } => {
                write!(f, "Address field too short: start {} end {}", start, end)
            }
            Self::FrameTooShort { len } => write!(f, "Frame is too short: len {}", len),
            Self::AddressInvalidUtf8 { .. } => write!(f, "Callsign is not valid UTF-8"),
            Self::ContentZeroLength => write!(f, "Content section of frame is empty"),
            Self::MissingPidField => write!(f, "Protocol ID field is missing"),
            Self::UnrecognisedUFieldType => write!(f, "Unrecognised U field type"),
            Self::UnrecognisedSFieldType => write!(f, "Unrecognised S field type"),
            Self::WrongSizeFrmrInfo => write!(f, "Wrong size for FRMR info"),
            Self::Xid => write!(f, "Unexpected XID format"),
        }
    }
}

/// Human-readable protocol identifiers, mostly from the AX.25 2.2 spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolIdentifier {
    Layer3Impl,
    X25Plp,
    CompressedTcpIp,
    UncompressedTcpIp,
    SegmentationFragment,
    TexnetDatagram,
    LinkQuality,
    Appletalk,
    AppletalkArp,
    ArpaIp,
    ArpaAddress,
    Flexnet,
    NetRom,
    None,
    Escape,
    Unknown(u8),
}

impl ProtocolIdentifier {
    fn from_byte(byte: u8) -> ProtocolIdentifier {
        match byte {
            pid if pid & 0b0011_0000 == 0b0001_0000 || pid & 0b0011_0000 == 0b0010_0000 => {
                ProtocolIdentifier::Layer3Impl
            }
            0x01 => ProtocolIdentifier::X25Plp,
            0x06 => ProtocolIdentifier::CompressedTcpIp,
            0x07 => ProtocolIdentifier::UncompressedTcpIp,
            0x08 => ProtocolIdentifier::SegmentationFragment,
            0xC3 => ProtocolIdentifier::TexnetDatagram,
            0xC4 => ProtocolIdentifier::LinkQuality,
            0xCA => ProtocolIdentifier::Appletalk,
            0xCB => ProtocolIdentifier::AppletalkArp,
            0xCC => ProtocolIdentifier::ArpaIp,
            0xCD => ProtocolIdentifier::ArpaAddress,
            0xCE => ProtocolIdentifier::Flexnet,
            0xCF => ProtocolIdentifier::NetRom,
            0xF0 => ProtocolIdentifier::None,
            0xFF => ProtocolIdentifier::Escape,
            pid => ProtocolIdentifier::Unknown(pid),
        }
    }

    fn to_byte(&self) -> u8 {
        match *self {
            ProtocolIdentifier::Layer3Impl => 0b0001_0000,
            ProtocolIdentifier::X25Plp => 0x01,
            ProtocolIdentifier::CompressedTcpIp => 0x06,
            ProtocolIdentifier::UncompressedTcpIp => 0x07,
            ProtocolIdentifier::SegmentationFragment => 0x08,
            ProtocolIdentifier::TexnetDatagram => 0xC3,
            ProtocolIdentifier::LinkQuality => 0xC4,
            ProtocolIdentifier::Appletalk => 0xCA,
            ProtocolIdentifier::AppletalkArp => 0xCB,
            ProtocolIdentifier::ArpaIp => 0xCC,
            ProtocolIdentifier::ArpaAddress => 0xCD,
            ProtocolIdentifier::Flexnet => 0xCE,
            ProtocolIdentifier::NetRom => 0xCF,
            ProtocolIdentifier::None => 0xF0,
            ProtocolIdentifier::Escape => 0xFF,
            ProtocolIdentifier::Unknown(pid) => pid,
        }
    }
}

/// Indicates whether a given frame is a Command or a Response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResponse {
    Command,
    Response,
}

/// Information (I) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Information {
    pub pid: ProtocolIdentifier,
    pub info: Vec<u8>,
    pub receive_sequence: u8,
    pub send_sequence: u8,
    pub poll: bool,
}

/// RR Supervisory (S) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveReady {
    pub receive_sequence: u8,
    pub poll_or_final: bool,
}

/// RNR Supervisory (S) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveNotReady {
    pub receive_sequence: u8,
    pub poll_or_final: bool,
}

/// REJ Supervisory (S) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject {
    pub receive_sequence: u8,
    pub poll_or_final: bool,
}

/// SREJ Supervisory (S) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectiveReject {
    pub receive_sequence: u8,
    pub poll_or_final: bool,
}

/// SABM Unnumbered (U) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetAsynchronousBalancedMode {
    pub poll: bool,
}

/// SABME Unnumbered (U) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetAsynchronousBalancedModeExtended {
    pub poll: bool,
}

/// DISC Unnumbered (U) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disconnect {
    pub poll: bool,
}

/// DM Unnumbered (U) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectedMode {
    pub final_bit: bool, // 'final' is a rust keyword
}

/// UA Unnumbered (U) frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnnumberedAcknowledge {
    pub final_bit: bool,
}

/// FRMR Unnumbered (U) frame. Flags correspond to names in the AX.25 specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameReject {
    pub final_bit: bool,
    /// A raw copy of the control field in the frame that was rejected
    pub rejected_control_field_raw: u8,
    /// The attached control field contained an invalid Receive Sequence Number
    pub z: bool,
    /// The information field of a received frame exceeded the maximum allowable length.
    pub y: bool,
    /// A U or S frame was received that contained an information field.
    pub x: bool,
    /// The received control field was invalid or not implemented.
    pub w: bool,
    pub receive_sequence: u8,
    pub send_sequence: u8,
    pub command_response: CommandResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectiveRejectMode {
    Multi,
    Single,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuloMode {
    Modulo8,
    Modulo128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeIdentification {
    pub poll_or_final: bool,
    pub full_duplex: Option<bool>,
    pub srej_mode: Option<SelectiveRejectMode>,
    pub modulo: Option<ModuloMode>,
    pub i_field_length_rx: Option<u32>,
    pub window_size_rx: Option<u32>,
    pub ack_timer: Option<u32>,
    pub retries: Option<u32>,
}

/// UI Unnumbered Information frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnnumberedInformation {
    pub pid: ProtocolIdentifier,
    pub info: Vec<u8>,
    pub poll_or_final: bool,
}

/// Test frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub poll_or_final: bool,
    pub info: Vec<u8>,
}

/// Placeholder for when the Address part was parseable but not the control field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownContent {
    pub raw: Vec<u8>,
}

/// The body of the frame after the end of the address field
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameContent {
    Information(Information),
    ReceiveReady(ReceiveReady),
    ReceiveNotReady(ReceiveNotReady),
    Reject(Reject),
    SelectiveReject(SelectiveReject),
    SetAsynchronousBalancedMode(SetAsynchronousBalancedMode),
    SetAsynchronousBalancedModeExtended(SetAsynchronousBalancedModeExtended),
    Disconnect(Disconnect),
    DisconnectedMode(DisconnectedMode),
    UnnumberedAcknowledge(UnnumberedAcknowledge),
    FrameReject(FrameReject),
    ExchangeIdentification(ExchangeIdentification),
    UnnumberedInformation(UnnumberedInformation),
    Test(Test),
    UnknownContent(UnknownContent),
}

impl FrameContent {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();

        match *self {
            FrameContent::Information(ref i) => {
                let mut c: u8 = 0;
                c |= (i.receive_sequence & 0b0000_0111) << 5;
                c |= if i.poll { 1 << 4 } else { 0 };
                c |= (i.send_sequence & 0b0000_0111) << 1;
                encoded.push(c);
                encoded.push(i.pid.to_byte());
                encoded.extend(&i.info);
            }
            FrameContent::ReceiveReady(ref rr) => {
                let mut c: u8 = 0b0000_0001;
                c |= if rr.poll_or_final { 1 << 4 } else { 0 };
                c |= (rr.receive_sequence & 0b0000_0111) << 5;
                encoded.push(c);
            }
            FrameContent::ReceiveNotReady(ref rnr) => {
                let mut c: u8 = 0b0000_0101;
                c |= if rnr.poll_or_final { 1 << 4 } else { 0 };
                c |= (rnr.receive_sequence & 0b0000_0111) << 5;
                encoded.push(c);
            }
            FrameContent::Reject(ref rej) => {
                let mut c: u8 = 0b0000_1001;
                c |= if rej.poll_or_final { 1 << 4 } else { 0 };
                c |= (rej.receive_sequence & 0b0000_0111) << 5;
                encoded.push(c);
            }
            FrameContent::SelectiveReject(ref srej) => {
                let mut c: u8 = 0b0000_1101;
                c |= if srej.poll_or_final { 1 << 4 } else { 0 };
                c |= (srej.receive_sequence & 0b0000_0111) << 5;
                encoded.push(c);
            }
            FrameContent::SetAsynchronousBalancedMode(ref sabm) => {
                let mut c: u8 = 0b0010_1111;
                c |= if sabm.poll { 1 << 4 } else { 0 };
                encoded.push(c);
            }
            FrameContent::SetAsynchronousBalancedModeExtended(ref sabme) => {
                let mut c: u8 = 0b0110_1111;
                c |= if sabme.poll { 1 << 4 } else { 0 };
                encoded.push(c);
            }
            FrameContent::Disconnect(ref disc) => {
                let mut c: u8 = 0b0100_0011;
                c |= if disc.poll { 1 << 4 } else { 0 };
                encoded.push(c);
            }
            FrameContent::DisconnectedMode(ref dm) => {
                let mut c: u8 = 0b0000_1111;
                c |= if dm.final_bit { 1 << 4 } else { 0 };
                encoded.push(c);
            }
            FrameContent::UnnumberedAcknowledge(ref ua) => {
                let mut c: u8 = 0b0110_0011;
                c |= if ua.final_bit { 1 << 4 } else { 0 };
                encoded.push(c);
            }
            FrameContent::FrameReject(ref fr) => {
                let mut c: u8 = 0b1000_0111;
                c |= if fr.final_bit { 1 << 4 } else { 0 };
                encoded.push(c);
                let mut frmr1: u8 = 0;
                frmr1 |= if fr.z { 1 << 3 } else { 0 };
                frmr1 |= if fr.y { 1 << 2 } else { 0 };
                frmr1 |= if fr.x { 1 << 1 } else { 0 };
                frmr1 |= if fr.w { 1 } else { 0 };
                encoded.push(frmr1);
                let mut frmr2: u8 = 0;
                frmr2 |= (fr.receive_sequence & 0b0000_0111) << 5;
                frmr2 |= if fr.command_response == CommandResponse::Response {
                    1 << 4
                } else {
                    0
                };
                frmr2 |= (fr.send_sequence & 0b0000_0111) << 1;
                encoded.push(frmr2);
                encoded.push(fr.rejected_control_field_raw);
            }
            FrameContent::UnnumberedInformation(ref ui) => {
                let mut c: u8 = 0b0000_0011;
                c |= if ui.poll_or_final { 1 << 4 } else { 0 };
                encoded.push(c);
                encoded.push(ui.pid.to_byte());
                encoded.extend(&ui.info);
            }
            FrameContent::ExchangeIdentification(ref xid) => {
                let mut c: u8 = 0b1010_1111;
                c |= if xid.poll_or_final { 1 << 4 } else { 0 };
                encoded.push(c);

                encoded.push(XID_FORMAT_IDENTIFIER);
                encoded.push(XID_GROUP_IDENTIFIER);

                let groups_length = 4
                    + 5
                    + xid.i_field_length_rx.map(|_| 4).unwrap_or_default()
                    + xid.window_size_rx.map(|_| 3).unwrap_or_default()
                    + xid.ack_timer.map(|_| 4).unwrap_or_default()
                    + xid.retries.map(|_| 3).unwrap_or_default();

                // Groups length.
                encoded.push(0);
                encoded.push(groups_length as u8);

                encoded.push(PI_CLASSES_OF_PROCEDURES);
                encoded.push(2);

                let value = PV_CLASSES_PROCEDURES_BALANCED_ABM
                    | match xid.full_duplex {
                        Some(true) => PV_CLASSES_PROCEDURES_FULL_DUPLEX,
                        Some(false) => PV_CLASSES_PROCEDURES_HALF_DUPLEX,
                        None => PV_CLASSES_PROCEDURES_HALF_DUPLEX,
                    };

                encoded.push((value >> 8) as u8);
                encoded.push(value as u8);

                encoded.push(PI_HDLC_OPTIONAL_FUNCTIONS);
                encoded.push(3);

                let value = PV_HDLC_OPTIONAL_FUNCTIONS_EXTENDED_ADDRESS
                    | PV_HDLC_OPTIONAL_FUNCTIONS_TEST_CMD_RESP
                    | PV_HDLC_OPTIONAL_FUNCTIONS_16_BIT_FCS
                    | PV_HDLC_OPTIONAL_FUNCTIONS_SYNCHRONOUS_TX;

                let value = value
                    | match xid.srej_mode {
                        Some(SelectiveRejectMode::Multi) => {
                            PV_HDLC_OPTIONAL_FUNCTIONS_MULTI_SREJ_CMD_RESP
                        }
                        Some(SelectiveRejectMode::Single) => {
                            PV_HDLC_OPTIONAL_FUNCTIONS_SREJ_CMD_RESP
                        }
                        Some(SelectiveRejectMode::None) | None => {
                            PV_HDLC_OPTIONAL_FUNCTIONS_REJ_CMD_RESP
                        }
                    };

                let value = value
                    | match xid.modulo {
                        Some(ModuloMode::Modulo128) => PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_128,
                        Some(ModuloMode::Modulo8) | None => PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_8,
                    };

                encoded.push((value >> 16) as u8);
                encoded.push((value >> 8) as u8);
                encoded.push(value as u8);

                if let Some(i_field_length_rx) = xid.i_field_length_rx {
                    encoded.push(PI_I_FIELD_LENGTH_RX);
                    encoded.push(2);
                    let value = i_field_length_rx * 8;
                    encoded.push((value >> 8) as u8);
                    encoded.push(value as u8);
                }

                if let Some(window_size_rx) = xid.window_size_rx {
                    encoded.push(PI_WINDOW_SIZE_RX);
                    encoded.push(1);
                    encoded.push(window_size_rx as u8);
                }

                if let Some(ack_timer) = xid.ack_timer {
                    encoded.push(PI_ACK_TIMER);
                    encoded.push(2);
                    encoded.push((ack_timer >> 8) as u8);
                    encoded.push(ack_timer as u8);
                }

                if let Some(retries) = xid.retries {
                    encoded.push(PI_RETRIES);
                    encoded.push(1);
                    encoded.push(retries as u8);
                }
            }
            FrameContent::Test(ref test) => {
                let mut c: u8 = 0b1110_0011;
                c |= if test.poll_or_final { 1 << 4 } else { 0 };
                encoded.push(c);
                encoded.extend(&test.info);
            }
            FrameContent::UnknownContent(ref uc) => {
                encoded.extend(&uc.raw);
            }
        }

        encoded
    }
}

/// A source or destination of an AX.25 frame, combining a callsign with an SSID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address {
    // An alphanumeric ASCII callsign of maximum length 6, e.g. "VK7NTK"
    pub callsign: String,
    /// Secondary Station Identifier, from 0 to 15
    pub ssid: u8,
    c_bit: bool,
}

impl Address {
    fn to_bytes(&self, high_bit: bool, final_in_address: bool) -> Vec<u8> {
        let mut encoded = Vec::new();
        // Shift by one bit as required for AX.25 address encoding
        for b in self.callsign.as_bytes() {
            encoded.push(b << 1);
        }
        // Pad with spaces up to length 6
        while encoded.len() != 6 {
            encoded.push(b' ' << 1);
        }
        // Now do the SSID byte
        let high = if high_bit { 0b1000_0000 } else { 0 };
        let low = if final_in_address { 0b0000_0001 } else { 0 };
        let ssid_byte = (self.ssid << 1) | 0b0110_0000 | high | low;
        encoded.push(ssid_byte);

        encoded
    }
}

impl Default for Address {
    fn default() -> Address {
        Address {
            callsign: "NOCALL".to_string(),
            ssid: 0,
            c_bit: false,
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ssid_str = match self.ssid {
            0 => "".to_string(),
            ssid => alloc::format!("-{}", ssid),
        };
        write!(f, "{}{}", self.callsign, ssid_str)
    }
}

impl FromStr for Address {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(AddressParseError::InvalidFormat);
        }

        let callsign = parts[0].to_uppercase();
        if callsign.is_empty() || callsign.len() > 6 {
            return Err(AddressParseError::InvalidFormat);
        }
        for c in callsign.chars() {
            if !c.is_alphanumeric() {
                return Err(AddressParseError::InvalidFormat);
            }
        }

        let ssid = parts[1]
            .parse::<u8>()
            .map_err(|e| AddressParseError::InvalidSsid { source: e })?;
        if ssid > 15 {
            return Err(AddressParseError::SsidOutOfRange);
        }

        // c_bit will be set on transmit
        Ok(Address {
            callsign,
            ssid,
            c_bit: false,
        })
    }
}

/// A single hop in the frame's route
#[derive(Debug, Clone)]
pub struct RouteEntry {
    /// Callsign-SSID of a repeater to use for source routing.
    pub repeater: Address,
    /// A flag that is set to true when it passes through the nominated repeater.
    pub has_repeated: bool,
}

/// A strongly-typed representation of a single AX.25 frame.
#[derive(Debug, Clone)]
pub struct Ax25Frame {
    /// Sending station
    pub source: Address,
    /// Destination station
    pub destination: Address,
    /// The route the frame has taken/will take according to repeater entries in the address field
    pub route: Vec<RouteEntry>,
    /// AX.25 2.0-compliant stations will indicate in every frame whether it is a command
    /// or a response, as part of the address field.
    pub command_or_response: Option<CommandResponse>,
    /// Various content depending on the packet type
    pub content: FrameContent,
}

impl Ax25Frame {
    /// Returns a UTF-8 string that is a "best effort" at displaying the information
    /// content of this frame. Returns None if there is no information field present.
    /// Most applications will need to work with the Vec<u8> info directly.
    pub fn info_string_lossy(&self) -> Option<String> {
        match self.content {
            FrameContent::Information(ref i) => Some(String::from_utf8_lossy(&i.info).into_owned()),
            FrameContent::UnnumberedInformation(ref ui) => {
                Some(String::from_utf8_lossy(&ui.info).into_owned())
            }
            _ => None,
        }
    }

    /// Parse raw bytes into an Ax25Frame if possible.
    pub fn from_bytes(bytes: &[u8]) -> Result<Ax25Frame, FrameParseError> {
        // Skip over leading null bytes
        // Linux AF_PACKET has oen of these - we will strip it out in the linux module
        // but also keep the protection here
        let addr_start = bytes
            .iter()
            .position(|&c| c != 0)
            .ok_or(FrameParseError::OnlyNullBytes)?;
        let addr_end = bytes
            .iter()
            .position(|&c| c & 0x01 == 0x01)
            .ok_or(FrameParseError::NoEndToAddressField)?;
        let control = addr_end + 1;
        // +1 because the "terminator" is actually within the last byte
        if addr_end - addr_start + 1 < 14 {
            return Err(FrameParseError::AddressFieldTooShort {
                start: addr_start,
                end: addr_end,
            });
        }
        if control >= bytes.len() {
            return Err(FrameParseError::FrameTooShort { len: bytes.len() });
        }

        let dest = parse_address(&bytes[addr_start..addr_start + 7])?;
        let src = parse_address(&bytes[addr_start + 7..addr_start + 14])?;
        let rpt_count = (addr_end + 1 - addr_start - 14) / 7;
        let mut route: Vec<RouteEntry> = Vec::new();
        for i in 0..rpt_count {
            let repeater =
                parse_address(&bytes[addr_start + 14 + i * 7..addr_start + 14 + (i + 1) * 7])?;
            let entry = RouteEntry {
                has_repeated: repeater.c_bit, // The "C" bit in an address happens to be the repeated bit for a repeater
                repeater,
            };
            route.push(entry);
        }

        let content = parse_content(&bytes[control..])?;
        let command_or_response = match (dest.c_bit, src.c_bit) {
            (true, false) => Some(CommandResponse::Command),
            (false, true) => Some(CommandResponse::Response),
            _ => None,
        };

        Ok(Ax25Frame {
            source: src,
            destination: dest,
            route,
            content,
            command_or_response,
        })
    }

    /// Encode an Ax25Frame struct as raw bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut frame = Vec::new();
        let (dest_c_bit, src_c_bit) = match self.command_or_response {
            Some(CommandResponse::Command) => (true, false),
            Some(CommandResponse::Response) => (false, true),
            _ => (true, false), // assume Command
        };
        frame.extend(self.destination.to_bytes(dest_c_bit, false));
        frame.extend(self.source.to_bytes(src_c_bit, self.route.is_empty()));

        for (i, entry) in self.route.iter().enumerate() {
            frame.extend(
                entry
                    .repeater
                    .to_bytes(entry.has_repeated, i + 1 == self.route.len()),
            );
        }

        frame.extend(self.content.encode());
        frame
    }
}

impl fmt::Display for Ax25Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let info_display = match self.info_string_lossy() {
            Some(ref info) => info.clone(),
            None => "-".to_string(),
        };
        write!(
            f,
            "Source\t\t{}\nDestination\t{}\n\
             Data\t\t\"{}\"",
            self.source, self.destination, info_display
        )
    }
}

fn parse_address(bytes: &[u8]) -> Result<Address, FrameParseError> {
    let mut dest_utf8: Vec<u8> = bytes[0..6]
        .iter()
        .rev()
        .map(|&c| c >> 1)
        .skip_while(|&c| c == b' ')
        .collect::<Vec<u8>>();
    dest_utf8.reverse();
    Ok(Address {
        callsign: String::from_utf8(dest_utf8)
            .map_err(|e| FrameParseError::AddressInvalidUtf8 { source: e })?,
        ssid: (bytes[6] >> 1) & 0x0f,
        c_bit: bytes[6] & 0b1000_0000 > 0,
    })
}

fn parse_i_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    if bytes.len() < 2 {
        return Err(FrameParseError::MissingPidField);
    }
    let c = bytes[0]; // control octet
    Ok(FrameContent::Information(Information {
        receive_sequence: (c & 0b1110_0000) >> 5,
        send_sequence: (c & 0b0000_1110) >> 1,
        poll: (c & 0b0001_0000) > 0,
        pid: ProtocolIdentifier::from_byte(bytes[1]),
        info: bytes[2..].to_vec(), // could be empty vec
    }))
}

fn parse_s_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    // These all have the same general layout
    // There should be no PID or info following this control byte
    let c = bytes[0];
    let n_r = (c & 0b1110_0000) >> 5;
    let poll_or_final = (c & 0b0001_0000) > 0;

    match c & 0b0000_1111 {
        0b0000_0001 => Ok(FrameContent::ReceiveReady(ReceiveReady {
            receive_sequence: n_r,
            poll_or_final,
        })),
        0b0000_0101 => Ok(FrameContent::ReceiveNotReady(ReceiveNotReady {
            receive_sequence: n_r,
            poll_or_final,
        })),
        0b0000_1001 => Ok(FrameContent::Reject(Reject {
            receive_sequence: n_r,
            poll_or_final,
        })),
        0b0000_1101 => Ok(FrameContent::SelectiveReject(SelectiveReject {
            receive_sequence: n_r,
            poll_or_final,
        })),
        _ => Err(FrameParseError::UnrecognisedSFieldType),
    }
}

fn parse_u_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    // The only moving part in control for U frames is the P/F bit
    // Two special cases to handle:
    // FRMR is followed by a 3-byte information field that must be parsed specially
    // UI is followed by PID and variable length information field
    let c = bytes[0];
    let poll_or_final = c & 0b0001_0000 > 0;

    // Ignore the P/F bit for identifying the command or response
    match c & 0b1110_1111 {
        0b0010_1111 => Ok(FrameContent::SetAsynchronousBalancedMode(
            SetAsynchronousBalancedMode {
                poll: poll_or_final,
            },
        )),
        0b0110_1111 => Ok(FrameContent::SetAsynchronousBalancedModeExtended(
            SetAsynchronousBalancedModeExtended {
                poll: poll_or_final,
            },
        )),
        0b0100_0011 => Ok(FrameContent::Disconnect(Disconnect {
            poll: poll_or_final,
        })),
        0b0000_1111 => Ok(FrameContent::DisconnectedMode(DisconnectedMode {
            final_bit: poll_or_final,
        })),
        0b0110_0011 => Ok(FrameContent::UnnumberedAcknowledge(UnnumberedAcknowledge {
            final_bit: poll_or_final,
        })),
        0b1000_0111 => parse_frmr_frame(bytes),
        0b0000_0011 => parse_ui_frame(bytes),
        0b1010_1111 => parse_xid_frame(bytes),
        0b1110_0011 => parse_test_frame(bytes),
        _ => Err(FrameParseError::UnrecognisedUFieldType),
    }
}

fn parse_ui_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    if bytes.len() < 2 {
        return Err(FrameParseError::MissingPidField);
    }
    // Control, then PID, then Info
    Ok(FrameContent::UnnumberedInformation(UnnumberedInformation {
        poll_or_final: bytes[0] & 0b0001_0000 > 0,
        pid: ProtocolIdentifier::from_byte(bytes[1]),
        info: bytes[2..].to_vec(),
    }))
}

const XID_FORMAT_IDENTIFIER: u8 = 0x82;
const XID_GROUP_IDENTIFIER: u8 = 0x80;

const PI_CLASSES_OF_PROCEDURES: u8 = 2;
const PI_HDLC_OPTIONAL_FUNCTIONS: u8 = 3;
const PI_I_FIELD_LENGTH_RX: u8 = 6;
const PI_WINDOW_SIZE_RX: u8 = 8;
const PI_ACK_TIMER: u8 = 9;
const PI_RETRIES: u8 = 10;

const PV_CLASSES_PROCEDURES_BALANCED_ABM: u16 = 0x0100;
const PV_CLASSES_PROCEDURES_HALF_DUPLEX: u16 = 0x2000;
const PV_CLASSES_PROCEDURES_FULL_DUPLEX: u16 = 0x4000;

const PV_HDLC_OPTIONAL_FUNCTIONS_REJ_CMD_RESP: u32 = 0x020000;
const PV_HDLC_OPTIONAL_FUNCTIONS_SREJ_CMD_RESP: u32 = 0x040000;
const PV_HDLC_OPTIONAL_FUNCTIONS_EXTENDED_ADDRESS: u32 = 0x800000;

const PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_8: u32 = 0x000400;
const PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_128: u32 = 0x000800;
const PV_HDLC_OPTIONAL_FUNCTIONS_TEST_CMD_RESP: u32 = 0x002000;
const PV_HDLC_OPTIONAL_FUNCTIONS_16_BIT_FCS: u32 = 0x008000;

const PV_HDLC_OPTIONAL_FUNCTIONS_MULTI_SREJ_CMD_RESP: u32 = 0x000020;

const PV_HDLC_OPTIONAL_FUNCTIONS_SYNCHRONOUS_TX: u32 = 0x000002;

fn parse_xid_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    let mut full_duplex = None;
    let mut srej_mode = None;
    let mut modulo = None;
    let mut i_field_length_rx = None;
    let mut window_size_rx = None;
    let mut ack_timer = None;
    let mut retries = None;

    let info = bytes[1..].to_vec();
    if info.len() < 4 {
        return Err(FrameParseError::Xid);
    }

    let (ids, info) = info.split_at(2);
    if ids[0] != XID_FORMAT_IDENTIFIER {
        return Err(FrameParseError::Xid);
    }
    if ids[1] != XID_GROUP_IDENTIFIER {
        return Err(FrameParseError::Xid);
    }
    let (group_len, mut info) = info.split_at(2);

    let group_len = u16::from_be_bytes(group_len.try_into().map_err(|_| FrameParseError::Xid)?);

    if info.len() != group_len as usize {
        return Err(FrameParseError::Xid);
    }

    while !info.is_empty() {
        let header;

        (header, info) = info.split_at(2);
        let indicator = header[0];
        let value_length = header[1];

        let vbytes;
        (vbytes, info) = info.split_at(value_length as usize);

        let value = match value_length {
            4 => {
                (vbytes[0] as u32) << 24
                    | (vbytes[1] as u32) << 16
                    | (vbytes[2] as u32) << 8
                    | (vbytes[3] as u32)
            }
            3 => (vbytes[0] as u32) << 16 | (vbytes[1] as u32) << 8 | (vbytes[2] as u32),
            2 => (vbytes[0] as u32) << 8 | (vbytes[1] as u32),
            1 => vbytes[0] as u32,
            _ => return Err(FrameParseError::Xid),
        };

        match indicator {
            PI_CLASSES_OF_PROCEDURES => {
                if (value as u16 & PV_CLASSES_PROCEDURES_BALANCED_ABM) == 0 {
                    // TODO warn
                }
                if (value as u16 & PV_CLASSES_PROCEDURES_FULL_DUPLEX) == 0
                    && (value as u16 & PV_CLASSES_PROCEDURES_HALF_DUPLEX) != 0
                {
                    full_duplex = Some(false);
                } else if (value as u16 & PV_CLASSES_PROCEDURES_FULL_DUPLEX) != 0
                    && (value as u16 & PV_CLASSES_PROCEDURES_HALF_DUPLEX) == 0
                {
                    full_duplex = Some(true);
                } else {
                    // TODO warn
                }
            }
            PI_HDLC_OPTIONAL_FUNCTIONS => {
                if value & PV_HDLC_OPTIONAL_FUNCTIONS_MULTI_SREJ_CMD_RESP != 0 {
                    srej_mode = Some(SelectiveRejectMode::Multi);
                } else if value & PV_HDLC_OPTIONAL_FUNCTIONS_SREJ_CMD_RESP != 0 {
                    srej_mode = Some(SelectiveRejectMode::Single);
                } else if value & PV_HDLC_OPTIONAL_FUNCTIONS_REJ_CMD_RESP != 0 {
                    srej_mode = Some(SelectiveRejectMode::None);
                } else {
                    // TODO warn
                }

                if value & PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_8 != 0
                    && value & PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_128 == 0
                {
                    modulo = Some(ModuloMode::Modulo8);
                }
                if value & PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_8 == 0
                    && value & PV_HDLC_OPTIONAL_FUNCTIONS_MODULO_128 != 0
                {
                    modulo = Some(ModuloMode::Modulo128);
                } else {
                    // TODO warn
                }

                if value & PV_HDLC_OPTIONAL_FUNCTIONS_EXTENDED_ADDRESS == 0 {
                    // TODO warn
                }
                if value & PV_HDLC_OPTIONAL_FUNCTIONS_TEST_CMD_RESP == 0 {
                    // TODO warn
                }
                if value & PV_HDLC_OPTIONAL_FUNCTIONS_16_BIT_FCS == 0 {
                    // TODO warn
                }
                if value & PV_HDLC_OPTIONAL_FUNCTIONS_SYNCHRONOUS_TX == 0 {
                    // TODO warn
                }
            }
            PI_I_FIELD_LENGTH_RX => {
                i_field_length_rx = Some(value / 8);
            }
            PI_WINDOW_SIZE_RX => {
                window_size_rx = Some(value);
            }
            PI_ACK_TIMER => {
                ack_timer = Some(value);
            }
            PI_RETRIES => {
                retries = Some(value);
            }
            _ => {
                // TODO warn
            }
        }
    }

    Ok(FrameContent::ExchangeIdentification(
        ExchangeIdentification {
            poll_or_final: bytes[0] & 0b0001_0000 > 0,
            full_duplex,
            srej_mode,
            modulo,
            i_field_length_rx,
            window_size_rx,
            ack_timer,
            retries,
        },
    ))
}

fn parse_test_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    Ok(FrameContent::Test(Test {
        poll_or_final: bytes[0] & 0b0001_0000 > 0,
        info: bytes[2..].to_vec(),
    }))
}

fn parse_frmr_frame(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    // Expect 24 bits following the control
    if bytes.len() != 4 {
        return Err(FrameParseError::WrongSizeFrmrInfo);
    }
    Ok(FrameContent::FrameReject(FrameReject {
        final_bit: bytes[0] & 0b0001_0000 > 0,
        rejected_control_field_raw: bytes[3],
        z: bytes[1] & 0b0000_1000 > 0,
        y: bytes[1] & 0b0000_0100 > 0,
        x: bytes[1] & 0b0000_0010 > 0,
        w: bytes[1] & 0b0000_0001 > 0,
        receive_sequence: (bytes[2] & 0b1110_0000) >> 5,
        command_response: if bytes[2] & 0b0001_0000 > 0 {
            CommandResponse::Response
        } else {
            CommandResponse::Command
        },
        send_sequence: (bytes[2] & 0b0000_1110) >> 1,
    }))
}

/// Parse the content of the frame starting from the control field
fn parse_content(bytes: &[u8]) -> Result<FrameContent, FrameParseError> {
    if bytes.is_empty() {
        return Err(FrameParseError::ContentZeroLength);
    }
    match bytes[0] {
        c if c & 0x01 == 0x00 => parse_i_frame(bytes),
        c if c & 0x03 == 0x01 => parse_s_frame(bytes),
        c if c & 0x03 == 0x03 => parse_u_frame(bytes),
        _ => Ok(FrameContent::UnknownContent(UnknownContent {
            raw: bytes.to_vec(),
        })),
    }
}

#[test]
fn pid_test() {
    assert_eq!(
        ProtocolIdentifier::from_byte(0x01),
        ProtocolIdentifier::X25Plp
    );
    assert_eq!(
        ProtocolIdentifier::from_byte(0xCA),
        ProtocolIdentifier::Appletalk
    );
    assert_eq!(
        ProtocolIdentifier::from_byte(0xFF),
        ProtocolIdentifier::Escape
    );
    assert_eq!(
        ProtocolIdentifier::from_byte(0x45),
        ProtocolIdentifier::Unknown(0x45)
    );
    assert_eq!(
        ProtocolIdentifier::from_byte(0x10),
        ProtocolIdentifier::Layer3Impl
    );
    assert_eq!(
        ProtocolIdentifier::from_byte(0x20),
        ProtocolIdentifier::Layer3Impl
    );
    assert_eq!(
        ProtocolIdentifier::from_byte(0xA5),
        ProtocolIdentifier::Layer3Impl
    );
}

#[test]
fn test_address_fromstr() {
    assert_eq!(
        Address::from_str("VK7NTK-1").unwrap(),
        Address {
            callsign: "VK7NTK".to_string(),
            ssid: 1,
            c_bit: false
        }
    );
    assert_eq!(
        Address::from_str("ID-15").unwrap(),
        Address {
            callsign: "ID".to_string(),
            ssid: 15,
            c_bit: false
        }
    );
    assert!(Address::from_str("vk7ntk-5").is_ok());

    assert!(Address::from_str("-1").is_err());
    assert!(Address::from_str("VK7NTK").is_err());
    assert!(Address::from_str("VK7N -5").is_err());
    assert!(Address::from_str("VK7NTK-16").is_err());
    assert!(Address::from_str("8").is_err());
    assert!(Address::from_str("vk7n--1").is_err());
}

#[test]
fn test_round_trips() {
    use std::fs::{read_dir, File};
    use std::io::Read;

    let mut paths: Vec<_> = read_dir("testdata/linux-ax0")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());
    for entry in paths {
        let entry_path = entry.path();
        println!("Testing round trip on {}", entry_path.display());
        let filename = entry_path.to_str().unwrap();
        let mut file = File::open(filename).unwrap();
        let mut frame_data: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut frame_data);
        // Skip the leading null byte. A quirk as they came from Linux AF_PACKET.
        let frame_data_fixed = &frame_data[1..];

        match Ax25Frame::from_bytes(frame_data_fixed) {
            Ok(parsed) => {
                // Should be identical when re-encoded
                assert_eq!(frame_data_fixed, &parsed.to_bytes()[..])
            }
            Err(e) => panic!("Could not parse! {}", e),
        };
    }
}
