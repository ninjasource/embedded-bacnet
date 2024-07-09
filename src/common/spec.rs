use crate::common::error::Error;

pub const BACNET_MAX_OBJECT: u32 = 0x3FF;
pub const BACNET_INSTANCE_BITS: u32 = 22;
pub const BACNET_MAX_INSTANCE: u32 = 0x3FFFFF;
pub const MAX_BITSTRING_BYTES: u32 = 15;
pub const BACNET_ARRAY_ALL: u32 = 0xFFFFFFFF;
pub const BACNET_NO_PRIORITY: u32 = 0;
pub const BACNET_MIN_PRIORITY: u32 = 1;
pub const BACNET_MAX_PRIORITY: u32 = 16;

/*
TODO: use derive_more when it reaches 1.0 (to automatically impl TryFrom for all enums)
#[derive(Debug, Clone, derive_more::TryFrom)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[try_from(repr)]
#[repr(u32)]
pub enum Segmentation {
    Both = 0,
    Transmit = 1,
    Receive = 2,
    None = 3,
    Max = 4,
}
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum Segmentation {
    Both = 0,
    Transmit = 1,
    Receive = 2,
    None = 3,
    Max = 4,
}

impl TryFrom<u32> for Segmentation {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Both),
            1 => Ok(Self::Transmit),
            2 => Ok(Self::Receive),
            3 => Ok(Self::None),
            4 => Ok(Self::Max),
            _ => Err(Error::InvalidValue("invalid segmentation value")),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum Binary {
    Off = 0,
    On = 1,
}

impl TryFrom<u32> for Binary {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::On),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u16)]
pub enum ErrorClass {
    Device = 0,
    Object = 1,
    Property = 2,
    Resources = 3,
    Security = 4,
    Services = 5,
    Vt = 6,
    Communication = 7,
    // codes 64 and above
    Proprietary(u16),
}

impl TryFrom<u32> for ErrorClass {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Device),
            1 => Ok(Self::Object),
            2 => Ok(Self::Property),
            3 => Ok(Self::Resources),
            4 => Ok(Self::Security),
            5 => Ok(Self::Services),
            6 => Ok(Self::Vt),
            7 => Ok(Self::Communication),
            // codes 64 and above
            x if x > 63 => Ok(Self::Proprietary(x as u16)),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u16)]
pub enum ErrorCode {
    // valid for all classes
    Other = 0,

    // Error Class - Device
    DeviceBusy = 3,
    ConfigurationInProgress = 2,
    OperationalProblem = 25,

    // Error Class - Object
    DynamicCreationNotSupported = 4,
    NoObjectsOfSpecifiedType = 17,
    ObjectDeletionNotPermitted = 23,
    ObjectIdentifierAlreadyExists = 24,
    ReadAccessDenied = 27,
    UnknownObject = 31,
    UnsupportedObjectType = 36,

    // Error Class - Property
    CharacterSetNotSupported = 41,
    DatatypeNotSupported = 47,
    InconsistentSelectionCriterion = 8,
    InvalidArrayIndex = 42,
    InvalidDataType = 9,
    NotCovProperty = 44,
    OptionalFunctionalityNotSupported = 45,
    PropertyIsNotAnArray = 50,
    // READ_ACCESS_DENIED = 27
    UnknownProperty = 32,
    ValueOutOfRange = 37,
    WriteAccessDenied = 40,

    // Error Class - Resources
    NoSpaceForObject = 18,
    NoSpaceToAddListElement = 19,
    NoSpaceToWriteProperty = 20,

    // Error Class - Security
    AuthenticationFailed = 1,
    // CHARACTER_SET_NOT_SUPPORTED = 41
    IncompatibleSecurityLevels = 6,
    InvalidOperatorName = 12,
    KeyGenerationError = 15,
    PasswordFailure = 26,
    SecurityNotSupported = 28,
    Timeout = 30,

    // Error Class - Services
    // CHARACTER_SET_NOT_SUPPORTED = 41
    CovSubscriptionFailed = 43,
    DuplicateName = 48,
    DuplicateObjectId = 49,
    FileAccessDenied = 5,
    InconsistentParameters = 7,
    InvalidConfigurationData = 46,
    InvalidFileAccessMethod = 10,
    InvalidFileStartPosition = 11,
    InvalidParameterDataType = 13,
    InvalidTimeStamp = 14,
    MissingRequiredParameter = 16,
    // OPTIONAL_FUNCTIONALITY_NOT_SUPPORTED = 45
    PropertyIsNotAList = 22,
    ServiceRequestDenied = 29,

    // Error Class - VT
    UnknownVtClass = 34,
    UnknownVtSession = 35,
    NoVtSessionsAvailable = 21,
    VtSessionAlreadyClosed = 38,
    VtSessionTerminationFailure = 39,

    // unused
    Reserved1 = 33,
    // new error codes from new addenda
    AbortBufferOverflow = 51,
    AbortInvalidApduInThisState = 52,
    AbortPreemptedByHigherPriorityTask = 53,
    AbortSegmentationNotSupported = 54,
    AbortProprietary = 55,
    AbortOther = 56,
    InvalidTag = 57,
    NetworkDown = 58,
    RejectBufferOverflow = 59,
    RejectInconsistentParameters = 60,
    RejectInvalidParameterDataType = 61,
    RejectInvalidTag = 62,
    RejectMissingRequiredParameter = 63,
    RejectParameterOutOfRange = 64,
    RejectTooManyArguments = 65,
    RejectUndefinedEnumeration = 66,
    RejectUnrecognizedService = 67,
    RejectProprietary = 68,
    RejectOther = 69,
    UnknownDevice = 70,
    UnknownRoute = 71,
    ValueNotInitialized = 72,
    InvalidEventState = 73,
    NoAlarmConfigured = 74,
    LogBufferFull = 75,
    LoggedValuePurged = 76,
    NoPropertySpecified = 77,
    NotConfiguredForTriggeredLogging = 78,
    UnknownSubscription = 79,
    ParameterOutOfRange = 80,
    ListElementNotFound = 81,
    Busy = 82,
    CommunicationDisabled = 83,
    Success = 84,
    AccessDenied = 85,
    BadDestinationAddress = 86,
    BadDestinationDeviceId = 87,
    BadSignature = 88,
    BadSourceAddress = 89,
    BadTimestamp = 90,
    CannotUseKey = 91,
    CannotVerifyMessageId = 92,
    CorrectKeyRevision = 93,
    DestinationDeviceIdRequired = 94,
    DuplicateMessage = 95,
    EncryptionNotConfigured = 96,
    EncryptionRequired = 97,
    IncorrectKey = 98,
    InvalidKeyData = 99,
    KeyUpdateInProgress = 100,
    MalformedMessage = 101,
    NotKeyServer = 102,
    SecurityNotConfigured = 103,
    SourceSecurityRequired = 104,
    TooManyKeys = 105,
    UnknownAuthenticationType = 106,
    UnknownKey = 107,
    UnknownKeyRevision = 108,
    UnknownSourceMessage = 109,
    NotRouterToDnet = 110,
    RouterBusy = 111,
    UnknownNetworkMessage = 112,
    MessageTooLong = 113,
    SecurityError = 114,
    AddressingError = 115,
    WriteBdtFailed = 116,
    ReadBdtFailed = 117,
    RegisterForeignDeviceFailed = 118,
    ReadFdtFailed = 119,
    DeleteFdtEntryFailed = 120,
    DistributeBroadcastFailed = 121,
    UnknownFileSize = 122,
    AbortApduTooLong = 123,
    AbortApplicationExceededReplyTime = 124,
    AbortOutOfResources = 125,
    AbortTsmTimeout = 126,
    AbortWindowSizeOutOfRange = 127,
    FileFull = 128,
    InconsistentConfiguration = 129,
    InconsistentObjectType = 130,
    InternalError = 131,
    NotConfigured = 132,
    OutOfMemory = 133,
    ValueTooLong = 134,
    AbortInsufficientSecurity = 135,
    AbortSecurityError = 136,
    DuplicateEntry = 137,
    InvalidValueInThisState = 138,
    InvalidOperationInThisState = 139,
    ListItemNotNumbered = 140,
    ListItemNotTimestamped = 141,
    InvalidDataEncoding = 142,
    BvlcFunctionUnknown = 143,
    BvlcProprietaryFunctionUnknown = 144,
    HeaderEncodingError = 145,
    HeaderNotUnderstood = 146,
    MessageIncomplete = 147,
    NotABacnetScHub = 148,
    PayloadExpected = 149,
    UnexpectedData = 150,
    NodeDuplicateVmac = 151,
    HttpUnexpectedResponseCode = 152,
    HttpNoUpgrade = 153,
    HttpResourceNotLocal = 154,
    HttpProxyAuthenticationFailed = 155,
    HttpResponseTimeout = 156,
    HttpResponseSyntaxError = 157,
    HttpResponseValueError = 158,
    HttpResponseMissingHeader = 159,
    HttpWebsocketHeaderError = 160,
    HttpUpgradeRequired = 161,
    HttpUpgradeError = 162,
    HttpTemporaryUnavailable = 163,
    HttpNotAServer = 164,
    HttpError = 165,
    WebsocketSchemeNotSupported = 166,
    WebsocketUnknownControlMessage = 167,
    WebsocketCloseError = 168,
    WebsocketClosedByPeer = 169,
    WebsocketEndpointLeaves = 170,
    WebsocketProtocolError = 171,
    WebsocketDataNotAccepted = 172,
    WebsocketClosedAbnormally = 173,
    WebsocketDataInconsistent = 174,
    WebsocketDataAgainstPolicy = 175,
    WebsocketFrameTooLong = 176,
    WebsocketExtensionMissing = 177,
    WebsocketRequestUnavailable = 178,
    WebsocketError = 179,
    TlsClientCertificateError = 180,
    TlsServerCertificateError = 181,
    TlsClientAuthenticationFailed = 182,
    TlsServerAuthenticationFailed = 183,
    TlsClientCertificateExpired = 184,
    TlsServerCertificateExpired = 185,
    TlsClientCertificateRevoked = 186,
    TlsServerCertificateRevoked = 187,
    TlsError = 188,
    DnsUnavailable = 189,
    DnsNameResolutionFailed = 190,
    DnsResolverFailure = 191,
    DnsError = 192,
    TcpConnectTimeout = 193,
    TcpConnectionRefused = 194,
    TcpClosedByLocal = 195,
    TcpClosedOther = 196,
    TcpError = 197,
    IpAddressNotReachable = 198,
    IpError = 199,
    CertificateExpired = 200,
    CertificateInvalid = 201,
    CertificateMalformed = 202,
    CertificateRevoked = 203,
    UnknownSecurityKey = 204,
    ReferencedPortInError = 205,
    // error codes 256 and above
    Proprietary(u16),
}

impl TryFrom<u32> for ErrorCode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Other),
            3 => Ok(Self::DeviceBusy),
            2 => Ok(Self::ConfigurationInProgress),
            25 => Ok(Self::OperationalProblem),
            4 => Ok(Self::DynamicCreationNotSupported),
            17 => Ok(Self::NoObjectsOfSpecifiedType),
            23 => Ok(Self::ObjectDeletionNotPermitted),
            24 => Ok(Self::ObjectIdentifierAlreadyExists),
            27 => Ok(Self::ReadAccessDenied),
            31 => Ok(Self::UnknownObject),
            36 => Ok(Self::UnsupportedObjectType),
            41 => Ok(Self::CharacterSetNotSupported),
            47 => Ok(Self::DatatypeNotSupported),
            8 => Ok(Self::InconsistentSelectionCriterion),
            42 => Ok(Self::InvalidArrayIndex),
            9 => Ok(Self::InvalidDataType),
            44 => Ok(Self::NotCovProperty),
            45 => Ok(Self::OptionalFunctionalityNotSupported),
            50 => Ok(Self::PropertyIsNotAnArray),
            32 => Ok(Self::UnknownProperty),
            37 => Ok(Self::ValueOutOfRange),
            40 => Ok(Self::WriteAccessDenied),
            18 => Ok(Self::NoSpaceForObject),
            19 => Ok(Self::NoSpaceToAddListElement),
            20 => Ok(Self::NoSpaceToWriteProperty),
            1 => Ok(Self::AuthenticationFailed),
            6 => Ok(Self::IncompatibleSecurityLevels),
            12 => Ok(Self::InvalidOperatorName),
            15 => Ok(Self::KeyGenerationError),
            26 => Ok(Self::PasswordFailure),
            28 => Ok(Self::SecurityNotSupported),
            30 => Ok(Self::Timeout),
            43 => Ok(Self::CovSubscriptionFailed),
            48 => Ok(Self::DuplicateName),
            49 => Ok(Self::DuplicateObjectId),
            5 => Ok(Self::FileAccessDenied),
            7 => Ok(Self::InconsistentParameters),
            46 => Ok(Self::InvalidConfigurationData),
            10 => Ok(Self::InvalidFileAccessMethod),
            11 => Ok(Self::InvalidFileStartPosition),
            13 => Ok(Self::InvalidParameterDataType),
            14 => Ok(Self::InvalidTimeStamp),
            16 => Ok(Self::MissingRequiredParameter),
            22 => Ok(Self::PropertyIsNotAList),
            29 => Ok(Self::ServiceRequestDenied),
            34 => Ok(Self::UnknownVtClass),
            35 => Ok(Self::UnknownVtSession),
            21 => Ok(Self::NoVtSessionsAvailable),
            38 => Ok(Self::VtSessionAlreadyClosed),
            39 => Ok(Self::VtSessionTerminationFailure),
            33 => Ok(Self::Reserved1),
            51 => Ok(Self::AbortBufferOverflow),
            52 => Ok(Self::AbortInvalidApduInThisState),
            53 => Ok(Self::AbortPreemptedByHigherPriorityTask),
            54 => Ok(Self::AbortSegmentationNotSupported),
            55 => Ok(Self::AbortProprietary),
            56 => Ok(Self::AbortOther),
            57 => Ok(Self::InvalidTag),
            58 => Ok(Self::NetworkDown),
            59 => Ok(Self::RejectBufferOverflow),
            60 => Ok(Self::RejectInconsistentParameters),
            61 => Ok(Self::RejectInvalidParameterDataType),
            62 => Ok(Self::RejectInvalidTag),
            63 => Ok(Self::RejectMissingRequiredParameter),
            64 => Ok(Self::RejectParameterOutOfRange),
            65 => Ok(Self::RejectTooManyArguments),
            66 => Ok(Self::RejectUndefinedEnumeration),
            67 => Ok(Self::RejectUnrecognizedService),
            68 => Ok(Self::RejectProprietary),
            69 => Ok(Self::RejectOther),
            70 => Ok(Self::UnknownDevice),
            71 => Ok(Self::UnknownRoute),
            72 => Ok(Self::ValueNotInitialized),
            73 => Ok(Self::InvalidEventState),
            74 => Ok(Self::NoAlarmConfigured),
            75 => Ok(Self::LogBufferFull),
            76 => Ok(Self::LoggedValuePurged),
            77 => Ok(Self::NoPropertySpecified),
            78 => Ok(Self::NotConfiguredForTriggeredLogging),
            79 => Ok(Self::UnknownSubscription),
            80 => Ok(Self::ParameterOutOfRange),
            81 => Ok(Self::ListElementNotFound),
            82 => Ok(Self::Busy),
            83 => Ok(Self::CommunicationDisabled),
            84 => Ok(Self::Success),
            85 => Ok(Self::AccessDenied),
            86 => Ok(Self::BadDestinationAddress),
            87 => Ok(Self::BadDestinationDeviceId),
            88 => Ok(Self::BadSignature),
            89 => Ok(Self::BadSourceAddress),
            90 => Ok(Self::BadTimestamp),
            91 => Ok(Self::CannotUseKey),
            92 => Ok(Self::CannotVerifyMessageId),
            93 => Ok(Self::CorrectKeyRevision),
            94 => Ok(Self::DestinationDeviceIdRequired),
            95 => Ok(Self::DuplicateMessage),
            96 => Ok(Self::EncryptionNotConfigured),
            97 => Ok(Self::EncryptionRequired),
            98 => Ok(Self::IncorrectKey),
            99 => Ok(Self::InvalidKeyData),
            100 => Ok(Self::KeyUpdateInProgress),
            101 => Ok(Self::MalformedMessage),
            102 => Ok(Self::NotKeyServer),
            103 => Ok(Self::SecurityNotConfigured),
            104 => Ok(Self::SourceSecurityRequired),
            105 => Ok(Self::TooManyKeys),
            106 => Ok(Self::UnknownAuthenticationType),
            107 => Ok(Self::UnknownKey),
            108 => Ok(Self::UnknownKeyRevision),
            109 => Ok(Self::UnknownSourceMessage),
            110 => Ok(Self::NotRouterToDnet),
            111 => Ok(Self::RouterBusy),
            112 => Ok(Self::UnknownNetworkMessage),
            113 => Ok(Self::MessageTooLong),
            114 => Ok(Self::SecurityError),
            115 => Ok(Self::AddressingError),
            116 => Ok(Self::WriteBdtFailed),
            117 => Ok(Self::ReadBdtFailed),
            118 => Ok(Self::RegisterForeignDeviceFailed),
            119 => Ok(Self::ReadFdtFailed),
            120 => Ok(Self::DeleteFdtEntryFailed),
            121 => Ok(Self::DistributeBroadcastFailed),
            122 => Ok(Self::UnknownFileSize),
            123 => Ok(Self::AbortApduTooLong),
            124 => Ok(Self::AbortApplicationExceededReplyTime),
            125 => Ok(Self::AbortOutOfResources),
            126 => Ok(Self::AbortTsmTimeout),
            127 => Ok(Self::AbortWindowSizeOutOfRange),
            128 => Ok(Self::FileFull),
            129 => Ok(Self::InconsistentConfiguration),
            130 => Ok(Self::InconsistentObjectType),
            131 => Ok(Self::InternalError),
            132 => Ok(Self::NotConfigured),
            133 => Ok(Self::OutOfMemory),
            134 => Ok(Self::ValueTooLong),
            135 => Ok(Self::AbortInsufficientSecurity),
            136 => Ok(Self::AbortSecurityError),
            137 => Ok(Self::DuplicateEntry),
            138 => Ok(Self::InvalidValueInThisState),
            139 => Ok(Self::InvalidOperationInThisState),
            140 => Ok(Self::ListItemNotNumbered),
            141 => Ok(Self::ListItemNotTimestamped),
            142 => Ok(Self::InvalidDataEncoding),
            143 => Ok(Self::BvlcFunctionUnknown),
            144 => Ok(Self::BvlcProprietaryFunctionUnknown),
            145 => Ok(Self::HeaderEncodingError),
            146 => Ok(Self::HeaderNotUnderstood),
            147 => Ok(Self::MessageIncomplete),
            148 => Ok(Self::NotABacnetScHub),
            149 => Ok(Self::PayloadExpected),
            150 => Ok(Self::UnexpectedData),
            151 => Ok(Self::NodeDuplicateVmac),
            152 => Ok(Self::HttpUnexpectedResponseCode),
            153 => Ok(Self::HttpNoUpgrade),
            154 => Ok(Self::HttpResourceNotLocal),
            155 => Ok(Self::HttpProxyAuthenticationFailed),
            156 => Ok(Self::HttpResponseTimeout),
            157 => Ok(Self::HttpResponseSyntaxError),
            158 => Ok(Self::HttpResponseValueError),
            159 => Ok(Self::HttpResponseMissingHeader),
            160 => Ok(Self::HttpWebsocketHeaderError),
            161 => Ok(Self::HttpUpgradeRequired),
            162 => Ok(Self::HttpUpgradeError),
            163 => Ok(Self::HttpTemporaryUnavailable),
            164 => Ok(Self::HttpNotAServer),
            165 => Ok(Self::HttpError),
            166 => Ok(Self::WebsocketSchemeNotSupported),
            167 => Ok(Self::WebsocketUnknownControlMessage),
            168 => Ok(Self::WebsocketCloseError),
            169 => Ok(Self::WebsocketClosedByPeer),
            170 => Ok(Self::WebsocketEndpointLeaves),
            171 => Ok(Self::WebsocketProtocolError),
            172 => Ok(Self::WebsocketDataNotAccepted),
            173 => Ok(Self::WebsocketClosedAbnormally),
            174 => Ok(Self::WebsocketDataInconsistent),
            175 => Ok(Self::WebsocketDataAgainstPolicy),
            176 => Ok(Self::WebsocketFrameTooLong),
            177 => Ok(Self::WebsocketExtensionMissing),
            178 => Ok(Self::WebsocketRequestUnavailable),
            179 => Ok(Self::WebsocketError),
            180 => Ok(Self::TlsClientCertificateError),
            181 => Ok(Self::TlsServerCertificateError),
            182 => Ok(Self::TlsClientAuthenticationFailed),
            183 => Ok(Self::TlsServerAuthenticationFailed),
            184 => Ok(Self::TlsClientCertificateExpired),
            185 => Ok(Self::TlsServerCertificateExpired),
            186 => Ok(Self::TlsClientCertificateRevoked),
            187 => Ok(Self::TlsServerCertificateRevoked),
            188 => Ok(Self::TlsError),
            189 => Ok(Self::DnsUnavailable),
            190 => Ok(Self::DnsNameResolutionFailed),
            191 => Ok(Self::DnsResolverFailure),
            192 => Ok(Self::DnsError),
            193 => Ok(Self::TcpConnectTimeout),
            194 => Ok(Self::TcpConnectionRefused),
            195 => Ok(Self::TcpClosedByLocal),
            196 => Ok(Self::TcpClosedOther),
            197 => Ok(Self::TcpError),
            198 => Ok(Self::IpAddressNotReachable),
            199 => Ok(Self::IpError),
            200 => Ok(Self::CertificateExpired),
            201 => Ok(Self::CertificateInvalid),
            202 => Ok(Self::CertificateMalformed),
            203 => Ok(Self::CertificateRevoked),
            204 => Ok(Self::UnknownSecurityKey),
            205 => Ok(Self::ReferencedPortInError),
            x if x > 255 => Ok(Self::Proprietary(x as u16)),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u16)]
pub enum EngineeringUnits {
    // Enumerated values 0-255 are reserved for definition by ASHRAE.
    // Acceleration
    MetersPerSecondPerSecond = 166,
    // Area
    SquareMeters = 0,
    SquareCentimeters = 116,
    SquareFeet = 1,
    SquareInches = 115,
    // Currency
    Currency1 = 105,
    Currency2 = 106,
    Currency3 = 107,
    Currency4 = 108,
    Currency5 = 109,
    Currency6 = 110,
    Currency7 = 111,
    Currency8 = 112,
    Currency9 = 113,
    Currency10 = 114,
    // Electrical
    Milliamperes = 2,
    Amperes = 3,
    AmperesPerMeter = 167,
    AmperesPerSquareMeter = 168,
    AmpereSquareMeters = 169,
    Decibels = 199,
    DecibelsMillivolt = 200,
    DecibelsVolt = 201,
    Farads = 170,
    Henrys = 171,
    Ohms = 4,
    OhmMeters = 172,
    Milliohms = 145,
    Kilohms = 122,
    Megohms = 123,
    Microsiemens = 190,
    Millisiemens = 202,
    Siemens = 173, // 1 mho equals 1 siemens
    SiemensPerMeter = 174,
    Teslas = 175,
    Volts = 5,
    Millivolts = 124,
    Kilovolts = 6,
    Megavolts = 7,
    VoltAmperes = 8,
    KilovoltAmperes = 9,
    MegavoltAmperes = 10,
    VoltAmperesReactive = 11,
    KilovoltAmperesReactive = 12,
    MegavoltAmperesReactive = 13,
    VoltsPerDegreeKelvin = 176,
    VoltsPerMeter = 177,
    DegreesPhase = 14,
    PowerFactor = 15,
    Webers = 178,
    // Energy
    Joules = 16,
    Kilojoules = 17,
    KilojoulesPerKilogram = 125,
    Megajoules = 126,
    WattHours = 18,
    KilowattHours = 19,
    MegawattHours = 146,
    WattHoursReactive = 203,
    KilowattHoursReactive = 204,
    MegawattHoursReactive = 205,
    Btus = 20,
    KiloBtus = 147,
    MegaBtus = 148,
    Therms = 21,
    TonHours = 22,
    // Enthalpy
    JoulesPerKilogramDryAir = 23,
    KilojoulesPerKilogramDryAir = 149,
    MegajoulesPerKilogramDryAir = 150,
    BtusPerPoundDryAir = 24,
    BtusPerPound = 117,
    // Entropy
    JoulesPerDegreeKelvin = 127,
    KilojoulesPerDegreeKelvin = 151,
    MegajoulesPerDegreeKelvin = 152,
    JoulesPerKilogramDegreeKelvin = 128,
    // Force
    Newton = 153,
    // Frequency
    CyclesPerHour = 25,
    CyclesPerMinute = 26,
    Hertz = 27,
    Kilohertz = 129,
    Megahertz = 130,
    PerHour = 131,
    // Humidity
    GramsOfWaterPerKilogramDryAir = 28,
    PercentRelativeHumidity = 29,
    // Length
    Micrometers = 194,
    Millimeters = 30,
    Centimeters = 118,
    Kilometers = 193,
    Meters = 31,
    Inches = 32,
    Feet = 33,
    // Light
    Candelas = 179,
    CandelasPerSquareMeter = 180,
    WattsPerSquareFoot = 34,
    WattsPerSquareMeter = 35,
    Lumens = 36,
    Luxes = 37,
    FootCandles = 38,
    // Mass
    Milligrams = 196,
    Grams = 195,
    Kilograms = 39,
    PoundsMass = 40,
    Tons = 41,
    // Mass Flow
    GramsPerSecond = 154,
    GramsPerMinute = 155,
    KilogramsPerSecond = 42,
    KilogramsPerMinute = 43,
    KilogramsPerHour = 44,
    PoundsMassPerSecond = 119,
    PoundsMassPerMinute = 45,
    PoundsMassPerHour = 46,
    TonsPerHour = 156,
    // Power
    Milliwatts = 132,
    Watts = 47,
    Kilowatts = 48,
    Megawatts = 49,
    BtusPerHour = 50,
    KiloBtusPerHour = 157,
    Horsepower = 51,
    TonsRefrigeration = 52,
    // Pressure
    Pascals = 53,
    Hectopascals = 133,
    Kilopascals = 54,
    Millibars = 134,
    Bars = 55,
    PoundsForcePerSquareInch = 56,
    MillimetersOfWater = 206,
    CentimetersOfWater = 57,
    InchesOfWater = 58,
    MillimetersOfMercury = 59,
    CentimetersOfMercury = 60,
    InchesOfMercury = 61,
    // Temperature
    DegreesCelsius = 62,
    DegreesKelvin = 63,
    DegreesKelvinPerHour = 181,
    DegreesKelvinPerMinute = 182,
    DegreesFahrenheit = 64,
    DegreeDaysCelsius = 65,
    DegreeDaysFahrenheit = 66,
    DeltaDegreesFahrenheit = 120,
    DeltaDegreesKelvin = 121,
    // Time
    Years = 67,
    Months = 68,
    Weeks = 69,
    Days = 70,
    Hours = 71,
    Minutes = 72,
    Seconds = 73,
    HundredthsSeconds = 158,
    Milliseconds = 159,
    // Torque
    NewtonMeters = 160,
    // Velocity
    MillimetersPerSecond = 161,
    MillimetersPerMinute = 162,
    MetersPerSecond = 74,
    MetersPerMinute = 163,
    MetersPerHour = 164,
    KilometersPerHour = 75,
    FeetPerSecond = 76,
    FeetPerMinute = 77,
    MilesPerHour = 78,
    // Volume
    CubicFeet = 79,
    CubicMeters = 80,
    ImperialGallons = 81,
    Milliliters = 197,
    Liters = 82,
    UsGallons = 83,
    // Volumetric Flow
    CubicFeetPerSecond = 142,
    CubicFeetPerMinute = 84,
    CubicFeetPerHour = 191,
    CubicMetersPerSecond = 85,
    CubicMetersPerMinute = 165,
    CubicMetersPerHour = 135,
    ImperialGallonsPerMinute = 86,
    MillilitersPerSecond = 198,
    LitersPerSecond = 87,
    LitersPerMinute = 88,
    LitersPerHour = 136,
    UsGallonsPerMinute = 89,
    UsGallonsPerHour = 192,
    // Other
    DegreesAngular = 90,
    DegreesCelsiusPerHour = 91,
    DegreesCelsiusPerMinute = 92,
    DegreesFahrenheitPerHour = 93,
    DegreesFahrenheitPerMinute = 94,
    JouleSeconds = 183,
    KilogramsPerCubicMeter = 186,
    KwHoursPerSquareMeter = 137,
    KwHoursPerSquareFoot = 138,
    MegajoulesPerSquareMeter = 139,
    MegajoulesPerSquareFoot = 140,
    NoUnits = 95,
    NewtonSeconds = 187,
    NewtonsPerMeter = 188,
    PartsPerMillion = 96,
    PartsPerBillion = 97,
    Percent = 98,
    PercentObscurationPerFoot = 143,
    PercentObscurationPerMeter = 144,
    PercentPerSecond = 99,
    PerMinute = 100,
    PerSecond = 101,
    PsiPerDegreeFahrenheit = 102,
    Radians = 103,
    RadiansPerSecond = 184,
    RevolutionsPerMinute = 104,
    SquareMetersPerNewton = 185,
    WattsPerMeterPerDegreeKelvin = 189,
    WattsPerSquareMeterDegreeKelvin = 141,
    PerMille = 207,
    GramsPerGram = 208,
    KilogramsPerKilogram = 209,
    GramsPerKilogram = 210,
    MilligramsPerGram = 211,
    MilligramsPerKilogram = 212,
    GramsPerMilliliter = 213,
    GramsPerLiter = 214,
    MilligramsPerLiter = 215,
    MicrogramsPerLiter = 216,
    GramsPerCubicMeter = 217,
    MilligramsPerCubicMeter = 218,
    MicrogramsPerCubicMeter = 219,
    NanogramsPerCubicMeter = 220,
    GramsPerCubicCentimeter = 221,
    Becquerels = 222,
    Kilobecquerels = 223,
    Megabecquerels = 224,
    Gray = 225,
    Milligray = 226,
    Microgray = 227,
    Sieverts = 228,
    Millisieverts = 229,
    Microsieverts = 230,
    MicrosievertsPerHour = 231,
    DecibelsA = 232,
    NephelometricTurbidityUnit = 233,
    Ph = 234,
    GramsPerSquareMeter = 235,
    MinutesPerDegreeKelvin = 236,
    OhmMeterSquaredPerMeter = 237,
    AmpereSeconds = 238,
    VoltAmpereHours = 239,
    KilovoltAmpereHours = 240,
    MegavoltAmpereHours = 241,
    VoltAmpereHoursReactive = 242,
    KilovoltAmpereHoursReactive = 243,
    MegavoltAmpereHoursReactive = 244,
    VoltSquareHours = 245,
    AmpereSquareHours = 246,
    JoulePerHours = 247,
    CubicFeetPerDay = 248,
    CubicMetersPerDay = 249,
    WattHoursPerCubicMeter = 250,
    JoulesPerCubicMeter = 251,
    MolePercent = 252,
    PascalSeconds = 253,
    MillionStandardCubicFeetPerMinute = 254,
    ReservedRangeMax = 255,
    // Enumerated values 256-47807 may be used by others
    // subject to the procedures and constraints described in Clause 23.
    ProprietaryRangeMin = 256,
    ProprietaryRangeMax = 47807,
    // Enumerated values 47808-49999 are reserved for definition by ASHRAE.
    StandardCubicFeetPerDay = 47808,
    MillionStandardCubicFeetPerDay = 47809,
    ThousandCubicFeetPerDay = 47810,
    ThousandStandardCubicFeetPerDay = 47811,
    PoundsMassPerDay = 47812,
    // 47813 - NOT USED
    Millirems = 47814,
    MilliremsPerHour = 47815,
    ReservedRangeMax2 = 49999,
    ProprietaryRangeMin2 = 50000,
    // Enumerated values 50000-65535 may be used by others
    // subject to the procedures and constraints described in Clause 23.
    // do the proprietary range inside of enum so that
    // compilers will allocate adequate sized datatype for enum
    // which is used to store decoding
    ProprietaryRangeMax2 = 65535,
}

impl EngineeringUnits {
    /// returns a string representation of the most common engineering units
    /// NOTE: Make sure the font can render the following unicode characters: ²³⁻¹Ω°Δ
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MetersPerSecondPerSecond => "m/s²",
            Self::SquareMeters => "m²",
            Self::SquareCentimeters => "cm²",
            Self::SquareFeet => "ft²",
            Self::SquareInches => "in²",
            Self::Currency1 => "$",
            Self::Milliamperes => "mA",
            Self::Amperes => "A",
            Self::AmperesPerMeter => "A/m",
            Self::AmperesPerSquareMeter => "A/m²",
            Self::AmpereSquareMeters => "Am²",
            Self::Farads => "F",
            Self::Henrys => "H",
            Self::Ohms => "Ω",
            Self::OhmMeters => "Ωm",
            Self::Milliohms => "mΩ",
            Self::Kilohms => "kΩ",
            Self::Megohms => "MΩ",
            Self::Siemens => "S",
            Self::SiemensPerMeter => "S/m",
            Self::Teslas => "T",
            Self::Volts => "V",
            Self::Millivolts => "mV",
            Self::Kilovolts => "kV",
            Self::Megavolts => "MV",
            Self::VoltAmperes => "VA",
            Self::KilovoltAmperes => "kVA",
            Self::MegavoltAmperes => "MVA",
            Self::VoltAmperesReactive => "var",
            Self::KilovoltAmperesReactive => "kvar",
            Self::MegavoltAmperesReactive => "Mvar",
            Self::DegreesPhase => "°P",
            Self::PowerFactor => "pf",
            Self::Webers => "Wb",
            Self::Joules => "J",
            Self::Kilojoules => "kJ",
            Self::KilojoulesPerKilogram => "kJ/kg",
            Self::Megajoules => "MJ",
            Self::WattHours => "Wh",
            Self::KilowattHours => "kWh",
            Self::MegawattHours => "MWh",
            Self::Btus => "btu",
            Self::KiloBtus => "kbtu",
            Self::MegaBtus => "Mbtu",
            Self::Therms => "thrm",
            Self::TonHours => "th",
            Self::JoulesPerKilogramDryAir => "J/kg",
            Self::KilojoulesPerKilogramDryAir => "kJ/kg",
            Self::MegajoulesPerKilogramDryAir => "MJ/kg",
            Self::BtusPerPoundDryAir => "b/lb",
            Self::BtusPerPound => "b/lb",
            Self::JoulesPerDegreeKelvin => "J/K",
            Self::KilojoulesPerDegreeKelvin => "kJ/K",
            Self::MegajoulesPerDegreeKelvin => "MJ/K",
            Self::Newton => "N",
            Self::CyclesPerHour => "c/h",
            Self::CyclesPerMinute => "c/m",
            Self::Hertz => "Hz",
            Self::Kilohertz => "kHz",
            Self::Megahertz => "MHz",
            Self::PerHour => "h⁻¹",
            Self::GramsOfWaterPerKilogramDryAir => "g/kg",
            Self::PercentRelativeHumidity => "%rh",
            Self::Milliliters => "mm",
            Self::Centimeters => "cm",
            Self::Meters => "m",
            Self::Inches => "inch",
            Self::Feet => "feet",
            Self::Candelas => "cd",
            Self::CandelasPerSquareMeter => "cd/m²",
            Self::WattsPerSquareFoot => "W/f²",
            Self::WattsPerSquareMeter => "W/m²",
            Self::Lumens => "lum",
            Self::Luxes => "lux",
            Self::FootCandles => "ftcd",
            Self::Kilograms => "kg",
            Self::PoundsMass => "lb",
            Self::Tons => "t",
            Self::GramsPerSecond => "g/s",
            Self::GramsPerMinute => "g/m",
            Self::KilogramsPerSecond => "kg/s",
            Self::KilogramsPerMinute => "kg/m",
            Self::KilogramsPerHour => "kg/h",
            Self::PoundsMassPerSecond => "lb/s",
            Self::PoundsMassPerMinute => "lb/m",
            Self::PoundsMassPerHour => "lb/h",
            Self::TonsPerHour => "t/h",
            Self::Milliwatts => "mW",
            Self::Watts => "W",
            Self::Kilowatts => "kW",
            Self::Megawatts => "MW",
            Self::BtusPerHour => "bt/h",
            Self::Horsepower => "hp",
            Self::TonsRefrigeration => "tr",
            Self::Pascals => "Pa",
            Self::Hectopascals => "hPa",
            Self::Kilopascals => "kPa",
            Self::Millibars => "mBar",
            Self::Bars => "Bar",
            Self::PoundsForcePerSquareInch => "psi",
            Self::CentimetersOfWater => "cmw",
            Self::InchesOfWater => "inwc",
            Self::MillimetersOfMercury => "mmHg",
            Self::CentimetersOfMercury => "cmHg",
            Self::InchesOfMercury => "inHg",
            Self::DegreesCelsius => "°C",
            Self::DegreesKelvin => "K",
            Self::DegreesKelvinPerHour => "K/h",
            Self::DegreesKelvinPerMinute => "K/m",
            Self::DegreesFahrenheit => "°F",
            Self::DegreeDaysCelsius => "°dyC",
            Self::DegreeDaysFahrenheit => "°dyF",
            Self::DeltaDegreesFahrenheit => "Δ°F",
            Self::Years => "year",
            Self::Months => "mnth",
            Self::Weeks => "week",
            Self::Days => "day",
            Self::Hours => "hour",
            Self::Minutes => "min",
            Self::Seconds => "sec",
            Self::HundredthsSeconds => "s⁻²",
            Self::Milliseconds => "ms",
            Self::NewtonMeters => "Nm",
            Self::MillimetersPerSecond => "mm/s",
            Self::MillimetersPerMinute => "mm/m",
            Self::MetersPerSecond => "m/s",
            Self::MetersPerMinute => "m/m",
            Self::MetersPerHour => "m/h",
            Self::KilometersPerHour => "km/h",
            Self::FeetPerSecond => "ft/s",
            Self::FeetPerMinute => "fpm",
            Self::MilesPerHour => "mph",
            Self::CubicFeet => "ft³",
            Self::CubicMeters => "m³",
            Self::ImperialGallons => "gall",
            Self::Liters => "l",
            Self::UsGallons => "USg",
            Self::CubicFeetPerSecond => "cfs",
            Self::CubicFeetPerMinute => "cfm",
            Self::CubicFeetPerHour => "cfh",
            Self::CubicMetersPerSecond => "m³/s",
            Self::CubicMetersPerMinute => "m³/m",
            Self::CubicMetersPerHour => "m³/h",
            Self::ImperialGallonsPerMinute => "igpm",
            Self::LitersPerSecond => "l/s",
            Self::LitersPerMinute => "l/m",
            Self::LitersPerHour => "l/h",
            Self::UsGallonsPerMinute => "gpm",
            Self::DegreesAngular => "deg",
            Self::DegreesCelsiusPerHour => "°C/h",
            Self::DegreesCelsiusPerMinute => "°C/m",
            Self::DegreesFahrenheitPerHour => "°F/h",
            Self::DegreesFahrenheitPerMinute => "°F/m",
            Self::JouleSeconds => "Js",
            Self::KilogramsPerCubicMeter => "kg/m³",
            Self::KwHoursPerSquareMeter => "kWhm",
            Self::KwHoursPerSquareFoot => "kWhf",
            Self::NoUnits => "",
            Self::NewtonSeconds => "Ns",
            Self::NewtonsPerMeter => "N/m",
            Self::PartsPerMillion => "ppm",
            Self::PartsPerBillion => "ppb",
            Self::Percent => "%",
            Self::PercentObscurationPerFoot => "%/ft",
            Self::PercentObscurationPerMeter => "%/m",
            Self::PercentPerSecond => "%/s",
            Self::PerMinute => "pm",
            Self::PerSecond => "ps",
            Self::PsiPerDegreeFahrenheit => "psiF",
            Self::Radians => "rad",
            Self::RadiansPerSecond => "rd/s",
            Self::RevolutionsPerMinute => "rpm",
            Self::SquareMetersPerNewton => "m²/N",
            Self::WattsPerMeterPerDegreeKelvin => "WmK",
            Self::WattsPerSquareMeterDegreeKelvin => "Wm²K",
            _x => {
                // unhandled units
                ""
            }
        }
    }
}

impl TryFrom<u32> for EngineeringUnits {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            166 => Ok(Self::MetersPerSecondPerSecond),
            0 => Ok(Self::SquareMeters),
            116 => Ok(Self::SquareCentimeters),
            1 => Ok(Self::SquareFeet),
            115 => Ok(Self::SquareInches),
            105 => Ok(Self::Currency1),
            106 => Ok(Self::Currency2),
            107 => Ok(Self::Currency3),
            108 => Ok(Self::Currency4),
            109 => Ok(Self::Currency5),
            110 => Ok(Self::Currency6),
            111 => Ok(Self::Currency7),
            112 => Ok(Self::Currency8),
            113 => Ok(Self::Currency9),
            114 => Ok(Self::Currency10),
            2 => Ok(Self::Milliamperes),
            3 => Ok(Self::Amperes),
            167 => Ok(Self::AmperesPerMeter),
            168 => Ok(Self::AmperesPerSquareMeter),
            169 => Ok(Self::AmpereSquareMeters),
            199 => Ok(Self::Decibels),
            200 => Ok(Self::DecibelsMillivolt),
            201 => Ok(Self::DecibelsVolt),
            170 => Ok(Self::Farads),
            171 => Ok(Self::Henrys),
            4 => Ok(Self::Ohms),
            172 => Ok(Self::OhmMeters),
            145 => Ok(Self::Milliohms),
            122 => Ok(Self::Kilohms),
            123 => Ok(Self::Megohms),
            190 => Ok(Self::Microsiemens),
            202 => Ok(Self::Millisiemens),
            173 => Ok(Self::Siemens),
            174 => Ok(Self::SiemensPerMeter),
            175 => Ok(Self::Teslas),
            5 => Ok(Self::Volts),
            124 => Ok(Self::Millivolts),
            6 => Ok(Self::Kilovolts),
            7 => Ok(Self::Megavolts),
            8 => Ok(Self::VoltAmperes),
            9 => Ok(Self::KilovoltAmperes),
            10 => Ok(Self::MegavoltAmperes),
            11 => Ok(Self::VoltAmperesReactive),
            12 => Ok(Self::KilovoltAmperesReactive),
            13 => Ok(Self::MegavoltAmperesReactive),
            176 => Ok(Self::VoltsPerDegreeKelvin),
            177 => Ok(Self::VoltsPerMeter),
            14 => Ok(Self::DegreesPhase),
            15 => Ok(Self::PowerFactor),
            178 => Ok(Self::Webers),
            16 => Ok(Self::Joules),
            17 => Ok(Self::Kilojoules),
            125 => Ok(Self::KilojoulesPerKilogram),
            126 => Ok(Self::Megajoules),
            18 => Ok(Self::WattHours),
            19 => Ok(Self::KilowattHours),
            146 => Ok(Self::MegawattHours),
            203 => Ok(Self::WattHoursReactive),
            204 => Ok(Self::KilowattHoursReactive),
            205 => Ok(Self::MegawattHoursReactive),
            20 => Ok(Self::Btus),
            147 => Ok(Self::KiloBtus),
            148 => Ok(Self::MegaBtus),
            21 => Ok(Self::Therms),
            22 => Ok(Self::TonHours),
            23 => Ok(Self::JoulesPerKilogramDryAir),
            149 => Ok(Self::KilojoulesPerKilogramDryAir),
            150 => Ok(Self::MegajoulesPerKilogramDryAir),
            24 => Ok(Self::BtusPerPoundDryAir),
            117 => Ok(Self::BtusPerPound),
            127 => Ok(Self::JoulesPerDegreeKelvin),
            151 => Ok(Self::KilojoulesPerDegreeKelvin),
            152 => Ok(Self::MegajoulesPerDegreeKelvin),
            128 => Ok(Self::JoulesPerKilogramDegreeKelvin),
            153 => Ok(Self::Newton),
            25 => Ok(Self::CyclesPerHour),
            26 => Ok(Self::CyclesPerMinute),
            27 => Ok(Self::Hertz),
            129 => Ok(Self::Kilohertz),
            130 => Ok(Self::Megahertz),
            131 => Ok(Self::PerHour),
            28 => Ok(Self::GramsOfWaterPerKilogramDryAir),
            29 => Ok(Self::PercentRelativeHumidity),
            194 => Ok(Self::Micrometers),
            30 => Ok(Self::Millimeters),
            118 => Ok(Self::Centimeters),
            193 => Ok(Self::Kilometers),
            31 => Ok(Self::Meters),
            32 => Ok(Self::Inches),
            33 => Ok(Self::Feet),
            179 => Ok(Self::Candelas),
            180 => Ok(Self::CandelasPerSquareMeter),
            34 => Ok(Self::WattsPerSquareFoot),
            35 => Ok(Self::WattsPerSquareMeter),
            36 => Ok(Self::Lumens),
            37 => Ok(Self::Luxes),
            38 => Ok(Self::FootCandles),
            196 => Ok(Self::Milligrams),
            195 => Ok(Self::Grams),
            39 => Ok(Self::Kilograms),
            40 => Ok(Self::PoundsMass),
            41 => Ok(Self::Tons),
            154 => Ok(Self::GramsPerSecond),
            155 => Ok(Self::GramsPerMinute),
            42 => Ok(Self::KilogramsPerSecond),
            43 => Ok(Self::KilogramsPerMinute),
            44 => Ok(Self::KilogramsPerHour),
            119 => Ok(Self::PoundsMassPerSecond),
            45 => Ok(Self::PoundsMassPerMinute),
            46 => Ok(Self::PoundsMassPerHour),
            156 => Ok(Self::TonsPerHour),
            132 => Ok(Self::Milliwatts),
            47 => Ok(Self::Watts),
            48 => Ok(Self::Kilowatts),
            49 => Ok(Self::Megawatts),
            50 => Ok(Self::BtusPerHour),
            157 => Ok(Self::KiloBtusPerHour),
            51 => Ok(Self::Horsepower),
            52 => Ok(Self::TonsRefrigeration),
            53 => Ok(Self::Pascals),
            133 => Ok(Self::Hectopascals),
            54 => Ok(Self::Kilopascals),
            134 => Ok(Self::Millibars),
            55 => Ok(Self::Bars),
            56 => Ok(Self::PoundsForcePerSquareInch),
            206 => Ok(Self::MillimetersOfWater),
            57 => Ok(Self::CentimetersOfWater),
            58 => Ok(Self::InchesOfWater),
            59 => Ok(Self::MillimetersOfMercury),
            60 => Ok(Self::CentimetersOfMercury),
            61 => Ok(Self::InchesOfMercury),
            62 => Ok(Self::DegreesCelsius),
            63 => Ok(Self::DegreesKelvin),
            181 => Ok(Self::DegreesKelvinPerHour),
            182 => Ok(Self::DegreesKelvinPerMinute),
            64 => Ok(Self::DegreesFahrenheit),
            65 => Ok(Self::DegreeDaysCelsius),
            66 => Ok(Self::DegreeDaysFahrenheit),
            120 => Ok(Self::DeltaDegreesFahrenheit),
            121 => Ok(Self::DeltaDegreesKelvin),
            67 => Ok(Self::Years),
            68 => Ok(Self::Months),
            69 => Ok(Self::Weeks),
            70 => Ok(Self::Days),
            71 => Ok(Self::Hours),
            72 => Ok(Self::Minutes),
            73 => Ok(Self::Seconds),
            158 => Ok(Self::HundredthsSeconds),
            159 => Ok(Self::Milliseconds),
            160 => Ok(Self::NewtonMeters),
            161 => Ok(Self::MillimetersPerSecond),
            162 => Ok(Self::MillimetersPerMinute),
            74 => Ok(Self::MetersPerSecond),
            163 => Ok(Self::MetersPerMinute),
            164 => Ok(Self::MetersPerHour),
            75 => Ok(Self::KilometersPerHour),
            76 => Ok(Self::FeetPerSecond),
            77 => Ok(Self::FeetPerMinute),
            78 => Ok(Self::MilesPerHour),
            79 => Ok(Self::CubicFeet),
            80 => Ok(Self::CubicMeters),
            81 => Ok(Self::ImperialGallons),
            197 => Ok(Self::Milliliters),
            82 => Ok(Self::Liters),
            83 => Ok(Self::UsGallons),
            142 => Ok(Self::CubicFeetPerSecond),
            84 => Ok(Self::CubicFeetPerMinute),
            191 => Ok(Self::CubicFeetPerHour),
            85 => Ok(Self::CubicMetersPerSecond),
            165 => Ok(Self::CubicMetersPerMinute),
            135 => Ok(Self::CubicMetersPerHour),
            86 => Ok(Self::ImperialGallonsPerMinute),
            198 => Ok(Self::MillilitersPerSecond),
            87 => Ok(Self::LitersPerSecond),
            88 => Ok(Self::LitersPerMinute),
            136 => Ok(Self::LitersPerHour),
            89 => Ok(Self::UsGallonsPerMinute),
            192 => Ok(Self::UsGallonsPerHour),
            90 => Ok(Self::DegreesAngular),
            91 => Ok(Self::DegreesCelsiusPerHour),
            92 => Ok(Self::DegreesCelsiusPerMinute),
            93 => Ok(Self::DegreesFahrenheitPerHour),
            94 => Ok(Self::DegreesFahrenheitPerMinute),
            183 => Ok(Self::JouleSeconds),
            186 => Ok(Self::KilogramsPerCubicMeter),
            137 => Ok(Self::KwHoursPerSquareMeter),
            138 => Ok(Self::KwHoursPerSquareFoot),
            139 => Ok(Self::MegajoulesPerSquareMeter),
            140 => Ok(Self::MegajoulesPerSquareFoot),
            95 => Ok(Self::NoUnits),
            187 => Ok(Self::NewtonSeconds),
            188 => Ok(Self::NewtonsPerMeter),
            96 => Ok(Self::PartsPerMillion),
            97 => Ok(Self::PartsPerBillion),
            98 => Ok(Self::Percent),
            143 => Ok(Self::PercentObscurationPerFoot),
            144 => Ok(Self::PercentObscurationPerMeter),
            99 => Ok(Self::PercentPerSecond),
            100 => Ok(Self::PerMinute),
            101 => Ok(Self::PerSecond),
            102 => Ok(Self::PsiPerDegreeFahrenheit),
            103 => Ok(Self::Radians),
            184 => Ok(Self::RadiansPerSecond),
            104 => Ok(Self::RevolutionsPerMinute),
            185 => Ok(Self::SquareMetersPerNewton),
            189 => Ok(Self::WattsPerMeterPerDegreeKelvin),
            141 => Ok(Self::WattsPerSquareMeterDegreeKelvin),
            207 => Ok(Self::PerMille),
            208 => Ok(Self::GramsPerGram),
            209 => Ok(Self::KilogramsPerKilogram),
            210 => Ok(Self::GramsPerKilogram),
            211 => Ok(Self::MilligramsPerGram),
            212 => Ok(Self::MilligramsPerKilogram),
            213 => Ok(Self::GramsPerMilliliter),
            214 => Ok(Self::GramsPerLiter),
            215 => Ok(Self::MilligramsPerLiter),
            216 => Ok(Self::MicrogramsPerLiter),
            217 => Ok(Self::GramsPerCubicMeter),
            218 => Ok(Self::MilligramsPerCubicMeter),
            219 => Ok(Self::MicrogramsPerCubicMeter),
            220 => Ok(Self::NanogramsPerCubicMeter),
            221 => Ok(Self::GramsPerCubicCentimeter),
            222 => Ok(Self::Becquerels),
            223 => Ok(Self::Kilobecquerels),
            224 => Ok(Self::Megabecquerels),
            225 => Ok(Self::Gray),
            226 => Ok(Self::Milligray),
            227 => Ok(Self::Microgray),
            228 => Ok(Self::Sieverts),
            229 => Ok(Self::Millisieverts),
            230 => Ok(Self::Microsieverts),
            231 => Ok(Self::MicrosievertsPerHour),
            232 => Ok(Self::DecibelsA),
            233 => Ok(Self::NephelometricTurbidityUnit),
            234 => Ok(Self::Ph),
            235 => Ok(Self::GramsPerSquareMeter),
            236 => Ok(Self::MinutesPerDegreeKelvin),
            237 => Ok(Self::OhmMeterSquaredPerMeter),
            238 => Ok(Self::AmpereSeconds),
            239 => Ok(Self::VoltAmpereHours),
            240 => Ok(Self::KilovoltAmpereHours),
            241 => Ok(Self::MegavoltAmpereHours),
            242 => Ok(Self::VoltAmpereHoursReactive),
            243 => Ok(Self::KilovoltAmpereHoursReactive),
            244 => Ok(Self::MegavoltAmpereHoursReactive),
            245 => Ok(Self::VoltSquareHours),
            246 => Ok(Self::AmpereSquareHours),
            247 => Ok(Self::JoulePerHours),
            248 => Ok(Self::CubicFeetPerDay),
            249 => Ok(Self::CubicMetersPerDay),
            250 => Ok(Self::WattHoursPerCubicMeter),
            251 => Ok(Self::JoulesPerCubicMeter),
            252 => Ok(Self::MolePercent),
            253 => Ok(Self::PascalSeconds),
            254 => Ok(Self::MillionStandardCubicFeetPerMinute),
            255 => Ok(Self::ReservedRangeMax),
            256 => Ok(Self::ProprietaryRangeMin),
            47807 => Ok(Self::ProprietaryRangeMax),
            47808 => Ok(Self::StandardCubicFeetPerDay),
            47809 => Ok(Self::MillionStandardCubicFeetPerDay),
            47810 => Ok(Self::ThousandCubicFeetPerDay),
            47811 => Ok(Self::ThousandStandardCubicFeetPerDay),
            47812 => Ok(Self::PoundsMassPerDay),
            47814 => Ok(Self::Millirems),
            47815 => Ok(Self::MilliremsPerHour),
            49999 => Ok(Self::ReservedRangeMax2),
            50000 => Ok(Self::ProprietaryRangeMin2),
            65535 => Ok(Self::ProprietaryRangeMax2),
            _ => Err(value),
        }
    }
}

#[repr(u8)]
pub enum LogBufferResultFlags {
    FirstItem = 0b1000_0000,
    LastItem = 0b0100_0000,
    MoreItems = 0b0010_0000,
}

#[repr(u8)]
pub enum StatusFlags {
    InAlarm = 0b1000_0000,
    Fault = 0b0100_0000,
    Overridden = 0b0010_0000,
    OutOfService = 0b0001_0000,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Status {
    pub inner: u8,
}

impl Status {
    pub fn new(inner: u8) -> Self {
        Self { inner }
    }

    pub const fn in_alarm(&self) -> bool {
        self.inner & StatusFlags::InAlarm as u8 == StatusFlags::InAlarm as u8
    }

    pub const fn fault(&self) -> bool {
        self.inner & StatusFlags::Fault as u8 == StatusFlags::Fault as u8
    }

    pub const fn overridden(&self) -> bool {
        self.inner & StatusFlags::Overridden as u8 == StatusFlags::Overridden as u8
    }

    pub const fn out_of_service(&self) -> bool {
        self.inner & StatusFlags::OutOfService as u8 == StatusFlags::OutOfService as u8
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogBufferResult {
    pub inner: u8,
}

impl LogBufferResult {
    pub fn new(inner: u8) -> Self {
        Self { inner }
    }

    pub const fn first_item(&self) -> bool {
        self.inner & LogBufferResultFlags::FirstItem as u8 == LogBufferResultFlags::FirstItem as u8
    }

    pub const fn last_item(&self) -> bool {
        self.inner & LogBufferResultFlags::LastItem as u8 == LogBufferResultFlags::LastItem as u8
    }

    pub const fn more_items(&self) -> bool {
        self.inner & LogBufferResultFlags::MoreItems as u8 == LogBufferResultFlags::MoreItems as u8
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogStatus {
    LogDisabled = 0,
    BufferPurged = 1,
    LogInterrupted = 2,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AcknowledgmentFilter {
    All = 0,
    Acked = 1,
    NotAcked = 2,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum EventState {
    Normal = 0,
    Fault = 1,
    OffNormal = 2,
    HighLimit = 3,
    LowLimit = 4,
}

impl TryFrom<u32> for EventState {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Fault),
            2 => Ok(Self::OffNormal),
            3 => Ok(Self::HighLimit),
            4 => Ok(Self::LowLimit),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum NotifyType {
    Alarm = 0,
    Event = 1,
    AckNotification = 2,
}

impl TryFrom<u32> for NotifyType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Alarm),
            1 => Ok(Self::Event),
            2 => Ok(Self::AckNotification),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum LoggingType {
    Polled = 0,
    Cov = 1,
    Triggered = 2,
}

impl TryFrom<u32> for LoggingType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Polled),
            1 => Ok(Self::Cov),
            2 => Ok(Self::Triggered),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectionLogic {
    And = 0,
    Or = 1,
    All = 2,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RelationSpecifier {
    Equal = 0,
    NotEqual = 1,
    LessThan = 2,
    GreaterThan = 3,
    LessThanOrEqual = 4,
    GreaterThanOrEqual = 5,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommunicationEnableDisable {
    Enable = 0,
    Disable = 1,
    DisableInitiation = 2,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessagePriority {
    Normal = 0,
    Urgent = 1,
    CriticalEquipment = 2,
    LifeSafety = 3,
}

// end of bit string enumerations
