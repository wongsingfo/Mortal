use super::{ActionCandidate, PlayerState};
use crate::hand::{hand, hand_with_aka, tile37_to_vec};
use crate::mjai::Event;
use crate::{must_tile, t, tuz};
use std::convert::TryInto;

// This is not only a helper but it also tests `encode_obs`.
fn state_from_log(player_id: u8, log: &str) -> PlayerState {
    let mut ps = PlayerState::new(player_id);
    for line in log.trim().split('\n') {
        let cans = ps.update_json(line).unwrap();
        if cans.can_act() {
            let _encoded = ps.encode_obs(false);
            if cans.can_daiminkan || cans.can_kakan || cans.can_ankan {
                let _encoded = ps.encode_obs(true);
            }
        }
    }
    ps
}

#[test]
fn waits() {
    let mut ps = PlayerState {
        tehai: hand("456m 78999p 789s 77z").unwrap(),
        tehai_len_div3: 4,
        ..Default::default()
    };
    ps.update_waits_and_furiten();
    let expected = t![6p, 9p, C];
    for (idx, &b) in ps.waits.iter().enumerate() {
        if expected.contains(&must_tile!(idx)) {
            assert!(b);
        } else {
            assert!(!b);
        }
    }

    let mut ps = PlayerState {
        tehai: hand("2344445666678s").unwrap(),
        tehai_len_div3: 4,
        ..Default::default()
    };
    ps.update_waits_and_furiten();
    let expected = t![1s, 2s, 3s, 5s, 7s, 8s, 9s];
    for (idx, &b) in ps.waits.iter().enumerate() {
        if expected.contains(&must_tile!(idx)) {
            assert!(b);
        } else {
            assert!(!b);
        }
    }
}

#[test]
fn can_chi() {
    let mut ps = PlayerState::new(0);
    ps.tehai = hand("1111234m").unwrap();
    ps.set_can_chi_from_tile(t!(1m));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: false,
            can_chi_low: false,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(4m));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: false,
            can_chi_low: false,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(2m));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: true,
            can_chi_low: true,
            ..
        },
    ));

    ps.tehai = hand("6666789999p").unwrap();
    ps.set_can_chi_from_tile(t!(5p));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: false,
            can_chi_low: true,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(7p));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: true,
            can_chi_low: true,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(8p));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: true,
            can_chi_mid: true,
            can_chi_low: false,
            ..
        },
    ));

    ps.tehai = hand("4556s").unwrap();
    ps.set_can_chi_from_tile(t!(3s));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: false,
            can_chi_low: true,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(4s));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: false,
            can_chi_low: true,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(5s));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: false,
            can_chi_mid: false,
            can_chi_low: false,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(6s));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: true,
            can_chi_mid: false,
            can_chi_low: false,
            ..
        },
    ));
    ps.set_can_chi_from_tile(t!(7s));
    assert!(matches!(
        ps.last_cans,
        ActionCandidate {
            can_chi_high: true,
            can_chi_mid: false,
            can_chi_low: false,
            ..
        },
    ));
}

