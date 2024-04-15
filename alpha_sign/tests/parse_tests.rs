use alpha_sign::text::ReadText;
use alpha_sign::text::WriteText;
use alpha_sign::Command;
use alpha_sign::Packet;
use alpha_sign::SignSelector;

#[test]
fn test_parse_write_text() {
    let pkt = Packet::new(
        vec![SignSelector::default()],
        vec![Command::WriteText(WriteText::new('A', "test".to_string()))],
    );

    let Ok((_, res)) = Packet::parse(pkt.encode().unwrap().as_slice()) else {
        panic!()
    };

    assert_eq!(res, pkt)
}

#[test]
fn test_parse_read_text() {
    let pkt = Packet::new(
        vec![SignSelector::default()],
        vec![Command::ReadText(ReadText::new('A'))],
    );

    match Packet::parse(pkt.encode().unwrap().as_slice()) {
        Ok((_, res)) => assert_eq!(pkt, res),
        Err(e) => println!("{:#?}", e),
    };
}

#[test]
fn test_parse_multiple_selectors() {
    let pkt = Packet::new(
        vec![
            SignSelector::default(),
            SignSelector {
                sign_type: alpha_sign::SignType::All,
                address: 0x69,
            },
        ],
        vec![Command::WriteText(WriteText::new('A', "test".to_string()))],
    );

    let Ok((_, res)) = Packet::parse(pkt.encode().unwrap().as_slice()) else {
        panic!()
    };

    assert_eq!(res, pkt)
}

#[test]
fn test_parse_multiple_commands() {
    let pkt = Packet::new(
        vec![SignSelector::default()],
        vec![
            Command::WriteText(WriteText::new('A', "test".to_string())),
            Command::WriteText(WriteText::new('B', "test 2".to_string())),
        ],
    );

    let Ok((_, res)) = Packet::parse(pkt.encode().unwrap().as_slice()) else {
        panic!()
    };

    assert_eq!(res, pkt)
}

#[test]
fn test_parse_multiple_commands_and_selectors() {
    let pkt = Packet::new(
        vec![
            SignSelector::default(),
            SignSelector {
                sign_type: alpha_sign::SignType::All,
                address: 0x69,
            },
        ],
        vec![
            Command::WriteText(WriteText::new('A', "test".to_string())),
            Command::WriteText(WriteText::new('B', "test 2".to_string())),
        ],
    );

    let Ok((_, res)) = Packet::parse(pkt.encode().unwrap().as_slice()) else {
        panic!()
    };

    assert_eq!(res, pkt)
}
