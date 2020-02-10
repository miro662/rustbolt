use std::mem;

pub trait BoltSerializable {
    fn bolt_serialize(&self) -> Vec<u8>;
}

impl BoltSerializable for bool {
    fn bolt_serialize(&self) -> Vec<u8> {
        match self {
            true => vec![0xC3],
            false => vec![0xC2]
        }
    }
}

impl BoltSerializable for i64 {
    fn bolt_serialize(&self) -> Vec<u8> {
        unsafe {
            let data_be = self.to_be();
            let raw_data = mem::transmute::<i64, [u8; 8]>(data_be);
            if (*self) >= -16 && (*self) <= 127 {
                // TINY_INT
                vec![raw_data[7]]
            } else if (*self) >= -128 && (*self) <= 127 {
                // INT_8
                vec![0xC8, raw_data[7]]
            } else if (*self) >= -32768 && (*self) <= 32767 {
                // INT_16
                vec![0xC9, raw_data[6], raw_data[7]]
            } else if (*self) >= -2147483648 && (*self) <= 2147483647 {
                // INT_32
                vec![0xCA, raw_data[4], raw_data[5], raw_data[6], raw_data[7]]
            } else {
                // INT_64
                vec![0xCB, raw_data[0], raw_data[1], raw_data[2], raw_data[3], raw_data[4], raw_data[5], raw_data[6], raw_data[7]]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_serialzied_correctly() {
        assert_eq!(vec![0xC3], true.bolt_serialize());
        assert_eq!(vec![0xC2], false.bolt_serialize());
    }

    #[test]
    fn i64_serialized_correctly_to_bolt_tiny_int() {
        assert_eq!(vec![0x00], 0i64.bolt_serialize());
        assert_eq!(vec![0x01], 1i64.bolt_serialize());
        assert_eq!(vec![0x7F], 127i64.bolt_serialize());
        assert_eq!(vec![0xFF], (-1i64).bolt_serialize());
        assert_eq!(vec![0xF0], (-16i64).bolt_serialize());
    }

    #[test]
    fn i64_serialized_correctly_to_bolt_int_8() {
        assert_eq!(vec![0xC8, 0xEF], (-17i64).bolt_serialize());
        assert_eq!(vec![0xC8, 0x9C], (-100i64).bolt_serialize());
        assert_eq!(vec![0xC8, 0x81], (-127i64).bolt_serialize());
        assert_eq!(vec![0xC8, 0x80], (-128i64).bolt_serialize());
    }

    #[test]
    fn i64_serialized_correctly_to_bolt_int_16() {    
        assert_eq!(vec![0xC9, 0x80, 0x00], (-32768i64).bolt_serialize());
        assert_eq!(vec![0xC9, 0xFF, 0x7F], (-129i64).bolt_serialize());
        assert_eq!(vec![0xC9, 0x00, 0x80], (128i64).bolt_serialize());
        assert_eq!(vec![0xC9, 0x7F, 0xFF], (32767i64).bolt_serialize());
    }

    #[test]
    fn i64_serialized_correctly_to_bolt_int_32() {   
        assert_eq!(vec![0xCA, 0x80, 0x00, 0x00, 0x00], (-2147483648i64).bolt_serialize());
        assert_eq!(vec![0xCA, 0xFF, 0xFF, 0x7F, 0xFF], (-32769i64).bolt_serialize());
        assert_eq!(vec![0xCA, 0x00, 0x00, 0x80, 0x00], (32768i64).bolt_serialize());
        assert_eq!(vec![0xCA, 0x7F, 0xFF, 0xFF, 0xFF], (2147483647i64).bolt_serialize());
    }

    #[test]
    fn i64_serialized_correctly_to_bolt_int_64() {   
        assert_eq!(
            vec![0xCB, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 
            (-9223372036854775808i64).bolt_serialize()
        );
        assert_eq!(
            vec![0xCB, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF], 
            (-2147483649i64).bolt_serialize()
        );
        assert_eq!(
            vec![0xCB, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00], 
            (2147483648i64).bolt_serialize()
        );
        assert_eq!(
            vec![0xCB, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 
            (9223372036854775807i64).bolt_serialize()
        );
    }
}