#[test]
fn furiten() {
    let mut ps = PlayerState::new(0);
    ps.update(&Event::StartKyoku {
        bakaze: t!(E),
        kyoku: 1,
        honba: 0,
        kyotaku: 0,
        oya: 0,
        scores: [25000; 4],
        dora_marker: t!(3p),
        tehais: [
            tile37_to_vec(&hand_with_aka("23406m 456789p 58s").unwrap())
                .try_into()
                .unwrap(),
            [t!(?); 13],
            [t!(?); 13],
            [t!(?); 13],
        ],
    });
    ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(8s),
    });
    assert!(ps.shanten == 1);
    assert!(ps.waits.iter().all(|&b| !b));
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(5s),
        tsumogiri: false,
    });
    assert!(ps.shanten == 0);
    assert!(ps.waits[tuz!(1m)] && ps.waits[tuz!(4m)] && ps.waits[tuz!(7m)]);
    assert!(!ps.at_furiten);

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    let cans = ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(1m),
        tsumogiri: false,
    });
    assert!(!ps.at_furiten);
    assert!(cans.can_ron_agari);

    ps.update(&Event::Tsumo {
        actor: 2,
        pai: t!(?),
    });
    assert!(ps.at_furiten);
    ps.update(&Event::Dahai {
        actor: 2,
        pai: t!(1s),
        tsumogiri: true,
    });

    ps.update(&Event::Tsumo {
        actor: 3,
        pai: t!(?),
    });
    let cans = ps.update(&Event::Dahai {
        actor: 3,
        pai: t!(1m),
        tsumogiri: false,
    });
    assert!(ps.shanten == 0);
    assert!(ps.waits[tuz!(1m)] && ps.waits[tuz!(4m)] && ps.waits[tuz!(7m)]);
    assert!(ps.at_furiten);
    assert!(!cans.can_ron_agari);

    ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(3s),
    });
    assert!(ps.at_furiten);
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(3s),
        tsumogiri: true,
    });
    assert!(!ps.at_furiten);

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(P),
        tsumogiri: true,
    });

    ps.update(&Event::Tsumo {
        actor: 2,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 2,
        pai: t!(C),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 3,
        pai: t!(?),
    });
    let cans = ps.update(&Event::Dahai {
        actor: 3,
        pai: t!(1m),
        tsumogiri: false,
    });
    assert!(!ps.at_furiten);
    assert!(cans.can_ron_agari);
    assert_eq!(ps.agari_points(true, &[]).unwrap().ron, 5800);

    // riichi furiten test
    let cans = ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(N),
    });
    assert!(cans.can_riichi);
    ps.update(&Event::Reach { actor: 0 });
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(N),
        tsumogiri: true,
    });
    ps.update(&Event::ReachAccepted { actor: 0 });

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(9m),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 2,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 2,
        pai: t!(9m),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 3,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 3,
        pai: t!(9m),
        tsumogiri: true,
    });

    // tsumo agari minogashi
    let cans = ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(1m),
    });
    assert!(ps.waits[tuz!(1m)] && ps.waits[tuz!(4m)] && ps.waits[tuz!(7m)]);
    assert!(!ps.at_furiten);
    assert!(cans.can_tsumo_agari);
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(1m),
        tsumogiri: true,
    });
    assert!(ps.at_furiten); // furiten forever from now on

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(4s),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 2,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 2,
        pai: t!(4s),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 3,
        pai: t!(?),
    });
    let cans = ps.update(&Event::Dahai {
        actor: 3,
        pai: t!(7m),
        tsumogiri: true,
    });
    assert!(ps.waits[tuz!(1m)] && ps.waits[tuz!(4m)] && ps.waits[tuz!(7m)]);
    assert!(ps.at_furiten);
    assert!(!cans.can_ron_agari);

    ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(8m),
    });
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(8m),
        tsumogiri: true,
    });
    assert!(ps.at_furiten); // still furiten

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(E),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 2,
        pai: t!(?),
    });
    let cans = ps.update(&Event::Dahai {
        actor: 2,
        pai: t!(4m),
        tsumogiri: true,
    });
    assert!(ps.at_furiten);
    assert!(!cans.can_ron_agari);
    ps.update(&Event::Tsumo {
        actor: 3,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 3,
        pai: t!(E),
        tsumogiri: true,
    });

    // tsumo agari is always possible regardless of furiten
    let cans = ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(4m),
    });
    assert!(ps.waits[0] && ps.waits[3] && ps.waits[6]);
    assert!(ps.at_furiten);
    assert!(cans.can_tsumo_agari);
    assert_eq!(ps.agari_points(false, &[t!(3m)]).unwrap().tsumo_ko, 6000);
}

#[test]
fn dora_count_after_kan() {
    let mut ps = PlayerState::new(0);
    ps.update(&Event::StartKyoku {
        bakaze: t!(E),
        kyoku: 1,
        honba: 0,
        kyotaku: 0,
        oya: 0,
        scores: [25000; 4],
        dora_marker: t!(N),
        tehais: [
            tile37_to_vec(&hand_with_aka("1111s 123456p 112z").unwrap())
                .try_into()
                .unwrap(),
            [t!(?); 13],
            [t!(?); 13],
            [t!(?); 13],
        ],
    });
    ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(8s),
    });
    assert_eq!(ps.doras_owned[0], 2);

    ps.update(&Event::Ankan {
        actor: 0,
        consumed: [t!(1s); 4],
    });
    ps.update(&Event::Dora {
        dora_marker: t!(9s),
    });
    ps.update(&Event::Tsumo {
        actor: 0,
        pai: t!(5pr),
    });
    assert_eq!(ps.doras_owned[0], 7);
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(E),
        tsumogiri: true,
    });
    assert_eq!(ps.doras_owned[0], 6);

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(5p),
        tsumogiri: true,
    });

    ps.update(&Event::Pon {
        actor: 0,
        target: 1,
        pai: t!(5p),
        consumed: t![5pr, 5p],
    });
    assert_eq!(ps.doras_owned[0], 6);
    ps.update(&Event::Dahai {
        actor: 0,
        pai: t!(E),
        tsumogiri: false,
    });
    assert_eq!(ps.doras_owned[0], 5);

    ps.update(&Event::Tsumo {
        actor: 1,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 1,
        pai: t!(P),
        tsumogiri: true,
    });
    ps.update(&Event::Tsumo {
        actor: 2,
        pai: t!(?),
    });
    ps.update(&Event::Dahai {
        actor: 2,
        pai: t!(P),
        tsumogiri: true,
    });

    ps.update(&Event::Tsumo {
        actor: 3,
        pai: t!(?),
    });
    ps.update(&Event::Ankan {
        actor: 3,
        consumed: [t!(1m); 4],
    });
    ps.update(&Event::Dora {
        dora_marker: t!(4p),
    });
    assert_eq!(ps.doras_owned[0], 8);
}

