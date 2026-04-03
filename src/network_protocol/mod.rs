pub mod ip;
pub mod network_pdu;

pub mod data_link {
    #[deprecated(note = "use IpFrame")]
    pub type DataLink<'a> = super::ip::IpFrame<'a>;

    #[deprecated(note = "use IpDataLinkFunction")]
    pub type DataLinkFunction = super::ip::BvllFunction;
}
