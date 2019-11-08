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
pub struct SipHeader(Vec<String>);

impl SipHeader {
    fn add_header(&mut self, header: String) {
        self.0.push(header)
    }
}

#[derive(Debug)]
pub struct Sdp(Vec<String>);

#[derive(Debug)]
enum SipParseState {
    Idle,
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
        for line in trace.lines() {
            let state = match self.state {
                Idle => {
                    if line.contains("SIP/2.0") {
                        SipParse(SipHeader(vec![line.to_owned()]))
                    } else {
                        Idle
                    }
                }

                SipParse(mut h) => {
                    if line.is_empty() {
                        // Here we should have a full SipPacket
                        // Lets see if it matches the search terms
                        // If not we go back to Idle state
                        if h.0
                            .iter()
                            .any(|h| term.iter().any(|t| h.contains(t)))
                        {
                            SdpParse(h, Sdp(Vec::new()))
                        } else {
                            Idle
                        }
                    } else {
                        h.0.push(line.to_owned());
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
