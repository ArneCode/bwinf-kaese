mod cheese;
mod path;
mod pieces_map;

use cheese::Cheese;
use clap::{arg, Parser};
use path::Path;
use pieces_map::PiecesMap;
use rand::{prelude::*, rngs::ThreadRng, seq::SliceRandom, thread_rng};
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, fs, hash::Hash, time::Instant};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Piece(u32, u32);
impl TryFrom<Vec<&str>> for Piece {
    type Error = std::num::ParseIntError;
    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        assert!(value.len() == 2);
        let width = value[0].parse()?;
        let height = value[1].parse()?;
        if width > height {
            // panic!();
            Ok(Self(width, height))
        } else {
            Ok(Self(height, width))
        }
    }
}
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
    println!("read {} pieces from the file", n_pieces);
    pieces
}
pub struct PossPath {
    cheese: Cheese,
    path: Path,
    pieces: Box<PiecesMap>,
}

impl PossPath {
    fn new(cheese: Cheese, path: Path, pieces: Box<PiecesMap>) -> Self {
        Self {
            cheese,
            path,
            pieces,
        }
    }
}
fn filter_multiple_paths(mut poss_paths: Vec<PossPath>) -> Vec<PossPath> {
    let mut other_cheeses: FxHashMap<&Cheese, (&Box<PiecesMap>, Path, usize)> =
        FxHashMap::default();
    let mut used_paths: Vec<usize> = vec![];
    for (
        i,
        PossPath {
            cheese,
            path,
            pieces,
        },
    ) in poss_paths.iter().enumerate()
    {
        let mut add_to_others = true;
        if let Some((other_pieces, other_path, idx)) = other_cheeses.get(&cheese) {
            add_to_others = false;
            let keys = if pieces.base_id == other_pieces.base_id {
                pieces.added.keys()
            } else {
                //FIXME
                pieces.base.keys()
            };
            for piece in keys {
                if pieces.get(piece) != other_pieces.get(piece) {
                    add_to_others = true;
                    break;
                }
            }
            if !add_to_others && other_path.n_added > path.n_added {
                used_paths[*idx] = i;
            }
        }
        if add_to_others {
            other_cheeses.insert(cheese, (pieces, path.clone(), used_paths.len()));
            used_paths.push(i);
        }
    }
    used_paths.sort();
    used_paths
        .into_iter()
        .rev()
        .map(|idx| poss_paths.swap_remove(idx))
        .collect()
}

