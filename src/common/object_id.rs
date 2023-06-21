use super::{
    error::Error,
    helper::{decode_unsigned, Buffer, Reader},
};

pub const BACNET_MAX_INSTANCE: u32 = 0x3FFFFF;
pub const BACNET_INSTANCE_BITS: u32 = 22;
pub const BACNET_MAX_OBJECT: u32 = 0x3FF;

#[derive(Debug)]
pub struct ObjectId {
    pub object_type: ObjectType,
    pub id: u32,
}

impl ObjectId {
    pub fn encode(&self, buffer: &mut Buffer) {
        let value = ((self.object_type as u32 & BACNET_MAX_OBJECT) << BACNET_INSTANCE_BITS)
            | (self.id & BACNET_MAX_INSTANCE);
        buffer.extend_from_slice(&value.to_be_bytes());
    }

    pub fn decode(reader: &mut Reader, size: u32) -> Result<Self, Error> {
        let value = decode_unsigned(reader, size)?;
        let object_type = value >> BACNET_INSTANCE_BITS & BACNET_MAX_OBJECT;
        let object_type = ObjectType::from(object_type);
        let id = value & BACNET_MAX_INSTANCE;
        let object_id = ObjectId { object_type, id };
        Ok(object_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum ObjectType {
    ObjectAnalogInput = 0,
    ObjectAnalogOutput,
    ObjectAnalogValue,
    ObjectBinaryInput,
    ObjectBinaryOutput,
    ObjectBinaryValue,
    ObjectCalendar,
    ObjectCommand,
    ObjectDevice,
    ObjectEventEnrollment,
    ObjectFile,
    ObjectGroup,
    ObjectLoop,
    ObjectMultiStateInput,
    ObjectMultiStateOutput,
    ObjectNotificationClass,
    ObjectProgram,
    ObjectSchedule,
    ObjectAveraging,
    ObjectMultiStateValue,
    ObjectTrendlog,
    ObjectLifeSafetyPoint,
    ObjectLifeSafetyZone,
    ObjectAccumulator,
    ObjectPulseConverter,
    ObjectEventLog,
    ObjectGlobalGroup,
    ObjectTrendLogMultiple,
    ObjectLoadControl,
    ObjectStructuredView,
    ObjectAccessDoor,
    ObjectTimer,
    ObjectAccessCredential, // addendum 2008-j
    ObjectAccessPoint,
    ObjectAccessRights,
    ObjectAccessUser,
    ObjectAccessZone,
    ObjectCredentialDataInput,   // authentication-factor-input
    ObjectNetworkSecurity,       // Addendum 2008-g
    ObjectBitstringValue,        // Addendum 2008-w
    ObjectCharacterstringValue,  // Addendum 2008-w
    ObjectDatePatternValue,      // Addendum 2008-w
    ObjectDateValue,             // Addendum 2008-w
    ObjectDatetimePatternValue,  // Addendum 2008-w
    ObjectDatetimeValue,         // Addendum 2008-w
    ObjectIntegerValue,          // Addendum 2008-w
    ObjectLargeAnalogValue,      // Addendum 2008-w
    ObjectOctetstringValue,      // Addendum 2008-w
    ObjectPositiveIntegerValue,  // Addendum 2008-w
    ObjectTimePatternValue,      // Addendum 2008-w
    ObjectTimeValue,             // Addendum 2008-w
    ObjectNotificationForwarder, // Addendum 2010-af
    ObjectAlertEnrollment,       // Addendum 2010-af
    ObjectChannel,               // Addendum 2010-aa
    ObjectLightingOutput,        // Addendum 2010-i
    ObjectBinaryLightingOutput,  // Addendum 135-2012az
    ObjectNetworkPort,           // Addendum 135-2012az
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
