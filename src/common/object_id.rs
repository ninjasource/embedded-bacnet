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
        let value = ((self.object_type.as_u32() & BACNET_MAX_OBJECT) << BACNET_INSTANCE_BITS)
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
    ObjectVendorSpecific(u32),    // 128..=1023
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
            Self::ObjectVendorSpecific(v) => *v,
        }
    }

    pub fn try_from(value: u32) -> Result<Self, u32> {
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
            57..=127 => Ok(Self::ObjectReserved(value)),
            128..=1023 => Ok(Self::ObjectVendorSpecific(value)),
            x => Err(x),
        }
    }
}
