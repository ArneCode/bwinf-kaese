mod cheese;
mod cheese_builder;
mod pieces_map;
mod prev_pieces;

use cheese::Piece;
use clap::{arg, Parser};
use pieces_map::PiecesMap;
use rand::{prelude::*, rngs::ThreadRng, seq::SliceRandom, thread_rng};
use std::{
    fs::{self, File},
    io::BufWriter,
    io::Write,
    time::Instant,
};

use crate::cheese_builder::construct_cheeses;

/// Lädt die Stücke aus einer Datei
fn load_pieces(path: &str) -> Vec<Piece> {
    let s = fs::read_to_string(path).expect("couldn't read file");
    let mut lines = s.split("\r\n");
    let n_pieces: usize = lines
        .next()
        .expect("empty data file")
        .parse()
        .expect("couldn't extract number of pieces");
    let pieces: Vec<Piece> = lines
        .filter_map(|line| -> Option<Piece> {
            if line.is_empty() {
                None
            } else {
                Some(
                    line.split(' ')
                        .collect::<Vec<&str>>()
                        .try_into()
                        .expect("couldn't parse line"),
                )
            }
        })
        .collect();
    assert_eq!(n_pieces, pieces.len());
    println!("\t{} Scheiben aus {} gelesen", n_pieces, path);
    pieces
}
/// Schreibt Käsescheiben in eine Datei
fn write_pieces(path: &str, pieces: &Vec<Piece>) {
    let file = File::create(path).expect("couldn't create file");
    let mut writer = BufWriter::new(file);
    writeln!(writer, "{}", pieces.len()).expect("couldn't write number of pieces");
    for piece in pieces {
        writeln!(writer, "{} {}", piece.0, piece.1).expect("couldn't write piece");
    }
    writer.flush().expect("couldn't flush writer");
}

/// Entfernt Scheiben mit einer bestimmten Wahrscheinlichkeit
fn eat_pieces(pieces: Vec<Piece>, rng: &mut ThreadRng, prob: f64) -> Vec<Piece> {
    let mut new_pieces = vec![];
    let mut last_eaten = false;
    let mut n_eaten = 0;
    for (_i, piece) in pieces.into_iter().enumerate() {
        if rng.gen_bool(1.0 - prob) || last_eaten {
            new_pieces.push(piece);
            last_eaten = false;
        } else {
            last_eaten = true;
            n_eaten += 1;
        }
    }
    println!("{} Scheiben wurden gegessen\n", n_eaten);
    new_pieces
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(long)]
    ///Wahrscheinlichkeit mit der Scheiben entfernt werden
    eat_prob: Option<f64>,
    ///Wird automatisch gesetzt wenn --eat-prob gesetzt ist
    #[arg(long, default_value = "false")]
    find_missing: bool,
    ///Die Dateien, aus denen die Scheiben geladen werden sollen
    #[arg(required = true)]
    files: Vec<String>,
}
// Lädt die Scheiben aus einer Datei, mischt sie und entfernt ggf. Stücke
fn prepare_pieces(opts: &Opts) -> Vec<Piece> {
    println!("Lade Scheiben...");
    let mut pieces = opts
        .files
        .iter()
        .flat_map(|path| load_pieces(path))
        .collect::<Vec<_>>();
    println!();
    let mut rng = thread_rng();
    println!("Mische Scheiben...\n");
    pieces.shuffle(&mut rng);
    if let Some(prob) = opts.eat_prob {
        pieces = eat_pieces(pieces, &mut rng, prob);
    }
    pieces
}
// Die Hauptfunktion
fn main() {
    let mut opts = Opts::parse();
    //println!("{:#?}", opts);
    if opts.eat_prob.is_some() {
        opts.find_missing = true;
    }
    // Scheiben werden vorbereitet
    let pieces = prepare_pieces(&opts);
    // Die Scheiben werden in eine HashMap geladen,
    // die die Anzahl der Scheiben mit einer bestimmten Größe speichert
    let pieces_map = Box::new(PiecesMap::new(&pieces));
    // Timer wird gestartet
    let start = Instant::now();
    // Es wird versucht Käse zu finden
    let result = construct_cheeses(pieces_map, pieces.len(), opts.find_missing);
    // Die Zeit wird gemessen
    let elapsed = start.elapsed();
    // Die Ergebnisse werden ausgegeben
    if result.is_empty() {
        println!("Kein Käse gefunden");
    } else {
        println!("{} Käse gefunden: ", result.len());
    }
    let n_results = result.len();
    for (i, (cheese, path)) in result.iter().enumerate() {
        if n_results > 1 {
            println!("   {}:", i);
        }
        println!("\tKäse: {:?}", cheese.size);
        if opts.find_missing {
            println!("\t{} Scheiben wurden hinzugefügt", path.n_added);
        }
        let path = path.curr.get_pieces().into_iter().rev().collect::<Vec<_>>();
        println!("\tStartscheibe: {:?}", path[0]);
        println!("\tLetzte Scheibe: {:?}", path.last().unwrap());
        let file_path = if n_results > 1 {
            format!("solution_{}.txt", i)
        } else {
            "solution.txt".to_string()
        };
        // Die Scheibenreihenfolge wird in eine Datei geschrieben
        write_pieces(&file_path, &path);
        println!("\tScheibenreihenfolge in {} gespeichert", file_path);
        println!();
    }

    println!("Suche hat {:?} gedauert", elapsed);
}
