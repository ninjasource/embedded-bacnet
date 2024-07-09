use crate::common::{
    error::Error,
    helper::decode_unsigned,
    io::{Reader, Writer},
    spec::{BACNET_INSTANCE_BITS, BACNET_MAX_INSTANCE, BACNET_MAX_OBJECT},
};

// NOTE: Copy is derived for usage convenience
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectId {
    pub object_type: ObjectType,
    pub id: u32,
}

impl ObjectId {
    pub const LEN: u32 = 4; // 4 bytes

    pub fn new(object_type: ObjectType, id: u32) -> Self {
        Self { object_type, id }
    }

    pub fn encode(&self, writer: &mut Writer) {
        let value = ((self.object_type as u32 & BACNET_MAX_OBJECT) << BACNET_INSTANCE_BITS)
            | (self.id & BACNET_MAX_INSTANCE);
        writer.extend_from_slice(&value.to_be_bytes());
    }

    pub fn decode(size: u32, reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let value = decode_unsigned(size, reader, buf)? as u32;
        let object_type = value >> BACNET_INSTANCE_BITS & BACNET_MAX_OBJECT;
        let object_type = ObjectType::try_from(object_type)
            .map_err(|x| Error::InvalidVariant(("ObjectId decode ObjectType", x)))?;
        let id = value & BACNET_MAX_INSTANCE;
        let object_id = ObjectId { object_type, id };
        Ok(object_id)
    }
}

// NOTE that copy is derived for usage convenience
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum ObjectType {
    ObjectAnalogInput = 0,
    ObjectAnalogOutput = 1,
    ObjectAnalogValue = 2,
    ObjectBinaryInput = 3,
    ObjectBinaryOutput = 4,
    ObjectBinaryValue = 5,
    ObjectCalendar = 6,
    ObjectCommand = 7,
    ObjectDevice = 8,
    ObjectEventEnrollment = 9,
    ObjectFile = 10,
    ObjectGroup = 11,
    ObjectLoop = 12,
    ObjectMultiStateInput = 13,
    ObjectMultiStateOutput = 14,
    ObjectNotificationClass = 15,
    ObjectProgram = 16,
    ObjectSchedule = 17,
    ObjectAveraging = 18,
    ObjectMultiStateValue = 19,
    ObjectTrendlog = 20,
    ObjectLifeSafetyPoint = 21,
    ObjectLifeSafetyZone = 22,
    ObjectAccumulator = 23,
    ObjectPulseConverter = 24,
    ObjectEventLog = 25,
    ObjectGlobalGroup = 26,
    ObjectTrendLogMultiple = 27,
    ObjectLoadControl = 28,
    ObjectStructuredView = 29,
    ObjectAccessDoor = 30,
    ObjectTimer = 31,
    ObjectAccessCredential = 32, // addendum 2008-j
    ObjectAccessPoint = 33,
    ObjectAccessRights = 34,
    ObjectAccessUser = 35,
    ObjectAccessZone = 36,
    ObjectCredentialDataInput = 37,   // authentication-factor-input
    ObjectNetworkSecurity = 38,       // Addendum 2008-g
    ObjectBitstringValue = 39,        // Addendum 2008-w
    ObjectCharacterstringValue = 40,  // Addendum 2008-w
    ObjectDatePatternValue = 41,      // Addendum 2008-w
    ObjectDateValue = 42,             // Addendum 2008-w
    ObjectDatetimePatternValue = 43,  // Addendum 2008-w
    ObjectDatetimeValue = 44,         // Addendum 2008-w
    ObjectIntegerValue = 45,          // Addendum 2008-w
    ObjectLargeAnalogValue = 46,      // Addendum 2008-w
    ObjectOctetstringValue = 47,      // Addendum 2008-w
    ObjectPositiveIntegerValue = 48,  // Addendum 2008-w
    ObjectTimePatternValue = 49,      // Addendum 2008-w
    ObjectTimeValue = 50,             // Addendum 2008-w
    ObjectNotificationForwarder = 51, // Addendum 2010-af
    ObjectAlertEnrollment = 52,       // Addendum 2010-af
    ObjectChannel = 53,               // Addendum 2010-aa
    ObjectLightingOutput = 54,        // Addendum 2010-i
    ObjectBinaryLightingOutput = 55,  // Addendum 135-2012az
    ObjectNetworkPort = 56,           // Addendum 135-2012az
    // Enumerated values 0-127 are reserved for definition by ASHRAE.
    // Enumerated values 128-1023 may be used by others subject to
    // the procedures and constraints described in Clause 23.
    // do the max range inside of enum so that
    // compilers will allocate adequate sized datatype for enum
    // which is used to store decoding
    Reserved = 57,
    Proprietary = 128,
    Invalid = 1024,
}

