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
        let type_value = self.object_type.as_u32();
        let value = ((type_value & BACNET_MAX_OBJECT) << BACNET_INSTANCE_BITS)
            | (self.id & BACNET_MAX_INSTANCE);
        writer.extend_from_slice(&value.to_be_bytes());
    }

    pub fn decode(size: u32, reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let value = decode_unsigned(size, reader, buf)? as u32;
        let object_type_value = value >> BACNET_INSTANCE_BITS & BACNET_MAX_OBJECT;
        let object_type = ObjectType::from_u32(object_type_value);
        let id = value & BACNET_MAX_INSTANCE;
        let object_id = ObjectId { object_type, id };
        Ok(object_id)
    }
}

// NOTE that copy is derived for usage convenience
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectType {
    ObjectAnalogInput,
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
    ObjectAccessCredential,    // addendum 2008-j
    ObjectAccessPoint,
    ObjectAccessRights,
    ObjectAccessUser,
    ObjectAccessZone,
    ObjectCredentialDataInput, // authentication-factor-input
    ObjectNetworkSecurity,     // Addendum 2008-g
    ObjectBitstringValue,      // Addendum 2008-w
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
    ObjectReserved(u32),       // 57..=127
    ObjectProprietary(u32),    // 128..=1023
    ObjectVendorSpecific(u32), // 1024+
}

impl ObjectType {
    pub fn as_u32(&self) -> u32 {
        match self {
            Self::ObjectAnalogInput => 0,
            Self::ObjectAnalogOutput => 1,
            Self::ObjectAnalogValue => 2,
            Self::ObjectBinaryInput => 3,
            Self::ObjectBinaryOutput => 4,
            Self::ObjectBinaryValue => 5,
            Self::ObjectCalendar => 6,
            Self::ObjectCommand => 7,
            Self::ObjectDevice => 8,
            Self::ObjectEventEnrollment => 9,
            Self::ObjectFile => 10,
            Self::ObjectGroup => 11,
            Self::ObjectLoop => 12,
            Self::ObjectMultiStateInput => 13,
            Self::ObjectMultiStateOutput => 14,
            Self::ObjectNotificationClass => 15,
            Self::ObjectProgram => 16,
            Self::ObjectSchedule => 17,
            Self::ObjectAveraging => 18,
            Self::ObjectMultiStateValue => 19,
            Self::ObjectTrendlog => 20,
            Self::ObjectLifeSafetyPoint => 21,
            Self::ObjectLifeSafetyZone => 22,
            Self::ObjectAccumulator => 23,
            Self::ObjectPulseConverter => 24,
            Self::ObjectEventLog => 25,
            Self::ObjectGlobalGroup => 26,
            Self::ObjectTrendLogMultiple => 27,
            Self::ObjectLoadControl => 28,
            Self::ObjectStructuredView => 29,
            Self::ObjectAccessDoor => 30,
            Self::ObjectTimer => 31,
            Self::ObjectAccessCredential => 32,
            Self::ObjectAccessPoint => 33,
            Self::ObjectAccessRights => 34,
            Self::ObjectAccessUser => 35,
            Self::ObjectAccessZone => 36,
            Self::ObjectCredentialDataInput => 37,
            Self::ObjectNetworkSecurity => 38,
            Self::ObjectBitstringValue => 39,
            Self::ObjectCharacterstringValue => 40,
            Self::ObjectDatePatternValue => 41,
            Self::ObjectDateValue => 42,
            Self::ObjectDatetimePatternValue => 43,
            Self::ObjectDatetimeValue => 44,
            Self::ObjectIntegerValue => 45,
            Self::ObjectLargeAnalogValue => 46,
            Self::ObjectOctetstringValue => 47,
            Self::ObjectPositiveIntegerValue => 48,
            Self::ObjectTimePatternValue => 49,
            Self::ObjectTimeValue => 50,
            Self::ObjectNotificationForwarder => 51,
            Self::ObjectAlertEnrollment => 52,
            Self::ObjectChannel => 53,
            Self::ObjectLightingOutput => 54,
            Self::ObjectBinaryLightingOutput => 55,
            Self::ObjectNetworkPort => 56,
            Self::ObjectReserved(v) => *v,
            Self::ObjectProprietary(v) => *v,
            Self::ObjectVendorSpecific(v) => *v,
        }
    }

    pub fn from_u32(value: u32) -> Self {
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
            57..=127 => Self::ObjectReserved(value),
            128..=1023 => Self::ObjectProprietary(value),
            _ => Self::ObjectVendorSpecific(value),
        }
    }
}
