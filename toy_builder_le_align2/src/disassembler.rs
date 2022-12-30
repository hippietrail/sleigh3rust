pub type AddrType = u32;
macro_rules! impl_read_to_type {
    ($ unsigned_type : ty , $ signed_type : ty , $ len : literal , $ read_unsigned : ident , $ read_signed : ident , $ write_unsigned : ident , $ write_signed : ident) => {
        const fn $read_unsigned<const BIG_ENDIAN: bool>(
            data: [u8; $len],
            start_bit: usize,
            len_bits: usize,
        ) -> $unsigned_type {
            const TYPE_BITS: usize = <$unsigned_type>::BITS as usize;
            assert!(TYPE_BITS / 8 == $len);
            assert!(len_bits > 0);
            assert!(len_bits + start_bit <= TYPE_BITS);
            let mut data = if BIG_ENDIAN {
                <$unsigned_type>::from_be_bytes(data)
            } else {
                <$unsigned_type>::from_le_bytes(data)
            };
            let value_mask = <$unsigned_type>::MAX >> (TYPE_BITS - len_bits);
            data = data >> start_bit;
            data = data & value_mask;
            data
        }
        const fn $read_signed<const BIG_ENDIAN: bool>(
            data: [u8; $len],
            start_bit: usize,
            len_bits: usize,
        ) -> $signed_type {
            const TYPE_BITS: usize = <$signed_type>::BITS as usize;
            assert!(len_bits > 1);
            assert!(TYPE_BITS / 8 == $len);
            let data = $read_unsigned::<BIG_ENDIAN>(data, start_bit, len_bits);
            let value_mask =
                <$signed_type>::MAX as $unsigned_type >> (TYPE_BITS - len_bits);
            let sign_mask = !value_mask;
            let value_part = data & value_mask;
            let sign_part = data & sign_mask;
            if sign_part != 0 {
                sign_mask as $signed_type | value_part as $signed_type
            } else {
                data as $signed_type
            }
        }
        const fn $write_unsigned<const BIG_ENDIAN: bool>(
            value: $unsigned_type,
            mem: $unsigned_type,
            start_bit: usize,
            len_bits: usize,
        ) -> [u8; $len] {
            const TYPE_BITS: usize = <$unsigned_type>::BITS as usize;
            assert!(len_bits > 0);
            assert!(len_bits + start_bit <= TYPE_BITS);
            let value_max = <$unsigned_type>::MAX >> (TYPE_BITS - len_bits);
            let mask = value_max << start_bit;
            let mut value = value;
            value <<= start_bit;
            value = (mem & !mask) | value;
            if BIG_ENDIAN {
                value.to_be_bytes()
            } else {
                value.to_le_bytes()
            }
        }
        const fn $write_signed<const BIG_ENDIAN: bool>(
            value: $signed_type,
            mem: $signed_type,
            start_bit: usize,
            len_bits: usize,
        ) -> [u8; $len] {
            const TYPE_BITS: usize = <$unsigned_type>::BITS as usize;
            assert!(len_bits > 0);
            assert!(len_bits + start_bit <= TYPE_BITS);
            let value_max = <$signed_type>::MAX >> (TYPE_BITS - len_bits);
            let value_min = <$signed_type>::MIN >> (TYPE_BITS - len_bits);
            let mask = <$unsigned_type>::MAX >> (TYPE_BITS - len_bits);
            let value = value as $unsigned_type & mask;
            let mem = mem as $unsigned_type;
            $write_unsigned::<BIG_ENDIAN>(value, mem, start_bit, len_bits)
        }
    };
}
impl_read_to_type!(u8, i8, 1, read_u8, read_i8, write_u8, write_i8);
impl_read_to_type!(u16, i16, 2, read_u16, read_i16, write_u16, write_i16);
impl_read_to_type!(u32, i32, 4, read_u32, read_i32, write_u32, write_i32);
impl_read_to_type!(u64, i64, 8, read_u64, read_i64, write_u64, write_i64);
impl_read_to_type!(
    u128, i128, 16, read_u128, read_i128, write_u128, write_i128
);
pub trait GlobalSetTrait {
    fn set_fctx(&mut self, address: Option<u32>, value: i64);
    fn set_nfctx(&mut self, address: Option<u32>, value: i64);
    fn set_phase(&mut self, address: Option<u32>, value: i64);
    fn set_counter(&mut self, address: Option<u32>, value: i64);
}
pub trait MemoryRead {
    type AddressType;
    fn read(&self, addr: Self::AddressType, buf: &mut [u8]);
}
pub trait MemoryWrite {
    type AddressType;
    fn write(&mut self, addr: Self::AddressType, buf: &[u8]);
}
pub trait ContextregisterTrait:
    MemoryRead<AddressType = u16> + MemoryWrite<AddressType = u16>
{
    fn read_fctx_raw(&self) -> u8 {
        let mut work_value = [0u8; 1u64 as usize];
        self.read(0u64 as u16, &mut work_value[0..1]);
        let value = read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
        u8::try_from(value).unwrap()
    }
    fn write_fctx_raw(&mut self, param: u8) {
        let mut mem = [0u8; 1];
        self.read(0u64 as u16, &mut mem[0..1]);
        let mem = u8::from_le_bytes(mem);
        let mem =
            write_u8::<false>(param as u8, mem, 0u64 as usize, 4u64 as usize);
        self.write(0u64 as u16, &mem[0..1]);
    }
    fn read_fctx_disassembly(&self) -> i64 {
        i64::try_from(self.read_fctx_raw()).unwrap()
    }
    fn write_fctx_disassembly(&mut self, param: i64) {
        self.write_fctx_raw(param as u8)
    }
    fn read_fctx_execution(&self) -> u8 {
        self.read_fctx_raw()
    }
    fn write_fctx_execution(&mut self, param: u8) {
        self.write_fctx_raw(param)
    }
    fn fctx_display(&self) -> DisplayElement {
        meaning_number(true, self.read_fctx_raw())
    }
    fn read_nfctx_raw(&self) -> u8 {
        let mut work_value = [0u8; 1u64 as usize];
        self.read(0u64 as u16, &mut work_value[0..1]);
        let value = read_u8::<false>(work_value, 4u64 as usize, 4u64 as usize);
        u8::try_from(value).unwrap()
    }
    fn write_nfctx_raw(&mut self, param: u8) {
        let mut mem = [0u8; 1];
        self.read(0u64 as u16, &mut mem[0..1]);
        let mem = u8::from_le_bytes(mem);
        let mem =
            write_u8::<false>(param as u8, mem, 4u64 as usize, 4u64 as usize);
        self.write(0u64 as u16, &mem[0..1]);
    }
    fn read_nfctx_disassembly(&self) -> i64 {
        i64::try_from(self.read_nfctx_raw()).unwrap()
    }
    fn write_nfctx_disassembly(&mut self, param: i64) {
        self.write_nfctx_raw(param as u8)
    }
    fn read_nfctx_execution(&self) -> u8 {
        self.read_nfctx_raw()
    }
    fn write_nfctx_execution(&mut self, param: u8) {
        self.write_nfctx_raw(param)
    }
    fn nfctx_display(&self) -> DisplayElement {
        meaning_number(true, self.read_nfctx_raw())
    }
    fn read_phase_raw(&self) -> u8 {
        let mut work_value = [0u8; 1u64 as usize];
        self.read(1u64 as u16, &mut work_value[0..1]);
        let value = read_u8::<false>(work_value, 0u64 as usize, 2u64 as usize);
        u8::try_from(value).unwrap()
    }
    fn write_phase_raw(&mut self, param: u8) {
        let mut mem = [0u8; 1];
        self.read(1u64 as u16, &mut mem[0..1]);
        let mem = u8::from_le_bytes(mem);
        let mem =
            write_u8::<false>(param as u8, mem, 0u64 as usize, 2u64 as usize);
        self.write(1u64 as u16, &mem[0..1]);
    }
    fn read_phase_disassembly(&self) -> i64 {
        i64::try_from(self.read_phase_raw()).unwrap()
    }
    fn write_phase_disassembly(&mut self, param: i64) {
        self.write_phase_raw(param as u8)
    }
    fn read_phase_execution(&self) -> u8 {
        self.read_phase_raw()
    }
    fn write_phase_execution(&mut self, param: u8) {
        self.write_phase_raw(param)
    }
    fn phase_display(&self) -> DisplayElement {
        meaning_number(true, self.read_phase_raw())
    }
    fn read_counter_raw(&self) -> u8 {
        let mut work_value = [0u8; 1u64 as usize];
        self.read(1u64 as u16, &mut work_value[0..1]);
        let value = read_u8::<false>(work_value, 2u64 as usize, 4u64 as usize);
        u8::try_from(value).unwrap()
    }
    fn write_counter_raw(&mut self, param: u8) {
        let mut mem = [0u8; 1];
        self.read(1u64 as u16, &mut mem[0..1]);
        let mem = u8::from_le_bytes(mem);
        let mem =
            write_u8::<false>(param as u8, mem, 2u64 as usize, 4u64 as usize);
        self.write(1u64 as u16, &mem[0..1]);
    }
    fn read_counter_disassembly(&self) -> i64 {
        i64::try_from(self.read_counter_raw()).unwrap()
    }
    fn write_counter_disassembly(&mut self, param: i64) {
        self.write_counter_raw(param as u8)
    }
    fn read_counter_execution(&self) -> u8 {
        self.read_counter_raw()
    }
    fn write_counter_execution(&mut self, param: u8) {
        self.write_counter_raw(param)
    }
    fn counter_display(&self) -> DisplayElement {
        meaning_number(true, self.read_counter_raw())
    }
}
pub trait ContextTrait {
    type Typeregister: ContextregisterTrait;
    fn register(&self) -> &Self::Typeregister;
    fn register_mut(&mut self) -> &mut Self::Typeregister;
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextregisterStruct {
    pub chunk_0x0: [u8; 8u64 as usize],
}
impl ContextregisterTrait for ContextregisterStruct {}
impl MemoryRead for ContextregisterStruct {
    type AddressType = u16;
    fn read(&self, addr: Self::AddressType, buf: &mut [u8]) {
        let addr = <u64>::try_from(addr).unwrap();
        let buf_len = <u64>::try_from(buf.len()).unwrap();
        let addr_end = addr + buf_len;
        match (addr, addr_end) {
            (0u64..=7u64, 0u64..=8u64) => {
                let start = addr - 0u64;
                let end = usize::try_from(start + buf_len).unwrap();
                let start = usize::try_from(start).unwrap();
                buf.copy_from_slice(&self.chunk_0x0[start..end]);
            }
            _ => panic!("undefined mem {}:{}", addr, buf.len()),
        }
    }
}
impl MemoryWrite for ContextregisterStruct {
    type AddressType = u16;
    fn write(&mut self, addr: Self::AddressType, buf: &[u8]) {
        let addr = <u64>::try_from(addr).unwrap();
        let buf_len = <u64>::try_from(buf.len()).unwrap();
        let addr_end = addr + buf_len;
        match (addr, addr_end) {
            (0u64..=7u64, 0u64..=8u64) => {
                let start = addr - 0u64;
                let end = usize::try_from(start + buf_len).unwrap();
                let start = usize::try_from(start).unwrap();
                self.chunk_0x0[start..end].copy_from_slice(buf);
            }
            _ => panic!("undefined mem {}:{}", addr, buf.len()),
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct SpacesStruct {
    pub register: ContextregisterStruct,
}
impl ContextTrait for SpacesStruct {
    type Typeregister = ContextregisterStruct;
    fn register(&self) -> &Self::Typeregister {
        &self.register
    }
    fn register_mut(&mut self) -> &mut Self::Typeregister {
        &mut self.register
    }
}
fn meaning_number<T>(hex: bool, num: T) -> DisplayElement
where
    i64: TryFrom<T>,
    <i64 as TryFrom<T>>::Error: core::fmt::Debug,
{
    DisplayElement::Number(hex, i64::try_from(num).unwrap())
}
fn meaning_0_display<T>(num: T) -> DisplayElement
where
    u8: TryFrom<T>,
    <u8 as TryFrom<T>>::Error: core::fmt::Debug,
{
    let value = meaning_0_value(num.try_into().unwrap());
    DisplayElement::Register(value)
}
fn meaning_0_value<T>(num: T) -> Register
where
    u8: TryFrom<T>,
    <u8 as TryFrom<T>>::Error: core::fmt::Debug,
{
    match u8::try_from(num).unwrap() {
        0 => Register::r0,
        1 => Register::r1,
        2 => Register::r2,
        3 => Register::r3,
        4 => Register::r4,
        5 => Register::r5,
        6 => Register::r6,
        7 => Register::r7,
        8 => Register::r8,
        9 => Register::r9,
        10 => Register::r10,
        11 => Register::r11,
        12 => Register::r12,
        13 => Register::sp,
        14 => Register::lr,
        15 => Register::pc,
        _ => unreachable!("Invalid Attach Value"),
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op8(u8);
impl TokenField_op8 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_xsimm8(i8);
impl TokenField_xsimm8 {
    fn execution(&self) -> i8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_nnnn(u8);
impl TokenField_nnnn {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op1515(u8);
impl TokenField_op1515 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op1415(u8);
impl TokenField_op1415 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op1215(u8);
impl TokenField_op1215 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op1111(u8);
impl TokenField_op1111 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op0811(u8);
impl TokenField_op0811 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op0407(u8);
impl TokenField_op0407 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op0007(u8);
impl TokenField_op0007 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op0003(u8);
impl TokenField_op0003 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_op0303(u8);
impl TokenField_op0303 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_rd(u8);
impl TokenField_rd {
    fn execution(&self) -> Register {
        meaning_0_value(self.0)
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_0_display(self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_rs(u8);
impl TokenField_rs {
    fn execution(&self) -> Register {
        meaning_0_value(self.0)
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_0_display(self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_rt(u8);
impl TokenField_rt {
    fn execution(&self) -> Register {
        meaning_0_value(self.0)
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_0_display(self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_imm1213(u8);
impl TokenField_imm1213 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_imm0007(u8);
impl TokenField_imm0007 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_imm0003(u8);
impl TokenField_imm0003 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_simm1213(i8);
impl TokenField_simm1213 {
    fn execution(&self) -> i8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_simm0010(i16);
impl TokenField_simm0010 {
    fn execution(&self) -> i16 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_simm0411(i8);
impl TokenField_simm0411 {
    fn execution(&self) -> i8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_simm0007(i8);
impl TokenField_simm0007 {
    fn execution(&self) -> i8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_simm0003(i8);
impl TokenField_simm0003 {
    fn execution(&self) -> i8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_cc0911(u8);
impl TokenField_cc0911 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
#[derive(Clone, Copy, Debug)]
struct TokenField_cc0002(u8);
impl TokenField_cc0002 {
    fn execution(&self) -> u8 {
        self.0
    }
    fn disassembly(&self) -> i64 {
        i64::try_from(self.0).unwrap()
    }
    fn display(&self) -> DisplayElement {
        meaning_number(true, self.0)
    }
}
struct TokenParser<const LEN: usize>([u8; LEN]);
impl<const LEN: usize> TokenParser<LEN> {
    fn new(data: &[u8]) -> Option<Self> {
        let token_slice: &[u8] = data.get(..LEN)?;
        let token_data = <[u8; LEN]>::try_from(token_slice).unwrap();
        Some(Self(token_data))
    }
    fn TokenFieldop8(&self) -> TokenField_op8 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 8u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op8(inner_value)
    }
    fn TokenFieldxsimm8(&self) -> TokenField_xsimm8 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_i8::<false>(work_value, 0u64 as usize, 8u64 as usize);
            i8::try_from(value).unwrap()
        };
        TokenField_xsimm8(inner_value)
    }
    fn TokenFieldnnnn(&self) -> TokenField_nnnn {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_nnnn(inner_value)
    }
    fn TokenFieldop1515(&self) -> TokenField_op1515 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 7u64 as usize, 1u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op1515(inner_value)
    }
    fn TokenFieldop1415(&self) -> TokenField_op1415 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 6u64 as usize, 2u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op1415(inner_value)
    }
    fn TokenFieldop1215(&self) -> TokenField_op1215 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 4u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op1215(inner_value)
    }
    fn TokenFieldop1111(&self) -> TokenField_op1111 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 3u64 as usize, 1u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op1111(inner_value)
    }
    fn TokenFieldop0811(&self) -> TokenField_op0811 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op0811(inner_value)
    }
    fn TokenFieldop0407(&self) -> TokenField_op0407 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 4u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op0407(inner_value)
    }
    fn TokenFieldop0007(&self) -> TokenField_op0007 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 8u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op0007(inner_value)
    }
    fn TokenFieldop0003(&self) -> TokenField_op0003 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op0003(inner_value)
    }
    fn TokenFieldop0303(&self) -> TokenField_op0303 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 3u64 as usize, 1u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_op0303(inner_value)
    }
    fn TokenFieldrd(&self) -> TokenField_rd {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_rd(inner_value)
    }
    fn TokenFieldrs(&self) -> TokenField_rs {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 4u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_rs(inner_value)
    }
    fn TokenFieldrt(&self) -> TokenField_rt {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_rt(inner_value)
    }
    fn TokenFieldimm1213(&self) -> TokenField_imm1213 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 4u64 as usize, 2u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_imm1213(inner_value)
    }
    fn TokenFieldimm0007(&self) -> TokenField_imm0007 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 8u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_imm0007(inner_value)
    }
    fn TokenFieldimm0003(&self) -> TokenField_imm0003 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_imm0003(inner_value)
    }
    fn TokenFieldsimm1213(&self) -> TokenField_simm1213 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_i8::<false>(work_value, 4u64 as usize, 2u64 as usize);
            i8::try_from(value).unwrap()
        };
        TokenField_simm1213(inner_value)
    }
    fn TokenFieldsimm0010(&self) -> TokenField_simm0010 {
        let inner_value = {
            let mut work_value = [0u8; 2u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 2u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_i16::<false>(work_value, 0u64 as usize, 11u64 as usize);
            i16::try_from(value).unwrap()
        };
        TokenField_simm0010(inner_value)
    }
    fn TokenFieldsimm0411(&self) -> TokenField_simm0411 {
        let inner_value = {
            let mut work_value = [0u8; 2u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 2u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_i16::<false>(work_value, 4u64 as usize, 8u64 as usize);
            i8::try_from(value).unwrap()
        };
        TokenField_simm0411(inner_value)
    }
    fn TokenFieldsimm0007(&self) -> TokenField_simm0007 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_i8::<false>(work_value, 0u64 as usize, 8u64 as usize);
            i8::try_from(value).unwrap()
        };
        TokenField_simm0007(inner_value)
    }
    fn TokenFieldsimm0003(&self) -> TokenField_simm0003 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_i8::<false>(work_value, 0u64 as usize, 4u64 as usize);
            i8::try_from(value).unwrap()
        };
        TokenField_simm0003(inner_value)
    }
    fn TokenFieldcc0911(&self) -> TokenField_cc0911 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 1u64 as usize;
            let token_end = 2u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 1u64 as usize, 3u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_cc0911(inner_value)
    }
    fn TokenFieldcc0002(&self) -> TokenField_cc0002 {
        let inner_value = {
            let mut work_value = [0u8; 1u64 as usize];
            let work_start = 0u64 as usize;
            let work_end = 1u64 as usize;
            let token_start = 0u64 as usize;
            let token_end = 1u64 as usize;
            work_value[work_start..work_end]
                .copy_from_slice(&self.0[token_start..token_end]);
            let value =
                read_u8::<false>(work_value, 0u64 as usize, 3u64 as usize);
            u8::try_from(value).unwrap()
        };
        TokenField_cc0002(inner_value)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Register {
    r0,
    r1,
    r2,
    r3,
    r4,
    r5,
    r6,
    r7,
    r8,
    r9,
    r10,
    r11,
    r12,
    sp,
    lr,
    pc,
    C,
    Z,
    N,
    V,
    r0l,
    r0h,
    r1l,
    r1h,
    r2l,
    r2h,
    r3l,
    r3h,
    r4l,
    r4h,
    r5l,
    r5h,
    r6l,
    r6h,
    r7l,
    r7h,
    r8l,
    r8h,
    r9l,
    r9h,
    r10l,
    r10h,
    r11l,
    r11h,
    r12l,
    r12h,
    spl,
    sph,
    lrl,
    lrh,
    pcl,
    pch,
    contextreg,
}
impl core::fmt::Display for Register {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::r0 => write!(f, "r0"),
            Self::r1 => write!(f, "r1"),
            Self::r2 => write!(f, "r2"),
            Self::r3 => write!(f, "r3"),
            Self::r4 => write!(f, "r4"),
            Self::r5 => write!(f, "r5"),
            Self::r6 => write!(f, "r6"),
            Self::r7 => write!(f, "r7"),
            Self::r8 => write!(f, "r8"),
            Self::r9 => write!(f, "r9"),
            Self::r10 => write!(f, "r10"),
            Self::r11 => write!(f, "r11"),
            Self::r12 => write!(f, "r12"),
            Self::sp => write!(f, "sp"),
            Self::lr => write!(f, "lr"),
            Self::pc => write!(f, "pc"),
            Self::C => write!(f, "C"),
            Self::Z => write!(f, "Z"),
            Self::N => write!(f, "N"),
            Self::V => write!(f, "V"),
            Self::r0l => write!(f, "r0l"),
            Self::r0h => write!(f, "r0h"),
            Self::r1l => write!(f, "r1l"),
            Self::r1h => write!(f, "r1h"),
            Self::r2l => write!(f, "r2l"),
            Self::r2h => write!(f, "r2h"),
            Self::r3l => write!(f, "r3l"),
            Self::r3h => write!(f, "r3h"),
            Self::r4l => write!(f, "r4l"),
            Self::r4h => write!(f, "r4h"),
            Self::r5l => write!(f, "r5l"),
            Self::r5h => write!(f, "r5h"),
            Self::r6l => write!(f, "r6l"),
            Self::r6h => write!(f, "r6h"),
            Self::r7l => write!(f, "r7l"),
            Self::r7h => write!(f, "r7h"),
            Self::r8l => write!(f, "r8l"),
            Self::r8h => write!(f, "r8h"),
            Self::r9l => write!(f, "r9l"),
            Self::r9h => write!(f, "r9h"),
            Self::r10l => write!(f, "r10l"),
            Self::r10h => write!(f, "r10h"),
            Self::r11l => write!(f, "r11l"),
            Self::r11h => write!(f, "r11h"),
            Self::r12l => write!(f, "r12l"),
            Self::r12h => write!(f, "r12h"),
            Self::spl => write!(f, "spl"),
            Self::sph => write!(f, "sph"),
            Self::lrl => write!(f, "lrl"),
            Self::lrh => write!(f, "lrh"),
            Self::pcl => write!(f, "pcl"),
            Self::pch => write!(f, "pch"),
            Self::contextreg => write!(f, "contextreg"),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum DisplayElement {
    Literal(&'static str),
    Register(Register),
    Number(bool, i64),
}
impl core::fmt::Display for DisplayElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(lit) => lit.fmt(f),
            Self::Register(reg) => reg.fmt(f),
            Self::Number(hex, value) => match (*hex, value.is_negative()) {
                (true, true) => write!(f, "-0x{:x}", value.abs()),
                (true, false) => write!(f, "0x{:x}", value),
                (false, _) => value.fmt(f),
            },
        }
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:31:1"]
#[derive(Clone, Debug)]
struct instructionVar0 {
    instruction: Box<Tableinstruction>,
}
impl instructionVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        self.instruction.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 0u64 as u32;
        if context_instance.register().read_phase_disassembly() != 0i64 {
            return None;
        }
        let tmp = 1i64;
        context_instance.register_mut().write_phase_disassembly(tmp);
        let instruction = if let Some((len, table)) = Tableinstruction::parse(
            tokens_current,
            &mut context_instance,
            inst_start,
        ) {
            block_0_len = block_0_len.max(len as u32);
            Box::new(table)
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { instruction }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:217:1"]
#[derive(Clone, Debug)]
struct instructionVar1 {}
impl instructionVar1 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("ret")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 4i64 {
            return None;
        }
        if token_parser.TokenFieldop0007().disassembly() != 0i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:227:1"]
#[derive(Clone, Debug)]
struct instructionVar2 {}
impl instructionVar2 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Literal("user_three")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 3i64 {
            return None;
        }
        if token_parser.TokenFieldop0007().disassembly() != 0i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:232:1"]
#[derive(Clone, Debug)]
struct instructionVar3 {}
impl instructionVar3 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Literal("unimpl")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 8i64 {
            return None;
        }
        if token_parser.TokenFieldop0007().disassembly() != 0i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:197:1"]
#[derive(Clone, Debug)]
struct instructionVar4 {
    rs: TokenField_rs,
}
impl instructionVar4 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("inv"),
            DisplayElement::Literal("  "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 14i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:198:1"]
#[derive(Clone, Debug)]
struct instructionVar5 {
    rs: TokenField_rs,
}
impl instructionVar5 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("neg"),
            DisplayElement::Literal("  "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 14i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 1i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:204:1"]
#[derive(Clone, Debug)]
struct instructionVar6 {
    CC: TableCC,
    Rel82: TableRel82,
}
impl instructionVar6 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("sk")];
        display.extend_from_slice(&extend);
        self.CC.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 8i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 0i64 {
            return None;
        }
        if token_parser.TokenFieldop0407().disassembly() != 0i64 {
            return None;
        }
        if token_parser.TokenFieldop0303().disassembly() != 0i64 {
            return None;
        }
        let CC = if let Some((len, table)) =
            TableCC::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let Rel82 = if let Some((len, table)) =
            TableRel82::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { CC, Rel82 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:214:1"]
#[derive(Clone, Debug)]
struct instructionVar7 {
    rs: TokenField_rs,
}
impl instructionVar7 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("push"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 2i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:215:1"]
#[derive(Clone, Debug)]
struct instructionVar8 {
    rs: TokenField_rs,
}
impl instructionVar8 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("pop"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 3i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:219:1"]
#[derive(Clone, Debug)]
struct instructionVar9 {
    rs: TokenField_rs,
}
impl instructionVar9 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("call"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 6i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:225:1"]
#[derive(Clone, Debug)]
struct instructionVar10 {
    rs: TokenField_rs,
}
impl instructionVar10 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("user_one"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:226:1"]
#[derive(Clone, Debug)]
struct instructionVar11 {
    rs: TokenField_rs,
}
impl instructionVar11 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("user_two"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 2i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:230:1"]
#[derive(Clone, Debug)]
struct instructionVar12 {
    rs: TokenField_rs,
}
impl instructionVar12 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("user_six"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 6i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:65:1"]
#[derive(Clone, Debug)]
struct instructionVar13 {
    rs: TokenField_rs,
}
impl instructionVar13 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("cop1"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        if context_instance.register().read_nfctx_disassembly() != 1i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:66:1"]
#[derive(Clone, Debug)]
struct instructionVar14 {
    rs: TokenField_rs,
}
impl instructionVar14 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("cop2"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        if context_instance.register().read_nfctx_disassembly() != 2i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:67:1"]
#[derive(Clone, Debug)]
struct instructionVar15 {
    rs: TokenField_rs,
}
impl instructionVar15 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("cop3"),
            DisplayElement::Literal(" "),
            self.rs.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0003().disassembly() != 0i64 {
            return None;
        }
        if context_instance.register().read_nfctx_disassembly() != 3i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:208:1"]
#[derive(Clone, Debug)]
struct instructionVar16 {
    rs: TokenField_rs,
    COND: TableCOND,
}
impl instructionVar16 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("br")];
        display.extend_from_slice(&extend);
        self.COND.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal(" "), self.rs.display()];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 0i64 {
            return None;
        }
        if token_parser.TokenFieldop0303().disassembly() != 0i64 {
            return None;
        }
        let COND = if let Some((len, table)) =
            TableCOND::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { COND, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:209:1"]
#[derive(Clone, Debug)]
struct instructionVar17 {
    rs: TokenField_rs,
    COND: TableCOND,
}
impl instructionVar17 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Literal("brds")];
        display.extend_from_slice(&extend);
        self.COND.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal(" "), self.rs.display()];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop0303().disassembly() != 0i64 {
            return None;
        }
        let COND = if let Some((len, table)) =
            TableCOND::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { COND, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:223:1"]
#[derive(Clone, Debug)]
struct instructionVar18 {
    rs: TokenField_rs,
    COND: TableCOND,
}
impl instructionVar18 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Literal("call")];
        display.extend_from_slice(&extend);
        self.COND.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal(" "), self.rs.display()];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 6i64 {
            return None;
        }
        if token_parser.TokenFieldop0303().disassembly() != 1i64 {
            return None;
        }
        let COND = if let Some((len, table)) =
            TableCOND::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { COND, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:54:1"]
#[derive(Clone, Debug)]
struct instructionVar19 {
    imm0003: TokenField_imm0003,
    Imm4: TableImm4,
}
impl instructionVar19 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        global_set.set_fctx(
            Some(inst_next),
            context.register().read_fctx_disassembly(),
        );
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("fctx"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 9i64 {
            return None;
        }
        if token_parser.TokenFieldrs().disassembly() != 0i64 {
            return None;
        }
        let tmp = token_parser.TokenFieldimm0003().disassembly();
        context_instance.register_mut().write_fctx_disassembly(tmp);
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let imm0003 = token_parser.TokenFieldimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, imm0003 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:58:1"]
#[derive(Clone, Debug)]
struct instructionVar20 {
    Imm4: TableImm4,
    nfctxSetAddr: TablenfctxSetAddr,
}
impl instructionVar20 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("nfctx"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.nfctxSetAddr.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal(",")];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 0u64 as u32;
        let mut sub_pattern_c30 = |tokens: &[u8], context_param: &mut T| {
            let mut pattern_len = 0 as u32;
            let mut context_instance = context_param.clone();
            let mut tokens = tokens;
            let mut block_0_len = 2u64 as u32;
            let token_parser = <TokenParser<2usize>>::new(tokens)?;
            if context_instance.register().read_phase_disassembly() != 1i64 {
                return None;
            }
            if token_parser.TokenFieldop1215().disassembly() != 13i64 {
                return None;
            }
            if token_parser.TokenFieldop0811().disassembly() != 9i64 {
                return None;
            }
            if token_parser.TokenFieldrs().disassembly() != 2i64 {
                return None;
            }
            let Imm4 = if let Some((len, table)) =
                TableImm4::parse(tokens, &mut context_instance, inst_start)
            {
                block_0_len = block_0_len.max(len as u32);
                table
            } else {
                return None;
            };
            pattern_len += block_0_len;
            tokens = &tokens[usize::try_from(block_0_len).unwrap()..];
            *context_param = context_instance;
            Some(((Imm4), (), pattern_len))
        };
        let ((mut Imm4), (), sub_len) =
            sub_pattern_c30(tokens_current, &mut context_instance)?;
        block_0_len = block_0_len.max(sub_len);
        let nfctxSetAddr = if let Some((len, table)) = TablenfctxSetAddr::parse(
            tokens_current,
            &mut context_instance,
            inst_start,
        ) {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, nfctxSetAddr }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:59:1"]
#[derive(Clone, Debug)]
struct instructionVar21 {
    imm0003: TokenField_imm0003,
    Imm4: TableImm4,
}
impl instructionVar21 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        global_set.set_nfctx(
            Some(inst_next),
            context.register().read_nfctx_disassembly(),
        );
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("nfctx"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 9i64 {
            return None;
        }
        if token_parser.TokenFieldrs().disassembly() != 1i64 {
            return None;
        }
        let tmp = token_parser.TokenFieldimm0003().disassembly();
        context_instance.register_mut().write_nfctx_disassembly(tmp);
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let imm0003 = token_parser.TokenFieldimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, imm0003 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:77:1"]
#[derive(Clone, Debug)]
struct instructionVar22 {
    imm0003: TokenField_imm0003,
    NopCnt: TableNopCnt,
    NopByte: TableNopByte,
}
impl instructionVar22 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal("nop"), DisplayElement::Literal(" ")];
        display.extend_from_slice(&extend);
        self.NopCnt.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 9i64 {
            return None;
        }
        if token_parser.TokenFieldrs().disassembly() != 3i64 {
            return None;
        }
        let tmp = token_parser.TokenFieldimm0003().disassembly();
        context_instance
            .register_mut()
            .write_counter_disassembly(tmp);
        let NopCnt = if let Some((len, table)) = TableNopCnt::parse(
            tokens_current,
            &mut context_instance,
            inst_start,
        ) {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let imm0003 = token_parser.TokenFieldimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        let mut block_1_len = 0u64 as u32;
        let NopByte = if let Some((len, table)) = TableNopByte::parse(
            tokens_current,
            &mut context_instance,
            inst_start,
        ) {
            block_1_len = block_1_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_1_len;
        tokens_current =
            &tokens_current[usize::try_from(block_1_len).unwrap()..];
        *context = context_instance;
        Some((
            pattern_len,
            Self {
                NopCnt,
                NopByte,
                imm0003,
            },
        ))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:158:1"]
#[derive(Clone, Debug)]
struct instructionVar23 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar23 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("add"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:159:1"]
#[derive(Clone, Debug)]
struct instructionVar24 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar24 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("sub"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 1i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:160:1"]
#[derive(Clone, Debug)]
struct instructionVar25 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar25 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("rsub"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 2i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:161:1"]
#[derive(Clone, Debug)]
struct instructionVar26 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar26 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("mul"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 3i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:162:1"]
#[derive(Clone, Debug)]
struct instructionVar27 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar27 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("div"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 4i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:163:1"]
#[derive(Clone, Debug)]
struct instructionVar28 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar28 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("mod"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 5i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:164:1"]
#[derive(Clone, Debug)]
struct instructionVar29 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar29 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("cmp"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 6i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:165:1"]
#[derive(Clone, Debug)]
struct instructionVar30 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar30 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("ucmp"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 7i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:167:1"]
#[derive(Clone, Debug)]
struct instructionVar31 {
    rs: TokenField_rs,
    Simm4: TableSimm4,
}
impl instructionVar31 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("add"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 8i64 {
            return None;
        }
        let Simm4 = if let Some((len, table)) =
            TableSimm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:168:1"]
#[derive(Clone, Debug)]
struct instructionVar32 {
    rs: TokenField_rs,
    Simm4: TableSimm4,
}
impl instructionVar32 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("sub"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 9i64 {
            return None;
        }
        let Simm4 = if let Some((len, table)) =
            TableSimm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:169:1"]
#[derive(Clone, Debug)]
struct instructionVar33 {
    rs: TokenField_rs,
    Simm4: TableSimm4,
}
impl instructionVar33 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("rsub"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 10i64 {
            return None;
        }
        let Simm4 = if let Some((len, table)) =
            TableSimm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:170:1"]
#[derive(Clone, Debug)]
struct instructionVar34 {
    rs: TokenField_rs,
    Simm4: TableSimm4,
}
impl instructionVar34 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("mul"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 11i64 {
            return None;
        }
        let Simm4 = if let Some((len, table)) =
            TableSimm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:171:1"]
#[derive(Clone, Debug)]
struct instructionVar35 {
    rs: TokenField_rs,
    Simm4: TableSimm4,
}
impl instructionVar35 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("div"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 12i64 {
            return None;
        }
        let Simm4 = if let Some((len, table)) =
            TableSimm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:172:1"]
#[derive(Clone, Debug)]
struct instructionVar36 {
    rs: TokenField_rs,
    Imm4: TableImm4,
}
impl instructionVar36 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("mod"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 13i64 {
            return None;
        }
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:173:1"]
#[derive(Clone, Debug)]
struct instructionVar37 {
    rs: TokenField_rs,
    Simm4: TableSimm4,
}
impl instructionVar37 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("cmp"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 14i64 {
            return None;
        }
        let Simm4 = if let Some((len, table)) =
            TableSimm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:174:1"]
#[derive(Clone, Debug)]
struct instructionVar38 {
    rs: TokenField_rs,
    Imm4: TableImm4,
}
impl instructionVar38 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("ucmp"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 12i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 15i64 {
            return None;
        }
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:176:1"]
#[derive(Clone, Debug)]
struct instructionVar39 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar39 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("and"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 0i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:177:1"]
#[derive(Clone, Debug)]
struct instructionVar40 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar40 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("or"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 1i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:178:1"]
#[derive(Clone, Debug)]
struct instructionVar41 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar41 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("xor"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 2i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:179:1"]
#[derive(Clone, Debug)]
struct instructionVar42 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar42 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("lsr"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 3i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:180:1"]
#[derive(Clone, Debug)]
struct instructionVar43 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar43 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("asr"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 4i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:181:1"]
#[derive(Clone, Debug)]
struct instructionVar44 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar44 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("lsl"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 5i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:192:1"]
#[derive(Clone, Debug)]
struct instructionVar45 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar45 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("saa"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 8i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:194:1"]
#[derive(Clone, Debug)]
struct instructionVar46 {
    rs: TokenField_rs,
    Imm4: TableImm4,
}
impl instructionVar46 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("lsr"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 11i64 {
            return None;
        }
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:195:1"]
#[derive(Clone, Debug)]
struct instructionVar47 {
    rs: TokenField_rs,
    Imm4: TableImm4,
}
impl instructionVar47 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("asr"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 12i64 {
            return None;
        }
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:196:1"]
#[derive(Clone, Debug)]
struct instructionVar48 {
    rs: TokenField_rs,
    Imm4: TableImm4,
}
impl instructionVar48 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("lsl"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Imm4.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 13i64 {
            return None;
        }
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm4, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:200:1"]
#[derive(Clone, Debug)]
struct instructionVar49 {
    rs: TokenField_rs,
    RT: TableRT,
}
impl instructionVar49 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("load"),
            DisplayElement::Literal("  "),
            self.rs.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.RT.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 6i64 {
            return None;
        }
        let RT = if let Some((len, table)) =
            TableRT::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { RT, rs }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:201:1"]
#[derive(Clone, Debug)]
struct instructionVar50 {
    rt: TokenField_rt,
    RS: TableRS,
}
impl instructionVar50 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("store"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.RS.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal(", "), self.rt.display()];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 7i64 {
            return None;
        }
        let RS = if let Some((len, table)) =
            TableRS::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { RS, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:202:1"]
#[derive(Clone, Debug)]
struct instructionVar51 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar51 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("mov"),
            DisplayElement::Literal("   "),
            self.rs.display(),
            DisplayElement::Literal(", "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 13i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 15i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:206:1"]
#[derive(Clone, Debug)]
struct instructionVar52 {
    COND: TableCOND,
    Rel82: TableRel82,
}
impl instructionVar52 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("br")];
        display.extend_from_slice(&extend);
        self.COND.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal(" ")];
        display.extend_from_slice(&extend);
        self.Rel82.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 14i64 {
            return None;
        }
        if token_parser.TokenFieldop0303().disassembly() != 0i64 {
            return None;
        }
        let COND = if let Some((len, table)) =
            TableCOND::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let Rel82 = if let Some((len, table)) =
            TableRel82::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { COND, Rel82 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:207:1"]
#[derive(Clone, Debug)]
struct instructionVar53 {
    COND: TableCOND,
    Rel82: TableRel82,
}
impl instructionVar53 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Literal("brds")];
        display.extend_from_slice(&extend);
        self.COND.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal(" ")];
        display.extend_from_slice(&extend);
        self.Rel82.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 14i64 {
            return None;
        }
        if token_parser.TokenFieldop0303().disassembly() != 1i64 {
            return None;
        }
        let COND = if let Some((len, table)) =
            TableCOND::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let Rel82 = if let Some((len, table)) =
            TableRel82::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { COND, Rel82 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:218:1"]
#[derive(Clone, Debug)]
struct instructionVar54 {
    Rel8: TableRel8,
}
impl instructionVar54 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("callds"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.Rel8.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 5i64 {
            return None;
        }
        let Rel8 = if let Some((len, table)) =
            TableRel8::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Rel8 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:228:1"]
#[derive(Clone, Debug)]
struct instructionVar55 {
    rs: TokenField_rs,
    rt: TokenField_rt,
}
impl instructionVar55 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 5usize] = [
            DisplayElement::Literal("user_four"),
            DisplayElement::Literal(" "),
            self.rs.display(),
            DisplayElement::Literal(" "),
            self.rt.display(),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 4i64 {
            return None;
        }
        let rs = token_parser.TokenFieldrs();
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs, rt }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:229:1"]
#[derive(Clone, Debug)]
struct instructionVar56 {
    Rel8: TableRel8,
}
impl instructionVar56 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("user_five"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.Rel8.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 10i64 {
            return None;
        }
        if token_parser.TokenFieldop0811().disassembly() != 5i64 {
            return None;
        }
        let Rel8 = if let Some((len, table)) =
            TableRel8::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Rel8 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:220:1"]
#[derive(Clone, Debug)]
struct instructionVar57 {
    Rel11: TableRel11,
}
impl instructionVar57 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("call"),
            DisplayElement::Literal(" "),
        ];
        display.extend_from_slice(&extend);
        self.Rel11.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1215().disassembly() != 15i64 {
            return None;
        }
        if token_parser.TokenFieldop1111().disassembly() != 1i64 {
            return None;
        }
        let Rel11 = if let Some((len, table)) =
            TableRel11::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Rel11 }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:156:1"]
#[derive(Clone, Debug)]
struct instructionVar58 {
    rd: TokenField_rd,
    Simm10: TableSimm10,
}
impl instructionVar58 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("simm"),
            DisplayElement::Literal(" "),
            self.rd.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Simm10.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1415().disassembly() != 2i64 {
            return None;
        }
        let Simm10 = if let Some((len, table)) = TableSimm10::parse(
            tokens_current,
            &mut context_instance,
            inst_start,
        ) {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rd = token_parser.TokenFieldrd();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Simm10, rd }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:155:1"]
#[derive(Clone, Debug)]
struct instructionVar59 {
    rd: TokenField_rd,
    Imm10: TableImm10,
}
impl instructionVar59 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 4usize] = [
            DisplayElement::Literal("imm"),
            DisplayElement::Literal("  "),
            self.rd.display(),
            DisplayElement::Literal(", "),
        ];
        display.extend_from_slice(&extend);
        self.Imm10.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop1515().disassembly() != 0i64 {
            return None;
        }
        let Imm10 = if let Some((len, table)) =
            TableImm10::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let rd = token_parser.TokenFieldrd();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { Imm10, rd }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:76:1"]
#[derive(Clone, Debug)]
struct instructionVar60 {
    One: TableOne,
}
impl instructionVar60 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal("nop"), DisplayElement::Literal(" ")];
        display.extend_from_slice(&extend);
        self.One.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 1u64 as u32;
        let token_parser = <TokenParser<1usize>>::new(tokens_current)?;
        if context_instance.register().read_phase_disassembly() != 1i64 {
            return None;
        }
        if token_parser.TokenFieldop8().disassembly() != 247i64 {
            return None;
        }
        let One = if let Some((len, table)) =
            TableOne::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { One }))
    }
}
#[derive(Clone, Debug)]
enum Tableinstruction {
    Var0(instructionVar0),
    Var1(instructionVar1),
    Var2(instructionVar2),
    Var3(instructionVar3),
    Var4(instructionVar4),
    Var5(instructionVar5),
    Var6(instructionVar6),
    Var7(instructionVar7),
    Var8(instructionVar8),
    Var9(instructionVar9),
    Var10(instructionVar10),
    Var11(instructionVar11),
    Var12(instructionVar12),
    Var13(instructionVar13),
    Var14(instructionVar14),
    Var15(instructionVar15),
    Var16(instructionVar16),
    Var17(instructionVar17),
    Var18(instructionVar18),
    Var19(instructionVar19),
    Var20(instructionVar20),
    Var21(instructionVar21),
    Var22(instructionVar22),
    Var23(instructionVar23),
    Var24(instructionVar24),
    Var25(instructionVar25),
    Var26(instructionVar26),
    Var27(instructionVar27),
    Var28(instructionVar28),
    Var29(instructionVar29),
    Var30(instructionVar30),
    Var31(instructionVar31),
    Var32(instructionVar32),
    Var33(instructionVar33),
    Var34(instructionVar34),
    Var35(instructionVar35),
    Var36(instructionVar36),
    Var37(instructionVar37),
    Var38(instructionVar38),
    Var39(instructionVar39),
    Var40(instructionVar40),
    Var41(instructionVar41),
    Var42(instructionVar42),
    Var43(instructionVar43),
    Var44(instructionVar44),
    Var45(instructionVar45),
    Var46(instructionVar46),
    Var47(instructionVar47),
    Var48(instructionVar48),
    Var49(instructionVar49),
    Var50(instructionVar50),
    Var51(instructionVar51),
    Var52(instructionVar52),
    Var53(instructionVar53),
    Var54(instructionVar54),
    Var55(instructionVar55),
    Var56(instructionVar56),
    Var57(instructionVar57),
    Var58(instructionVar58),
    Var59(instructionVar59),
    Var60(instructionVar60),
}
impl Tableinstruction {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var1(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var2(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var3(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var4(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var5(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var6(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var7(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var8(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var9(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var10(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var11(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var12(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var13(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var14(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var15(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var16(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var17(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var18(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var19(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var20(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var21(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var22(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var23(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var24(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var25(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var26(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var27(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var28(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var29(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var30(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var31(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var32(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var33(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var34(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var35(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var36(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var37(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var38(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var39(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var40(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var41(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var42(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var43(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var44(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var45(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var46(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var47(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var48(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var49(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var50(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var51(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var52(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var53(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var54(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var55(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var56(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var57(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var58(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var59(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var60(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) = instructionVar0::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar1::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var1(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar2::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var2(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar3::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var3(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar4::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var4(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar5::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var5(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar6::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var6(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar7::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var7(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar8::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var8(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar9::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var9(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar10::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var10(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar11::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var11(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar12::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var12(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar13::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var13(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar14::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var14(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar15::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var15(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar16::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var16(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar17::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var17(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar18::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var18(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar19::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var19(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar20::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var20(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar21::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var21(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar22::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var22(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar23::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var23(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar24::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var24(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar25::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var25(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar26::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var26(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar27::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var27(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar28::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var28(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar29::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var29(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar30::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var30(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar31::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var31(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar32::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var32(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar33::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var33(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar34::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var34(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar35::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var35(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar36::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var36(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar37::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var37(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar38::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var38(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar39::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var39(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar40::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var40(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar41::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var41(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar42::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var42(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar43::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var43(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar44::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var44(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar45::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var45(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar46::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var46(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar47::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var47(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar48::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var48(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar49::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var49(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar50::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var50(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar51::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var51(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar52::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var52(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar53::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var53(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar54::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var54(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar55::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var55(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar56::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var56(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar57::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var57(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar58::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var58(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar59::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var59(parsed)));
        }
        if let Some((inst_len, parsed)) = instructionVar60::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var60(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:107:1"]
#[derive(Clone, Debug)]
struct Simm4Var0 {
    simm0003: TokenField_simm0003,
}
impl Simm4Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal("#"), self.simm0003.display()];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let simm0003 = token_parser.TokenFieldsimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { simm0003 }))
    }
}
#[derive(Clone, Debug)]
enum TableSimm4 {
    Var0(Simm4Var0),
}
impl TableSimm4 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Simm4Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:108:1"]
#[derive(Clone, Debug)]
struct Simm10Var0 {
    simm1213: TokenField_simm1213,
    imm0007: TokenField_imm0007,
}
impl Simm10Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut computed: i64 = 0;
        computed = (self
            .simm1213
            .disassembly()
            .checked_shl(u32::try_from(8i64).unwrap())
            .unwrap_or(0)
            | self.imm0007.disassembly());
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("#"),
            DisplayElement::Number(true, computed),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut computed: i64 = 0;
        computed = (token_parser
            .TokenFieldsimm1213()
            .disassembly()
            .checked_shl(u32::try_from(8i64).unwrap())
            .unwrap_or(0)
            | token_parser.TokenFieldimm0007().disassembly());
        let simm1213 = token_parser.TokenFieldsimm1213();
        let imm0007 = token_parser.TokenFieldimm0007();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { simm1213, imm0007 }))
    }
}
#[derive(Clone, Debug)]
enum TableSimm10 {
    Var0(Simm10Var0),
}
impl TableSimm10 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Simm10Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:110:1"]
#[derive(Clone, Debug)]
struct Imm4Var0 {
    imm0003: TokenField_imm0003,
}
impl Imm4Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal("#"), self.imm0003.display()];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let imm0003 = token_parser.TokenFieldimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { imm0003 }))
    }
}
#[derive(Clone, Debug)]
enum TableImm4 {
    Var0(Imm4Var0),
}
impl TableImm4 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Imm4Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:111:1"]
#[derive(Clone, Debug)]
struct Imm10Var0 {
    imm1213: TokenField_imm1213,
    imm0007: TokenField_imm0007,
}
impl Imm10Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut computed: i64 = 0;
        computed = (self
            .imm1213
            .disassembly()
            .checked_shl(u32::try_from(8i64).unwrap())
            .unwrap_or(0)
            | self.imm0007.disassembly());
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("#"),
            DisplayElement::Number(true, computed),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut computed: i64 = 0;
        computed = (token_parser
            .TokenFieldimm1213()
            .disassembly()
            .checked_shl(u32::try_from(8i64).unwrap())
            .unwrap_or(0)
            | token_parser.TokenFieldimm0007().disassembly());
        let imm1213 = token_parser.TokenFieldimm1213();
        let imm0007 = token_parser.TokenFieldimm0007();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { imm1213, imm0007 }))
    }
}
#[derive(Clone, Debug)]
enum TableImm10 {
    Var0(Imm10Var0),
}
impl TableImm10 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Imm10Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:113:1"]
#[derive(Clone, Debug)]
struct Rel8Var0 {
    simm0007: TokenField_simm0007,
}
impl Rel8Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(self.simm0007.disassembly());
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Number(true, addr)];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(token_parser.TokenFieldsimm0007().disassembly());
        let simm0007 = token_parser.TokenFieldsimm0007();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { simm0007 }))
    }
}
#[derive(Clone, Debug)]
enum TableRel8 {
    Var0(Rel8Var0),
}
impl TableRel8 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Rel8Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:114:1"]
#[derive(Clone, Debug)]
struct Rel82Var0 {
    simm0411: TokenField_simm0411,
}
impl Rel82Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(self.simm0411.disassembly());
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Number(true, addr)];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(token_parser.TokenFieldsimm0411().disassembly());
        let simm0411 = token_parser.TokenFieldsimm0411();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { simm0411 }))
    }
}
#[derive(Clone, Debug)]
enum TableRel82 {
    Var0(Rel82Var0),
}
impl TableRel82 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Rel82Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:115:1"]
#[derive(Clone, Debug)]
struct Rel11Var0 {
    simm0010: TokenField_simm0010,
}
impl Rel11Var0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(self.simm0010.disassembly());
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Number(true, addr)];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(token_parser.TokenFieldsimm0010().disassembly());
        let simm0010 = token_parser.TokenFieldsimm0010();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { simm0010 }))
    }
}
#[derive(Clone, Debug)]
enum TableRel11 {
    Var0(Rel11Var0),
}
impl TableRel11 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            Rel11Var0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:117:1"]
#[derive(Clone, Debug)]
struct RSVar0 {
    rs: TokenField_rs,
}
impl RSVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("["),
            self.rs.display(),
            DisplayElement::Literal("]"),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let rs = token_parser.TokenFieldrs();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rs }))
    }
}
#[derive(Clone, Debug)]
enum TableRS {
    Var0(RSVar0),
}
impl TableRS {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            RSVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:118:1"]
#[derive(Clone, Debug)]
struct RTVar0 {
    rt: TokenField_rt,
}
impl RTVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 3usize] = [
            DisplayElement::Literal("["),
            self.rt.display(),
            DisplayElement::Literal("]"),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let rt = token_parser.TokenFieldrt();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { rt }))
    }
}
#[derive(Clone, Debug)]
enum TableRT {
    Var0(RTVar0),
}
impl TableRT {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            RTVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:120:1"]
#[derive(Clone, Debug)]
struct CCVar0 {}
impl CCVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("eq")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 0i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:121:1"]
#[derive(Clone, Debug)]
struct CCVar1 {}
impl CCVar1 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("ne")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 1i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:122:1"]
#[derive(Clone, Debug)]
struct CCVar2 {}
impl CCVar2 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("lt")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 2i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:123:1"]
#[derive(Clone, Debug)]
struct CCVar3 {}
impl CCVar3 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("le")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 3i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:124:1"]
#[derive(Clone, Debug)]
struct CCVar4 {}
impl CCVar4 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("lo")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 4i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:125:1"]
#[derive(Clone, Debug)]
struct CCVar5 {}
impl CCVar5 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("mi")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 5i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:126:1"]
#[derive(Clone, Debug)]
struct CCVar6 {}
impl CCVar6 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 1usize] = [DisplayElement::Literal("vs")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 6i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:127:1"]
#[derive(Clone, Debug)]
struct CCVar7 {}
impl CCVar7 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let extend: [DisplayElement; 2usize] =
            [DisplayElement::Literal(""), DisplayElement::Literal("")];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 7i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[derive(Clone, Debug)]
enum TableCC {
    Var0(CCVar0),
    Var1(CCVar1),
    Var2(CCVar2),
    Var3(CCVar3),
    Var4(CCVar4),
    Var5(CCVar5),
    Var6(CCVar6),
    Var7(CCVar7),
}
impl TableCC {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var1(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var2(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var3(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var4(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var5(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var6(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var7(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            CCVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar1::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var1(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar2::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var2(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar3::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var3(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar4::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var4(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar5::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var5(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar6::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var6(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CCVar7::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var7(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:129:1"]
#[derive(Clone, Debug)]
struct CONDVar0 {
    CC: TableCC,
}
impl CONDVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        self.CC.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 0u64 as u32;
        let CC = if let Some((len, table)) =
            TableCC::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { CC }))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toyInstructions.sinc:130:1"]
#[derive(Clone, Debug)]
struct CONDVar1 {
    CC: TableCC,
}
impl CONDVar1 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        self.CC.display_extend(
            display, context, inst_start, inst_next, global_set,
        );
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        if token_parser.TokenFieldcc0002().disassembly() != 7i64 {
            return None;
        }
        let CC = if let Some((len, table)) =
            TableCC::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { CC }))
    }
}
#[derive(Clone, Debug)]
enum TableCOND {
    Var0(CONDVar0),
    Var1(CONDVar1),
}
impl TableCOND {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var1(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            CONDVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        if let Some((inst_len, parsed)) =
            CONDVar1::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var1(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:56:1"]
#[derive(Clone, Debug)]
struct nfctxSetAddrVar0 {
    xsimm8: TokenField_xsimm8,
    imm0003: TokenField_imm0003,
    Imm4: TableImm4,
}
impl nfctxSetAddrVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(self.xsimm8.disassembly());
        global_set.set_nfctx(
            Some(u32::try_from(addr).unwrap()),
            context.register().read_nfctx_disassembly(),
        );
        let extend: [DisplayElement; 1usize] =
            [DisplayElement::Number(true, addr)];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut addr: i64 = 0;
        addr = i64::try_from(inst_start)
            .unwrap()
            .wrapping_add(token_parser.TokenFieldxsimm8().disassembly());
        let tmp = token_parser.TokenFieldimm0003().disassembly();
        context_instance.register_mut().write_nfctx_disassembly(tmp);
        let Imm4 = if let Some((len, table)) =
            TableImm4::parse(tokens_current, &mut context_instance, inst_start)
        {
            block_0_len = block_0_len.max(len as u32);
            table
        } else {
            return None;
        };
        let imm0003 = token_parser.TokenFieldimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        let mut block_1_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let xsimm8 = token_parser.TokenFieldxsimm8();
        pattern_len += block_1_len;
        tokens_current =
            &tokens_current[usize::try_from(block_1_len).unwrap()..];
        *context = context_instance;
        Some((
            pattern_len,
            Self {
                Imm4,
                imm0003,
                xsimm8,
            },
        ))
    }
}
#[derive(Clone, Debug)]
enum TablenfctxSetAddr {
    Var0(nfctxSetAddrVar0),
}
impl TablenfctxSetAddr {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) = nfctxSetAddrVar0::parse(
            tokens_param,
            &mut context_current,
            inst_start,
        ) {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:69:1"]
#[derive(Clone, Debug)]
struct NopCntVar0 {
    imm0003: TokenField_imm0003,
}
impl NopCntVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut cnt: i64 = 0;
        cnt = self.imm0003.disassembly().wrapping_add(2i64);
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("#"),
            DisplayElement::Number(true, cnt),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 2u64 as u32;
        let token_parser = <TokenParser<2usize>>::new(tokens_current)?;
        let mut cnt: i64 = 0;
        cnt = token_parser
            .TokenFieldimm0003()
            .disassembly()
            .wrapping_add(2i64);
        let imm0003 = token_parser.TokenFieldimm0003();
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { imm0003 }))
    }
}
#[derive(Clone, Debug)]
enum TableNopCnt {
    Var0(NopCntVar0),
}
impl TableNopCnt {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            NopCntVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:71:1"]
#[derive(Clone, Debug)]
struct NopByteVar0 {}
impl NopByteVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 0u64 as u32;
        if context_instance.register().read_counter_disassembly() != 0i64 {
            return None;
        }
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:72:1"]
#[derive(Clone, Debug)]
struct NopByteVar1 {
    NopByte: Box<TableNopByte>,
}
impl NopByteVar1 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 0u64 as u32;
        let tmp = context_instance
            .register()
            .read_counter_disassembly()
            .wrapping_sub(1i64);
        context_instance
            .register_mut()
            .write_counter_disassembly(tmp);
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        let mut block_1_len = 1u64 as u32;
        let token_parser = <TokenParser<1usize>>::new(tokens_current)?;
        let nnnn = token_parser.TokenFieldnnnn();
        pattern_len += block_1_len;
        tokens_current =
            &tokens_current[usize::try_from(block_1_len).unwrap()..];
        let mut block_2_len = 0u64 as u32;
        let NopByte = if let Some((len, table)) = TableNopByte::parse(
            tokens_current,
            &mut context_instance,
            inst_start,
        ) {
            block_2_len = block_2_len.max(len as u32);
            Box::new(table)
        } else {
            return None;
        };
        pattern_len += block_2_len;
        tokens_current =
            &tokens_current[usize::try_from(block_2_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self { NopByte }))
    }
}
#[derive(Clone, Debug)]
enum TableNopByte {
    Var0(NopByteVar0),
    Var1(NopByteVar1),
}
impl TableNopByte {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
            Self::Var1(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            NopByteVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        if let Some((inst_len, parsed)) =
            NopByteVar1::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var1(parsed)));
        }
        None
    }
}
#[doc = "Constructor at /home/rbran/src/ghidra/Ghidra/Processors/Toy/data/languages/toy_builder.sinc:74:1"]
#[derive(Clone, Debug)]
struct OneVar0 {}
impl OneVar0 {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        let mut cnt: i64 = 0;
        cnt = 1i64;
        let extend: [DisplayElement; 2usize] = [
            DisplayElement::Literal("#"),
            DisplayElement::Number(true, cnt),
        ];
        display.extend_from_slice(&extend);
    }
    fn parse<T>(
        mut tokens_current: &[u8],
        context: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut pattern_len = 0 as u32;
        let mut context_instance = context.clone();
        let mut block_0_len = 0u64 as u32;
        let mut cnt: i64 = 0;
        cnt = 1i64;
        pattern_len += block_0_len;
        tokens_current =
            &tokens_current[usize::try_from(block_0_len).unwrap()..];
        *context = context_instance;
        Some((pattern_len, Self {}))
    }
}
#[derive(Clone, Debug)]
enum TableOne {
    Var0(OneVar0),
}
impl TableOne {
    fn display_extend<T>(
        &self,
        display: &mut Vec<DisplayElement>,
        context: &T,
        inst_start: u32,
        inst_next: u32,
        global_set_param: &mut impl GlobalSetTrait,
    ) where
        T: ContextTrait + Clone,
    {
        match self {
            Self::Var0(x) => x.display_extend(
                display,
                context,
                inst_start,
                inst_next,
                global_set_param,
            ),
        }
    }
    fn parse<T>(
        tokens_param: &[u8],
        context_param: &mut T,
        inst_start: u32,
    ) -> Option<(u32, Self)>
    where
        T: ContextTrait + Clone,
    {
        let mut context_current = context_param.clone();
        if let Some((inst_len, parsed)) =
            OneVar0::parse(tokens_param, &mut context_current, inst_start)
        {
            *context_param = context_current;
            return Some((inst_len, Self::Var0(parsed)));
        }
        None
    }
}
pub fn parse_instruction<T>(
    tokens: &[u8],
    context: &mut T,
    inst_start: u32,
    global_set: &mut impl GlobalSetTrait,
) -> Option<(u32, Vec<DisplayElement>)>
where
    T: ContextTrait + Clone,
{
    let (inst_len, instruction) =
        Tableinstruction::parse(tokens, context, inst_start)?;
    let inst_next = inst_start + inst_len;
    let mut display = vec![];
    instruction.display_extend(
        &mut display,
        context,
        inst_start,
        inst_next,
        global_set,
    );
    Some((inst_next, display))
}
