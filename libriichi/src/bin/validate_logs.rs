use riichi::chi_type::ChiType;
use riichi::mjai::Event;
use riichi::state::{ActionCandidate, PlayerState};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

use anyhow::{ensure, Context, Result};
use flate2::read::GzDecoder;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json as json;

const USAGE: &str = "Usage: validate_logs <DIR>";

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let dir = args.get(1).context(USAGE)?;

    const TEMPLATE: &str = "{spinner:.cyan} [{elapsed_precise}] {pos} ({per_sec})";
    let bar = ProgressBar::new_spinner()
        .with_style(ProgressStyle::with_template(TEMPLATE)?.tick_chars(".oOo"));
    bar.enable_steady_tick(Duration::from_millis(150));

    glob(&format!("{dir}/**/*.json"))?
        .chain(glob(&format!("{dir}/**/*.json.gz"))?)
        .par_bridge()
        .try_for_each(|path| {
            bar.inc(1);
            let path = path?;

            let result = process_path(&path).with_context(|| format!("in log {}", path.display()));
            if let Err(err) = result {
                println!("\n{err:?}");
            }

            anyhow::Ok(())
        })?;

    bar.abandon();

    Ok(())
}

fn process_path(path: &Path) -> Result<()> {
    let mut raw_log = String::new();
    if matches!(path.extension(), Some(s) if s.eq_ignore_ascii_case("gz")) {
        let mut gz = GzDecoder::new(File::open(path)?);
        gz.read_to_string(&mut raw_log)?;
    } else {
        let mut f = File::open(path)?;
        f.read_to_string(&mut raw_log)?;
    }
    let events: Vec<Event> = raw_log
        .lines()
        .map(|l| Ok(json::from_str(l)?))
        .collect::<Result<_>>()?;

    let mut states = [
        PlayerState::new(0),
        PlayerState::new(1),
        PlayerState::new(2),
        PlayerState::new(3),
    ];
    let mut cans = [ActionCandidate::default(); 4];

    for (idx, ev) in events.iter().enumerate() {
        let line = idx + 1;
        match ev {
            Event::Dahai { actor, pai, .. } => {
                ensure!(
                    cans[*actor as usize].can_discard,
                    "fails can_discard at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );

                let discard_candidates = states[*actor as usize].discard_candidates_aka();
                ensure!(
                    discard_candidates[pai.as_usize()],
                    "fails discard_candidates at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );
            }
            Event::Chi {
                actor,
                consumed,
                pai,
                target,
            } => {
                ensure!(
                    (target + 1) % 4 == *actor,
                    "chi from non-kamicha at line {}\nstate:\n{}",
                    line,
                    states[*actor as usize].brief_info(),
                );

                match ChiType::new(*consumed, *pai) {
                    ChiType::Low => {
                        ensure!(
                            cans[*actor as usize].can_chi_low,
                            "fails can_chi_low at line {}\nstate:\n{}",
                            line,
                            states[*actor as usize].brief_info(),
                        );
                    }
                    ChiType::Mid => {
                        ensure!(
                            cans[*actor as usize].can_chi_mid,
                            "fails can_chi_mid at line {}\nstate:\n{}",
                            line,
                            states[*actor as usize].brief_info(),
                        );
                    }
                    ChiType::High => {
                        ensure!(
                            cans[*actor as usize].can_chi_high,
                            "fails can_chi_high at line {}\nstate:\n{}",
                            line,
                            states[*actor as usize].brief_info(),
                        );
                    }
                }
            }
            Event::Pon { actor, .. } => {
                ensure!(
                    cans[*actor as usize].can_pon,
                    "fails can_pon at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );
            }
            Event::Daiminkan { actor, .. } => {
                ensure!(
                    cans[*actor as usize].can_daiminkan,
                    "fails can_daiminkan at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );
            }
            Event::Ankan { actor, consumed } => {
                ensure!(
                    cans[*actor as usize].can_ankan,
                    "fails can_ankan at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );

                let ankan_candidates = states[*actor as usize].ankan_candidates();
                ensure!(
                    ankan_candidates.contains(&consumed[0].deaka()),
                    "fails ankan_candidates at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );
            }
            Event::Kakan { actor, pai, .. } => {
                ensure!(
                    cans[*actor as usize].can_kakan,
                    "fails can_kakan at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );

                let kakan_candidates = states[*actor as usize].kakan_candidates();
                ensure!(
                    kakan_candidates.contains(&pai.deaka()),
                    "fails kakan_candidates at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );
            }
            Event::Reach { actor } => {
                ensure!(
                    cans[*actor as usize].can_riichi,
                    "fails can_riichi at line {line}\nstate:\n{}",
                    states[*actor as usize].brief_info(),
                );
            }
            Event::Hora {
                actor,
                target,
                ura_markers,
                deltas,
            } => {
                let is_ron = actor != target;
                if is_ron {
                    ensure!(
                        cans[*actor as usize].can_ron_agari,
                        "fails can_ron_agari at line {line}\nstate:\n{}",
                        states[*actor as usize].brief_info(),
                    );
                } else {
                    ensure!(
                        cans[*actor as usize].can_tsumo_agari,
                        "fails can_tsumo_agari at line {line}\nstate:\n{}",
                        states[*actor as usize].brief_info(),
                    );
                }

                // This is a rough test
                // TODO: fix bug for double chankan ron
                let ura = ura_markers
                    .as_ref()
                    .context("missing field `ura_markers`")?;
                let deltas = deltas.context("missing field `deltas`")?;
                let points = states[*actor as usize]
                    .agari_points(is_ron, ura)
                    .with_context(|| {
                        format!(
                            "failed to get agari points at line {line}\nstate:\n{}",
                            states[*actor as usize].brief_info()
                        )
                    })?;

                if is_ron {
                    ensure!(deltas[*actor as usize] >= points.ron);
                } else if states[*actor as usize].is_oya() {
                    ensure!(deltas[*actor as usize] >= points.tsumo_oya);
                } else {
                    ensure!(deltas[*actor as usize] >= points.tsumo_ko);
                }
            }
            _ => (),
        }

        states
            .iter_mut()
            .zip(&mut cans)
            .for_each(|(s, c)| *c = s.update_with_skip(ev, true));
    }

    Ok(())
}
