use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use vikett::types::RefereeKind;
use vikett::{dump_ontology, eval, eval_on, Catalog, CATALOG};

#[derive(Parser)]
#[command(
    name = "vikett",
    about = "Closed-door control protocol — test rig and TUI"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Interactive decision-tree TUI (default)
    Tui,
    /// Evaluate one utterance and print take + walk
    Eval {
        #[arg(short, long)]
        utterance: String,
        #[arg(short, long, default_value = "desk")]
        snap: String,
        #[arg(short, long, default_value = "lexical")]
        referee: RefereeArg,
        /// Use a snap built from this Hyprland session
        #[arg(long)]
        live: bool,
    },
    /// Print the decision tree for an utterance
    Tree {
        #[arg(short, long)]
        utterance: String,
        #[arg(short, long, default_value = "desk")]
        snap: String,
        #[arg(short, long, default_value = "lexical")]
        referee: RefereeArg,
        #[arg(long)]
        live: bool,
    },
    /// Run lexical goldens (and optional model paraphrases)
    Test {
        #[arg(long, default_value = "lexical")]
        referee: RefereeArg,
    },
    /// Extensive prompt suite (goldens + coverage + refuses + paraphrases)
    Suite {
        #[arg(long, default_value = "lexical")]
        referee: RefereeArg,
        /// Filter by tag (wm, audio, refuse, guest, compound, paraphrase, …)
        #[arg(long)]
        tag: Option<String>,
        /// Print passing cases too
        #[arg(long)]
        show_pass: bool,
        /// Fail on paraphrase/Laya misses. A wrong act fails without this flag.
        #[arg(long)]
        strict: bool,
        /// Also run Laya if the System One server is up
        #[arg(long)]
        compare: bool,
        #[arg(long, value_enum, default_value = "text")]
        format: ReportFormat,
    },
    /// Rewrite ontology/*.json from the loaded catalogue + extras
    Dump,
}

#[derive(Clone, Copy, ValueEnum)]
enum ReportFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum RefereeArg {
    Lexical,
    Laya,
}