#[test]
fn rule_based_agari_all_last_minogashi() {
    let log = r#"
        {"type":"start_kyoku","bakaze":"S","dora_marker":"5m","kyoku":4,"honba":0,"kyotaku":0,"oya":3,"scores":[35300,3000,38400,23300],"tehais":[["4m","5mr","8m","1p","3p","3p","5p","2s","5sr","9s","W","P","P"],["2m","3m","5m","7m","7p","9p","4s","5s","5s","6s","7s","7s","E"],["3m","5m","6m","2p","6p","9p","1s","5s","8s","9s","S","S","C"],["1m","4m","3p","4p","5pr","7p","1s","2s","7s","8s","W","N","P"]]}
        {"type":"tsumo","actor":3,"pai":"F"}
        {"type":"dahai","actor":3,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"5p"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"9m"}
        {"type":"dahai","actor":1,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"N"}
        {"type":"dahai","actor":2,"pai":"9p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2p"}
        {"type":"dahai","actor":3,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"6m"}
        {"type":"dahai","actor":0,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"7m"}
        {"type":"dahai","actor":1,"pai":"9m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3s"}
        {"type":"dahai","actor":2,"pai":"2p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"4s"}
        {"type":"dahai","actor":3,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1m"}
        {"type":"dahai","actor":0,"pai":"1m","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"9m"}
        {"type":"dahai","actor":1,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"3m"}
        {"type":"dahai","actor":2,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2s"}
        {"type":"dahai","actor":3,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"2m"}
        {"type":"dahai","actor":0,"pai":"2s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"1m"}
        {"type":"dahai","actor":1,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3p"}
        {"type":"dahai","actor":2,"pai":"3p","tsumogiri":true}
        {"type":"pon","actor":0,"target":2,"pai":"3p","consumed":["3p","3p"]}
        {"type":"dahai","actor":0,"pai":"2m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"6p"}
        {"type":"dahai","actor":1,"pai":"9p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"6s"}
        {"type":"dahai","actor":2,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"7p"}
        {"type":"dahai","actor":3,"pai":"P","tsumogiri":false}
        {"type":"pon","actor":0,"target":3,"pai":"P","consumed":["P","P"]}
        {"type":"dahai","actor":0,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"7s"}
        {"type":"dahai","actor":1,"pai":"5s","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3s"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2m"}
        {"type":"dahai","actor":3,"pai":"1s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1p"}
        {"type":"dahai","actor":0,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"7m"}
        {"type":"dahai","actor":1,"pai":"4s","tsumogiri":false}
        {"type":"chi","actor":2,"target":1,"pai":"4s","consumed":["5s","6s"]}
        {"type":"dahai","actor":2,"pai":"6p","tsumogiri":false}
        {"type":"chi","actor":3,"target":2,"pai":"6p","consumed":["5pr","7p"]}
        {"type":"dahai","actor":3,"pai":"7p","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1s"}
        {"type":"dahai","actor":0,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"1s"}
        {"type":"reach","actor":1}
        {"type":"dahai","actor":1,"pai":"1s","tsumogiri":true}
        {"type":"reach_accepted","actor":1}
        {"type":"tsumo","actor":2,"pai":"9s"}
        {"type":"dahai","actor":2,"pai":"8s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"4p"}
        {"type":"dahai","actor":3,"pai":"4p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4m"}
        {"type":"dahai","actor":0,"pai":"4m","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"1p"}
        {"type":"dahai","actor":1,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8m"}
        {"type":"dahai","actor":2,"pai":"8m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"C"}
        {"type":"dahai","actor":3,"pai":"C","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"2s"}
        {"type":"dahai","actor":0,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"8p"}
    "#;
    let mut ps = state_from_log(1, log);

    assert!(ps.last_cans.can_tsumo_agari);
    let should_hora = ps.rule_based_agari();
    assert!(!should_hora);

    ps.add_dora_indicator(t!(5m));
    let should_hora = ps.rule_based_agari();
    assert!(should_hora);
}

#[test]
fn get_rank() {
    let ps = PlayerState::new(0);
    let rank = ps.get_rank([20000, 25000, 25000, 30000]);
    assert_eq!(rank, 3);

    let ps = PlayerState::new(3);
    let rank = ps.get_rank([25000, 25000, 25000, 25000]);
    assert_eq!(rank, 3);

    let ps = PlayerState::new(1);
    let rank = ps.get_rank([25000, 30000, 20000, 25000]);
    assert_eq!(rank, 2);

    let ps = PlayerState::new(1);
    let rank = ps.get_rank([32000, 32000, 18000, 18000]);
    assert_eq!(rank, 0);

    let ps = PlayerState::new(2);
    let rank = ps.get_rank([32000, 18000, 18000, 32000]);
    assert_eq!(rank, 1);

    let ps = PlayerState::new(2);
    let rank = ps.get_rank([5, 2, 5, 3]);
    assert_eq!(rank, 1);
}

#[test]
fn kakan_from_hand() {
    let log = r#"
        {"type":"start_kyoku","bakaze":"S","dora_marker":"6m","kyoku":2,"honba":0,"kyotaku":0,"oya":1,"scores":[16100,36600,16800,30500],"tehais":[["5p","5s","1s","9m","9m","W","E","N","1p","F","9m","3p","6p"],["4s","9s","S","4s","1m","P","N","7s","F","2m","3s","2s","2s"],["6m","8p","8p","2p","8m","N","7p","C","1s","2p","N","9s","9p"],["2m","6s","7p","9s","2m","9s","6m","7s","8m","3m","S","5mr","C"]]}
        {"type":"tsumo","actor":1,"pai":"S"}
        {"type":"dahai","actor":1,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"1s"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"P"}
        {"type":"dahai","actor":3,"pai":"S","tsumogiri":false}
        {"type":"pon","actor":1,"target":3,"pai":"S","consumed":["S","S"]}
        {"type":"dahai","actor":1,"pai":"P","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"4p"}
        {"type":"dahai","actor":2,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"5s"}
        {"type":"dahai","actor":3,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"7m"}
        {"type":"dahai","actor":0,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"P"}
        {"type":"dahai","actor":1,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"9p"}
        {"type":"dahai","actor":2,"pai":"6m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"C"}
        {"type":"dahai","actor":3,"pai":"C","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"7p"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"5s"}
        {"type":"dahai","actor":1,"pai":"2m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"5m"}
        {"type":"dahai","actor":2,"pai":"5m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"1p"}
        {"type":"dahai","actor":3,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4m"}
        {"type":"dahai","actor":0,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"E"}
        {"type":"dahai","actor":1,"pai":"P","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"1s"}
        {"type":"dahai","actor":2,"pai":"8m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6p"}
        {"type":"dahai","actor":3,"pai":"8m","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"5p"}
        {"type":"dahai","actor":0,"pai":"1s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"2s"}
        {"type":"dahai","actor":1,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"5m"}
        {"type":"dahai","actor":2,"pai":"5m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"3s"}
        {"type":"dahai","actor":3,"pai":"3s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"7p"}
        {"type":"dahai","actor":0,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"E"}
        {"type":"dahai","actor":1,"pai":"E","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"W"}
        {"type":"dahai","actor":2,"pai":"W","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"7m"}
        {"type":"dahai","actor":3,"pai":"2m","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"5m"}
        {"type":"dahai","actor":0,"pai":"5s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"S"}
        {"type":"dahai","actor":1,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"6p"}
        {"type":"dahai","actor":2,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2p"}
        {"type":"dahai","actor":3,"pai":"2p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6p"}
        {"type":"dahai","actor":0,"pai":"3p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"4m"}
        {"type":"dahai","actor":1,"pai":"4m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"3s"}
        {"type":"dahai","actor":2,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"8p"}
        {"type":"reach","actor":3}
        {"type":"dahai","actor":3,"pai":"P","tsumogiri":false}
        {"type":"reach_accepted","actor":3}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"8s"}
        {"type":"kakan","actor":1,"pai":"S","consumed":["S","S","S"]}
        {"type":"tsumo","actor":1,"pai":"4s"}
    "#;
    let ps = state_from_log(1, log);

    assert!(ps.last_cans.can_tsumo_agari);
}

#[test]
fn discard_candidates_with_unconditional_tenpai() {
    let log = r#"
        {"type":"start_kyoku","bakaze":"S","dora_marker":"2s","kyoku":3,"honba":0,"kyotaku":0,"oya":2,"scores":[25600,15600,21200,37600],"tehais":[["3m","3m","1p","6p","7p","9p","5sr","7s","8s","8s","E","E","W"],["4m","5mr","6m","1p","4p","5p","8p","3s","3s","4s","5s","S","P"],["1m","5m","7m","2p","9p","3s","5s","9s","S","W","N","P","C"],["1m","4m","6m","2p","3p","4p","6p","9p","2s","4s","7s","S","N"]]}
        {"type":"tsumo","actor":2,"pai":"C"}
        {"type":"dahai","actor":2,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2m"}
        {"type":"dahai","actor":3,"pai":"2m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"2p"}
        {"type":"dahai","actor":0,"pai":"9p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"7p"}
        {"type":"dahai","actor":1,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"4p"}
        {"type":"dahai","actor":2,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"P"}
        {"type":"dahai","actor":3,"pai":"P","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6m"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"C"}
        {"type":"dahai","actor":1,"pai":"P","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"8m"}
        {"type":"dahai","actor":2,"pai":"9p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"9m"}
        {"type":"dahai","actor":3,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"1p"}
        {"type":"dahai","actor":0,"pai":"2p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"7m"}
        {"type":"dahai","actor":1,"pai":"S","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"P"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"N"}
        {"type":"dahai","actor":3,"pai":"N","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6p"}
        {"type":"dahai","actor":0,"pai":"7p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"9m"}
        {"type":"dahai","actor":1,"pai":"C","tsumogiri":false}
        {"type":"pon","actor":2,"target":1,"pai":"C","consumed":["C","C"]}
        {"type":"dahai","actor":2,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"7s"}
        {"type":"dahai","actor":3,"pai":"7s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"2p"}
        {"type":"dahai","actor":0,"pai":"2p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"5pr"}
        {"type":"dahai","actor":1,"pai":"9m","tsumogiri":false}
        {"type":"chi","actor":2,"target":1,"pai":"9m","consumed":["7m","8m"]}
        {"type":"dahai","actor":2,"pai":"S","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"E"}
        {"type":"dahai","actor":3,"pai":"E","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"5m"}
        {"type":"dahai","actor":0,"pai":"7s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"3p"}
        {"type":"dahai","actor":1,"pai":"5p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"F"}
        {"type":"dahai","actor":2,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"2s"}
        {"type":"dahai","actor":3,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4s"}
        {"type":"dahai","actor":0,"pai":"4s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"1p"}
        {"type":"dahai","actor":1,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"6s"}
        {"type":"dahai","actor":2,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6p"}
        {"type":"dahai","actor":3,"pai":"6p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"9p"}
        {"type":"dahai","actor":0,"pai":"9p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"5p"}
        {"type":"dahai","actor":1,"pai":"5p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"5s"}
        {"type":"dahai","actor":2,"pai":"5s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"9s"}
        {"type":"dahai","actor":3,"pai":"9s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"8m"}
        {"type":"dahai","actor":0,"pai":"8m","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"9m"}
        {"type":"dahai","actor":1,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"9s"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"1s"}
        {"type":"dahai","actor":3,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"2m"}
        {"type":"dahai","actor":0,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"8m"}
        {"type":"dahai","actor":1,"pai":"8m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8p"}
        {"type":"dahai","actor":2,"pai":"8p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"7m"}
        {"type":"dahai","actor":3,"pai":"7m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"7p"}
        {"type":"dahai","actor":0,"pai":"7p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"8p"}
        {"type":"dahai","actor":1,"pai":"7m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3m"}
        {"type":"dahai","actor":2,"pai":"3m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"1s"}
        {"type":"dahai","actor":3,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4p"}
        {"type":"dahai","actor":0,"pai":"2m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"F"}
        {"type":"dahai","actor":1,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"9s"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"7m"}
        {"type":"dahai","actor":3,"pai":"7m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"F"}
        {"type":"dahai","actor":0,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"8s"}
        {"type":"dahai","actor":1,"pai":"8s","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"F"}
        {"type":"dahai","actor":2,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"1m"}
        {"type":"dahai","actor":3,"pai":"1m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"9m"}
        {"type":"dahai","actor":1,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"2m"}
        {"type":"dahai","actor":2,"pai":"2m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"7p"}
        {"type":"dahai","actor":3,"pai":"7p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"3p"}
        {"type":"dahai","actor":0,"pai":"6m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"6m"}
        {"type":"dahai","actor":1,"pai":"6m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"1s"}
        {"type":"dahai","actor":2,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"8m"}
        {"type":"dahai","actor":3,"pai":"8m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"S"}
        {"type":"dahai","actor":0,"pai":"S","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"2m"}
        {"type":"dahai","actor":1,"pai":"2m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"4s"}
        {"type":"dahai","actor":2,"pai":"6s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"8s"}
        {"type":"dahai","actor":3,"pai":"8s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"N"}
        {"type":"dahai","actor":0,"pai":"N","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"3s"}
    "#;
    let ps = state_from_log(1, log);

    let expected = t![7p, 8p];
    ps.discard_candidates_with_unconditional_tenpai()
        .iter()
        .enumerate()
        .for_each(|(idx, &b)| {
            if expected.contains(&must_tile!(idx)) {
                assert!(b);
            } else {
                assert!(!b);
            }
        });

    let log = r#"
        {"type":"start_kyoku","bakaze":"E","dora_marker":"2p","kyoku":4,"honba":0,"kyotaku":0,"oya":3,"scores":[25000,20100,24000,30900],"tehais":[["1m","1m","4m","5m","5m","1p","4p","6p","7p","4s","5s","6s","S"],["5m","6p","7p","2s","3s","4s","4s","5s","7s","9s","S","C","C"],["2m","3m","6m","7m","9m","9m","1p","6p","1s","6s","9s","P","P"],["5mr","6m","8m","8m","2p","5p","7p","8p","9p","3s","9s","W","N"]]}
        {"type":"tsumo","actor":3,"pai":"C"}
        {"type":"dahai","actor":3,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"E"}
        {"type":"dahai","actor":0,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"2m"}
        {"type":"dahai","actor":1,"pai":"2m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"9s"}
        {"type":"dahai","actor":2,"pai":"1s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"8p"}
        {"type":"dahai","actor":3,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"P"}
        {"type":"dahai","actor":0,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"3m"}
        {"type":"dahai","actor":1,"pai":"3m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8s"}
        {"type":"dahai","actor":2,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"S"}
        {"type":"dahai","actor":3,"pai":"S","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"N"}
        {"type":"dahai","actor":0,"pai":"N","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"5pr"}
        {"type":"dahai","actor":1,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"1s"}
        {"type":"dahai","actor":2,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"9p"}
        {"type":"dahai","actor":3,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"2p"}
        {"type":"dahai","actor":0,"pai":"P","tsumogiri":false}
        {"type":"pon","actor":2,"target":0,"pai":"P","consumed":["P","P"]}
        {"type":"dahai","actor":2,"pai":"6p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"3p"}
        {"type":"dahai","actor":3,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"7m"}
        {"type":"dahai","actor":0,"pai":"S","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"2m"}
        {"type":"dahai","actor":1,"pai":"2m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"3s"}
        {"type":"dahai","actor":2,"pai":"3s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"3p"}
        {"type":"dahai","actor":3,"pai":"3s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"8s"}
        {"type":"dahai","actor":0,"pai":"7m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"F"}
        {"type":"dahai","actor":1,"pai":"S","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"E"}
        {"type":"dahai","actor":2,"pai":"6s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"4s"}
        {"type":"dahai","actor":3,"pai":"4s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"7s"}
        {"type":"dahai","actor":0,"pai":"4p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"6s"}
        {"type":"dahai","actor":1,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"7m"}
        {"type":"dahai","actor":2,"pai":"8s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6m"}
        {"type":"dahai","actor":3,"pai":"2p","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"3p"}
        {"type":"dahai","actor":0,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"6p"}
        {"type":"dahai","actor":1,"pai":"6p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"N"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2p"}
        {"type":"dahai","actor":3,"pai":"2p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4p"}
        {"type":"dahai","actor":0,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"F"}
        {"type":"dahai","actor":1,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"3m"}
        {"type":"dahai","actor":2,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"8p"}
        {"type":"dahai","actor":3,"pai":"5p","tsumogiri":false}
        {"type":"chi","actor":0,"target":3,"pai":"5p","consumed":["6p","7p"]}
        {"type":"dahai","actor":0,"pai":"4m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"1p"}
        {"type":"dahai","actor":1,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"5s"}
        {"type":"dahai","actor":2,"pai":"5s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"9m"}
        {"type":"dahai","actor":3,"pai":"9m","tsumogiri":true}
        {"type":"pon","actor":2,"target":3,"pai":"9m","consumed":["9m","9m"]}
        {"type":"dahai","actor":2,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"7s"}
        {"type":"dahai","actor":3,"pai":"7s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"3m"}
        {"type":"dahai","actor":0,"pai":"3m","tsumogiri":true}
        {"type":"pon","actor":2,"target":0,"pai":"3m","consumed":["3m","3m"]}
        {"type":"dahai","actor":2,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"1s"}
        {"type":"dahai","actor":3,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"7p"}
        {"type":"dahai","actor":0,"pai":"7p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"9m"}
        {"type":"dahai","actor":1,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"4m"}
        {"type":"dahai","actor":2,"pai":"2m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"P"}
        {"type":"dahai","actor":3,"pai":"P","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"F"}
        {"type":"dahai","actor":1,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8m"}
        {"type":"dahai","actor":2,"pai":"8m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"7s"}
        {"type":"dahai","actor":3,"pai":"7s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4p"}
        {"type":"dahai","actor":0,"pai":"4p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"3p"}
        {"type":"dahai","actor":1,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"8s"}
        {"type":"dahai","actor":2,"pai":"8s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"2s"}
        {"type":"dahai","actor":3,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4p"}
        {"type":"dahai","actor":0,"pai":"4p","tsumogiri":true}
        {"type":"chi","actor":1,"target":0,"pai":"4p","consumed":["3p","5pr"]}
        {"type":"dahai","actor":1,"pai":"7s","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"5p"}
        {"type":"dahai","actor":2,"pai":"5p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"1m"}
        {"type":"dahai","actor":3,"pai":"8p","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"8s"}
        {"type":"dahai","actor":1,"pai":"8s","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8p"}
        {"type":"dahai","actor":2,"pai":"8p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"F"}
        {"type":"dahai","actor":3,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1p"}
        {"type":"dahai","actor":0,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"1m"}
        {"type":"dahai","actor":1,"pai":"1m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"5sr"}
        {"type":"dahai","actor":2,"pai":"7m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"9p"}
        {"type":"dahai","actor":3,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1s"}
        {"type":"dahai","actor":0,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"6s"}
    "#;
    let ps = state_from_log(1, log);

    let expected = t![5p, 8p];
    for (idx, &b) in ps.waits.iter().enumerate() {
        if expected.contains(&must_tile!(idx)) {
            assert!(b);
        } else {
            assert!(!b);
        }
    }

    let discard_candidates = ps.discard_candidates_with_unconditional_tenpai();
    assert_eq!(discard_candidates, [false; 34]);
}

#[test]
fn double_chankan_ron() {
    let log = r#"
        {"type":"start_kyoku","bakaze":"S","dora_marker":"2p","kyoku":2,"honba":0,"kyotaku":0,"oya":1,"scores":[44400,1600,25700,28300],"tehais":[["1m","5m","9m","9m","9m","3p","9p","8s","9s","W","W","N","C"],["7m","8m","3p","6p","8p","1s","1s","3s","6s","9s","E","F","C"],["3m","9m","2p","5p","8p","1s","2s","5s","6s","7s","S","F","C"],["2m","2m","5m","5mr","8m","1p","1p","7p","8p","3s","5s","8s","9s"]]}
        {"type":"tsumo","actor":1,"pai":"P"}
        {"type":"dahai","actor":1,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3m"}
        {"type":"dahai","actor":2,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6m"}
        {"type":"dahai","actor":3,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1s"}
        {"type":"dahai","actor":0,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"9p"}
        {"type":"dahai","actor":1,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"9p"}
        {"type":"dahai","actor":2,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"7s"}
        {"type":"dahai","actor":3,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"7p"}
        {"type":"dahai","actor":0,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"5m"}
        {"type":"dahai","actor":1,"pai":"P","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"8s"}
        {"type":"dahai","actor":2,"pai":"9m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"7m"}
        {"type":"dahai","actor":3,"pai":"1p","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"1m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"P"}
        {"type":"dahai","actor":1,"pai":"P","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"4m"}
        {"type":"dahai","actor":2,"pai":"S","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"8m"}
        {"type":"dahai","actor":3,"pai":"8m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"8p"}
        {"type":"dahai","actor":0,"pai":"N","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"5sr"}
        {"type":"dahai","actor":1,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"E"}
        {"type":"dahai","actor":2,"pai":"E","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"4p"}
        {"type":"dahai","actor":3,"pai":"4p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"1m"}
        {"type":"dahai","actor":0,"pai":"5m","tsumogiri":false}
        {"type":"pon","actor":3,"target":0,"pai":"5m","consumed":["5m","5mr"]}
        {"type":"dahai","actor":3,"pai":"8s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"4s"}
        {"type":"dahai","actor":0,"pai":"4s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"N"}
        {"type":"dahai","actor":1,"pai":"N","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"9p"}
        {"type":"dahai","actor":2,"pai":"8p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"C"}
        {"type":"dahai","actor":3,"pai":"C","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4s"}
        {"type":"dahai","actor":0,"pai":"4s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"1m"}
        {"type":"dahai","actor":1,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"4p"}
        {"type":"dahai","actor":2,"pai":"2p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"P"}
        {"type":"dahai","actor":3,"pai":"P","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"3m"}
        {"type":"dahai","actor":0,"pai":"3p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"6s"}
        {"type":"dahai","actor":1,"pai":"9p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"8s"}
        {"type":"dahai","actor":2,"pai":"3m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"4m"}
        {"type":"dahai","actor":3,"pai":"4m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"P"}
        {"type":"dahai","actor":0,"pai":"P","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"E"}
        {"type":"dahai","actor":1,"pai":"E","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"7s"}
        {"type":"dahai","actor":2,"pai":"2s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"F"}
        {"type":"dahai","actor":3,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4m"}
        {"type":"dahai","actor":0,"pai":"4m","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"2m"}
        {"type":"dahai","actor":1,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"7p"}
        {"type":"dahai","actor":2,"pai":"7p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"2s"}
        {"type":"dahai","actor":3,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"4p"}
        {"type":"dahai","actor":0,"pai":"4p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"5pr"}
        {"type":"dahai","actor":1,"pai":"8p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"2s"}
        {"type":"dahai","actor":2,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"F"}
        {"type":"dahai","actor":3,"pai":"F","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6p"}
        {"type":"dahai","actor":0,"pai":"6p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"7m"}
        {"type":"dahai","actor":1,"pai":"3p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"1p"}
        {"type":"dahai","actor":2,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"9s"}
        {"type":"dahai","actor":3,"pai":"9s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"S"}
        {"type":"dahai","actor":0,"pai":"S","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"7s"}
        {"type":"dahai","actor":1,"pai":"6s","tsumogiri":false}
        {"type":"chi","actor":2,"target":1,"pai":"6s","consumed":["5s","7s"]}
        {"type":"dahai","actor":2,"pai":"1s","tsumogiri":false}
        {"type":"pon","actor":1,"target":2,"pai":"1s","consumed":["1s","1s"]}
        {"type":"dahai","actor":1,"pai":"3s","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"2p"}
        {"type":"dahai","actor":2,"pai":"2p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"3p"}
        {"type":"dahai","actor":3,"pai":"3p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6s"}
        {"type":"dahai","actor":0,"pai":"6s","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"6p"}
        {"type":"dahai","actor":1,"pai":"6p","tsumogiri":true}
        {"type":"chi","actor":2,"target":1,"pai":"6p","consumed":["4p","5p"]}
        {"type":"dahai","actor":2,"pai":"8s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6m"}
        {"type":"dahai","actor":3,"pai":"3s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"7m"}
        {"type":"dahai","actor":0,"pai":"8s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"6p"}
        {"type":"dahai","actor":1,"pai":"6p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"5s"}
        {"type":"dahai","actor":2,"pai":"8s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"1p"}
        {"type":"dahai","actor":3,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"2s"}
        {"type":"dahai","actor":0,"pai":"9s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"1m"}
        {"type":"dahai","actor":1,"pai":"2m","tsumogiri":false}
        {"type":"pon","actor":3,"target":1,"pai":"2m","consumed":["2m","2m"]}
        {"type":"dahai","actor":3,"pai":"6m","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"2s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"N"}
        {"type":"dahai","actor":1,"pai":"N","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"5p"}
        {"type":"dahai","actor":2,"pai":"5p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"3m"}
        {"type":"dahai","actor":3,"pai":"3m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6m"}
        {"type":"ankan","actor":0,"consumed":["W","W","W","W"]}
        {"type":"dora","dora_marker":"7p"}
        {"type":"tsumo","actor":0,"pai":"8m"}
        {"type":"dahai","actor":0,"pai":"6m","tsumogiri":false}
        {"type":"chi","actor":1,"target":0,"pai":"6m","consumed":["7m","8m"]}
        {"type":"dahai","actor":1,"pai":"7m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3s"}
        {"type":"dahai","actor":2,"pai":"3s","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"2m"}
    "#;
    let mut ps = state_from_log(2, log);

    let mut ps_kakan = ps.clone();
    let cans = ps_kakan
        .update_json(r#"{"type":"kakan","actor":3,"pai":"2m","consumed":["2m","2m","2m"]}"#)
        .unwrap();
    assert!(cans.can_ron_agari);
    assert_eq!(ps_kakan.agari_points(true, &[]).unwrap().ron, 1000);

    let cans = ps
        .update_json(r#"{"type":"dahai","actor":3,"pai":"2m","tsumogiri":true}"#)
        .unwrap();
    assert!(!cans.can_ron_agari);
}
