use std::mem;

use macros::bitfield_struct;

const FLAG123_FLAG1: bool = true;
const FLAG123_FLAG2: u8 = 0b10;
const FLAG123_FLAG3: u8 = 0b100;
const FLAG123_FLAG4: u16 = 0b1001000010;
const FLAG123: u16 = (FLAG123_FLAG4 << 6)
    | ((FLAG123_FLAG3 as u16) << 3)
    | ((FLAG123_FLAG2 as u16) << 1)
    | (FLAG123_FLAG1 as u16);

const FLAGS_0_FLAG5: u8 = 0b01;
const FLAGS_0_FLAG6: u8 = 0b10;
const FLAGS_0_FLAG7: u8 = 0b11;
const FLAGS_0_FLAG8: u8 = 0b01;
const FLAGS_0: u8 =
    (FLAGS_0_FLAG8 << 6) | (FLAGS_0_FLAG7 << 4) | (FLAGS_0_FLAG6 << 2) | FLAGS_0_FLAG5;

const FLAGS_1_FLAG9: u8 = 0b00;
const FLAGS_1_FLAG10: u8 = 0b01;
const FLAGS_1_FLAG11: u8 = 0b11;
const FLAGS_1_FLAG12: u8 = 0b10;
const FLAGS_1: u8 =
    (FLAGS_1_FLAG12 << 6) | (FLAGS_1_FLAG11 << 4) | (FLAGS_1_FLAG10 << 2) | FLAGS_1_FLAG9;

#[test]
fn test() {
    let hoge = Hoge::new()
        .with_flag123_flag1(FLAG123_FLAG1)
        .with_flag123_flag2(FLAG123_FLAG2)
        .with_flag123_flag3(FLAG123_FLAG3)
        .with_flag123_flag4(FLAG123_FLAG4)
        .with_int2(0xfedcba)
        .with_flags_0_flag5(FLAGS_0_FLAG5)
        .with_flags_0_flag6(FLAGS_0_FLAG6)
        .with_flags_0_flag7(FLAGS_0_FLAG7)
        .with_flags_0_flag8(FLAGS_0_FLAG8)
        .with_flags_1_flag9(FLAGS_1_FLAG9)
        .with_flags_1_flag10(FLAGS_1_FLAG10)
        .with_flags_1_flag11(FLAGS_1_FLAG11)
        .with_flags_1_flag12(FLAGS_1_FLAG12);

    assert_eq!(mem::size_of::<Hoge>(), 8);
    check_flag123(&hoge);
    check_flags(&hoge);
}

fn check_flag123(hoge: &Hoge) {
    assert_eq!(hoge.get_flag123_flag1(), FLAG123_FLAG1);
    assert_eq!(hoge.get_flag123_flag2(), FLAG123_FLAG2);
    assert_eq!(hoge.get_flag123_flag3(), FLAG123_FLAG3);
    assert_eq!(hoge.get_flag123_flag4(), FLAG123_FLAG4);

    assert_eq!(hoge.get_flag123(), FLAG123);
}

fn check_flags(hoge: &Hoge) {
    assert_eq!(hoge.get_flags_0_flag5(), FLAGS_0_FLAG5);
    assert_eq!(hoge.get_flags_0_flag6(), FLAGS_0_FLAG6);
    assert_eq!(hoge.get_flags_0_flag7(), FLAGS_0_FLAG7);
    assert_eq!(hoge.get_flags_0_flag8(), FLAGS_0_FLAG8);

    assert_eq!(hoge.get_flags_1_flag9(), FLAGS_1_FLAG9);
    assert_eq!(hoge.get_flags_1_flag10(), FLAGS_1_FLAG10);
    assert_eq!(hoge.get_flags_1_flag11(), FLAGS_1_FLAG11);
    assert_eq!(hoge.get_flags_1_flag12(), FLAGS_1_FLAG12);

    assert_eq!(hoge.get_flags(), [FLAGS_0, FLAGS_1]);
}

bitfield_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Hoge {
        flag123: u16 => {
            #[bits(1)]
            flag1: bool,
            #[bits(2)]
            flag2: u8,
            #[bits(3)]
            flag3: u8,
            #[bits(10)]
            flag4: u16,
         },

        int2: u32,

        flags: [u8; 2] => [
            {
                #[bits(2)]
                flag5: u8,
                #[bits(2)]
                flag6: u8,
                #[bits(2)]
                flag7: u8,
                #[bits(2)]
                flag8: u8,
            },
            {
                #[bits(2)]
                flag9: u8,
                #[bits(2)]
                flag10: u8,
                #[bits(2)]
                flag11: u8,
                #[bits(2)]
                flag12: u8,
            }
        ]
    }
}