fn filter_top_paths(paths: Vec<Vec<PossPath>>) -> Vec<Vec<PossPath>> {
    //only leave the path with the min amount of added pieces
    let min_added_path = paths
        .into_iter()
        .min_by_key(|path| {
            path.iter()
                .min_by_key(|poss_path| poss_path.path.n_added)
                .unwrap()
                .path
                .n_added
        })
        .unwrap();
    vec![min_added_path]
}
fn filter_sub_paths(mut paths: Vec<PossPath>) -> Vec<PossPath> {
    //only leave the path with the min amount of added pieces
    //remove the paths with more added pieces
    let min_added = paths
        .iter()
        .fold(u32::MAX, |acc, poss_path| acc.min(poss_path.path.n_added));
    paths.retain(|poss_path| poss_path.path.n_added <= min_added);
    paths
}
// Setzt nur einen Käse zusammen, wird immer wieder von construct_cheeses aufgerufen
fn construct_cheese(
    mut top_paths: Vec<Vec<PossPath>>, // Die Pfade, nach Startstück getrennt
    min_path_len: usize,               // Die minimale Länge eines Pfades
    find_missing: bool,                // Ob nach fehlenden Stücken gesucht werden soll
) -> Option<(Cheese, Path)> {
    let mut i = 0; // Die aktuelle Länge der Pfade
    while !top_paths.is_empty() {
        if i % 10000 == 0 {
            //println!("n_paths (top): {}, i: {i}", top_paths.len());
        }
        if i == min_path_len / 2 && i > 4 && find_missing {
            // Wenn etwas Zeit vergangen ist, wird nur der Pfad mit den wenigsten
            // hinzugefügten Stücken weiterverfolgt
            top_paths = filter_top_paths(top_paths);
        }
        let mut new_top_paths = vec![]; // Die neuen top-Pfade
        let n_top = top_paths.len();
        for sub_paths in top_paths {
            if i % 10000 == 0 && i != 0 || sub_paths.len() > 4 {
                //println!("n_paths (sub): {}", sub_paths.len());
            }
            let mut new_paths = vec![]; // Die neuen sub-Pfade
            let curr_result = {
                let PossPath { cheese, path, .. } = &sub_paths[0];
                (*cheese, path.clone())
            };
            for PossPath {
                cheese,
                path,
                pieces,
            } in sub_paths.into_iter()
            {
                let paths = cheese.gen_poss_paths(path, pieces, find_missing);
                new_paths.extend(paths);
            }
            if new_paths.len() > 1 {
                new_paths = filter_sub_paths(new_paths);
            }
            if new_paths.is_empty() {
                if i >= min_path_len {
                    let (cheese, path) = &curr_result;
                    println!(
                        "found cheese: {:?}, len: {}, n_added: {}, n_top: {}",
                        cheese.size, path.len, path.n_added, n_top
                    );
                    return Some(curr_result);
                }
            } else {
                new_top_paths.push(filter_multiple_paths(new_paths));
            }
        }
        top_paths = new_top_paths;
        i += 1;
    }
    None
}
fn construct_cheeses(
    pieces: Box<PiecesMap>,
    n_pieces: usize,
    find_missing: bool,
) -> Vec<(Cheese, Path)> {
    // let poss_paths = {
    let mut pieces_map = pieces;
    //find possible start pieces_map
    let mut results = vec![];
    let mut used_pieces = vec![];
    let mut min_path_len = n_pieces * 3 / 4;
    while min_path_len > 2 {
        println!("min_path_len: {}", min_path_len);
        let keys = pieces_map.base.keys().cloned().collect::<Vec<Piece>>();
        let top_paths = keys
            .iter()
            .map(|piece| {
                let cheese = Cheese::new([piece.0, piece.1, 0]);
                let start = PossPath::new(cheese, Path::new(*piece), pieces_map.clone());
                vec![start]
            })
            .collect::<Vec<_>>();
        if let Some((cheese, path)) = construct_cheese(top_paths, min_path_len, find_missing) {
            let new_used_pieces = path.curr.get_real_pieces();
            pieces_map = Box::new(pieces_map.clone_without(&new_used_pieces));
            used_pieces.extend(new_used_pieces);
            //println!("total len: {}/{}", path.len, n_pieces);
            min_path_len = (n_pieces - used_pieces.len()) * 3 / 4;
            results.push((cheese, path));
            // found_cheese = true;
        } else {
            min_path_len /= 2;
        }
    }
    //check whether the correct amount of pieces was used
    match used_pieces.len().cmp(&n_pieces) {
        Ordering::Less => {
            println!("not all pieces used!!, {:?}", pieces_map);
            println!("total len: {}/{}", used_pieces.len(), n_pieces);
            println!("pieces left: {}", pieces_map.base.values().sum::<u32>());
        }
        Ordering::Greater => {
            println!("used: {:?}", used_pieces);
            panic!("error, used {} out of {}", used_pieces.len(), n_pieces);
        }
        Ordering::Equal => {}
    }

    results
}
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
    println!("\nate {} pieces\n", n_eaten);
    new_pieces
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(short, long)]
    ///Wahrscheinlichkeit, dass ein stück gegessen wird
    prob: Option<f64>,
    ///Ob fehlende Stücke gesucht werden sollen, wird automatisch gesetzt,
    /// wenn prob gesetzt ist
    #[arg(short, long, default_value = "false")]
    missing: bool,
    ///Die Dateien, aus denen die Stücke geladen werden sollen
    #[arg(required = true)]
    files: Vec<String>,
}
fn prepare_pieces(opts: &Opts) -> Vec<Piece> {
    let mut pieces = opts
        .files
        .iter()
        .flat_map(|path| load_pieces(path))
        .collect::<Vec<_>>();
    let mut rng = thread_rng();
    pieces.shuffle(&mut rng);
    if let Some(prob) = opts.prob {
        pieces = eat_pieces(pieces, &mut rng, prob);
    }
    pieces
}
fn main() {
    let mut opts = Opts::parse();
    if opts.prob.is_some() {
        opts.missing = true;
    }
    let pieces = prepare_pieces(&opts);
    let start = Instant::now();
    let pieces_map = Box::new(PiecesMap::new(&pieces));
    let result = construct_cheeses(pieces_map, pieces.len(), opts.missing);
    if result.is_empty() {
        println!("found no solution");
    }
    for (cheese, path) in &result {
        println!("found solution: {:#?}", cheese);
        let path = path.curr.get_pieces().into_iter().rev().collect::<Vec<_>>();
        println!("first piece: {:?}", path[0]);
        println!("last piece: {:?}", path.last().unwrap());
    }
    let elapsed = start.elapsed();
    println!("took: {:?}", elapsed);
}
