#![doc = r" SPIR-V instruction definitions"]
#![doc = r""]
#![doc = r" This file is auto-generated, do not modify manually."]
use crate::{core::*, op::*};
use core::fmt::{self, Display};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Code(pub(crate) u16);
impl Code {
    pub const NOP: Self = Self(0);
    pub const UNDEF: Self = Self(1);
    pub const SOURCE_CONTINUED: Self = Self(2);
    pub const SOURCE: Self = Self(3);
    pub const SOURCE_EXTENSION: Self = Self(4);
    pub const NAME: Self = Self(5);
    pub const MEMBER_NAME: Self = Self(6);
    pub const STRING: Self = Self(7);
    pub const LINE: Self = Self(8);
    pub const EXTENSION: Self = Self(10);
    pub const EXT_INST_IMPORT: Self = Self(11);
    pub const EXT_INST: Self = Self(12);
    pub const MEMORY_MODEL: Self = Self(14);
    pub const ENTRY_POINT: Self = Self(15);
    pub const EXECUTION_MODE: Self = Self(16);
    pub const CAPABILITY: Self = Self(17);
    pub const TYPE_VOID: Self = Self(19);
    pub const TYPE_BOOL: Self = Self(20);
    pub const TYPE_INT: Self = Self(21);
    pub const TYPE_FLOAT: Self = Self(22);
    pub const TYPE_VECTOR: Self = Self(23);
    pub const TYPE_MATRIX: Self = Self(24);
    pub const TYPE_IMAGE: Self = Self(25);
    pub const TYPE_SAMPLER: Self = Self(26);
    pub const TYPE_SAMPLED_IMAGE: Self = Self(27);
    pub const TYPE_ARRAY: Self = Self(28);
    pub const TYPE_RUNTIME_ARRAY: Self = Self(29);
    pub const TYPE_STRUCT: Self = Self(30);
    pub const TYPE_OPAQUE: Self = Self(31);
    pub const TYPE_POINTER: Self = Self(32);
    pub const TYPE_FUNCTION: Self = Self(33);
    pub const TYPE_EVENT: Self = Self(34);
    pub const TYPE_DEVICE_EVENT: Self = Self(35);
    pub const TYPE_RESERVE_ID: Self = Self(36);
    pub const TYPE_QUEUE: Self = Self(37);
    pub const TYPE_PIPE: Self = Self(38);
    pub const TYPE_FORWARD_POINTER: Self = Self(39);
    pub const CONSTANT_TRUE: Self = Self(41);
    pub const CONSTANT_FALSE: Self = Self(42);
    pub const CONSTANT: Self = Self(43);
    pub const CONSTANT_COMPOSITE: Self = Self(44);
    pub const CONSTANT_SAMPLER: Self = Self(45);
    pub const CONSTANT_NULL: Self = Self(46);
    pub const SPEC_CONSTANT_TRUE: Self = Self(48);
    pub const SPEC_CONSTANT_FALSE: Self = Self(49);
    pub const SPEC_CONSTANT: Self = Self(50);
    pub const SPEC_CONSTANT_COMPOSITE: Self = Self(51);
    pub const SPEC_CONSTANT_OP: Self = Self(52);
    pub const FUNCTION: Self = Self(54);
    pub const FUNCTION_PARAMETER: Self = Self(55);
    pub const FUNCTION_END: Self = Self(56);
    pub const FUNCTION_CALL: Self = Self(57);
    pub const VARIABLE: Self = Self(59);
    pub const IMAGE_TEXEL_POINTER: Self = Self(60);
    pub const LOAD: Self = Self(61);
    pub const STORE: Self = Self(62);
    pub const COPY_MEMORY: Self = Self(63);
    pub const COPY_MEMORY_SIZED: Self = Self(64);
    pub const ACCESS_CHAIN: Self = Self(65);
    pub const IN_BOUNDS_ACCESS_CHAIN: Self = Self(66);
    pub const PTR_ACCESS_CHAIN: Self = Self(67);
    pub const ARRAY_LENGTH: Self = Self(68);
    pub const GENERIC_PTR_MEM_SEMANTICS: Self = Self(69);
    pub const IN_BOUNDS_PTR_ACCESS_CHAIN: Self = Self(70);
    pub const DECORATE: Self = Self(71);
    pub const MEMBER_DECORATE: Self = Self(72);
    pub const DECORATION_GROUP: Self = Self(73);
    pub const GROUP_DECORATE: Self = Self(74);
    pub const GROUP_MEMBER_DECORATE: Self = Self(75);
    pub const VECTOR_EXTRACT_DYNAMIC: Self = Self(77);
    pub const VECTOR_INSERT_DYNAMIC: Self = Self(78);
    pub const VECTOR_SHUFFLE: Self = Self(79);
    pub const COMPOSITE_CONSTRUCT: Self = Self(80);
    pub const COMPOSITE_EXTRACT: Self = Self(81);
    pub const COMPOSITE_INSERT: Self = Self(82);
    pub const COPY_OBJECT: Self = Self(83);
    pub const TRANSPOSE: Self = Self(84);
    pub const SAMPLED_IMAGE: Self = Self(86);
    pub const IMAGE_SAMPLE_IMPLICIT_LOD: Self = Self(87);
    pub const IMAGE_SAMPLE_EXPLICIT_LOD: Self = Self(88);
    pub const IMAGE_SAMPLE_DREF_IMPLICIT_LOD: Self = Self(89);
    pub const IMAGE_SAMPLE_DREF_EXPLICIT_LOD: Self = Self(90);
    pub const IMAGE_SAMPLE_PROJ_IMPLICIT_LOD: Self = Self(91);
    pub const IMAGE_SAMPLE_PROJ_EXPLICIT_LOD: Self = Self(92);
    pub const IMAGE_SAMPLE_PROJ_DREF_IMPLICIT_LOD: Self = Self(93);
    pub const IMAGE_SAMPLE_PROJ_DREF_EXPLICIT_LOD: Self = Self(94);
    pub const IMAGE_FETCH: Self = Self(95);
    pub const IMAGE_GATHER: Self = Self(96);
    pub const IMAGE_DREF_GATHER: Self = Self(97);
    pub const IMAGE_READ: Self = Self(98);
    pub const IMAGE_WRITE: Self = Self(99);
    pub const IMAGE: Self = Self(100);
    pub const IMAGE_QUERY_FORMAT: Self = Self(101);
    pub const IMAGE_QUERY_ORDER: Self = Self(102);
    pub const IMAGE_QUERY_SIZE_LOD: Self = Self(103);
    pub const IMAGE_QUERY_SIZE: Self = Self(104);
    pub const IMAGE_QUERY_LOD: Self = Self(105);
    pub const IMAGE_QUERY_LEVELS: Self = Self(106);
    pub const IMAGE_QUERY_SAMPLES: Self = Self(107);
    pub const CONVERT_FTO_U: Self = Self(109);
    pub const CONVERT_FTO_S: Self = Self(110);
    pub const CONVERT_STO_F: Self = Self(111);
    pub const CONVERT_UTO_F: Self = Self(112);
    pub const UCONVERT: Self = Self(113);
    pub const SCONVERT: Self = Self(114);
    pub const FCONVERT: Self = Self(115);
    pub const QUANTIZE_TO_F16: Self = Self(116);
    pub const CONVERT_PTR_TO_U: Self = Self(117);
    pub const SAT_CONVERT_STO_U: Self = Self(118);
    pub const SAT_CONVERT_UTO_S: Self = Self(119);
    pub const CONVERT_UTO_PTR: Self = Self(120);
    pub const PTR_CAST_TO_GENERIC: Self = Self(121);
    pub const GENERIC_CAST_TO_PTR: Self = Self(122);
    pub const GENERIC_CAST_TO_PTR_EXPLICIT: Self = Self(123);
    pub const BITCAST: Self = Self(124);
    pub const SNEGATE: Self = Self(126);
    pub const FNEGATE: Self = Self(127);
    pub const IADD: Self = Self(128);
    pub const FADD: Self = Self(129);
    pub const ISUB: Self = Self(130);
    pub const FSUB: Self = Self(131);
    pub const IMUL: Self = Self(132);
    pub const FMUL: Self = Self(133);
    pub const UDIV: Self = Self(134);
    pub const SDIV: Self = Self(135);
    pub const FDIV: Self = Self(136);
    pub const UMOD: Self = Self(137);
    pub const SREM: Self = Self(138);
    pub const SMOD: Self = Self(139);
    pub const FREM: Self = Self(140);
    pub const FMOD: Self = Self(141);
    pub const VECTOR_TIMES_SCALAR: Self = Self(142);
    pub const MATRIX_TIMES_SCALAR: Self = Self(143);
    pub const VECTOR_TIMES_MATRIX: Self = Self(144);
    pub const MATRIX_TIMES_VECTOR: Self = Self(145);
    pub const MATRIX_TIMES_MATRIX: Self = Self(146);
    pub const OUTER_PRODUCT: Self = Self(147);
    pub const DOT: Self = Self(148);
    pub const IADD_CARRY: Self = Self(149);
    pub const ISUB_BORROW: Self = Self(150);
    pub const UMUL_EXTENDED: Self = Self(151);
    pub const SMUL_EXTENDED: Self = Self(152);
    pub const ANY: Self = Self(154);
    pub const ALL: Self = Self(155);
    pub const IS_NAN: Self = Self(156);
    pub const IS_INF: Self = Self(157);
    pub const IS_FINITE: Self = Self(158);
    pub const IS_NORMAL: Self = Self(159);
    pub const SIGN_BIT_SET: Self = Self(160);
    pub const LESS_OR_GREATER: Self = Self(161);
    pub const ORDERED: Self = Self(162);
    pub const UNORDERED: Self = Self(163);
    pub const LOGICAL_EQUAL: Self = Self(164);
    pub const LOGICAL_NOT_EQUAL: Self = Self(165);
    pub const LOGICAL_OR: Self = Self(166);
    pub const LOGICAL_AND: Self = Self(167);
    pub const LOGICAL_NOT: Self = Self(168);
    pub const SELECT: Self = Self(169);
    pub const IEQUAL: Self = Self(170);
    pub const INOT_EQUAL: Self = Self(171);
    pub const UGREATER_THAN: Self = Self(172);
    pub const SGREATER_THAN: Self = Self(173);
    pub const UGREATER_THAN_EQUAL: Self = Self(174);
    pub const SGREATER_THAN_EQUAL: Self = Self(175);
    pub const ULESS_THAN: Self = Self(176);
    pub const SLESS_THAN: Self = Self(177);
    pub const ULESS_THAN_EQUAL: Self = Self(178);
    pub const SLESS_THAN_EQUAL: Self = Self(179);
    pub const FORD_EQUAL: Self = Self(180);
    pub const FUNORD_EQUAL: Self = Self(181);
    pub const FORD_NOT_EQUAL: Self = Self(182);
    pub const FUNORD_NOT_EQUAL: Self = Self(183);
    pub const FORD_LESS_THAN: Self = Self(184);
    pub const FUNORD_LESS_THAN: Self = Self(185);
    pub const FORD_GREATER_THAN: Self = Self(186);
    pub const FUNORD_GREATER_THAN: Self = Self(187);
    pub const FORD_LESS_THAN_EQUAL: Self = Self(188);
    pub const FUNORD_LESS_THAN_EQUAL: Self = Self(189);
    pub const FORD_GREATER_THAN_EQUAL: Self = Self(190);
    pub const FUNORD_GREATER_THAN_EQUAL: Self = Self(191);
    pub const SHIFT_RIGHT_LOGICAL: Self = Self(194);
    pub const SHIFT_RIGHT_ARITHMETIC: Self = Self(195);
    pub const SHIFT_LEFT_LOGICAL: Self = Self(196);
    pub const BITWISE_OR: Self = Self(197);
    pub const BITWISE_XOR: Self = Self(198);
    pub const BITWISE_AND: Self = Self(199);
    pub const NOT: Self = Self(200);
    pub const BIT_FIELD_INSERT: Self = Self(201);
    pub const BIT_FIELD_SEXTRACT: Self = Self(202);
    pub const BIT_FIELD_UEXTRACT: Self = Self(203);
    pub const BIT_REVERSE: Self = Self(204);
    pub const BIT_COUNT: Self = Self(205);
    pub const DPDX: Self = Self(207);
    pub const DPDY: Self = Self(208);
    pub const FWIDTH: Self = Self(209);
    pub const DPDX_FINE: Self = Self(210);
    pub const DPDY_FINE: Self = Self(211);
    pub const FWIDTH_FINE: Self = Self(212);
    pub const DPDX_COARSE: Self = Self(213);
    pub const DPDY_COARSE: Self = Self(214);
    pub const FWIDTH_COARSE: Self = Self(215);
    pub const EMIT_VERTEX: Self = Self(218);
    pub const END_PRIMITIVE: Self = Self(219);
    pub const EMIT_STREAM_VERTEX: Self = Self(220);
    pub const END_STREAM_PRIMITIVE: Self = Self(221);
    pub const CONTROL_BARRIER: Self = Self(224);
    pub const MEMORY_BARRIER: Self = Self(225);
    pub const ATOMIC_LOAD: Self = Self(227);
    pub const ATOMIC_STORE: Self = Self(228);
    pub const ATOMIC_EXCHANGE: Self = Self(229);
    pub const ATOMIC_COMPARE_EXCHANGE: Self = Self(230);
    pub const ATOMIC_COMPARE_EXCHANGE_WEAK: Self = Self(231);
    pub const ATOMIC_IINCREMENT: Self = Self(232);
    pub const ATOMIC_IDECREMENT: Self = Self(233);
    pub const ATOMIC_IADD: Self = Self(234);
    pub const ATOMIC_ISUB: Self = Self(235);
    pub const ATOMIC_SMIN: Self = Self(236);
    pub const ATOMIC_UMIN: Self = Self(237);
    pub const ATOMIC_SMAX: Self = Self(238);
    pub const ATOMIC_UMAX: Self = Self(239);
    pub const ATOMIC_AND: Self = Self(240);
    pub const ATOMIC_OR: Self = Self(241);
    pub const ATOMIC_XOR: Self = Self(242);
    pub const PHI: Self = Self(245);
    pub const LOOP_MERGE: Self = Self(246);
    pub const SELECTION_MERGE: Self = Self(247);
    pub const LABEL: Self = Self(248);
    pub const BRANCH: Self = Self(249);
    pub const BRANCH_CONDITIONAL: Self = Self(250);
    pub const SWITCH: Self = Self(251);
    pub const KILL: Self = Self(252);
    pub const RETURN: Self = Self(253);
    pub const RETURN_VALUE: Self = Self(254);
    pub const UNREACHABLE: Self = Self(255);
    pub const LIFETIME_START: Self = Self(256);
    pub const LIFETIME_STOP: Self = Self(257);
    pub const GROUP_ASYNC_COPY: Self = Self(259);
    pub const GROUP_WAIT_EVENTS: Self = Self(260);
    pub const GROUP_ALL: Self = Self(261);
    pub const GROUP_ANY: Self = Self(262);
    pub const GROUP_BROADCAST: Self = Self(263);
    pub const GROUP_IADD: Self = Self(264);
    pub const GROUP_FADD: Self = Self(265);
    pub const GROUP_FMIN: Self = Self(266);
    pub const GROUP_UMIN: Self = Self(267);
    pub const GROUP_SMIN: Self = Self(268);
    pub const GROUP_FMAX: Self = Self(269);
    pub const GROUP_UMAX: Self = Self(270);
    pub const GROUP_SMAX: Self = Self(271);
    pub const READ_PIPE: Self = Self(274);
    pub const WRITE_PIPE: Self = Self(275);
    pub const RESERVED_READ_PIPE: Self = Self(276);
    pub const RESERVED_WRITE_PIPE: Self = Self(277);
    pub const RESERVE_READ_PIPE_PACKETS: Self = Self(278);
    pub const RESERVE_WRITE_PIPE_PACKETS: Self = Self(279);
    pub const COMMIT_READ_PIPE: Self = Self(280);
    pub const COMMIT_WRITE_PIPE: Self = Self(281);
    pub const IS_VALID_RESERVE_ID: Self = Self(282);
    pub const GET_NUM_PIPE_PACKETS: Self = Self(283);
    pub const GET_MAX_PIPE_PACKETS: Self = Self(284);
    pub const GROUP_RESERVE_READ_PIPE_PACKETS: Self = Self(285);
    pub const GROUP_RESERVE_WRITE_PIPE_PACKETS: Self = Self(286);
    pub const GROUP_COMMIT_READ_PIPE: Self = Self(287);
    pub const GROUP_COMMIT_WRITE_PIPE: Self = Self(288);
    pub const ENQUEUE_MARKER: Self = Self(291);
    pub const ENQUEUE_KERNEL: Self = Self(292);
    pub const GET_KERNEL_NDRANGE_SUB_GROUP_COUNT: Self = Self(293);
    pub const GET_KERNEL_NDRANGE_MAX_SUB_GROUP_SIZE: Self = Self(294);
    pub const GET_KERNEL_WORK_GROUP_SIZE: Self = Self(295);
    pub const GET_KERNEL_PREFERRED_WORK_GROUP_SIZE_MULTIPLE: Self = Self(296);
    pub const RETAIN_EVENT: Self = Self(297);
    pub const RELEASE_EVENT: Self = Self(298);
    pub const CREATE_USER_EVENT: Self = Self(299);
    pub const IS_VALID_EVENT: Self = Self(300);
    pub const SET_USER_EVENT_STATUS: Self = Self(301);
    pub const CAPTURE_EVENT_PROFILING_INFO: Self = Self(302);
    pub const GET_DEFAULT_QUEUE: Self = Self(303);
    pub const BUILD_NDRANGE: Self = Self(304);
    pub const IMAGE_SPARSE_SAMPLE_IMPLICIT_LOD: Self = Self(305);
    pub const IMAGE_SPARSE_SAMPLE_EXPLICIT_LOD: Self = Self(306);
    pub const IMAGE_SPARSE_SAMPLE_DREF_IMPLICIT_LOD: Self = Self(307);
    pub const IMAGE_SPARSE_SAMPLE_DREF_EXPLICIT_LOD: Self = Self(308);
    pub const IMAGE_SPARSE_SAMPLE_PROJ_IMPLICIT_LOD: Self = Self(309);
    pub const IMAGE_SPARSE_SAMPLE_PROJ_EXPLICIT_LOD: Self = Self(310);
    pub const IMAGE_SPARSE_SAMPLE_PROJ_DREF_IMPLICIT_LOD: Self = Self(311);
    pub const IMAGE_SPARSE_SAMPLE_PROJ_DREF_EXPLICIT_LOD: Self = Self(312);
    pub const IMAGE_SPARSE_FETCH: Self = Self(313);
    pub const IMAGE_SPARSE_GATHER: Self = Self(314);
    pub const IMAGE_SPARSE_DREF_GATHER: Self = Self(315);
    pub const IMAGE_SPARSE_TEXELS_RESIDENT: Self = Self(316);
    pub const NO_LINE: Self = Self(317);
    pub const ATOMIC_FLAG_TEST_AND_SET: Self = Self(318);
    pub const ATOMIC_FLAG_CLEAR: Self = Self(319);
    pub const IMAGE_SPARSE_READ: Self = Self(320);
    pub const SIZE_OF: Self = Self(321);
    pub const TYPE_PIPE_STORAGE: Self = Self(322);
    pub const CONSTANT_PIPE_STORAGE: Self = Self(323);
    pub const CREATE_PIPE_FROM_PIPE_STORAGE: Self = Self(324);
    pub const GET_KERNEL_LOCAL_SIZE_FOR_SUBGROUP_COUNT: Self = Self(325);
    pub const GET_KERNEL_MAX_NUM_SUBGROUPS: Self = Self(326);
    pub const TYPE_NAMED_BARRIER: Self = Self(327);
    pub const NAMED_BARRIER_INITIALIZE: Self = Self(328);
    pub const MEMORY_NAMED_BARRIER: Self = Self(329);
    pub const MODULE_PROCESSED: Self = Self(330);
    pub const EXECUTION_MODE_ID: Self = Self(331);
    pub const DECORATE_ID: Self = Self(332);
    pub const GROUP_NON_UNIFORM_ELECT: Self = Self(333);
    pub const GROUP_NON_UNIFORM_ALL: Self = Self(334);
    pub const GROUP_NON_UNIFORM_ANY: Self = Self(335);
    pub const GROUP_NON_UNIFORM_ALL_EQUAL: Self = Self(336);
    pub const GROUP_NON_UNIFORM_BROADCAST: Self = Self(337);
    pub const GROUP_NON_UNIFORM_BROADCAST_FIRST: Self = Self(338);
    pub const GROUP_NON_UNIFORM_BALLOT: Self = Self(339);
    pub const GROUP_NON_UNIFORM_INVERSE_BALLOT: Self = Self(340);
    pub const GROUP_NON_UNIFORM_BALLOT_BIT_EXTRACT: Self = Self(341);
    pub const GROUP_NON_UNIFORM_BALLOT_BIT_COUNT: Self = Self(342);
    pub const GROUP_NON_UNIFORM_BALLOT_FIND_LSB: Self = Self(343);
    pub const GROUP_NON_UNIFORM_BALLOT_FIND_MSB: Self = Self(344);
    pub const GROUP_NON_UNIFORM_SHUFFLE: Self = Self(345);
    pub const GROUP_NON_UNIFORM_SHUFFLE_XOR: Self = Self(346);
    pub const GROUP_NON_UNIFORM_SHUFFLE_UP: Self = Self(347);
    pub const GROUP_NON_UNIFORM_SHUFFLE_DOWN: Self = Self(348);
    pub const GROUP_NON_UNIFORM_IADD: Self = Self(349);
    pub const GROUP_NON_UNIFORM_FADD: Self = Self(350);
    pub const GROUP_NON_UNIFORM_IMUL: Self = Self(351);
    pub const GROUP_NON_UNIFORM_FMUL: Self = Self(352);
    pub const GROUP_NON_UNIFORM_SMIN: Self = Self(353);
    pub const GROUP_NON_UNIFORM_UMIN: Self = Self(354);
    pub const GROUP_NON_UNIFORM_FMIN: Self = Self(355);
    pub const GROUP_NON_UNIFORM_SMAX: Self = Self(356);
    pub const GROUP_NON_UNIFORM_UMAX: Self = Self(357);
    pub const GROUP_NON_UNIFORM_FMAX: Self = Self(358);
    pub const GROUP_NON_UNIFORM_BITWISE_AND: Self = Self(359);
    pub const GROUP_NON_UNIFORM_BITWISE_OR: Self = Self(360);
    pub const GROUP_NON_UNIFORM_BITWISE_XOR: Self = Self(361);
    pub const GROUP_NON_UNIFORM_LOGICAL_AND: Self = Self(362);
    pub const GROUP_NON_UNIFORM_LOGICAL_OR: Self = Self(363);
    pub const GROUP_NON_UNIFORM_LOGICAL_XOR: Self = Self(364);
    pub const GROUP_NON_UNIFORM_QUAD_BROADCAST: Self = Self(365);
    pub const GROUP_NON_UNIFORM_QUAD_SWAP: Self = Self(366);
    pub const COPY_LOGICAL: Self = Self(400);
    pub const PTR_EQUAL: Self = Self(401);
    pub const PTR_NOT_EQUAL: Self = Self(402);
    pub const PTR_DIFF: Self = Self(403);
    pub const COLOR_ATTACHMENT_READ_EXT: Self = Self(4160);
    pub const DEPTH_ATTACHMENT_READ_EXT: Self = Self(4161);
    pub const STENCIL_ATTACHMENT_READ_EXT: Self = Self(4162);
    pub const TYPE_TENSOR_ARM: Self = Self(4163);
    pub const TENSOR_READ_ARM: Self = Self(4164);
    pub const TENSOR_WRITE_ARM: Self = Self(4165);
    pub const TENSOR_QUERY_SIZE_ARM: Self = Self(4166);
    pub const GRAPH_CONSTANT_ARM: Self = Self(4181);
    pub const GRAPH_ENTRY_POINT_ARM: Self = Self(4182);
    pub const GRAPH_ARM: Self = Self(4183);
    pub const GRAPH_INPUT_ARM: Self = Self(4184);
    pub const GRAPH_SET_OUTPUT_ARM: Self = Self(4185);
    pub const GRAPH_END_ARM: Self = Self(4186);
    pub const TYPE_GRAPH_ARM: Self = Self(4190);
    pub const TERMINATE_INVOCATION: Self = Self(4416);
    pub const TYPE_UNTYPED_POINTER_KHR: Self = Self(4417);
    pub const UNTYPED_VARIABLE_KHR: Self = Self(4418);
    pub const UNTYPED_ACCESS_CHAIN_KHR: Self = Self(4419);
    pub const UNTYPED_IN_BOUNDS_ACCESS_CHAIN_KHR: Self = Self(4420);
    pub const SUBGROUP_BALLOT_KHR: Self = Self(4421);
    pub const SUBGROUP_FIRST_INVOCATION_KHR: Self = Self(4422);
    pub const UNTYPED_PTR_ACCESS_CHAIN_KHR: Self = Self(4423);
    pub const UNTYPED_IN_BOUNDS_PTR_ACCESS_CHAIN_KHR: Self = Self(4424);
    pub const UNTYPED_ARRAY_LENGTH_KHR: Self = Self(4425);
    pub const UNTYPED_PREFETCH_KHR: Self = Self(4426);
    pub const FMA_KHR: Self = Self(4427);
    pub const SUBGROUP_ALL_KHR: Self = Self(4428);
    pub const SUBGROUP_ANY_KHR: Self = Self(4429);
    pub const SUBGROUP_ALL_EQUAL_KHR: Self = Self(4430);
    pub const GROUP_NON_UNIFORM_ROTATE_KHR: Self = Self(4431);
    pub const SUBGROUP_READ_INVOCATION_KHR: Self = Self(4432);
    pub const EXT_INST_WITH_FORWARD_REFS_KHR: Self = Self(4433);
    pub const UNTYPED_GROUP_ASYNC_COPY_KHR: Self = Self(4434);
    pub const TRACE_RAY_KHR: Self = Self(4445);
    pub const EXECUTE_CALLABLE_KHR: Self = Self(4446);
    pub const CONVERT_UTO_ACCELERATION_STRUCTURE_KHR: Self = Self(4447);
    pub const IGNORE_INTERSECTION_KHR: Self = Self(4448);
    pub const TERMINATE_RAY_KHR: Self = Self(4449);
    pub const SDOT: Self = Self(4450);
    pub const UDOT: Self = Self(4451);
    pub const SUDOT: Self = Self(4452);
    pub const SDOT_ACC_SAT: Self = Self(4453);
    pub const UDOT_ACC_SAT: Self = Self(4454);
    pub const SUDOT_ACC_SAT: Self = Self(4455);
    pub const TYPE_COOPERATIVE_MATRIX_KHR: Self = Self(4456);
    pub const COOPERATIVE_MATRIX_LOAD_KHR: Self = Self(4457);
    pub const COOPERATIVE_MATRIX_STORE_KHR: Self = Self(4458);
    pub const COOPERATIVE_MATRIX_MUL_ADD_KHR: Self = Self(4459);
    pub const COOPERATIVE_MATRIX_LENGTH_KHR: Self = Self(4460);
    pub const CONSTANT_COMPOSITE_REPLICATE_EXT: Self = Self(4461);
    pub const SPEC_CONSTANT_COMPOSITE_REPLICATE_EXT: Self = Self(4462);
    pub const COMPOSITE_CONSTRUCT_REPLICATE_EXT: Self = Self(4463);
    pub const TYPE_RAY_QUERY_KHR: Self = Self(4472);
    pub const RAY_QUERY_INITIALIZE_KHR: Self = Self(4473);
    pub const RAY_QUERY_TERMINATE_KHR: Self = Self(4474);
    pub const RAY_QUERY_GENERATE_INTERSECTION_KHR: Self = Self(4475);
    pub const RAY_QUERY_CONFIRM_INTERSECTION_KHR: Self = Self(4476);
    pub const RAY_QUERY_PROCEED_KHR: Self = Self(4477);
    pub const RAY_QUERY_GET_INTERSECTION_TYPE_KHR: Self = Self(4479);
    pub const IMAGE_SAMPLE_WEIGHTED_QCOM: Self = Self(4480);
    pub const IMAGE_BOX_FILTER_QCOM: Self = Self(4481);
    pub const IMAGE_BLOCK_MATCH_SSDQCOM: Self = Self(4482);
    pub const IMAGE_BLOCK_MATCH_SADQCOM: Self = Self(4483);
    pub const BIT_CAST_ARRAY_QCOM: Self = Self(4497);
    pub const IMAGE_BLOCK_MATCH_WINDOW_SSDQCOM: Self = Self(4500);
    pub const IMAGE_BLOCK_MATCH_WINDOW_SADQCOM: Self = Self(4501);
    pub const IMAGE_BLOCK_MATCH_GATHER_SSDQCOM: Self = Self(4502);
    pub const IMAGE_BLOCK_MATCH_GATHER_SADQCOM: Self = Self(4503);
    pub const COMPOSITE_CONSTRUCT_COOP_MAT_QCOM: Self = Self(4540);
    pub const COMPOSITE_EXTRACT_COOP_MAT_QCOM: Self = Self(4541);
    pub const EXTRACT_SUB_ARRAY_QCOM: Self = Self(4542);
    pub const GROUP_IADD_NON_UNIFORM_AMD: Self = Self(5000);
    pub const GROUP_FADD_NON_UNIFORM_AMD: Self = Self(5001);
    pub const GROUP_FMIN_NON_UNIFORM_AMD: Self = Self(5002);
    pub const GROUP_UMIN_NON_UNIFORM_AMD: Self = Self(5003);
    pub const GROUP_SMIN_NON_UNIFORM_AMD: Self = Self(5004);
    pub const GROUP_FMAX_NON_UNIFORM_AMD: Self = Self(5005);
    pub const GROUP_UMAX_NON_UNIFORM_AMD: Self = Self(5006);
    pub const GROUP_SMAX_NON_UNIFORM_AMD: Self = Self(5007);
    pub const FRAGMENT_MASK_FETCH_AMD: Self = Self(5011);
    pub const FRAGMENT_FETCH_AMD: Self = Self(5012);
    pub const READ_CLOCK_KHR: Self = Self(5056);
    pub const ALLOCATE_NODE_PAYLOADS_AMDX: Self = Self(5074);
    pub const ENQUEUE_NODE_PAYLOADS_AMDX: Self = Self(5075);
    pub const TYPE_NODE_PAYLOAD_ARRAY_AMDX: Self = Self(5076);
    pub const FINISH_WRITING_NODE_PAYLOAD_AMDX: Self = Self(5078);
    pub const NODE_PAYLOAD_ARRAY_LENGTH_AMDX: Self = Self(5090);
    pub const IS_NODE_PAYLOAD_VALID_AMDX: Self = Self(5101);
    pub const CONSTANT_STRING_AMDX: Self = Self(5103);
    pub const SPEC_CONSTANT_STRING_AMDX: Self = Self(5104);
    pub const GROUP_NON_UNIFORM_QUAD_ALL_KHR: Self = Self(5110);
    pub const GROUP_NON_UNIFORM_QUAD_ANY_KHR: Self = Self(5111);
    pub const TYPE_BUFFER_EXT: Self = Self(5115);
    pub const BUFFER_POINTER_EXT: Self = Self(5119);
    pub const ABORT_KHR: Self = Self(5121);
    pub const UNTYPED_IMAGE_TEXEL_POINTER_EXT: Self = Self(5126);
    pub const MEMBER_DECORATE_ID_EXT: Self = Self(5127);
    pub const CONSTANT_SIZE_OF_EXT: Self = Self(5129);
    pub const CONSTANT_DATA_KHR: Self = Self(5147);
    pub const SPEC_CONSTANT_DATA_KHR: Self = Self(5148);
    pub const POISON_KHR: Self = Self(5158);
    pub const FREEZE_KHR: Self = Self(5159);
    pub const HIT_OBJECT_RECORD_HIT_MOTION_NV: Self = Self(5249);
    pub const HIT_OBJECT_RECORD_HIT_WITH_INDEX_MOTION_NV: Self = Self(5250);
    pub const HIT_OBJECT_RECORD_MISS_MOTION_NV: Self = Self(5251);
    pub const HIT_OBJECT_GET_WORLD_TO_OBJECT_NV: Self = Self(5252);
    pub const HIT_OBJECT_GET_OBJECT_TO_WORLD_NV: Self = Self(5253);
    pub const HIT_OBJECT_GET_OBJECT_RAY_DIRECTION_NV: Self = Self(5254);
    pub const HIT_OBJECT_GET_OBJECT_RAY_ORIGIN_NV: Self = Self(5255);
    pub const HIT_OBJECT_TRACE_RAY_MOTION_NV: Self = Self(5256);
    pub const HIT_OBJECT_GET_SHADER_RECORD_BUFFER_HANDLE_NV: Self = Self(5257);
    pub const HIT_OBJECT_GET_SHADER_BINDING_TABLE_RECORD_INDEX_NV: Self = Self(5258);
    pub const HIT_OBJECT_RECORD_EMPTY_NV: Self = Self(5259);
    pub const HIT_OBJECT_TRACE_RAY_NV: Self = Self(5260);
    pub const HIT_OBJECT_RECORD_HIT_NV: Self = Self(5261);
    pub const HIT_OBJECT_RECORD_HIT_WITH_INDEX_NV: Self = Self(5262);
    pub const HIT_OBJECT_RECORD_MISS_NV: Self = Self(5263);
    pub const HIT_OBJECT_EXECUTE_SHADER_NV: Self = Self(5264);
    pub const HIT_OBJECT_GET_CURRENT_TIME_NV: Self = Self(5265);
    pub const HIT_OBJECT_GET_ATTRIBUTES_NV: Self = Self(5266);
    pub const HIT_OBJECT_GET_HIT_KIND_NV: Self = Self(5267);
    pub const HIT_OBJECT_GET_PRIMITIVE_INDEX_NV: Self = Self(5268);
    pub const HIT_OBJECT_GET_GEOMETRY_INDEX_NV: Self = Self(5269);
    pub const HIT_OBJECT_GET_INSTANCE_ID_NV: Self = Self(5270);
    pub const HIT_OBJECT_GET_INSTANCE_CUSTOM_INDEX_NV: Self = Self(5271);
    pub const HIT_OBJECT_GET_WORLD_RAY_DIRECTION_NV: Self = Self(5272);
    pub const HIT_OBJECT_GET_WORLD_RAY_ORIGIN_NV: Self = Self(5273);
    pub const HIT_OBJECT_GET_RAY_TMAX_NV: Self = Self(5274);
    pub const HIT_OBJECT_GET_RAY_TMIN_NV: Self = Self(5275);
    pub const HIT_OBJECT_IS_EMPTY_NV: Self = Self(5276);
    pub const HIT_OBJECT_IS_HIT_NV: Self = Self(5277);
    pub const HIT_OBJECT_IS_MISS_NV: Self = Self(5278);
    pub const REORDER_THREAD_WITH_HIT_OBJECT_NV: Self = Self(5279);
    pub const REORDER_THREAD_WITH_HINT_NV: Self = Self(5280);
    pub const TYPE_HIT_OBJECT_NV: Self = Self(5281);
    pub const IMAGE_SAMPLE_FOOTPRINT_NV: Self = Self(5283);
    pub const TYPE_VECTOR_ID_EXT: Self = Self(5288);
    pub const COOPERATIVE_VECTOR_MATRIX_MUL_NV: Self = Self(5289);
    pub const COOPERATIVE_VECTOR_OUTER_PRODUCT_ACCUMULATE_NV: Self = Self(5290);
    pub const COOPERATIVE_VECTOR_REDUCE_SUM_ACCUMULATE_NV: Self = Self(5291);
    pub const COOPERATIVE_VECTOR_MATRIX_MUL_ADD_NV: Self = Self(5292);
    pub const COOPERATIVE_MATRIX_CONVERT_NV: Self = Self(5293);
    pub const EMIT_MESH_TASKS_EXT: Self = Self(5294);
    pub const SET_MESH_OUTPUTS_EXT: Self = Self(5295);
    pub const GROUP_NON_UNIFORM_PARTITION_EXT: Self = Self(5296);
    pub const WRITE_PACKED_PRIMITIVE_INDICES4X8_NV: Self = Self(5299);
    pub const FETCH_MICRO_TRIANGLE_VERTEX_POSITION_NV: Self = Self(5300);
    pub const FETCH_MICRO_TRIANGLE_VERTEX_BARYCENTRIC_NV: Self = Self(5301);
    pub const COOPERATIVE_VECTOR_LOAD_NV: Self = Self(5302);
    pub const COOPERATIVE_VECTOR_STORE_NV: Self = Self(5303);
    pub const HIT_OBJECT_RECORD_FROM_QUERY_EXT: Self = Self(5304);
    pub const HIT_OBJECT_RECORD_MISS_EXT: Self = Self(5305);
    pub const HIT_OBJECT_RECORD_MISS_MOTION_EXT: Self = Self(5306);
    pub const HIT_OBJECT_GET_INTERSECTION_TRIANGLE_VERTEX_POSITIONS_EXT: Self = Self(5307);
    pub const HIT_OBJECT_GET_RAY_FLAGS_EXT: Self = Self(5308);
    pub const HIT_OBJECT_SET_SHADER_BINDING_TABLE_RECORD_INDEX_EXT: Self = Self(5309);
    pub const HIT_OBJECT_REORDER_EXECUTE_SHADER_EXT: Self = Self(5310);
    pub const HIT_OBJECT_TRACE_REORDER_EXECUTE_EXT: Self = Self(5311);
    pub const HIT_OBJECT_TRACE_MOTION_REORDER_EXECUTE_EXT: Self = Self(5312);
    pub const TYPE_HIT_OBJECT_EXT: Self = Self(5313);
    pub const REORDER_THREAD_WITH_HINT_EXT: Self = Self(5314);
    pub const REORDER_THREAD_WITH_HIT_OBJECT_EXT: Self = Self(5315);
    pub const HIT_OBJECT_TRACE_RAY_EXT: Self = Self(5316);
    pub const HIT_OBJECT_TRACE_RAY_MOTION_EXT: Self = Self(5317);
    pub const HIT_OBJECT_RECORD_EMPTY_EXT: Self = Self(5318);
    pub const HIT_OBJECT_EXECUTE_SHADER_EXT: Self = Self(5319);
    pub const HIT_OBJECT_GET_CURRENT_TIME_EXT: Self = Self(5320);
    pub const HIT_OBJECT_GET_ATTRIBUTES_EXT: Self = Self(5321);
    pub const HIT_OBJECT_GET_HIT_KIND_EXT: Self = Self(5322);
    pub const HIT_OBJECT_GET_PRIMITIVE_INDEX_EXT: Self = Self(5323);
    pub const HIT_OBJECT_GET_GEOMETRY_INDEX_EXT: Self = Self(5324);
    pub const HIT_OBJECT_GET_INSTANCE_ID_EXT: Self = Self(5325);
    pub const HIT_OBJECT_GET_INSTANCE_CUSTOM_INDEX_EXT: Self = Self(5326);
    pub const HIT_OBJECT_GET_OBJECT_RAY_ORIGIN_EXT: Self = Self(5327);
    pub const HIT_OBJECT_GET_OBJECT_RAY_DIRECTION_EXT: Self = Self(5328);
    pub const HIT_OBJECT_GET_WORLD_RAY_DIRECTION_EXT: Self = Self(5329);
    pub const HIT_OBJECT_GET_WORLD_RAY_ORIGIN_EXT: Self = Self(5330);
    pub const HIT_OBJECT_GET_OBJECT_TO_WORLD_EXT: Self = Self(5331);
    pub const HIT_OBJECT_GET_WORLD_TO_OBJECT_EXT: Self = Self(5332);
    pub const HIT_OBJECT_GET_RAY_TMAX_EXT: Self = Self(5333);
    pub const REPORT_INTERSECTION_KHR: Self = Self(5334);
    pub const IGNORE_INTERSECTION_NV: Self = Self(5335);
    pub const TERMINATE_RAY_NV: Self = Self(5336);
    pub const TRACE_NV: Self = Self(5337);
    pub const TRACE_MOTION_NV: Self = Self(5338);
    pub const TRACE_RAY_MOTION_NV: Self = Self(5339);
    pub const RAY_QUERY_GET_INTERSECTION_TRIANGLE_VERTEX_POSITIONS_KHR: Self = Self(5340);
    pub const TYPE_ACCELERATION_STRUCTURE_KHR: Self = Self(5341);
    pub const EXECUTE_CALLABLE_NV: Self = Self(5344);
    pub const RAY_QUERY_GET_INTERSECTION_CLUSTER_ID_NV: Self = Self(5345);
    pub const HIT_OBJECT_GET_CLUSTER_ID_NV: Self = Self(5346);
    pub const HIT_OBJECT_GET_RAY_TMIN_EXT: Self = Self(5347);
    pub const HIT_OBJECT_GET_SHADER_BINDING_TABLE_RECORD_INDEX_EXT: Self = Self(5348);
    pub const HIT_OBJECT_GET_SHADER_RECORD_BUFFER_HANDLE_EXT: Self = Self(5349);
    pub const HIT_OBJECT_IS_EMPTY_EXT: Self = Self(5350);
    pub const HIT_OBJECT_IS_HIT_EXT: Self = Self(5351);
    pub const HIT_OBJECT_IS_MISS_EXT: Self = Self(5352);
    pub const TYPE_COOPERATIVE_MATRIX_NV: Self = Self(5358);
    pub const COOPERATIVE_MATRIX_LOAD_NV: Self = Self(5359);
    pub const COOPERATIVE_MATRIX_STORE_NV: Self = Self(5360);
    pub const COOPERATIVE_MATRIX_MUL_ADD_NV: Self = Self(5361);
    pub const COOPERATIVE_MATRIX_LENGTH_NV: Self = Self(5362);
    pub const BEGIN_INVOCATION_INTERLOCK_EXT: Self = Self(5364);
    pub const END_INVOCATION_INTERLOCK_EXT: Self = Self(5365);
    pub const COOPERATIVE_MATRIX_REDUCE_NV: Self = Self(5366);
    pub const COOPERATIVE_MATRIX_LOAD_TENSOR_NV: Self = Self(5367);
    pub const COOPERATIVE_MATRIX_STORE_TENSOR_NV: Self = Self(5368);
    pub const COOPERATIVE_MATRIX_PER_ELEMENT_OP_NV: Self = Self(5369);
    pub const TYPE_TENSOR_LAYOUT_NV: Self = Self(5370);
    pub const TYPE_TENSOR_VIEW_NV: Self = Self(5371);
    pub const CREATE_TENSOR_LAYOUT_NV: Self = Self(5372);
    pub const TENSOR_LAYOUT_SET_DIMENSION_NV: Self = Self(5373);
    pub const TENSOR_LAYOUT_SET_STRIDE_NV: Self = Self(5374);
    pub const TENSOR_LAYOUT_SLICE_NV: Self = Self(5375);
    pub const TENSOR_LAYOUT_SET_CLAMP_VALUE_NV: Self = Self(5376);
    pub const CREATE_TENSOR_VIEW_NV: Self = Self(5377);
    pub const TENSOR_VIEW_SET_DIMENSION_NV: Self = Self(5378);
    pub const TENSOR_VIEW_SET_STRIDE_NV: Self = Self(5379);
    pub const DEMOTE_TO_HELPER_INVOCATION: Self = Self(5380);
    pub const IS_HELPER_INVOCATION_EXT: Self = Self(5381);
    pub const TENSOR_VIEW_SET_CLIP_NV: Self = Self(5382);
    pub const TENSOR_LAYOUT_SET_BLOCK_SIZE_NV: Self = Self(5384);
    pub const COOPERATIVE_MATRIX_TRANSPOSE_NV: Self = Self(5390);
    pub const CONVERT_UTO_IMAGE_NV: Self = Self(5391);
    pub const CONVERT_UTO_SAMPLER_NV: Self = Self(5392);
    pub const CONVERT_IMAGE_TO_UNV: Self = Self(5393);
    pub const CONVERT_SAMPLER_TO_UNV: Self = Self(5394);
    pub const CONVERT_UTO_SAMPLED_IMAGE_NV: Self = Self(5395);
    pub const CONVERT_SAMPLED_IMAGE_TO_UNV: Self = Self(5396);
    pub const SAMPLER_IMAGE_ADDRESSING_MODE_NV: Self = Self(5397);
    pub const RAW_ACCESS_CHAIN_NV: Self = Self(5398);
    pub const RAY_QUERY_GET_INTERSECTION_SPHERE_POSITION_NV: Self = Self(5427);
    pub const RAY_QUERY_GET_INTERSECTION_SPHERE_RADIUS_NV: Self = Self(5428);
    pub const RAY_QUERY_GET_INTERSECTION_LSSPOSITIONS_NV: Self = Self(5429);
    pub const RAY_QUERY_GET_INTERSECTION_LSSRADII_NV: Self = Self(5430);
    pub const RAY_QUERY_GET_INTERSECTION_LSSHIT_VALUE_NV: Self = Self(5431);
    pub const HIT_OBJECT_GET_SPHERE_POSITION_NV: Self = Self(5432);
    pub const HIT_OBJECT_GET_SPHERE_RADIUS_NV: Self = Self(5433);
    pub const HIT_OBJECT_GET_LSSPOSITIONS_NV: Self = Self(5434);
    pub const HIT_OBJECT_GET_LSSRADII_NV: Self = Self(5435);
    pub const HIT_OBJECT_IS_SPHERE_HIT_NV: Self = Self(5436);
    pub const HIT_OBJECT_IS_LSSHIT_NV: Self = Self(5437);
    pub const RAY_QUERY_IS_SPHERE_HIT_NV: Self = Self(5438);
    pub const RAY_QUERY_IS_LSSHIT_NV: Self = Self(5439);
    pub const SUBGROUP_SHUFFLE_INTEL: Self = Self(5571);
    pub const SUBGROUP_SHUFFLE_DOWN_INTEL: Self = Self(5572);
    pub const SUBGROUP_SHUFFLE_UP_INTEL: Self = Self(5573);
    pub const SUBGROUP_SHUFFLE_XOR_INTEL: Self = Self(5574);
    pub const SUBGROUP_BLOCK_READ_INTEL: Self = Self(5575);
    pub const SUBGROUP_BLOCK_WRITE_INTEL: Self = Self(5576);
    pub const SUBGROUP_IMAGE_BLOCK_READ_INTEL: Self = Self(5577);
    pub const SUBGROUP_IMAGE_BLOCK_WRITE_INTEL: Self = Self(5578);
    pub const SUBGROUP_IMAGE_MEDIA_BLOCK_READ_INTEL: Self = Self(5580);
    pub const SUBGROUP_IMAGE_MEDIA_BLOCK_WRITE_INTEL: Self = Self(5581);
    pub const UCOUNT_LEADING_ZEROS_INTEL: Self = Self(5585);
    pub const UCOUNT_TRAILING_ZEROS_INTEL: Self = Self(5586);
    pub const ABS_ISUB_INTEL: Self = Self(5587);
    pub const ABS_USUB_INTEL: Self = Self(5588);
    pub const IADD_SAT_INTEL: Self = Self(5589);
    pub const UADD_SAT_INTEL: Self = Self(5590);
    pub const IAVERAGE_INTEL: Self = Self(5591);
    pub const UAVERAGE_INTEL: Self = Self(5592);
    pub const IAVERAGE_ROUNDED_INTEL: Self = Self(5593);
    pub const UAVERAGE_ROUNDED_INTEL: Self = Self(5594);
    pub const ISUB_SAT_INTEL: Self = Self(5595);
    pub const USUB_SAT_INTEL: Self = Self(5596);
    pub const IMUL32X16_INTEL: Self = Self(5597);
    pub const UMUL32X16_INTEL: Self = Self(5598);
    pub const ATOMIC_FMIN_EXT: Self = Self(5614);
    pub const ATOMIC_FMAX_EXT: Self = Self(5615);
    pub const ASSUME_TRUE_KHR: Self = Self(5630);
    pub const EXPECT_KHR: Self = Self(5631);
    pub const DECORATE_STRING: Self = Self(5632);
    pub const MEMBER_DECORATE_STRING: Self = Self(5633);
    pub const VARIABLE_LENGTH_ARRAY_INTEL: Self = Self(5818);
    pub const SAVE_MEMORY_INTEL: Self = Self(5819);
    pub const RESTORE_MEMORY_INTEL: Self = Self(5820);
    pub const LOOP_CONTROL_INTEL: Self = Self(5887);
    pub const RAY_QUERY_GET_RAY_TMIN_KHR: Self = Self(6016);
    pub const RAY_QUERY_GET_RAY_FLAGS_KHR: Self = Self(6017);
    pub const RAY_QUERY_GET_INTERSECTION_TKHR: Self = Self(6018);
    pub const RAY_QUERY_GET_INTERSECTION_INSTANCE_CUSTOM_INDEX_KHR: Self = Self(6019);
    pub const RAY_QUERY_GET_INTERSECTION_INSTANCE_ID_KHR: Self = Self(6020);
    pub const RAY_QUERY_GET_INTERSECTION_INSTANCE_SHADER_BINDING_TABLE_RECORD_OFFSET_KHR: Self =
        Self(6021);
    pub const RAY_QUERY_GET_INTERSECTION_GEOMETRY_INDEX_KHR: Self = Self(6022);
    pub const RAY_QUERY_GET_INTERSECTION_PRIMITIVE_INDEX_KHR: Self = Self(6023);
    pub const RAY_QUERY_GET_INTERSECTION_BARYCENTRICS_KHR: Self = Self(6024);
    pub const RAY_QUERY_GET_INTERSECTION_FRONT_FACE_KHR: Self = Self(6025);
    pub const RAY_QUERY_GET_INTERSECTION_CANDIDATE_AABBOPAQUE_KHR: Self = Self(6026);
    pub const RAY_QUERY_GET_INTERSECTION_OBJECT_RAY_DIRECTION_KHR: Self = Self(6027);
    pub const RAY_QUERY_GET_INTERSECTION_OBJECT_RAY_ORIGIN_KHR: Self = Self(6028);
    pub const RAY_QUERY_GET_WORLD_RAY_DIRECTION_KHR: Self = Self(6029);
    pub const RAY_QUERY_GET_WORLD_RAY_ORIGIN_KHR: Self = Self(6030);
    pub const RAY_QUERY_GET_INTERSECTION_OBJECT_TO_WORLD_KHR: Self = Self(6031);
    pub const RAY_QUERY_GET_INTERSECTION_WORLD_TO_OBJECT_KHR: Self = Self(6032);
    pub const ATOMIC_FADD_EXT: Self = Self(6035);
    pub const TYPE_BUFFER_SURFACE_INTEL: Self = Self(6086);
    pub const TYPE_STRUCT_CONTINUED_INTEL: Self = Self(6090);
    pub const CONSTANT_COMPOSITE_CONTINUED_INTEL: Self = Self(6091);
    pub const SPEC_CONSTANT_COMPOSITE_CONTINUED_INTEL: Self = Self(6092);
    pub const COMPOSITE_CONSTRUCT_CONTINUED_INTEL: Self = Self(6096);
    pub const CONVERT_FTO_BF16_INTEL: Self = Self(6116);
    pub const CONVERT_BF16_TO_FINTEL: Self = Self(6117);
    pub const CONTROL_BARRIER_ARRIVE_INTEL: Self = Self(6142);
    pub const CONTROL_BARRIER_WAIT_INTEL: Self = Self(6143);
    pub const ARITHMETIC_FENCE_EXT: Self = Self(6145);
    pub const SUBGROUP_BLOCK_PREFETCH_INTEL: Self = Self(6221);
    pub const SUBGROUP2_DBLOCK_LOAD_INTEL: Self = Self(6231);
    pub const SUBGROUP2_DBLOCK_LOAD_TRANSFORM_INTEL: Self = Self(6232);
    pub const SUBGROUP2_DBLOCK_LOAD_TRANSPOSE_INTEL: Self = Self(6233);
    pub const SUBGROUP2_DBLOCK_PREFETCH_INTEL: Self = Self(6234);
    pub const SUBGROUP2_DBLOCK_STORE_INTEL: Self = Self(6235);
    pub const SUBGROUP_MATRIX_MULTIPLY_ACCUMULATE_INTEL: Self = Self(6237);
    pub const BITWISE_FUNCTION_INTEL: Self = Self(6242);
    pub const UNTYPED_VARIABLE_LENGTH_ARRAY_INTEL: Self = Self(6244);
    pub const CONDITIONAL_EXTENSION_INTEL: Self = Self(6248);
    pub const CONDITIONAL_ENTRY_POINT_INTEL: Self = Self(6249);
    pub const CONDITIONAL_CAPABILITY_INTEL: Self = Self(6250);
    pub const SPEC_CONSTANT_TARGET_INTEL: Self = Self(6251);
    pub const SPEC_CONSTANT_ARCHITECTURE_INTEL: Self = Self(6252);
    pub const SPEC_CONSTANT_CAPABILITIES_INTEL: Self = Self(6253);
    pub const CONDITIONAL_COPY_OBJECT_INTEL: Self = Self(6254);
    pub const GROUP_IMUL_KHR: Self = Self(6401);
    pub const GROUP_FMUL_KHR: Self = Self(6402);
    pub const GROUP_BITWISE_AND_KHR: Self = Self(6403);
    pub const GROUP_BITWISE_OR_KHR: Self = Self(6404);
    pub const GROUP_BITWISE_XOR_KHR: Self = Self(6405);
    pub const GROUP_LOGICAL_AND_KHR: Self = Self(6406);
    pub const GROUP_LOGICAL_OR_KHR: Self = Self(6407);
    pub const GROUP_LOGICAL_XOR_KHR: Self = Self(6408);
    pub const ROUND_FTO_TF32_INTEL: Self = Self(6426);
    pub const MASKED_GATHER_INTEL: Self = Self(6428);
    pub const MASKED_SCATTER_INTEL: Self = Self(6429);
    pub const CONVERT_HANDLE_TO_IMAGE_INTEL: Self = Self(6529);
    pub const CONVERT_HANDLE_TO_SAMPLER_INTEL: Self = Self(6530);
    pub const CONVERT_HANDLE_TO_SAMPLED_IMAGE_INTEL: Self = Self(6531);
    pub const FDOT2_MIX_ACC32_VALVE: Self = Self(6916);
    pub const FDOT2_MIX_ACC16_VALVE: Self = Self(6917);
    pub const FDOT4_MIX_ACC32_VALVE: Self = Self(6918);
}
impl Word for Code {
    #[inline]
    fn from_word(word: u32) -> Self {
        Self(word as u16)
    }
}
impl Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", INST_INFOS[self.0 as usize].name)
    }
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformElect {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformAll {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformAny {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformAllEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBroadcast {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub invocation_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBroadcastFirst {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBallot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformInverseBallot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBallotBitExtract {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBallotBitCount {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBallotFindLSB {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBallotFindMSB {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformShuffle {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub invocation_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformShuffleXor {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub mask: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformShuffleUp {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub delta: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformShuffleDown {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub delta: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformIAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformFAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformIMul {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformFMul {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformSMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformUMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformFMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformSMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformUMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformFMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBitwiseAnd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBitwiseOr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformBitwiseXor {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformLogicalAnd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformLogicalOr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformLogicalXor {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub value: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformQuadBroadcast {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformQuadSwap {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub direction: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformQuadAllKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformQuadAnyKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformPartitionEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstString<'a> {
    pub id_result: IdResult,
    pub string: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstSourceContinued<'a> {
    pub continued_source: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstSource<'a> {
    pub source_language: SourceLanguage,
    pub version: LiteralInteger,
    pub file: Option<IdRef>,
    pub source: Option<LiteralString<'a>>,
}
#[derive(Clone, Copy)]
pub struct InstSourceExtension<'a> {
    pub extension: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstName<'a> {
    pub target: IdRef,
    pub name: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstMemberName<'a> {
    pub ty: IdRef,
    pub member: LiteralInteger,
    pub name: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstLine {
    pub file: IdRef,
    pub line: LiteralInteger,
    pub column: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstNoLine;
#[derive(Clone, Copy)]
pub struct InstModuleProcessed<'a> {
    pub process: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstGraphConstantARM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub graph_constant_id: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstGraphARM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstGraphInputARM<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub input_index: IdRef,
    pub element_index: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstGraphEntryPointARM<'a> {
    pub graph: IdRef,
    pub name: LiteralString<'a>,
    pub interface: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstGraphSetOutputARM<'a> {
    pub value: IdRef,
    pub output_index: IdRef,
    pub element_index: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstGraphEndARM;
#[derive(Clone, Copy)]
pub struct InstSampledImage {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub sampler: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleDrefImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleDrefExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleProjImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleProjExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleProjDrefImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleProjDrefExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageFetch {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageGather {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub component: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageDrefGather {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageRead {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImage {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQueryFormat {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQueryOrder {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQuerySizeLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub level_of_detail: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQuerySize {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQueryLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQueryLevels {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageQuerySamples {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleDrefImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleDrefExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleProjImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleProjExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleProjDrefImplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseSampleProjDrefExplicitLod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: ImageOperands,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseFetch {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseGather {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub component: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseDrefGather {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub dref: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseTexelsResident {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub resident_code: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageSparseRead {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstColorAttachmentReadEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub attachment: IdRef,
    pub sample: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstDepthAttachmentReadEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sample: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstStencilAttachmentReadEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sample: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleWeightedQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub texture: IdRef,
    pub coordinates: IdRef,
    pub weights: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBoxFilterQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub texture: IdRef,
    pub coordinates: IdRef,
    pub box_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBlockMatchSSDQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target: IdRef,
    pub target_coordinates: IdRef,
    pub reference: IdRef,
    pub reference_coordinates: IdRef,
    pub block_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBlockMatchSADQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target: IdRef,
    pub target_coordinates: IdRef,
    pub reference: IdRef,
    pub reference_coordinates: IdRef,
    pub block_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBlockMatchWindowSSDQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target_sampled_image: IdRef,
    pub target_coordinates: IdRef,
    pub reference_sampled_image: IdRef,
    pub reference_coordinates: IdRef,
    pub block_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBlockMatchWindowSADQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target_sampled_image: IdRef,
    pub target_coordinates: IdRef,
    pub reference_sampled_image: IdRef,
    pub reference_coordinates: IdRef,
    pub block_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBlockMatchGatherSSDQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target_sampled_image: IdRef,
    pub target_coordinates: IdRef,
    pub reference_sampled_image: IdRef,
    pub reference_coordinates: IdRef,
    pub block_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageBlockMatchGatherSADQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target_sampled_image: IdRef,
    pub target_coordinates: IdRef,
    pub reference_sampled_image: IdRef,
    pub reference_coordinates: IdRef,
    pub block_size: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageSampleFootprintNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampled_image: IdRef,
    pub coordinate: IdRef,
    pub granularity: IdRef,
    pub coarse: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstConvertHandleToImageINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertHandleToSamplerINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertHandleToSampledImageINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstImageWrite {
    pub image: IdRef,
    pub coordinate: IdRef,
    pub texel: IdRef,
    pub image_operands: Option<ImageOperands>,
}
#[derive(Clone, Copy)]
pub struct InstFunction {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub function_control: FunctionControl,
    pub function_type: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFunctionParameter {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstFunctionCall<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub function: IdRef,
    pub arguments: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixPerElementOpNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
    pub func: IdRef,
    pub operands: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstFunctionEnd;
#[derive(Clone, Copy)]
pub struct InstExtInstImport<'a> {
    pub id_result: IdResult,
    pub name: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstExtInst<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub set: IdRef,
    pub instruction: LiteralExtInstInteger,
    pub operands: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstExtInstWithForwardRefsKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub set: IdRef,
    pub instruction: LiteralExtInstInteger,
    pub operands: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstExtension<'a> {
    pub name: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstConditionalExtensionINTEL<'a> {
    pub condition: IdRef,
    pub name: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstGroupAsyncCopy {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub destination: IdRef,
    pub source: IdRef,
    pub num_elements: IdRef,
    pub stride: IdRef,
    pub event: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupAll {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupAny {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupBroadcast {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub local_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupIAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupUMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupSMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupUMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupSMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupBallotKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupFirstInvocationKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupAllKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupAnyKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupAllEqualKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub predicate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupNonUniformRotateKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub value: IdRef,
    pub delta: IdRef,
    pub cluster_size: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupReadInvocationKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
    pub index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUntypedGroupAsyncCopyKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdRef,
    pub destination: IdRef,
    pub source: IdRef,
    pub element_num_bytes: IdRef,
    pub num_elements: IdRef,
    pub stride: IdRef,
    pub event: IdRef,
    pub destination_memory_operands: Option<MemoryAccess>,
    pub source_memory_operands: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstGroupIAddNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFAddNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFMinNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupUMinNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupSMinNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFMaxNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupUMaxNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupSMaxNonUniformAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupShuffleINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub data: IdRef,
    pub invocation_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupShuffleDownINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub current: IdRef,
    pub next: IdRef,
    pub delta: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupShuffleUpINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub previous: IdRef,
    pub current: IdRef,
    pub delta: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupShuffleXorINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub data: IdRef,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupBlockReadINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ptr: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupImageBlockReadINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupImageMediaBlockReadINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub width: IdRef,
    pub height: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupMatrixMultiplyAccumulateINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub k_dim: IdRef,
    pub matrix_a: IdRef,
    pub matrix_b: IdRef,
    pub matrix_c: IdRef,
    pub matrix_multiply_accumulate_operands: Option<MatrixMultiplyAccumulateOperands>,
}
#[derive(Clone, Copy)]
pub struct InstGroupIMulKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupFMulKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupBitwiseAndKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupBitwiseOrKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupBitwiseXorKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupLogicalAndKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupLogicalOrKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupLogicalXorKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub operation: GroupOperation,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupWaitEvents {
    pub execution: IdScope,
    pub num_events: IdRef,
    pub events_list: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupBlockWriteINTEL {
    pub ptr: IdRef,
    pub data: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupImageBlockWriteINTEL {
    pub image: IdRef,
    pub coordinate: IdRef,
    pub data: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupImageMediaBlockWriteINTEL {
    pub image: IdRef,
    pub coordinate: IdRef,
    pub width: IdRef,
    pub height: IdRef,
    pub data: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroupBlockPrefetchINTEL {
    pub ptr: IdRef,
    pub num_bytes: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstSubgroup2DBlockLoadINTEL {
    pub element_size: IdRef,
    pub block_width: IdRef,
    pub block_height: IdRef,
    pub block_count: IdRef,
    pub src_base_pointer: IdRef,
    pub memory_width: IdRef,
    pub memory_height: IdRef,
    pub memory_pitch: IdRef,
    pub coordinate: IdRef,
    pub dst_pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroup2DBlockLoadTransformINTEL {
    pub element_size: IdRef,
    pub block_width: IdRef,
    pub block_height: IdRef,
    pub block_count: IdRef,
    pub src_base_pointer: IdRef,
    pub memory_width: IdRef,
    pub memory_height: IdRef,
    pub memory_pitch: IdRef,
    pub coordinate: IdRef,
    pub dst_pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroup2DBlockLoadTransposeINTEL {
    pub element_size: IdRef,
    pub block_width: IdRef,
    pub block_height: IdRef,
    pub block_count: IdRef,
    pub src_base_pointer: IdRef,
    pub memory_width: IdRef,
    pub memory_height: IdRef,
    pub memory_pitch: IdRef,
    pub coordinate: IdRef,
    pub dst_pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroup2DBlockPrefetchINTEL {
    pub element_size: IdRef,
    pub block_width: IdRef,
    pub block_height: IdRef,
    pub block_count: IdRef,
    pub src_base_pointer: IdRef,
    pub memory_width: IdRef,
    pub memory_height: IdRef,
    pub memory_pitch: IdRef,
    pub coordinate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSubgroup2DBlockStoreINTEL {
    pub element_size: IdRef,
    pub block_width: IdRef,
    pub block_height: IdRef,
    pub block_count: IdRef,
    pub src_pointer: IdRef,
    pub dst_base_pointer: IdRef,
    pub memory_width: IdRef,
    pub memory_height: IdRef,
    pub memory_pitch: IdRef,
    pub coordinate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertFToU {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub float_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertFToS {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub float_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertSToF {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub signed_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertUToF {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub unsigned_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUConvert {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub unsigned_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSConvert {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub signed_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFConvert {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub float_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstQuantizeToF16 {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertPtrToU {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSatConvertSToU {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub signed_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSatConvertUToS {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub unsigned_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertUToPtr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub integer_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstPtrCastToGeneric {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGenericCastToPtr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGenericCastToPtrExplicit {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub storage: StorageClass,
}
#[derive(Clone, Copy)]
pub struct InstBitcast {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitCastArrayQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub source_array: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixConvertNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixTransposeNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertFToBF16INTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub float_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertBF16ToFINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub bfloat16_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRoundFToTF32INTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub float_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReadPipe {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub pointer: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstWritePipe {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub pointer: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReservedReadPipe {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub reserve_id: IdRef,
    pub index: IdRef,
    pub pointer: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReservedWritePipe {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub reserve_id: IdRef,
    pub index: IdRef,
    pub pointer: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReserveReadPipePackets {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub num_packets: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReserveWritePipePackets {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub num_packets: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIsValidReserveId {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub reserve_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetNumPipePackets {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetMaxPipePackets {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupReserveReadPipePackets {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub pipe: IdRef,
    pub num_packets: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupReserveWritePipePackets {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub execution: IdScope,
    pub pipe: IdRef,
    pub num_packets: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConstantPipeStorage {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub packet_size: LiteralInteger,
    pub packet_alignment: LiteralInteger,
    pub capacity: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstCreatePipeFromPipeStorage {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pipe_storage: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCommitReadPipe {
    pub pipe: IdRef,
    pub reserve_id: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCommitWritePipe {
    pub pipe: IdRef,
    pub reserve_id: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupCommitReadPipe {
    pub execution: IdScope,
    pub pipe: IdRef,
    pub reserve_id: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGroupCommitWritePipe {
    pub execution: IdScope,
    pub pipe: IdRef,
    pub reserve_id: IdRef,
    pub packet_size: IdRef,
    pub packet_alignment: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstEnqueueMarker {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub queue: IdRef,
    pub num_events: IdRef,
    pub wait_events: IdRef,
    pub ret_event: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstEnqueueKernel<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub queue: IdRef,
    pub flags: IdRef,
    pub nd_range: IdRef,
    pub num_events: IdRef,
    pub wait_events: IdRef,
    pub ret_event: IdRef,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
    pub local_size: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstGetKernelNDrangeSubGroupCount {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub nd_range: IdRef,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetKernelNDrangeMaxSubGroupSize {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub nd_range: IdRef,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetKernelWorkGroupSize {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetKernelPreferredWorkGroupSizeMultiple {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCreateUserEvent {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstIsValidEvent {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub event: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetDefaultQueue {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstBuildNDRange {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub global_work_size: IdRef,
    pub local_work_size: IdRef,
    pub global_work_offset: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetKernelLocalSizeForSubgroupCount {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub subgroup_count: IdRef,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstGetKernelMaxNumSubgroups {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub invoke: IdRef,
    pub param: IdRef,
    pub param_size: IdRef,
    pub param_align: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRetainEvent {
    pub event: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReleaseEvent {
    pub event: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSetUserEventStatus {
    pub event: IdRef,
    pub status: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCaptureEventProfilingInfo {
    pub event: IdRef,
    pub profiling_info: IdRef,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertUToAccelerationStructureKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub accel: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryProceedKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionTypeKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFragmentMaskFetchAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFragmentFetchAMD {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub fragment_index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReadClockKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub scope: IdScope,
}
#[derive(Clone, Copy)]
pub struct InstAllocateNodePayloadsAMDX {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub visibility: IdScope,
    pub payload_count: IdRef,
    pub node_index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeNodePayloadArrayAMDX {
    pub id_result: IdResult,
    pub payload_type: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFinishWritingNodePayloadAMDX {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstNodePayloadArrayLengthAMDX {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub payload_array: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIsNodePayloadValidAMDX {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub payload_type: IdRef,
    pub node_index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConstantStringAMDX<'a> {
    pub id_result: IdResult,
    pub literal_string: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantStringAMDX<'a> {
    pub id_result: IdResult,
    pub literal_string: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetWorldToObjectNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetObjectToWorldNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetObjectRayDirectionNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetObjectRayOriginNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetShaderRecordBufferHandleNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetShaderBindingTableRecordIndexNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetCurrentTimeNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetHitKindNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetPrimitiveIndexNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetGeometryIndexNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetInstanceIdNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetInstanceCustomIndexNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetWorldRayDirectionNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetWorldRayOriginNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetRayTMaxNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetRayTMinNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsEmptyNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsHitNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsMissNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeVectorMatrixMulNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub input: IdRef,
    pub input_interpretation: IdRef,
    pub matrix: IdRef,
    pub matrix_offset: IdRef,
    pub matrix_interpretation: IdRef,
    pub m: IdRef,
    pub k: IdRef,
    pub memory_layout: IdRef,
    pub transpose: IdRef,
    pub matrix_stride: Option<IdRef>,
    pub cooperative_matrix_operands: Option<CooperativeMatrixOperands>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeVectorMatrixMulAddNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub input: IdRef,
    pub input_interpretation: IdRef,
    pub matrix: IdRef,
    pub matrix_offset: IdRef,
    pub matrix_interpretation: IdRef,
    pub bias: IdRef,
    pub bias_offset: IdRef,
    pub bias_interpretation: IdRef,
    pub m: IdRef,
    pub k: IdRef,
    pub memory_layout: IdRef,
    pub transpose: IdRef,
    pub matrix_stride: Option<IdRef>,
    pub cooperative_matrix_operands: Option<CooperativeMatrixOperands>,
}
#[derive(Clone, Copy)]
pub struct InstFetchMicroTriangleVertexPositionNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub accel: IdRef,
    pub instance_id: IdRef,
    pub geometry_index: IdRef,
    pub primitive_index: IdRef,
    pub barycentric: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFetchMicroTriangleVertexBarycentricNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub accel: IdRef,
    pub instance_id: IdRef,
    pub geometry_index: IdRef,
    pub primitive_index: IdRef,
    pub barycentric: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetIntersectionTriangleVertexPositionsEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetRayFlagsEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetCurrentTimeEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetHitKindEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetPrimitiveIndexEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetGeometryIndexEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetInstanceIdEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetInstanceCustomIndexEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetObjectRayOriginEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetObjectRayDirectionEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetWorldRayDirectionEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetWorldRayOriginEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetObjectToWorldEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetWorldToObjectEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetRayTMaxEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReportIntersectionKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit: IdRef,
    pub hit_kind: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionTriangleVertexPositionsKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionClusterIdNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetClusterIdNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetRayTMinEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetShaderBindingTableRecordIndexEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetShaderRecordBufferHandleEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsEmptyEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsHitEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsMissEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixLoadNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub stride: IdRef,
    pub column_major: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixMulAddNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub a: IdRef,
    pub b: IdRef,
    pub c: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixLengthNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ty: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCreateTensorLayoutNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTensorLayoutSetDimensionNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_layout: IdRef,
    pub dim: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTensorLayoutSetStrideNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_layout: IdRef,
    pub stride: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTensorLayoutSliceNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_layout: IdRef,
    pub operands: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTensorLayoutSetClampValueNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_layout: IdRef,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCreateTensorViewNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTensorViewSetDimensionNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_view: IdRef,
    pub dim: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTensorViewSetStrideNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_view: IdRef,
    pub stride: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstIsHelperInvocationEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTensorViewSetClipNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_view: IdRef,
    pub clip_row_offset: IdRef,
    pub clip_row_span: IdRef,
    pub clip_col_offset: IdRef,
    pub clip_col_span: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTensorLayoutSetBlockSizeNV<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor_layout: IdRef,
    pub block_size: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstConvertUToImageNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertUToSamplerNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertImageToUNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertSamplerToUNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertUToSampledImageNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConvertSampledImageToUNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionSpherePositionNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionSphereRadiusNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionLSSPositionsNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionLSSRadiiNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionLSSHitValueNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetSpherePositionNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetSphereRadiusNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetLSSPositionsNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetLSSRadiiNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsSphereHitNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectIsLSSHitNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryIsSphereHitNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryIsLSSHitNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUCountLeadingZerosINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUCountTrailingZerosINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAbsISubINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAbsUSubINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIAddSatINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUAddSatINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIAverageINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUAverageINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIAverageRoundedINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUAverageRoundedINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstISubSatINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUSubSatINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIMul32x16INTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUMul32x16INTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetRayTMinKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetRayFlagsKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionTKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionInstanceCustomIndexKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionInstanceIdKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionInstanceShaderBindingTableRecordOffsetKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionGeometryIndexKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionPrimitiveIndexKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionBarycentricsKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionFrontFaceKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionCandidateAABBOpaqueKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionObjectRayDirectionKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionObjectRayOriginKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetWorldRayDirectionKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetWorldRayOriginKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionObjectToWorldKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGetIntersectionWorldToObjectKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ray_query: IdRef,
    pub intersection: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFDot2MixAcc32VALVE {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub accumulator: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFDot2MixAcc16VALVE {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub accumulator: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFDot4MixAcc32VALVE {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub accumulator: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTraceRayKHR {
    pub accel: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstExecuteCallableKHR {
    pub sbt_index: IdRef,
    pub callable_data: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIgnoreIntersectionKHR;
#[derive(Clone, Copy)]
pub struct InstTerminateRayKHR;
#[derive(Clone, Copy)]
pub struct InstRayQueryInitializeKHR {
    pub ray_query: IdRef,
    pub accel: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryTerminateKHR {
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryGenerateIntersectionKHR {
    pub ray_query: IdRef,
    pub hit_t: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstRayQueryConfirmIntersectionKHR {
    pub ray_query: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstEnqueueNodePayloadsAMDX {
    pub payload_array: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordHitMotionNV {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub instance_id: IdRef,
    pub primitive_id: IdRef,
    pub geometry_index: IdRef,
    pub hit_kind: IdRef,
    pub sbt_record_offset: IdRef,
    pub sbt_record_stride: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub current_time: IdRef,
    pub hit_object_attributes: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordHitWithIndexMotionNV {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub instance_id: IdRef,
    pub primitive_id: IdRef,
    pub geometry_index: IdRef,
    pub hit_kind: IdRef,
    pub sbt_record_index: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub current_time: IdRef,
    pub hit_object_attributes: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordMissMotionNV {
    pub hit_object: IdRef,
    pub sbt_index: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub current_time: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectTraceRayMotionNV {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub ray_flags: IdRef,
    pub cullmask: IdRef,
    pub sbt_record_offset: IdRef,
    pub sbt_record_stride: IdRef,
    pub miss_index: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub time: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordEmptyNV {
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectTraceRayNV {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub ray_flags: IdRef,
    pub cullmask: IdRef,
    pub sbt_record_offset: IdRef,
    pub sbt_record_stride: IdRef,
    pub miss_index: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordHitNV {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub instance_id: IdRef,
    pub primitive_id: IdRef,
    pub geometry_index: IdRef,
    pub hit_kind: IdRef,
    pub sbt_record_offset: IdRef,
    pub sbt_record_stride: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub hit_object_attributes: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordHitWithIndexNV {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub instance_id: IdRef,
    pub primitive_id: IdRef,
    pub geometry_index: IdRef,
    pub hit_kind: IdRef,
    pub sbt_record_index: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
    pub hit_object_attributes: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordMissNV {
    pub hit_object: IdRef,
    pub sbt_index: IdRef,
    pub origin: IdRef,
    pub tmin: IdRef,
    pub direction: IdRef,
    pub tmax: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectExecuteShaderNV {
    pub hit_object: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetAttributesNV {
    pub hit_object: IdRef,
    pub hit_object_attribute: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReorderThreadWithHitObjectNV {
    pub hit_object: IdRef,
    pub hint: Option<IdRef>,
    pub bits: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstReorderThreadWithHintNV {
    pub hint: IdRef,
    pub bits: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeVectorOuterProductAccumulateNV {
    pub pointer: IdRef,
    pub offset: IdRef,
    pub a: IdRef,
    pub b: IdRef,
    pub memory_layout: IdRef,
    pub matrix_interpretation: IdRef,
    pub matrix_stride: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeVectorReduceSumAccumulateNV {
    pub pointer: IdRef,
    pub offset: IdRef,
    pub v: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstEmitMeshTasksEXT {
    pub group_count_x: IdRef,
    pub group_count_y: IdRef,
    pub group_count_z: IdRef,
    pub payload: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstSetMeshOutputsEXT {
    pub vertex_count: IdRef,
    pub primitive_count: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstWritePackedPrimitiveIndices4x8NV {
    pub index_offset: IdRef,
    pub packed_indices: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordFromQueryEXT {
    pub hit_object: IdRef,
    pub ray_query: IdRef,
    pub sbt_record_index: IdRef,
    pub hit_object_attributes: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordMissEXT {
    pub hit_object: IdRef,
    pub ray_flags: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordMissMotionEXT {
    pub hit_object: IdRef,
    pub ray_flags: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub current_time: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectSetShaderBindingTableRecordIndexEXT {
    pub hit_object: IdRef,
    pub sbt_record_index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectReorderExecuteShaderEXT {
    pub hit_object: IdRef,
    pub payload: IdRef,
    pub hint: Option<IdRef>,
    pub bits: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectTraceReorderExecuteEXT {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub payload: IdRef,
    pub hint: Option<IdRef>,
    pub bits: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectTraceMotionReorderExecuteEXT {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub current_time: IdRef,
    pub payload: IdRef,
    pub hint: Option<IdRef>,
    pub bits: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstReorderThreadWithHintEXT {
    pub hint: IdRef,
    pub bits: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstReorderThreadWithHitObjectEXT {
    pub hit_object: IdRef,
    pub hint: Option<IdRef>,
    pub bits: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectTraceRayEXT {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectTraceRayMotionEXT {
    pub hit_object: IdRef,
    pub acceleration_structure: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub current_time: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectRecordEmptyEXT {
    pub hit_object: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectExecuteShaderEXT {
    pub hit_object: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstHitObjectGetAttributesEXT {
    pub hit_object: IdRef,
    pub hit_object_attribute: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIgnoreIntersectionNV;
#[derive(Clone, Copy)]
pub struct InstTerminateRayNV;
#[derive(Clone, Copy)]
pub struct InstTraceNV {
    pub accel: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub payload_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTraceMotionNV {
    pub accel: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub time: IdRef,
    pub payload_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTraceRayMotionNV {
    pub accel: IdRef,
    pub ray_flags: IdRef,
    pub cull_mask: IdRef,
    pub sbt_offset: IdRef,
    pub sbt_stride: IdRef,
    pub miss_index: IdRef,
    pub ray_origin: IdRef,
    pub ray_tmin: IdRef,
    pub ray_direction: IdRef,
    pub ray_tmax: IdRef,
    pub time: IdRef,
    pub payload: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstExecuteCallableNV {
    pub sbt_index: IdRef,
    pub callable_data_id: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixStoreNV {
    pub pointer: IdRef,
    pub object: IdRef,
    pub stride: IdRef,
    pub column_major: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstBeginInvocationInterlockEXT;
#[derive(Clone, Copy)]
pub struct InstEndInvocationInterlockEXT;
#[derive(Clone, Copy)]
pub struct InstSamplerImageAddressingModeNV {
    pub bit_width: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstLoopControlINTEL<'a> {
    pub loop_control_parameters: &'a [LiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstAtomicLoad {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstAtomicExchange {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicCompareExchange {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub equal: IdMemorySemantics,
    pub unequal: IdMemorySemantics,
    pub value: IdRef,
    pub comparator: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicCompareExchangeWeak {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub equal: IdMemorySemantics,
    pub unequal: IdMemorySemantics,
    pub value: IdRef,
    pub comparator: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicIIncrement {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstAtomicIDecrement {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstAtomicIAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicISub {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicSMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicUMin {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicSMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicUMax {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicAnd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicOr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicXor {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicFlagTestAndSet {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstAtomicFMinEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicFMaxEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicFAddEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicStore {
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAtomicFlagClear {
    pub pointer: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstVectorExtractDynamic {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector: IdRef,
    pub index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstVectorInsertDynamic {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector: IdRef,
    pub component: IdRef,
    pub index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstVectorShuffle<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub components: &'a [LiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstCompositeConstruct<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub constituents: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstCompositeExtract<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub composite: IdRef,
    pub indexes: &'a [LiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstCompositeInsert<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub object: IdRef,
    pub composite: IdRef,
    pub indexes: &'a [LiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstCopyObject {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTranspose {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCopyLogical {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCompositeConstructReplicateEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCompositeConstructCoopMatQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub source_array: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCompositeExtractCoopMatQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub source_cooperative_matrix: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstExtractSubArrayQCOM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub source_array: IdRef,
    pub index: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCompositeConstructContinuedINTEL<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub constituents: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstConditionalCopyObjectINTEL<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub conditions_and_operands_and: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstSNegate {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFNegate {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFAdd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstISub {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFSub {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIMul {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFMul {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUDiv {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSDiv {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFDiv {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUMod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSRem {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSMod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFRem {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFMod {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstVectorTimesScalar {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector: IdRef,
    pub scalar: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstMatrixTimesScalar {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
    pub scalar: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstVectorTimesMatrix {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector: IdRef,
    pub matrix: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstMatrixTimesVector {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
    pub vector: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstMatrixTimesMatrix {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub left_matrix: IdRef,
    pub right_matrix: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstOuterProduct {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIAddCarry {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstISubBorrow {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUMulExtended {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSMulExtended {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFmaKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
    pub operand_3: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSDot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub packed_vector_format: Option<PackedVectorFormat>,
}
#[derive(Clone, Copy)]
pub struct InstUDot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub packed_vector_format: Option<PackedVectorFormat>,
}
#[derive(Clone, Copy)]
pub struct InstSUDot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub packed_vector_format: Option<PackedVectorFormat>,
}
#[derive(Clone, Copy)]
pub struct InstSDotAccSat {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub accumulator: IdRef,
    pub packed_vector_format: Option<PackedVectorFormat>,
}
#[derive(Clone, Copy)]
pub struct InstUDotAccSat {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub accumulator: IdRef,
    pub packed_vector_format: Option<PackedVectorFormat>,
}
#[derive(Clone, Copy)]
pub struct InstSUDotAccSat {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector_1: IdRef,
    pub vector_2: IdRef,
    pub accumulator: IdRef,
    pub packed_vector_format: Option<PackedVectorFormat>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixMulAddKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub a: IdRef,
    pub b: IdRef,
    pub c: IdRef,
    pub cooperative_matrix_operands: Option<CooperativeMatrixOperands>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixReduceNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub matrix: IdRef,
    pub reduce: CooperativeMatrixReduce,
    pub combine_func: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTensorReadARM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor: IdRef,
    pub coordinates: IdRef,
    pub tensor_operands: Option<TensorOperands>,
}
#[derive(Clone, Copy)]
pub struct InstTensorQuerySizeARM {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub tensor: IdRef,
    pub dimension: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTensorWriteARM {
    pub tensor: IdRef,
    pub coordinates: IdRef,
    pub object: IdRef,
    pub tensor_operands: Option<TensorOperands>,
}
#[derive(Clone, Copy)]
pub struct InstConstantTrue {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstConstantFalse {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstConstant {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: LiteralContextDependentNumber,
}
#[derive(Clone, Copy)]
pub struct InstConstantComposite<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub constituents: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstConstantSampler {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub sampler_addressing_mode: SamplerAddressingMode,
    pub param: LiteralInteger,
    pub sampler_filter_mode: SamplerFilterMode,
}
#[derive(Clone, Copy)]
pub struct InstConstantNull {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantTrue {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantFalse {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstant {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: LiteralContextDependentNumber,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantComposite<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub constituents: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantOp<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub opcode: LiteralSpecConstantOpInteger<'a>,
}
#[derive(Clone, Copy)]
pub struct InstConstantCompositeReplicateEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantCompositeReplicateEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConstantSizeOfEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ty: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstConstantDataKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub data: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantDataKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub data: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantTargetINTEL<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target: LiteralInteger,
    pub features: &'a [LiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantArchitectureINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub category: LiteralInteger,
    pub family: LiteralInteger,
    pub opcode: LiteralInteger,
    pub architecture: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantCapabilitiesINTEL<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub capabilities: &'a [Capability],
}
#[derive(Clone, Copy)]
pub struct InstConstantCompositeContinuedINTEL<'a> {
    pub constituents: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstSpecConstantCompositeContinuedINTEL<'a> {
    pub constituents: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstUndef {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstSizeOf {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixLengthKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ty: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstPoisonKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstFreezeKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstExpectKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub value: IdRef,
    pub expected_value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstArithmeticFenceEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub target: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstNop;
#[derive(Clone, Copy)]
pub struct InstAssumeTrueKHR {
    pub condition: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstMemoryModel {
    pub addressing_model: AddressingModel,
    pub memory_model: MemoryModel,
}
#[derive(Clone, Copy)]
pub struct InstEntryPoint<'a> {
    pub execution_model: ExecutionModel,
    pub entry_point: IdRef,
    pub name: LiteralString<'a>,
    pub interface: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstExecutionMode {
    pub entry_point: IdRef,
    pub mode: ExecutionMode,
}
#[derive(Clone, Copy)]
pub struct InstCapability {
    pub capability: Capability,
}
#[derive(Clone, Copy)]
pub struct InstExecutionModeId {
    pub entry_point: IdRef,
    pub mode: ExecutionMode,
}
#[derive(Clone, Copy)]
pub struct InstConditionalEntryPointINTEL<'a> {
    pub condition: IdRef,
    pub execution_model: ExecutionModel,
    pub entry_point: IdRef,
    pub name: LiteralString<'a>,
    pub interface: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstConditionalCapabilityINTEL {
    pub condition: IdRef,
    pub capability: Capability,
}
#[derive(Clone, Copy)]
pub struct InstTypeVoid {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeBool {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeInt {
    pub id_result: IdResult,
    pub width: LiteralInteger,
    pub signedness: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstTypeFloat {
    pub id_result: IdResult,
    pub width: LiteralInteger,
    pub floating_point_encoding: Option<FPEncoding>,
}
#[derive(Clone, Copy)]
pub struct InstTypeVector {
    pub id_result: IdResult,
    pub component_type: IdRef,
    pub component_count: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstTypeMatrix {
    pub id_result: IdResult,
    pub column_type: IdRef,
    pub column_count: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstTypeImage {
    pub id_result: IdResult,
    pub sampled_type: IdRef,
    pub dim: Dim,
    pub depth: LiteralInteger,
    pub arrayed: LiteralInteger,
    pub ms: LiteralInteger,
    pub sampled: LiteralInteger,
    pub image_format: ImageFormat,
    pub access_qualifier: Option<AccessQualifier>,
}
#[derive(Clone, Copy)]
pub struct InstTypeSampler {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeSampledImage {
    pub id_result: IdResult,
    pub image_type: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeArray {
    pub id_result: IdResult,
    pub element_type: IdRef,
    pub length: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeRuntimeArray {
    pub id_result: IdResult,
    pub element_type: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeStruct<'a> {
    pub id_result: IdResult,
    pub member_types: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTypeOpaque<'a> {
    pub id_result: IdResult,
    pub the_name_of_the_opaque_type: LiteralString<'a>,
}
#[derive(Clone, Copy)]
pub struct InstTypePointer {
    pub id_result: IdResult,
    pub storage_class: StorageClass,
    pub ty: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeFunction<'a> {
    pub id_result: IdResult,
    pub return_type: IdRef,
    pub parameter_types: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTypeEvent {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeDeviceEvent {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeReserveId {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeQueue {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypePipe {
    pub id_result: IdResult,
    pub qualifier: AccessQualifier,
}
#[derive(Clone, Copy)]
pub struct InstTypePipeStorage {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeNamedBarrier {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeTensorARM {
    pub id_result: IdResult,
    pub element_type: IdRef,
    pub rank: Option<IdRef>,
    pub shape: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstTypeGraphARM<'a> {
    pub id_result: IdResult,
    pub num_inputs: LiteralInteger,
    pub in_out_types: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTypeUntypedPointerKHR {
    pub id_result: IdResult,
    pub storage_class: StorageClass,
}
#[derive(Clone, Copy)]
pub struct InstTypeCooperativeMatrixKHR {
    pub id_result: IdResult,
    pub component_type: IdRef,
    pub scope: IdScope,
    pub rows: IdRef,
    pub columns: IdRef,
    pub using: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeRayQueryKHR {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeBufferEXT {
    pub id_result: IdResult,
    pub storage_class: StorageClass,
}
#[derive(Clone, Copy)]
pub struct InstTypeHitObjectNV {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeVectorIdEXT {
    pub id_result: IdResult,
    pub component_type: IdRef,
    pub component_count: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeHitObjectEXT {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeAccelerationStructureKHR {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstTypeCooperativeMatrixNV {
    pub id_result: IdResult,
    pub component_type: IdRef,
    pub execution: IdScope,
    pub rows: IdRef,
    pub columns: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeTensorLayoutNV {
    pub id_result: IdResult,
    pub dim: IdRef,
    pub clamp_mode: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstTypeTensorViewNV<'a> {
    pub id_result: IdResult,
    pub dim: IdRef,
    pub has_dimensions: IdRef,
    pub p: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstTypeBufferSurfaceINTEL {
    pub id_result: IdResult,
    pub access_qualifier: AccessQualifier,
}
#[derive(Clone, Copy)]
pub struct InstTypeForwardPointer {
    pub pointer_type: IdRef,
    pub storage_class: StorageClass,
}
#[derive(Clone, Copy)]
pub struct InstTypeStructContinuedINTEL<'a> {
    pub member_types: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstVariable {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub storage_class: StorageClass,
    pub initializer: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstImageTexelPointer {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub sample: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLoad {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstAccessChain<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstInBoundsAccessChain<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstPtrAccessChain<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub element: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstArrayLength {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub structure: IdRef,
    pub array_member: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstGenericPtrMemSemantics {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstInBoundsPtrAccessChain<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub element: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstPtrEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstPtrNotEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstPtrDiff {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUntypedVariableKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub storage_class: StorageClass,
    pub data_type: Option<IdRef>,
    pub initializer: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstUntypedAccessChainKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base_type: IdRef,
    pub base: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstUntypedInBoundsAccessChainKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base_type: IdRef,
    pub base: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstUntypedPtrAccessChainKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base_type: IdRef,
    pub base: IdRef,
    pub element: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstUntypedInBoundsPtrAccessChainKHR<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base_type: IdRef,
    pub base: IdRef,
    pub element: IdRef,
    pub indexes: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstUntypedArrayLengthKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub structure: IdRef,
    pub pointer: IdRef,
    pub array_member: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixLoadKHR {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub memory_layout: IdRef,
    pub stride: Option<IdRef>,
    pub memory_operand: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstBufferPointerEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub buffer: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUntypedImageTexelPointerEXT {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub image_type: IdRef,
    pub image: IdRef,
    pub coordinate: IdRef,
    pub sample: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeVectorLoadNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub offset: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixLoadTensorNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub pointer: IdRef,
    pub object: IdRef,
    pub tensor_layout: IdRef,
    pub memory_operand: MemoryAccess,
    pub tensor_addressing_operands: TensorAddressingOperands,
}
#[derive(Clone, Copy)]
pub struct InstRawAccessChainNV {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub byte_stride: IdRef,
    pub element_index: IdRef,
    pub byte_offset: IdRef,
    pub raw_access_chain_operands: Option<RawAccessChainOperands>,
}
#[derive(Clone, Copy)]
pub struct InstVariableLengthArrayINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub length: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSaveMemoryINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstUntypedVariableLengthArrayINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub element_type: IdRef,
    pub length: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstMaskedGatherINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub ptr_vector: IdRef,
    pub alignment: LiteralInteger,
    pub mask: IdRef,
    pub fill_empty: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstStore {
    pub pointer: IdRef,
    pub object: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstCopyMemory {
    pub target: IdRef,
    pub source: IdRef,
    pub memory_access_1: Option<MemoryAccess>,
    pub memory_access_2: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstCopyMemorySized {
    pub target: IdRef,
    pub source: IdRef,
    pub size: IdRef,
    pub memory_access_1: Option<MemoryAccess>,
    pub memory_access_2: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstUntypedPrefetchKHR {
    pub pointer_type: IdRef,
    pub num_bytes: IdRef,
    pub rw: Option<IdRef>,
    pub locality: Option<IdRef>,
    pub cache_type: Option<IdRef>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixStoreKHR {
    pub pointer: IdRef,
    pub object: IdRef,
    pub memory_layout: IdRef,
    pub stride: Option<IdRef>,
    pub memory_operand: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeVectorStoreNV {
    pub pointer: IdRef,
    pub offset: IdRef,
    pub object: IdRef,
    pub memory_access: Option<MemoryAccess>,
}
#[derive(Clone, Copy)]
pub struct InstCooperativeMatrixStoreTensorNV {
    pub pointer: IdRef,
    pub object: IdRef,
    pub tensor_layout: IdRef,
    pub memory_operand: MemoryAccess,
    pub tensor_addressing_operands: TensorAddressingOperands,
}
#[derive(Clone, Copy)]
pub struct InstRestoreMemoryINTEL {
    pub ptr: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstMaskedScatterINTEL {
    pub input_vector: IdRef,
    pub ptr_vector: IdRef,
    pub alignment: LiteralInteger,
    pub mask: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAny {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstAll {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub vector: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIsNan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIsInf {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIsFinite {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIsNormal {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSignBitSet {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLessOrGreater {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
    pub y: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstOrdered {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
    pub y: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUnordered {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub x: IdRef,
    pub y: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLogicalEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLogicalNotEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLogicalOr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLogicalAnd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstLogicalNot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSelect {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub condition: IdRef,
    pub object_1: IdRef,
    pub object_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstIEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstINotEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUGreaterThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSGreaterThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUGreaterThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSGreaterThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstULessThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSLessThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstULessThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstSLessThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFOrdEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFUnordEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFOrdNotEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFUnordNotEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFOrdLessThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFUnordLessThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFOrdGreaterThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFUnordGreaterThan {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFOrdLessThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFUnordLessThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFOrdGreaterThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFUnordGreaterThanEqual {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDPdx {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDPdy {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFwidth {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDPdxFine {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDPdyFine {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFwidthFine {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDPdxCoarse {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDPdyCoarse {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstFwidthCoarse {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub p: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDecorationGroup {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstDecorate<'a> {
    pub target: IdRef,
    pub decoration: Decoration<'a>,
}
#[derive(Clone, Copy)]
pub struct InstMemberDecorate<'a> {
    pub structure_type: IdRef,
    pub member: LiteralInteger,
    pub decoration: Decoration<'a>,
}
#[derive(Clone, Copy)]
pub struct InstGroupDecorate<'a> {
    pub decoration_group: IdRef,
    pub targets: &'a [IdRef],
}
#[derive(Clone, Copy)]
pub struct InstGroupMemberDecorate<'a> {
    pub decoration_group: IdRef,
    pub targets: &'a [PairIdRefLiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstDecorateId<'a> {
    pub target: IdRef,
    pub decoration: Decoration<'a>,
}
#[derive(Clone, Copy)]
pub struct InstMemberDecorateIdEXT<'a> {
    pub structure_type: IdRef,
    pub member: LiteralInteger,
    pub decoration: Decoration<'a>,
}
#[derive(Clone, Copy)]
pub struct InstDecorateString<'a> {
    pub target: IdRef,
    pub decoration: Decoration<'a>,
}
#[derive(Clone, Copy)]
pub struct InstMemberDecorateString<'a> {
    pub struct_type: IdRef,
    pub member: LiteralInteger,
    pub decoration: Decoration<'a>,
}
#[derive(Clone, Copy)]
pub struct InstShiftRightLogical {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub shift: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstShiftRightArithmetic {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub shift: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstShiftLeftLogical {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub shift: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitwiseOr {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitwiseXor {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitwiseAnd {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand_1: IdRef,
    pub operand_2: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstNot {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub operand: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitFieldInsert {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub insert: IdRef,
    pub offset: IdRef,
    pub count: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitFieldSExtract {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub offset: IdRef,
    pub count: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitFieldUExtract {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
    pub offset: IdRef,
    pub count: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitReverse {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitCount {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub base: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBitwiseFunctionINTEL {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub a: IdRef,
    pub b: IdRef,
    pub c: IdRef,
    pub lutindex: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstEmitVertex;
#[derive(Clone, Copy)]
pub struct InstEndPrimitive;
#[derive(Clone, Copy)]
pub struct InstEmitStreamVertex {
    pub stream: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstEndStreamPrimitive {
    pub stream: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstNamedBarrierInitialize {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub subgroup_count: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstControlBarrier {
    pub execution: IdScope,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstMemoryBarrier {
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstMemoryNamedBarrier {
    pub named_barrier: IdRef,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstControlBarrierArriveINTEL {
    pub execution: IdScope,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstControlBarrierWaitINTEL {
    pub execution: IdScope,
    pub memory: IdScope,
    pub semantics: IdMemorySemantics,
}
#[derive(Clone, Copy)]
pub struct InstPhi<'a> {
    pub id_result_type: IdResultType,
    pub id_result: IdResult,
    pub variables: &'a [PairIdRefIdRef],
}
#[derive(Clone, Copy)]
pub struct InstLabel {
    pub id_result: IdResult,
}
#[derive(Clone, Copy)]
pub struct InstLoopMerge {
    pub merge_block: IdRef,
    pub continue_target: IdRef,
    pub loop_control: LoopControl,
}
#[derive(Clone, Copy)]
pub struct InstSelectionMerge {
    pub merge_block: IdRef,
    pub selection_control: SelectionControl,
}
#[derive(Clone, Copy)]
pub struct InstBranch {
    pub target_label: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstBranchConditional<'a> {
    pub condition: IdRef,
    pub true_label: IdRef,
    pub false_label: IdRef,
    pub branch_weights: &'a [LiteralInteger],
}
#[derive(Clone, Copy)]
pub struct InstSwitch<'a> {
    pub selector: IdRef,
    pub default: IdRef,
    pub target: &'a [PairLiteralIntegerIdRef],
}
#[derive(Clone, Copy)]
pub struct InstKill;
#[derive(Clone, Copy)]
pub struct InstReturn;
#[derive(Clone, Copy)]
pub struct InstReturnValue {
    pub value: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstUnreachable;
#[derive(Clone, Copy)]
pub struct InstLifetimeStart {
    pub pointer: IdRef,
    pub size: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstLifetimeStop {
    pub pointer: IdRef,
    pub size: LiteralInteger,
}
#[derive(Clone, Copy)]
pub struct InstTerminateInvocation;
#[derive(Clone, Copy)]
pub struct InstAbortKHR {
    pub message_type: IdRef,
    pub message: IdRef,
}
#[derive(Clone, Copy)]
pub struct InstDemoteToHelperInvocation;
