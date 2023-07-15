use super::{
    error::Error,
    helper::{decode_unsigned, Reader, Writer},
    spec::{BACNET_INSTANCE_BITS, BACNET_MAX_INSTANCE, BACNET_MAX_OBJECT},
};

#[derive(Debug, Clone, Copy)]
pub struct ObjectId {
    pub object_type: ObjectType,
    pub id: u32,
}

impl ObjectId {
    pub fn new(object_type: ObjectType, id: u32) -> Self {
        Self { object_type, id }
    }

    pub fn encode(&self, buffer: &mut Writer) {
        let value = ((self.object_type as u32 & BACNET_MAX_OBJECT) << BACNET_INSTANCE_BITS)
            | (self.id & BACNET_MAX_INSTANCE);
        buffer.extend_from_slice(&value.to_be_bytes());
    }

    pub fn decode(size: u32, reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let value = decode_unsigned(size, reader, buf) as u32;
        let object_type = value >> BACNET_INSTANCE_BITS & BACNET_MAX_OBJECT;
        let object_type = ObjectType::from(object_type);
        let id = value & BACNET_MAX_INSTANCE;
        let object_id = ObjectId { object_type, id };
        Ok(object_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

impl From<u32> for ObjectType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::ObjectAnalogInput,
            1 => Self::ObjectAnalogOutput,
            2 => Self::ObjectAnalogValue,
            3 => Self::ObjectBinaryInput,
            4 => Self::ObjectBinaryOutput,
            5 => Self::ObjectBinaryValue,
            6 => Self::ObjectCalendar,
            7 => Self::ObjectCommand,
            8 => Self::ObjectDevice,
            9 => Self::ObjectEventEnrollment,
            10 => Self::ObjectFile,
            11 => Self::ObjectGroup,
            12 => Self::ObjectLoop,
            13 => Self::ObjectMultiStateInput,
            14 => Self::ObjectMultiStateOutput,
            15 => Self::ObjectNotificationClass,
            16 => Self::ObjectProgram,
            17 => Self::ObjectSchedule,
            18 => Self::ObjectAveraging,
            19 => Self::ObjectMultiStateValue,
            20 => Self::ObjectTrendlog,
            21 => Self::ObjectLifeSafetyPoint,
            22 => Self::ObjectLifeSafetyZone,
            23 => Self::ObjectAccumulator,
            24 => Self::ObjectPulseConverter,
            25 => Self::ObjectEventLog,
            26 => Self::ObjectGlobalGroup,
            27 => Self::ObjectTrendLogMultiple,
            28 => Self::ObjectLoadControl,
            29 => Self::ObjectStructuredView,
            30 => Self::ObjectAccessDoor,
            31 => Self::ObjectTimer,
            32 => Self::ObjectAccessCredential,
            33 => Self::ObjectAccessPoint,
            34 => Self::ObjectAccessRights,
            35 => Self::ObjectAccessUser,
            36 => Self::ObjectAccessZone,
            37 => Self::ObjectCredentialDataInput,
            38 => Self::ObjectNetworkSecurity,
            39 => Self::ObjectBitstringValue,
            40 => Self::ObjectCharacterstringValue,
            41 => Self::ObjectDatePatternValue,
            42 => Self::ObjectDateValue,
            43 => Self::ObjectDatetimePatternValue,
            44 => Self::ObjectDatetimeValue,
            45 => Self::ObjectIntegerValue,
            46 => Self::ObjectLargeAnalogValue,
            47 => Self::ObjectOctetstringValue,
            48 => Self::ObjectPositiveIntegerValue,
            49 => Self::ObjectTimePatternValue,
            50 => Self::ObjectTimeValue,
            51 => Self::ObjectNotificationForwarder,
            52 => Self::ObjectAlertEnrollment,
            53 => Self::ObjectChannel,
            54 => Self::ObjectLightingOutput,
            55 => Self::ObjectBinaryLightingOutput,
            56 => Self::ObjectNetworkPort,
            57..=127 => Self::Reserved,
            128..=1023 => Self::Proprietary,
            _ => Self::Invalid,
        }
    }
}