impl From<RefereeArg> for RefereeKind {
    fn from(v: RefereeArg) -> Self {
        match v {
            RefereeArg::Lexical => RefereeKind::Lexical,
            RefereeArg::Laya => RefereeKind::Laya,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd.unwrap_or(Cmd::Tui) {
        Cmd::Tui => vikett::tui::run(&CATALOG),
        Cmd::Eval {
            utterance,
            snap,
            referee,
            live,
        } => {
            let r = run_one(&utterance, &snap, referee.into(), live)?;
            println!("{}", serde_json::to_string_pretty(&r)?);
            Ok(())
        }
        Cmd::Tree {
            utterance,
            snap,
            referee,
            live,
        } => {
            let r = run_one(&utterance, &snap, referee.into(), live)?;
            print!("{}", r.trace.render());
            println!(
                "take {}  conf={:.2}",
                r.take.page_id.as_deref().unwrap_or("∅"),
                r.take.confidence
            );
            if let Some(w) = r.walk {
                println!("walk {}", w.command);
            }
            if let Some(a) = r.answer {
                println!("ask  {a}");
            }
            Ok(())
        }
        Cmd::Test { referee } => run_tests(referee.into()),
        Cmd::Suite {
            referee,
            tag,
            show_pass,
            strict,
            compare,
            format,
        } => run_suite_cmd(referee.into(), tag, show_pass, strict, compare, format),
        Cmd::Dump => {
            dump_ontology(&CATALOG)?;
            println!("wrote ontology/*.json");
            Ok(())
        }
    }
}

fn run_one(
    utterance: &str,
    snap_id: &str,
    referee: RefereeKind,
    live: bool,
) -> Result<vikett::EngineResult> {
    if live {
        let snap = vikett::host::live_snap().context("live snap")?;
        eval_on(&CATALOG, utterance, &snap, referee)
    } else {
        eval(utterance, snap_id, referee)
    }
}

fn run_tests(referee: RefereeKind) -> Result<()> {
    let cat: &Catalog = &CATALOG;
    let mut fail = 0;
    println!("L1 lexical goldens");
    for g in &cat.goldens {
        let r = vikett::run_golden(cat, g);
        if r.pass {
            println!("  ✓ {}", r.id);
        } else {
            fail += 1;
            println!(
                "  ✗ {}  got {:?} expected {:?} slots_ok={} ({})",
                r.id, r.got, r.expected, r.slots_ok, r.detail
            );
        }
    }

    if referee != RefereeKind::Lexical {
        if !vikett::model::available(referee) {
            println!("{referee} unavailable — skip L2 paraphrases");
        } else {
            println!("L2 {referee} paraphrases");
            for p in &cat.phrases {
                let Some(snap) = cat.snap(
                    cat.goldens
                        .iter()
                        .find(|g| g.id == p.golden)
                        .map(|g| g.snap.as_str())
                        .unwrap_or("desk"),
                ) else {
                    continue;
                };
                match eval_on(cat, &p.utterance, snap, referee) {
                    Ok(r) => {
                        let pass = r.take.page_id == p.expect_page;
                        if pass {
                            println!("  ✓ {} -> {:?}", p.id, r.take.page_id);
                        } else {
                            fail += 1;
                            println!(
                                "  ✗ {} got {:?} expected {:?}",
                                p.id, r.take.page_id, p.expect_page
                            );
                        }
                    }
                    Err(e) => {
                        fail += 1;
                        println!("  ✗ {} error {e}", p.id);
                    }
                }
            }
        }
    }

    if fail > 0 {
        anyhow::bail!("{fail} failing");
    }
    println!("ok");
    Ok(())
}

fn run_suite_cmd(
    referee: RefereeKind,
    tag: Option<String>,
    show_pass: bool,
    strict: bool,
    compare: bool,
    format: ReportFormat,
) -> Result<()> {
    let cat: &Catalog = &CATALOG;
    let tag_ref = tag.as_deref();
    match format {
        ReportFormat::Json => {
            let report = vikett::suite::run_suite(cat, referee, tag_ref)?;
            if report.skipped {
                eprintln!("Laya unavailable — skip");
            }
            println!("{}", serde_json::to_string_pretty(&json_report(&report))?);
            if !report.skipped && referee == RefereeKind::Lexical && report.must_fail > 0 {
                anyhow::bail!("{} lexical must-pass cases failed", report.must_fail);
            }
            if !report.skipped && report.wrong_acts > 0 {
                anyhow::bail!("{} wrong acts on refuse/dead cases", report.wrong_acts);
            }
        }
        ReportFormat::Text => {
            vikett::suite::run_and_print(cat, referee, tag_ref, show_pass, strict)?;
        }
    }
    if compare && referee == RefereeKind::Lexical {
        println!();
        vikett::suite::run_and_print(cat, RefereeKind::Laya, tag_ref, false, false)?;
    }
    Ok(())
}

fn json_report(report: &vikett::suite::SuiteReport) -> serde_json::Value {
    serde_json::json!({
        "must_fail": report.must_fail,
        "soft_fail": report.soft_fail,
        "wrong_acts": report.wrong_acts,
        "skipped": report.skipped,
        "pass": report.results.iter().filter(|r| r.pass).count(),
        "total": report.results.len(),
        "results": report.results.iter().map(|r| serde_json::json!({
            "id": r.id,
            "snap": r.snap,
            "utterance": r.utterance,
            "referee": r.referee.as_str(),
            "expected": r.expected,
            "got": r.got,
            "pass": r.pass,
            "wrong_act": r.wrong_act,
            "slots_ok": r.slots_ok,
            "walk_ok": r.walk_ok,
            "walk": r.walk,
            "reason": r.reason,
            "tags": r.tags,
        })).collect::<Vec<_>>(),
    })
}
