pub trait BoltSerializable {
    fn bolt_serialize(&self) -> Vec<u8>;
}

impl BoltSerializable for bool {
    fn bolt_serialize(&self) -> Vec<u8> {
        match self {
            true => vec![0xC3],
            false => vec![0xC2],
        }
    }
}

impl BoltSerializable for i64 {
    fn bolt_serialize(&self) -> Vec<u8> {
        let raw_data = self.to_be_bytes();
        if (*self) >= -16 && (*self) <= 127 {
            // TINY_INT
            vec![raw_data[7]]
        } else {
            let (header, bytes): (u8, usize) = if (*self) >= -128 && (*self) <= 127 {
                (0xC8, 1)
            } else if (*self) >= -32768 && (*self) <= 32767 {
                (0xC9, 2)
            } else if (*self) >= -2147483648 && (*self) <= 2147483647 {
                (0xCA, 4)
            } else {
                (0xCB, 8)
            };

            let mut data = vec![header];
            data.extend(raw_data.iter().skip(8 - bytes));
            data
        }
    }
}

impl BoltSerializable for f64 {
    fn bolt_serialize(&self) -> Vec<u8> {
        let raw_data = self.to_be_bytes();
        let mut data = vec![0xC1];
        data.extend(raw_data.iter());
        data
    }
}

impl BoltSerializable for &str {
    fn bolt_serialize(&self) -> Vec<u8> {
        let bytes = self.bytes();
        let len = bytes.len();

        let mut data = if len < 16 {
            vec![len as u8 + 0x80]
        } else {
            let (header, bytes): (u8, usize) = if len < 256 {
                (0xD0, 1)
            } else if len < 65536 {
                (0xD1, 2)
            } else {
                (0xD2, 4)
            };
            let mut data_vec = vec![header];
            data_vec.extend(len.to_be_bytes().iter().skip(8 - bytes));
            data_vec
        };
        data.extend(bytes);
        data
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
        assert_eq!(
            vec![0xCA, 0x80, 0x00, 0x00, 0x00],
            (-2147483648i64).bolt_serialize()
        );
        assert_eq!(
            vec![0xCA, 0xFF, 0xFF, 0x7F, 0xFF],
            (-32769i64).bolt_serialize()
        );
        assert_eq!(
            vec![0xCA, 0x00, 0x00, 0x80, 0x00],
            (32768i64).bolt_serialize()
        );
        assert_eq!(
            vec![0xCA, 0x7F, 0xFF, 0xFF, 0xFF],
            (2147483647i64).bolt_serialize()
        );
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

    #[test]
    fn f64_serialized_correctly_to_bolt() {
        assert_eq!(
            vec![0xC1, 0x3F, 0xF1, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9A],
            (1.1).bolt_serialize()
        );

        assert_eq!(
            vec![0xC1, 0xBF, 0xF1, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9A],
            (-1.1).bolt_serialize()
        );
    }

    #[test]
    fn str_serialized_correctly_to_bolt() {
        assert_eq!(vec![0x81, 0x61], "a".bolt_serialize());

        assert_eq!(
            vec![
                0xD0, 0x1A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C,
                0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A
            ],
            "abcdefghijklmnopqrstuvwxyz".bolt_serialize()
        );

        assert_eq!(
            vec![
                0xD0, 0x18, 0x45, 0x6E, 0x20, 0xC3, 0xA5, 0x20, 0x66, 0x6C, 0xC3, 0xB6, 0x74, 0x20,
                0xC3, 0xB6, 0x76, 0x65, 0x72, 0x20, 0xC3, 0xA4, 0x6E, 0x67, 0x65, 0x6E
            ],
            "En å flöt över ängen".bolt_serialize()
        );
    }
}