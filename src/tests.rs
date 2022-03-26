use super::*;
use hex;

#[test]
fn test_ptrbuffer() {
    let data = hex::decode("deadbeefabad1deadeadbea7defaced1").unwrap();
    let buffer = PtrBuffer::new(data.as_ptr(), data.len());

    assert_eq!(buffer.as_ptr(), data.as_ptr());
    assert_eq!(buffer.len(), data.len());
    assert_eq!(buffer.as_ptr(), data.as_ptr());
    unsafe { assert_eq!(buffer.eob(), data.as_ptr().add(data.len())); }

    let byte_result = buffer.get_ref::<i8>(0);
    assert!(byte_result.is_ok());
    assert!(*byte_result.unwrap() == -34);

    let error_result = buffer.get_ref::<i8>(buffer.len());
    assert!(error_result.is_err());

    #[repr(packed)]
    #[derive(Copy, Clone, Debug)]
    struct StructTest {
        deadbe: [u8; 3],
        efab: u16,
        ad1deade: u32,
    }
    unsafe impl Pod for StructTest { }
    unsafe impl Zeroable for StructTest {
        fn zeroed() -> Self {
            Self { deadbe: [0,0,0], efab: 0, ad1deade: 0 }
        }
    }

    let struct_result = buffer.get_ref::<StructTest>(0);
    assert!(struct_result.is_ok());

    let struct_data = struct_result.unwrap();
    assert_eq!(struct_data.deadbe, [0xDE, 0xAD, 0xBE]);

    let efab_unaligned = struct_data.efab;
    assert_eq!(efab_unaligned, 0xABEF);

    let ad1deade_unaligned = struct_data.ad1deade;
    assert_eq!(ad1deade_unaligned, 0xDEEA1DAD);

    let struct_result = buffer.get_ref::<StructTest>(4);
    assert!(struct_result.is_ok());

    let struct_data = struct_result.unwrap();
    assert_eq!(struct_data.deadbe, [0xAB, 0xAD, 0x1D]);

    let efab_unaligned = struct_data.efab;
    assert_eq!(efab_unaligned, 0xDEEA);

    let ad1deade_unaligned = struct_data.ad1deade;
    assert_eq!(ad1deade_unaligned, 0xDEA7BEAD);

    let read_result = buffer.read(8, 4);
    assert!(read_result.is_ok());

    let read_data = read_result.unwrap();
    assert_eq!(read_data, [0xDE, 0xAD, 0xBE, 0xA7]);

    let read_result = buffer.read(0xC, 4);
    assert!(read_result.is_ok());

    let read_data = read_result.unwrap();
    assert_eq!(read_data, [0xDE, 0xFA, 0xCE, 0xD1]);

    let itered_vec = buffer.iter().copied().collect::<Vec<u8>>();
    assert_eq!(itered_vec, data);

    let error_slice = buffer.get_slice_ref::<StructTest>(0,2);
    assert!(error_slice.is_err());

    let addr24 = buffer.get_slice_ref::<[u8; 3]>(0,2);
    assert!(addr24.is_ok());
    assert_eq!(addr24.unwrap(), [[0xDE, 0xAD, 0xBE],[0xEF, 0xAB, 0xAD]]);

    let search_results = buffer.search([0xDE, 0xFA, 0xCE, 0xD1]);
    assert!(search_results.is_ok());
    assert!(search_results.unwrap().next().is_some());

    let search_results = buffer.search_ref::<u32>(&0xFACEBABE);
    assert!(search_results.is_ok());
    assert!(search_results.unwrap().next().is_none());

    let search_results = buffer.search_ref::<u32>(&0xADABEFBE);
    assert!(search_results.is_ok());
    assert!(search_results.unwrap().next().is_some());

    let search_results = buffer.search_slice_ref::<u16>(&[0xADDE, 0xEFBE]);
    assert!(search_results.is_ok());
    assert!(search_results.unwrap().next().is_some());

    assert!(buffer.contains([0xDE, 0xAD, 0xBE, 0xA7]));
    assert!(!buffer.contains_ref::<u32>(&0xFACEBABE).unwrap());
    assert!(buffer.contains_ref::<u32>(&0xEA1DADAB).unwrap());
    assert!(buffer.contains_slice_ref::<u32>(&[0xA7BEADDE, 0xD1CEFADE]).unwrap());
    
    assert_eq!(buffer[0x8..0xC], [0xDE, 0xAD, 0xBE, 0xA7]);
}

#[test]
fn test_vecbuffer() {
    let data = hex::decode("deadbeefabad1deadeadbea7defaced1").unwrap();
    let mut buffer = VecBuffer::from_data(&data);

    assert!(buffer.write(0, &[0xFA, 0xCE, 0xBA, 0xBE]).is_ok());
    assert!(!buffer.contains([0xDE, 0xAD, 0xBE, 0xEF]));

    assert!(buffer.write_ref::<u32>(4, &0xEFBEADDE).is_ok());
    assert!(buffer.contains_ref::<u32>(&0xEFBEADDE).unwrap());

    buffer.append(&vec![0xAB, 0xAD, 0x1D, 0xEA]);
    assert!(buffer.contains([0xAB, 0xAD, 0x1D, 0xEA]));

    let rhs = buffer.split_off(0x8);
    assert!(!buffer.contains([0xAB, 0xAD, 0x1D, 0xEA]));

    buffer.resize(0xC, 0x00);
    assert!(buffer.write_ref::<u32>(0x8, &0x74EEFFC0).is_ok());

    buffer.append(&rhs);
    assert!(buffer.contains([0xAB, 0xAD, 0x1D, 0xEA]));
    assert!(buffer.contains([0xC0, 0xFF, 0xEE, 0x74]));

    assert_eq!(buffer, hex::decode("facebabedeadbeefc0ffee74deadbea7defaced1abad1dea").unwrap());
}
