#[cfg(all(test, feature = "std"))]
#[macro_export]
macro_rules! packet_test_cases {
    ($rawnam: ident, $pnam: ident, $varnam: ident, $bodnam: ident, $testnam: ident, $benchnams: ident, $benchnamd: ident) => {
        #[test]
        fn $testnam() {
            use crate::protocol::{RawPacket, HasPacketBody, HasPacketId};
            for k in 0..50 {
                let packet = $pnam::$varnam($bodnam::test_gen_random());
                if k == 0 {
                    println!("{:?}", packet);
                }
                let mut out = crate::types::BytesSerializer::default();
                packet.mc_serialize_body(&mut out).expect("serialize succeeds");
                let bytes = out.into_bytes();

                let raw_packet = $rawnam::create(packet.id(), bytes.as_slice()).expect("valid id");
                let deserialized = match raw_packet.deserialize() {
                    Err(err) => {
                        eprintln!("expected: {:?}", packet);
                        panic!("error: {:?}", err);
                    },
                    Ok(out) => out
                };
                assert_eq!(packet, deserialized, "deserialize(serialize(packet)) == packet");
                assert_eq!(packet.clone(), deserialized.clone(), "deserialized.clone() == packet.clone()")
            }
        }

        #[cfg(feature = "bench")]
        #[bench]
        fn $benchnams(b: &mut test::Bencher) {
            use crate::protocol::{HasPacketBody};
            let packet = $pnam::$varnam($bodnam::test_gen_random());
            let mut serializer = crate::test_macros::BenchSerializer::default();
            packet
                .mc_serialize_body(&mut serializer)
                .expect("serialize succeeds");
            b.bytes = serializer.len() as u64;
            serializer.reset();

            b.iter(|| {
                packet
                    .mc_serialize_body(&mut serializer)
                    .expect("serialize succeeds");
                serializer.reset();
            })
        }

        #[cfg(feature = "bench")]
        #[bench]
        fn $benchnamd(b: &mut test::Bencher) {
            use crate::protocol::{RawPacket, HasPacketBody, HasPacketId};
            let packet = $pnam::$varnam($bodnam::test_gen_random());
            let mut serializer = crate::types::BytesSerializer::default();
            packet
                .mc_serialize_body(&mut serializer)
                .expect("serialize succeeds");

            let bytes = serializer.into_bytes();
            b.bytes = bytes.len() as u64;
            let raw_packet = $rawnam::create(packet.id(), bytes.as_slice()).expect("valid id");
            b.iter(|| {
                raw_packet.deserialize().expect("deserialize succeeds");
            })
        }
    };
}

#[cfg(all(test, feature = "std", feature = "bench"))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchSerializer {
    data: alloc::vec::Vec<u8>,
}

#[cfg(all(test, feature = "std", feature = "bench"))]
impl crate::Serializer for BenchSerializer {
    fn serialize_bytes(&mut self, data: &[u8]) -> crate::SerializeResult {
        self.data.extend_from_slice(data);
        Ok(())
    }
}

#[cfg(all(test, feature = "std", feature = "bench"))]
impl BenchSerializer {
    pub fn reset(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