impl TryFrom<u32> for ObjectType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, u32> {
        match value {
            0 => Ok(Self::ObjectAnalogInput),
            1 => Ok(Self::ObjectAnalogOutput),
            2 => Ok(Self::ObjectAnalogValue),
            3 => Ok(Self::ObjectBinaryInput),
            4 => Ok(Self::ObjectBinaryOutput),
            5 => Ok(Self::ObjectBinaryValue),
            6 => Ok(Self::ObjectCalendar),
            7 => Ok(Self::ObjectCommand),
            8 => Ok(Self::ObjectDevice),
            9 => Ok(Self::ObjectEventEnrollment),
            10 => Ok(Self::ObjectFile),
            11 => Ok(Self::ObjectGroup),
            12 => Ok(Self::ObjectLoop),
            13 => Ok(Self::ObjectMultiStateInput),
            14 => Ok(Self::ObjectMultiStateOutput),
            15 => Ok(Self::ObjectNotificationClass),
            16 => Ok(Self::ObjectProgram),
            17 => Ok(Self::ObjectSchedule),
            18 => Ok(Self::ObjectAveraging),
            19 => Ok(Self::ObjectMultiStateValue),
            20 => Ok(Self::ObjectTrendlog),
            21 => Ok(Self::ObjectLifeSafetyPoint),
            22 => Ok(Self::ObjectLifeSafetyZone),
            23 => Ok(Self::ObjectAccumulator),
            24 => Ok(Self::ObjectPulseConverter),
            25 => Ok(Self::ObjectEventLog),
            26 => Ok(Self::ObjectGlobalGroup),
            27 => Ok(Self::ObjectTrendLogMultiple),
            28 => Ok(Self::ObjectLoadControl),
            29 => Ok(Self::ObjectStructuredView),
            30 => Ok(Self::ObjectAccessDoor),
            31 => Ok(Self::ObjectTimer),
            32 => Ok(Self::ObjectAccessCredential),
            33 => Ok(Self::ObjectAccessPoint),
            34 => Ok(Self::ObjectAccessRights),
            35 => Ok(Self::ObjectAccessUser),
            36 => Ok(Self::ObjectAccessZone),
            37 => Ok(Self::ObjectCredentialDataInput),
            38 => Ok(Self::ObjectNetworkSecurity),
            39 => Ok(Self::ObjectBitstringValue),
            40 => Ok(Self::ObjectCharacterstringValue),
            41 => Ok(Self::ObjectDatePatternValue),
            42 => Ok(Self::ObjectDateValue),
            43 => Ok(Self::ObjectDatetimePatternValue),
            44 => Ok(Self::ObjectDatetimeValue),
            45 => Ok(Self::ObjectIntegerValue),
            46 => Ok(Self::ObjectLargeAnalogValue),
            47 => Ok(Self::ObjectOctetstringValue),
            48 => Ok(Self::ObjectPositiveIntegerValue),
            49 => Ok(Self::ObjectTimePatternValue),
            50 => Ok(Self::ObjectTimeValue),
            51 => Ok(Self::ObjectNotificationForwarder),
            52 => Ok(Self::ObjectAlertEnrollment),
            53 => Ok(Self::ObjectChannel),
            54 => Ok(Self::ObjectLightingOutput),
            55 => Ok(Self::ObjectBinaryLightingOutput),
            56 => Ok(Self::ObjectNetworkPort),
            57..=127 => Ok(Self::Reserved),
            128..=1023 => Ok(Self::Proprietary),
            x => Err(x),
        }
    }
}
