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

const ENUMS_FLAG: Flag = Flag::B;
const ENUMS_REMAIN: u32 = 0b10101010101010101010101010;
const ENUMS_FLAG2: Flag = Flag::C(7);
const ENUNMS: u32 = (ENUMS_FLAG2.to_ne()) << 29 | (ENUMS_REMAIN << 3) | ENUMS_FLAG.to_ne();

#[test]
fn test_hoge() {
    let hoge = Hoge::default()
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
        .with_flags_1_flag12(FLAGS_1_FLAG12)
        .with_enums_flag(ENUMS_FLAG)
        .with_enums_flag2(ENUMS_FLAG2)
        .with_enums_remain(ENUMS_REMAIN);

    assert_eq!(mem::size_of::<Hoge>(), 12);
    check_flag123(&hoge);
    check_flags(&hoge);
    check_enum(&hoge);
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

fn check_enum(hoge: &Hoge) {
    assert_eq!(hoge.get_enums(), ENUNMS);
    assert_eq!(hoge.get_enums_flag(), ENUMS_FLAG);
    assert_eq!(hoge.get_enums_remain(), ENUMS_REMAIN);
    assert_eq!(hoge.get_enums_flag2(), ENUMS_FLAG2);
}

#[test]
fn test_lshift_overflow() {
    let mut fuga = Fuga::default();
    fuga.set_ptrs_ptr_lo(0);
}

#[test]
fn test_arr() {
    bitfield_struct! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        struct Arr {
            p: [u32; 1] => [
                {
                    #[bits(32)]
                    hoge: u32,
                },
            ]
        }
    }

    let mut arr = Arr::default();
    arr.set_p_0_hoge(0x1234);

    assert_eq!(arr.get_p_0_hoge(), 0x1234);
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
enum Flag {
    A,
    B,
    C(u32),
}

impl Flag {
    fn from_be(v: u32) -> Self {
        Self::from_ne(u32::from_be(v))
    }

    const fn to_ne(self) -> u32 {
        match self {
            Flag::A => 1,
            Flag::B => 2,
            Flag::C(x) => x,
        }
    }

    fn from_ne(v: u32) -> Self {
        match v {
            1 => Self::A,
            2 => Self::B,
            x => Self::C(x),
        }
    }
}

bitfield_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
        ],

        enums: u32 => {
            #[bits(3)]
            flag: Flag,
            #[bits(26)]
            remain: u32,
            #[bits(3)]
            flag2: Flag,
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Fuga {
        ptrs: u16 => {
            #[bits(16)]
            ptr_lo: u16,
        }
    }


}

#[test]
fn test_setup_stage() {
    bitfield_struct! {
        #[repr(C, packed)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[endian = "little"]
        pub struct SetupStage {
            parameter0: u32 => {
                #[bits(8)]
                bm_request_type: u8,
                #[bits(8)]
                b_ruquest: u8,
                #[bits(16)]
                w_value: u16,
            },
            parameter1: u32 => {
                #[bits(16)]
                w_index: u16,
                #[bits(16)]
                w_length: u16,
            },
            status: u32 => {
                #[bits(17)]
                trb_transfer_length: u32,
                #[bits(5)]
                _rsvdz: u8,
                #[bits(10)]
                interrupter_target: u16,
            },
            remain: u16 => {
                #[bits(1)]
                cycle_bit: bool,
                #[bits(4)]
                _rsvdz1: u16,
                #[bits(1)]
                interrupt_on_completion: bool,
                #[bits(1)]
                immediate_data: bool,
                #[bits(3)]
                _rsvdz2: u16,
                #[bits(6)]
                trb_type: u8,
            },
            control: u16 => {
                #[bits(2)]
                transfer_type: u8,
                #[bits(14)]
                _rsvdz: u16,
            }
        }
    }

    let setup = SetupStage::default()
        .with_parameter0_bm_request_type(0b10000000)
        .with_parameter0_b_ruquest(6)
        .with_parameter0_w_value(0x0100)
        .with_parameter1_w_index(0)
        .with_parameter1_w_length(18)
        .with_status_trb_transfer_length(8)
        .with_control_transfer_type(3)
        .with_remain_interrupt_on_completion(true)
        .with_remain_immediate_data(true)
        .with_remain_trb_type(2)
        .with_remain_cycle_bit(true);

    let expected = SetupStage {
        parameter0: 0b00000001_00000000_00000110_10000000,
        parameter1: 0x0012_0000,
        status: 0x0000_0008,
        remain: 0b00001000_01100001,
        control: 0x0000_0003,
    };

    assert_eq!(setup, expected);
}
