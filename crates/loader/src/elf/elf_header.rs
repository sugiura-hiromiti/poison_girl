use {
	crate::elf::{
		Context, ELF_IDENT_SIZE, ElfHeaderIdent, ElfType, ProgramHeader,
		SHT_SYMTAB, SectionHeader, StringTable, SymbolTable, get_string_table,
		read_le_bytes,
	},
	alloc::vec::Vec,
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, poison_girl_err,
	},
};

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfHeader
{
	pub ident: ElfHeaderIdent,
	pub ty: ElfType,
	pub machine: u16,
	pub version: u32,
	pub entry: u64,
	pub program_header_offset: u64,
	pub section_header_offset: u64,
	pub flags: u32,
	pub elf_header_size: u16,
	pub program_header_entry_size: u16,
	pub program_header_count: u16,
	pub section_header_entry_size: u16,
	pub section_header_count: u16,
	pub section_header_index_of_section_name_string_table: u16,
}

impl ElfHeader
{
	/// Intel 80386
	pub const EM_386: u16 = 3;
	/// Freescale 56800EX DSC
	pub const EM_56800EX: u16 = 200;
	/// Motorola MC68HC05 microcontroller
	pub const EM_68HC05: u16 = 72;
	/// Motorola MC68HC08 microcontroller
	pub const EM_68HC08: u16 = 71;
	/// Motorola MC68HC11 microcontroller
	pub const EM_68HC11: u16 = 70;
	/// Motorola M68HC12
	pub const EM_68HC12: u16 = 53;
	/// Motorola MC68HC16 microcontroller
	pub const EM_68HC16: u16 = 69;
	/// Motorola m68k family
	pub const EM_68K: u16 = 4;
	/// Renesas 78KOR
	pub const EM_78KOR: u16 = 199;
	/// Intel 8051 and variants
	pub const EM_8051: u16 = 165;
	/// Intel 80860
	pub const EM_860: u16 = 7;
	/// Motorola m88k family
	pub const EM_88K: u16 = 5;
	/// Intel 80960
	pub const EM_960: u16 = 19;
	// reserved 182
	/// ARM AARCH64
	pub const EM_AARCH64: u16 = 183;
	/// Altera Nios II
	pub const EM_ALTERA_NIOS2: u16 = 113;
	/// AMD GPU
	pub const EM_AMDGPU: u16 = 224;
	/// Argonaut RISC Core
	pub const EM_ARC: u16 = 45;
	/// Arca RISC
	pub const EM_ARCA: u16 = 109;
	/// ARC International ARCompact
	pub const EM_ARC_COMPACT: u16 = 93;
	/// Synopsys ARCompact V2
	pub const EM_ARC_COMPACT2: u16 = 195;
	/// ARM
	pub const EM_ARM: u16 = 40;
	/// Atmel AVR 8-bit microcontroller
	pub const EM_AVR: u16 = 83;
	// reserved 184
	/// Amtel 32-bit microprocessor
	pub const EM_AVR32: u16 = 185;
	/// Beyond BA1
	pub const EM_BA1: u16 = 201;
	/// Beyond BA2
	pub const EM_BA2: u16 = 202;
	/// Analog Devices Blackfin DSP
	pub const EM_BLACKFIN: u16 = 106;
	/// Linux BPF -- in-kernel virtual machine
	pub const EM_BPF: u16 = 247;
	/// Infineon C16x/XC16x
	pub const EM_C166: u16 = 116;
	/// Paneve CDP
	pub const EM_CDP: u16 = 215;
	/// Freescale Communication Engine RISC
	pub const EM_CE: u16 = 119;
	/// CloudShield
	pub const EM_CLOUDSHIELD: u16 = 192;
	/// Cognitive Smart Memory Processor
	pub const EM_COGE: u16 = 216;
	/// Motorola Coldfire
	pub const EM_COLDFIRE: u16 = 52;
	/// Bluechip CoolEngine
	pub const EM_COOL: u16 = 217;
	/// KIPO-KAIST Core-A 1st gen.
	pub const EM_COREA_1ST: u16 = 193;
	/// KIPO-KAIST Core-A 2nd gen.
	pub const EM_COREA_2ND: u16 = 194;
	/// National Semi. CompactRISC
	pub const EM_CR: u16 = 103;
	/// National Semi. CompactRISC CR16
	pub const EM_CR16: u16 = 177;
	/// Cray NV2 vector architecture
	pub const EM_CRAYNV2: u16 = 172;
	/// Axis Communications 32-bit emb.proc
	pub const EM_CRIS: u16 = 76;
	/// National Semi. CompactRISC CRX
	pub const EM_CRX: u16 = 114;
	/// C-SKY
	pub const EM_CSKY: u16 = 252;
	/// CSR Kalimba
	pub const EM_CSR_KALIMBA: u16 = 219;
	/// NVIDIA CUDA
	pub const EM_CUDA: u16 = 190;
	/// Cypress M8C
	pub const EM_CYPRESS_M8C: u16 = 161;
	/// Mitsubishi D10V
	pub const EM_D10V: u16 = 85;
	/// Mitsubishi D30V
	pub const EM_D30V: u16 = 86;
	/// New Japan Radio (NJR) 24-bit DSP
	pub const EM_DSP24: u16 = 136;
	/// Microchip Technology dsPIC30F
	pub const EM_DSPIC30F: u16 = 118;
	/// Icera Semi. Deep Execution Processor
	pub const EM_DXP: u16 = 112;
	/// Cyan Technology eCOG16
	pub const EM_ECOG16: u16 = 176;
	/// Cyan Technology eCOG1X
	pub const EM_ECOG1X: u16 = 168;
	/// Cyan Technology eCOG2
	pub const EM_ECOG2: u16 = 134;
	/// KM211 KMX16
	pub const EM_EMX16: u16 = 212;
	/// KM211 KMX8
	pub const EM_EMX8: u16 = 213;
	/// Freescale Extended Time Processing Unit
	pub const EM_ETPU: u16 = 178;
	/// eXcess configurable cpu
	pub const EM_EXCESS: u16 = 111;
	/// Fujitsu F2MC16
	pub const EM_F2MC16: u16 = 104;
	/// Digital Alpha
	pub const EM_FAKE_ALPHA: u16 = 41;
	/// Element 14 64-bit DSP Processor
	pub const EM_FIREPATH: u16 = 78;
	/// Fujitsu FR20
	pub const EM_FR20: u16 = 37;
	/// Fujitsu FR30
	pub const EM_FR30: u16 = 84;
	/// FTDI Chip FT32
	pub const EM_FT32: u16 = 222;
	/// Siemens FX66 microcontroller
	pub const EM_FX66: u16 = 66;
	/// Hitachi H8S
	pub const EM_H8S: u16 = 48;
	/// Hitachi H8/300
	pub const EM_H8_300: u16 = 46;
	/// Hitachi H8/300H
	pub const EM_H8_300H: u16 = 47;
	/// Hitachi H8/500
	pub const EM_H8_500: u16 = 49;
	/// Harvard University machine-independent object files
	pub const EM_HUANY: u16 = 81;
	/// Intel MCU
	pub const EM_IAMCU: u16 = 6;
	/// Intel Merced
	pub const EM_IA_64: u16 = 50;
	/// Intel Graphics Technology
	pub const EM_INTELGT: u16 = 205;
	/// Ubicom IP2xxx
	pub const EM_IP2K: u16 = 101;
	/// Infineon Technologies 32-bit emb.proc
	pub const EM_JAVELIN: u16 = 77;
	/// Intel K10M
	pub const EM_K10M: u16 = 181;
	// reserved 206-209
	/// KM211 KM32
	pub const EM_KM32: u16 = 210;
	/// KM211 KMX32
	pub const EM_KMX32: u16 = 211;
	/// KM211 KVARC
	pub const EM_KVARC: u16 = 214;
	/// Intel L10M
	pub const EM_L10M: u16 = 180;
	/// RISC for Lattice FPGA
	pub const EM_LATTICEMICO32: u16 = 138;
	// Loongarch 64
	pub const EM_LOONGARCH: u16 = 258;
	/// Renesas M16C
	pub const EM_M16C: u16 = 117;
	/// AT&T WE 32100
	pub const EM_M32: u16 = 1;
	/// Renesas M32C
	pub const EM_M32C: u16 = 120;
	/// Mitsubishi M32R
	pub const EM_M32R: u16 = 88;
	/// M2000 Reconfigurable RISC
	pub const EM_MANIK: u16 = 171;
	/// MAX processor
	pub const EM_MAX: u16 = 102;
	/// Dallas Semi. MAXQ30 mc
	pub const EM_MAXQ30: u16 = 169;
	/// Microchip 8-bit PIC(r)
	pub const EM_MCHP_PIC: u16 = 204;
	/// MCST Elbrus
	pub const EM_MCST_ELBRUS: u16 = 175;
	/// Toyota ME16 processor
	pub const EM_ME16: u16 = 59;
	/// Imagination Tech. META
	pub const EM_METAG: u16 = 174;
	/// Xilinx MicroBlaze
	pub const EM_MICROBLAZE: u16 = 189;
	/// MIPS R3000 big-endian
	pub const EM_MIPS: u16 = 8;
	/// MIPS R3000 little-endian
	pub const EM_MIPS_RS3_LE: u16 = 10;
	/// Stanford MIPS-X
	pub const EM_MIPS_X: u16 = 51;
	/// Fujitsu MMA Multimedia Accelerator
	pub const EM_MMA: u16 = 54;
	// reserved 145-159
	/// STMicroelectronics 64bit VLIW DSP
	pub const EM_MMDSP_PLUS: u16 = 160;
	/// Donald Knuth's educational 64-bit proc
	pub const EM_MMIX: u16 = 80;
	/// Matsushita MN10200
	pub const EM_MN10200: u16 = 90;
	/// Matsushita MN10300
	pub const EM_MN10300: u16 = 89;
	/// Moxie processor
	pub const EM_MOXIE: u16 = 223;
	/// Texas Instruments msp430
	pub const EM_MSP430: u16 = 105;
	/// Sony nCPU embeeded RISC
	pub const EM_NCPU: u16 = 56;
	/// Denso NDR1 microprocessor
	pub const EM_NDR1: u16 = 57;
	/// Andes Tech. compact code emb. RISC
	pub const EM_NDS32: u16 = 167;
	pub const EM_NONE: u16 = 0;
	/// Nanoradio Optimized RISC
	pub const EM_NORC: u16 = 218;
	/// National Semi. 32000
	pub const EM_NS32K: u16 = 97;
	pub const EM_NUM: u16 = 248;
	/// Open8 RISC
	pub const EM_OPEN8: u16 = 196;
	/// OpenRISC 32-bit embedded processor
	pub const EM_OPENRISC: u16 = 92;
	// reserved 11-14
	/// HPPA
	pub const EM_PARISC: u16 = 15;
	/// Siemens PCP
	pub const EM_PCP: u16 = 55;
	/// Digital PDP-10
	pub const EM_PDP10: u16 = 64;
	/// Digital PDP-11
	pub const EM_PDP11: u16 = 65;
	/// Sony DSP Processor
	pub const EM_PDSP: u16 = 63;
	/// picoJava
	pub const EM_PJ: u16 = 91;
	/// PowerPC
	pub const EM_PPC: u16 = 20;
	/// PowerPC 64-bit
	pub const EM_PPC64: u16 = 21;
	/// SiTera Prism
	pub const EM_PRISM: u16 = 82;
	/// QUALCOMM DSP6
	pub const EM_QDSP6: u16 = 164;
	/// Renesas R32C
	pub const EM_R32C: u16 = 162;
	/// Motorola RCE
	pub const EM_RCE: u16 = 39;
	/// TRW RH-32
	pub const EM_RH32: u16 = 38;
	// reserved 225-242
	/// RISC-V
	pub const EM_RISCV: u16 = 243;
	/// Renesas RL78
	pub const EM_RL78: u16 = 197;
	/// Freescale RS08
	pub const EM_RS08: u16 = 132;
	/// Renesas RX
	pub const EM_RX: u16 = 173;
	/// IBM System/370
	pub const EM_S370: u16 = 9;
	/// IBM S390
	pub const EM_S390: u16 = 22;
	/// Sunplus S+core7 RISC
	pub const EM_SCORE7: u16 = 135;
	/// Sharp embedded microprocessor
	pub const EM_SEP: u16 = 108;
	/// Seiko Epson C17
	pub const EM_SE_C17: u16 = 139;
	/// Seiko Epson S1C33 family
	pub const EM_SE_C33: u16 = 107;
	/// Hitachi SH
	pub const EM_SH: u16 = 42;
	/// Analog Devices SHARC family
	pub const EM_SHARC: u16 = 133;
	/// Infineon Tech. SLE9X
	pub const EM_SLE9X: u16 = 179;
	/// Trebia SNP 1000
	pub const EM_SNP1K: u16 = 99;
	/// SUN SPARC
	pub const EM_SPARC: u16 = 2;
	/// Sun's "v8plus"
	pub const EM_SPARC32PLUS: u16 = 18;
	/// SPARC v9 64-bit
	pub const EM_SPARCV9: u16 = 43;
	/// IBM SPU/SPC
	pub const EM_SPU: u16 = 23;
	/// STMicroelectronic ST100 processor
	pub const EM_ST100: u16 = 60;
	/// STMicroelectronics ST19 8 bit mc
	pub const EM_ST19: u16 = 74;
	/// STMicroelectronics ST200
	pub const EM_ST200: u16 = 100;
	/// STmicroelectronics ST7 8 bit mc
	pub const EM_ST7: u16 = 68;
	/// STMicroelectronics ST9+ 8/16 mc
	pub const EM_ST9PLUS: u16 = 67;
	/// Motorola Start*Core processor
	pub const EM_STARCORE: u16 = 58;
	/// STMicroelectronics STM8
	pub const EM_STM8: u16 = 186;
	/// STMicroelectronics STxP7x
	pub const EM_STXP7X: u16 = 166;
	/// Silicon Graphics SVx
	pub const EM_SVX: u16 = 73;
	/// Tileta TILE64
	pub const EM_TILE64: u16 = 187;
	/// Tilera TILE-Gx
	pub const EM_TILEGX: u16 = 191;
	/// Tilera TILEPro
	pub const EM_TILEPRO: u16 = 188;
	/// Advanced Logic Corp. Tinyj emb.fam
	pub const EM_TINYJ: u16 = 61;
	/// Texas Instruments App. Specific RISC
	pub const EM_TI_ARP32: u16 = 143;
	/// Texas Instruments TMS320C2000 DSP
	pub const EM_TI_C2000: u16 = 141;
	/// Texas Instruments TMS320C55x DSP
	pub const EM_TI_C5500: u16 = 142;
	/// Texas Instruments TMS320C6000 DSP
	pub const EM_TI_C6000: u16 = 140;
	/// Texas Instruments Prog. Realtime Unit
	pub const EM_TI_PRU: u16 = 144;
	/// Thompson Multimedia General Purpose Proc
	pub const EM_TMM_GPP: u16 = 96;
	/// Tenor Network TPC
	pub const EM_TPC: u16 = 98;
	/// Siemens Tricore
	pub const EM_TRICORE: u16 = 44;
	/// NXP Semi. TriMedia
	pub const EM_TRIMEDIA: u16 = 163;
	// reserved 121-130
	/// Altium TSK3000
	pub const EM_TSK3000: u16 = 131;
	/// PKU-Unity & MPRC Peking Uni. mc series
	pub const EM_UNICORE: u16 = 110;
	// reserved 24-35
	/// NEC V800 series
	pub const EM_V800: u16 = 36;
	/// NEC v850
	pub const EM_V850: u16 = 87;
	/// Digital VAX
	pub const EM_VAX: u16 = 75;
	/// Alphamosaic VideoCore
	pub const EM_VIDEOCORE: u16 = 95;
	/// Broadcom VideoCore III
	pub const EM_VIDEOCORE3: u16 = 137;
	/// Broadcom VideoCore V
	pub const EM_VIDEOCORE5: u16 = 198;
	/// Controls and Data Services VISIUMcore
	pub const EM_VISIUM: u16 = 221;
	// reserved 16
	/// Fujitsu VPP500
	pub const EM_VPP500: u16 = 17;
	/// AMD x86-64 architecture
	pub const EM_X86_64: u16 = 62;
	/// XMOS xCORE
	pub const EM_XCORE: u16 = 203;
	/// Motorola XGATE
	pub const EM_XGATE: u16 = 115;
	/// New Japan Radio (NJR) 16-bit DSP
	pub const EM_XIMO16: u16 = 170;
	/// Tensilica Xtensa Architecture
	pub const EM_XTENSA: u16 = 94;
	/// Zilog Z80
	pub const EM_Z80: u16 = 220;
	/// LSI Logic 16-bit DSP Processor
	pub const EM_ZSP: u16 = 79;

