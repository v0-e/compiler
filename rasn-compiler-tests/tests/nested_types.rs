#![allow(non_camel_case_types)]
use rasn_compiler_tests::e2e_pdu;

#[test]
fn t() {
    println!(
        "{}",
        rasn_compiler::RasnCompiler::new()
            .add_asn_literal(
                r#" 
        TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN

        Test-String ::= BMPString
        test-string-val Test-String ::= "012345"

        END
        "#
            )
            .compile_to_string()
            .unwrap()
            .0
    )
}

e2e_pdu!(
    boolean,
    r#" Test-Boolean ::= BOOLEAN
        Wrapping-Boolean ::= Test-Boolean
        value Wrapping-Boolean ::= FALSE"#,
    r#" #[derive(AsnType, Debug, Clone, Copy, Decode, Encode, PartialEq)]
        #[rasn(delegate)]
        pub struct TestBoolean(pub bool);
        
        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
        #[rasn(delegate)]
        pub struct WrappingBoolean(pub TestBoolean);                                 
        
        pub const VALUE: WrappingBoolean = WrappingBoolean(TestBoolean(false));         "#
);

e2e_pdu!(
    integer,
    r#" Test-Int ::= INTEGER (0..123723)
        Wrapping-Int ::= Test-Int (0..123)
        value Wrapping-Int ::= 4"#,
    r#" #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
        #[rasn(delegate, value("0..=123723"))]
        pub struct TestInt(pub u32);
        
        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
        #[rasn(delegate, value("0..=123"))]
        pub struct WrappingInt(pub TestInt);                                 
        
        pub const VALUE: WrappingInt = WrappingInt(TestInt(4));         "#
);

e2e_pdu!(
    sequence,
    r#" Test-Int ::= INTEGER (0..123723)
        Wrapping-Int ::= Test-Int (0..123)
        Test-Boolean ::= BOOLEAN
        Wrapping-Boolean ::= Test-Boolean
        Test-Sequence ::= SEQUENCE {
            int Wrapping-Int DEFAULT 5,
            boolean Wrapping-Boolean,
        }
        value Test-Sequence ::= { boolean TRUE }"#,
    r#" #[derive(AsnType, Debug, Clone, Copy, Decode, Encode, PartialEq)]
        #[rasn(delegate)]
        pub struct TestBoolean(pub bool);

        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
        #[rasn(delegate, value("0..=123723"))]
        pub struct TestInt(pub u32);

        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
        #[rasn(automatic_tags)]
        pub struct TestSequence {
            #[rasn(default = "test_sequence_int_default")]
            pub int: WrappingInt,
            pub boolean: WrappingBoolean,
        }
        impl TestSequence {
            pub fn new(int: WrappingInt, boolean: WrappingBoolean) -> Self {
                Self { int, boolean }
            }
        }
        fn test_sequence_int_default() -> WrappingInt {
            WrappingInt(TestInt(5))
        }

        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
        #[rasn(delegate)]
        pub struct WrappingBoolean(pub TestBoolean);

        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
        #[rasn(delegate, value("0..=123"))]
        pub struct WrappingInt(pub TestInt);

        lazy_static! {
            pub static ref VALUE: TestSequence = TestSequence::new(
                WrappingInt(TestInt(5)),
                WrappingBoolean(TestBoolean(true))
            );
        }                                                                        "#
);

e2e_pdu!(
    constraint_cross_reference,
    r#" Test-Int ::= INTEGER (0..123723)
        Wrapping-Int ::= Test-Int (0..value)
        value Test-Int ::= 5"#,
    r#" #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq, PartialOrd, Eq, Ord, Hash)]
        #[rasn(delegate, value("0..=123723"))]
        pub struct TestInt(pub u32);
        
        #[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
        #[rasn(delegate, value("0..=5"))]
        pub struct WrappingInt(pub TestInt);                                 
        
        pub const VALUE: TestInt = TestInt(5);         "#
);
