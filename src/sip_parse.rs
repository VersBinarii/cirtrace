const SDP_ATTRIBUTES: &[&str] = &[
    "v=", "o=", "s=", "i=", "u=", "e=", "p=", "c=", "b=", "z=", "k=", "a=",
    "t=", "r=", "m=",
];

#[derive(Debug)]
pub struct SipPacket {
    header: SipHeader,
    sdp: Option<Sdp>,
}

impl std::fmt::Display for SipPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for h in self.header.0.iter() {
            let _ = write!(f, "{}\n", h);
        }

        if let Some(sdp) = &self.sdp {
            let _ = write!(f, "\n");
            for s in sdp.0.iter() {
                let _ = write!(f, "{}\n", s);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct SipHeader(Vec<String>);

impl SipHeader {
    fn add_header(&mut self, header: String) {
        self.0.push(header)
    }

    fn get_call_id(&self) -> Option<&String> {
        self.0.iter().find(|&h| h.starts_with("Call-ID"))
    }
}

#[derive(Debug)]
struct Sdp(Vec<String>);

#[derive(Debug)]
enum SipParseState {
    Idle,
    InviteSipParse(SipHeader),
    SipParse(SipHeader),
    SdpParse(SipHeader, Sdp),
    Done(SipPacket),
}

#[derive(Debug)]
pub struct SipParser {
    state: SipParseState,
}

impl SipParser {
    pub fn new() -> Self {
        Self {
            state: SipParseState::Idle,
        }
    }
    pub fn extract_sip(
        mut self,
        trace: &str,
        term: &[&str],
        _with_sdp: bool,
    ) -> Vec<SipPacket> {
        use SipParseState::*;

        println!("Searching for {:?} terms in SIP packets", term);
        let mut packets = Vec::new();
        let mut current_call_id: Option<String> = None;

        for line in trace.lines() {
            let state = match self.state {
                Idle => {
                    if line.contains("SIP/2.0") {
                        if line.starts_with("INVITE") {
                            InviteSipParse(SipHeader(vec![line.to_owned()]))
                        } else {
                            SipParse(SipHeader(vec![line.to_owned()]))
                        }
                    } else {
                        Idle
                    }
                }
                InviteSipParse(mut h) => {
                    if line.is_empty() {
                        // Here we should have a full INVITE SipPacket
                        // Lets see if it matches the search terms
                        // If not we go back to Idle state
                        if h.0
                            .iter()
                            .any(|h| term.iter().any(|t| h.contains(t)))
                        {
                            current_call_id = h.get_call_id().cloned();
                            SdpParse(h, Sdp(Vec::new()))
                        } else {
                            Idle
                        }
                    } else {
                        h.add_header(line.to_owned());
                        InviteSipParse(h)
                    }
                }
                SipParse(mut h) => {
                    if line.is_empty() {
                        // If the call-id corressponds to
                        // what we're currently looking for
                        // then continue otherwise lets skip this one
                        // and look for new packet
                        if current_call_id.as_ref() == h.get_call_id() {
                            SdpParse(h, Sdp(Vec::new()))
                        } else {
                            Idle
                        }
                    } else {
                        h.add_header(line.to_owned());
                        SipParse(h)
                    }
                }
                SdpParse(h, mut s) => {
                    if line.is_empty() {
                        Done(SipPacket {
                            header: h,
                            sdp: if s.0.len() > 0 { Some(s) } else { None },
                        })
                    } else if SDP_ATTRIBUTES.iter().any(|a| line.starts_with(a))
                    {
                        s.0.push(line.to_owned());
                        SdpParse(h, s)
                    } else {
                        Idle
                    }
                }
                Done(p) => {
                    packets.push(p);
                    Idle
                }
            };

            self.state = state;
        }
        packets
    }
}