	pub fn parse(binary: &[u8],) -> PoisonGirlB<Self,>
	{
		let ident = &binary[..ELF_IDENT_SIZE];
		let ident = ElfHeaderIdent::new(ident,)?;
		let remain = &binary[ELF_IDENT_SIZE..];
		header_flag_fields(ident, remain,)
	}

	pub fn program_headers(
		&self,
		binary: &[u8],
	) -> PoisonGirlB<Vec<ProgramHeader,>,>
	{
		let mut offset = self.program_header_offset as usize;
		let count = self.program_header_count as usize;
		ProgramHeader::parse(binary, &mut offset, count,)
	}

	pub fn section_headers(
		&self,
		binary: &[u8],
	) -> PoisonGirlB<Vec<SectionHeader,>,>
	{
		let mut offset = self.section_header_offset as usize;
		let count = self.section_header_count as usize;
		SectionHeader::parse(binary, &mut offset, count,)
	}

	/// # Return
	/// 返り値は(セクションヘッダストリングテーブル, シンボルテーブル,
	/// シンボルテーブルのストリングテーブル)
	pub(crate) fn section_tables(
		&self,
		ctx: &Context,
		binary: &[u8],
		section_headers: &[SectionHeader],
	) -> PoisonGirlB<(StringTable, SymbolTable, StringTable,),>
	{
		let string_table_index =
			self.section_header_index_of_section_name_string_table as usize;
		let section_header_string_table =
			get_string_table(&section_headers, string_table_index, binary,)?;

		let mut symbol_table = SymbolTable::default();
		let mut string_table_for_symbol_table = StringTable::default();
		if let Some(section_header,) = section_headers
			.iter()
			.rfind(|section_header| section_header.ty == SHT_SYMTAB,)
		{
			let size = section_header.entry_size;
			let count = if size == 0 { 0 } else { section_header.size / size };
			symbol_table = SymbolTable::parse(
				binary,
				section_header.offset as usize,
				count as usize,
				ctx,
			)?;
			string_table_for_symbol_table = get_string_table(
				&section_headers,
				section_header.link as usize,
				binary,
			)?;
		}

		X((
			section_header_string_table,
			symbol_table,
			string_table_for_symbol_table,
		),)
	}

