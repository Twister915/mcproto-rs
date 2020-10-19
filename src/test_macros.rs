#[cfg(all(test, feature = "std"))]
#[macro_export]
macro_rules! packet_test_cases {
    ($pnam: ident, $varnam: ident, $bodnam: ident, $testnam: ident, $benchnams: ident, $benchnamd: ident) => {
        #[test]
        fn $testnam() {
            for k in 0..10 {
                let packet = $pnam::$varnam($bodnam::test_gen_random());
                if k == 0 {
                    println!("{:?}", packet);
                }
                let mut out = BytesSerializer::default();
                packet.mc_serialize(&mut out).expect("serialize succeeds");
                let bytes = out.into_bytes();

                let raw_packet = RawPacket {
                    id: packet.id(),
                    data: bytes.as_slice(),
                };

                let deserialized = match <$pnam>::mc_deserialize(raw_packet) {
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
            let packet = $pnam::$varnam($bodnam::test_gen_random());
            let mut serializer = BenchSerializer::default();
            packet
                .mc_serialize(&mut serializer)
                .expect("serialize succeeds");
            b.bytes = serializer.len() as u64;
            serializer.reset();

            b.iter(|| {
                packet
                    .mc_serialize(&mut serializer)
                    .expect("serialize succeeds");
                serializer.reset();
            })
        }

        #[cfg(feature = "bench")]
        #[bench]
        fn $benchnamd(b: &mut test::Bencher) {
            let packet = $pnam::$varnam($bodnam::test_gen_random());
            let mut serializer = BytesSerializer::default();
            packet
                .mc_serialize(&mut serializer)
                .expect("serialize succeeds");

            let bytes = serializer.into_bytes();
            b.bytes = bytes.len() as u64;
            let raw_packet = RawPacket {
                id: packet.id(),
                data: bytes.as_slice(),
            };
            b.iter(|| {
                $pnam::mc_deserialize(raw_packet.clone()).expect("deserialize succeeds");
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