	pub(super) fn is_64(&self,) -> bool
	{
		self.ident.is_64()
	}

	pub(super) fn is_lib(&self,) -> bool
	{
		self.ty.is_lib()
	}

	pub(super) fn is_little_endian(&self,) -> bool
	{
		self.ident.is_little_endian()
	}
}

fn header_flag_fields(
	ident: ElfHeaderIdent,
	ident_remain: &[u8],
) -> PoisonGirlB<ElfHeader,>
{
	let offset = &mut 0;

	macro_rules! fields {
		($field:ident) => {
			let $field =
				read_le_bytes(offset, ident_remain,).ok_or_else(|| {
					let field = stringify!($field);
					poison_girl_err!(poison_girl_no_std_error::ElfParseError::EndOfBinary{
						parser_pos: field,
						stage: poison_girl_no_std_error::ElfParseStage::Header
					})
				})?;
		};
		($($fields:ident,)*)=>{
			$(
				fields!($fields);
			)*
		};
	}

	let ty: u16 = read_le_bytes(offset, ident_remain,).ok_or(
		poison_girl_err!(ElfParseError::EndOfBinary {
			parser_pos: "ty",
			stage:      ElfParseStage::Header,
		}),
	)?;
	let ty = ElfType::try_from(ty,)?;
	fields!(
		machine,
		version,
		entry,
		program_header_offset,
		section_header_offset,
		flags,
		elf_header_size,
		program_header_entry_size,
		program_header_count,
		section_header_entry_size,
		section_header_count,
		section_header_index_of_section_name_string_table,
	);

	X(ElfHeader {
		ident,
		ty,
		machine,
		version,
		entry,
		program_header_offset,
		section_header_offset,
		flags,
		elf_header_size,
		program_header_entry_size,
		program_header_count,
		section_header_entry_size,
		section_header_count,
		section_header_index_of_section_name_string_table,
	},)
}
